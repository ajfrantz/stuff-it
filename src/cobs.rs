pub fn max_encoded_len(raw_len: usize) -> usize {
    let max_overhead = match raw_len {
        0 => 0,
        _ => 1 + ((raw_len - 1) / 254),
    };

    raw_len + max_overhead
}

pub fn encode(src: &[u8], dst: &mut [u8]) -> Result<usize, ()> {
    if src.is_empty() {
        return Ok(0);
    }

    let mut block_start_idx = 0;
    let mut next_data_idx = 1;
    let mut block_len = 1;
    let mut skip_last = false;

    for &b in src {
        skip_last = false;

        if b == 0 {
            *dst.get_mut(block_start_idx).ok_or(())? = block_len;
            block_start_idx = next_data_idx;
            next_data_idx += 1;
            block_len = 1;
        } else {
            *dst.get_mut(next_data_idx).ok_or(())? = b;
            next_data_idx += 1;
            block_len += 1;

            if block_len == 0xff {
                *dst.get_mut(block_start_idx).ok_or(())? = block_len;
                block_start_idx = next_data_idx;
                next_data_idx += 1;
                block_len = 1;
                skip_last = true;
            }
        }
    }

    if skip_last {
        Ok(next_data_idx - 1)
    } else {
        *dst.get_mut(block_start_idx).ok_or(())? = block_len;
        Ok(next_data_idx)
    }
}

pub fn decode(buffer: &mut [u8]) -> Result<&[u8], ()> {
    let mut read_idx = 0;
    let mut write_idx = 0;

    let input_len = buffer.len();
    while read_idx < input_len {
        let block_len = buffer[read_idx] as usize;
        read_idx += 1;

        if block_len > 1 {
            let copy_len = block_len - 1;
            if read_idx + copy_len > input_len {
                return Err(());
            }
            buffer.copy_within(read_idx..read_idx + copy_len, write_idx);
            read_idx += copy_len;
            write_idx += copy_len;
        }

        if block_len != 0xff && read_idx < input_len {
            buffer[write_idx] = 0;
            write_idx += 1;
        }
    }

    Ok(&buffer[..write_idx])
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    struct Example {
        input: Vec<u8>,
        output: Vec<u8>,
    }

    fn examples() -> Vec<Example> {
        vec![
            // Short sequences
            Example {
                input: vec![],
                output: vec![],
            },
            Example {
                input: vec![42],
                output: vec![2, 42],
            },
            Example {
                input: vec![0, 1, 0],
                output: vec![1, 2, 1, 1],
            },
            // Paper's example
            Example {
                input: vec![
                    0x45, 0x00, 0x00, 0x2c, 0x4c, 0x79, 0x00, 0x00, 0x40, 0x06, 0x4f, 0x37,
                ],
                output: vec![
                    0x02, 0x45, 0x01, 0x04, 0x2c, 0x4c, 0x79, 0x01, 0x05, 0x40, 0x06, 0x4f, 0x37,
                ],
            },
            // Wikipedia's examples
            Example {
                input: vec![0x00],
                output: vec![0x01, 0x01],
            },
            Example {
                input: vec![0x00, 0x00],
                output: vec![0x01, 0x01, 0x01],
            },
            Example {
                input: vec![0x11, 0x22, 0x00, 0x33],
                output: vec![0x03, 0x11, 0x22, 0x02, 0x33],
            },
            Example {
                input: vec![0x11, 0x22, 0x33, 0x44],
                output: vec![0x05, 0x11, 0x22, 0x33, 0x44],
            },
            Example {
                input: vec![0x11, 0x00, 0x00, 0x00],
                output: vec![0x02, 0x11, 0x01, 0x01, 0x01],
            },
            Example {
                input: (1..=254).collect(),
                output: [vec![0xff], (1..=254).collect()].concat(),
            },
            Example {
                input: (0..=254).collect(),
                output: [vec![0x01, 0xff], (1..=254).collect()].concat(),
            },
            Example {
                input: (1..=255).collect(),
                output: [vec![0xff], (1..=254).collect(), vec![0x02, 0xff]].concat(),
            },
            Example {
                input: [(2..=255).collect(), vec![0x00]].concat(),
                output: [vec![0xff], (2..=255).collect(), vec![0x01, 0x01]].concat(),
            },
            Example {
                input: [(3..=255).collect(), vec![0x00, 0x01]].concat(),
                output: [vec![0xfe], (3..=255).collect(), vec![0x02, 0x01]].concat(),
            },
        ]
    }

    #[test]
    fn test_encode_examples() {
        for (i, example) in examples().iter().enumerate() {
            let output_len = example.output.len();
            let mut output_buf = vec![0; output_len];
            assert_eq!(
                Ok(output_len),
                encode(&example.input, &mut output_buf),
                "example {}",
                i
            );
            assert_eq!(example.output, output_buf, "example {}", i);
        }
    }

    #[test]
    fn test_decode_examples() {
        for (i, example) in examples().iter().enumerate() {
            let mut buffer = example.output.clone();
            let result = decode(&mut buffer);

            let expected: &[u8] = &example.input;
            assert_eq!(Ok(expected), result, "example {}", i);
        }
    }

    proptest! {
        #[test]
        fn encode_props(ref data in any::<Vec<u8>>()) {
            // Should add no more than 1 byte per 254 bytes of input.
            let mut output = vec![0; max_encoded_len(data.len())];

            let result = encode(&data, &mut output);

            // We allocated enough space that this should always succeed.
            assert!(result.is_ok());
            let output = &output[..result.unwrap()];

            // If we had data, we should have gotten more bytes out than in.
            if !data.is_empty() {
                assert!(output.len() > data.len());
            }

            // The output should never include a zero.
            assert!(!output.iter().any(|&b| b == 0));
        }

        #[test]
        fn round_trip(ref data in any::<Vec<u8>>()) {
            let mut buffer = vec![0; 2 * data.len()];

            let encoded_len = encode(&data, &mut buffer).unwrap();
            let decoded = decode(&mut buffer[..encoded_len]).unwrap();

            // We should always get the same data back out.
            assert_eq!(data, &decoded);
        }
    }
}

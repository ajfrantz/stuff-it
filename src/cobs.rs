pub fn encode(src: &[u8], dst: &mut [u8]) -> Result<usize, ()> {
    if src.is_empty() {
        return Ok(0);
    }

    let src_len = src.len();
    let mut read_idx = 0;
    let mut write_idx = 0;

    let mut zero_idx = 0;
    while zero_idx < src_len && src[zero_idx] != 0 {
        zero_idx += 1;
    }

    while read_idx < src_len {
        let block_end = zero_idx;
        let block_len = (block_end - read_idx).min(254);

        let write_end = write_idx + 1 + block_len;
        if write_end > dst.len() {
            return Err(());
        }

        dst[write_idx] = (block_len + 1) as u8;
        dst[write_idx + 1..write_end].copy_from_slice(&src[read_idx..read_idx + block_len]);

        read_idx += block_len;
        write_idx = write_end;

        if block_len != 254 {
            read_idx += 1;
            zero_idx += 1;
            while zero_idx < src_len && src[zero_idx] != 0 {
                zero_idx += 1;
            }
        }
    }

    // If our input happened to end in a zero, we have to go out of our way to encode the
    // "phantom zero" at the end.
    if src[src_len - 1] == 0 {
        dst[write_idx] = 1;
        write_idx += 1;
    }

    Ok(write_idx)
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
}

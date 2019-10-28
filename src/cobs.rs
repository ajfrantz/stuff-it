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
}

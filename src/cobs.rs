struct BlockIter<'a> {
    slice: Option<&'a [u8]>,
    next_zero: Option<usize>,
}

fn cobs_blocks<'a>(slice: &'a [u8]) -> BlockIter<'a> {
    BlockIter {
        slice: Some(slice),
        next_zero: slice.iter().position(|&b| b == 0),
    }
}

impl<'a> Iterator for BlockIter<'a> {
    type Item = &'a [u8];

    fn next(&mut self) -> Option<&'a [u8]> {
        self.slice.map(|slice| {
            // Peel off up to 254 non-zero bytes as the block contents.
            match self.next_zero {
                Some(zero_idx) if zero_idx < 254 => {
                    // There's a zero within the next 254 bytes which determines where we stop.
                    let (block, remain) = slice.split_at(zero_idx);
                    // Skip the zero itself; we never explicitly output those as part of a block.
                    let remain = &remain[1..];
                    self.slice = Some(remain);
                    self.next_zero = remain.iter().position(|&b| b == 0);
                    block
                }
                _ => {
                    // There was no terminating zero in the next 254 bytes; output as many bytes as are
                    // left (up to a max-length block) and adjust the remainder.
                    if slice.len() <= 254 {
                        // Take the rest of the input and mark us as complete.
                        self.slice = None;
                        slice
                    } else {
                        // There's more to go, just take a full block's worth of non-zero bytes.
                        let (block, remain) = slice.split_at(254);
                        self.slice = Some(remain);
                        self.next_zero = self.next_zero.map(|idx| idx - 254);
                        block
                    }
                }
            }
        })
    }
}

pub fn encode(src: &[u8], dst: &mut [u8]) -> Result<usize, ()> {
    let dst_len = dst.len();
    let mut out_idx = 0;

    for block in cobs_blocks(src) {
        let len = block.len();
        let dst_begin = out_idx + 1;
        let dst_end = out_idx + 1 + len;

        if dst_end > dst_len {
            return Err(());
        }

        dst[out_idx] = (len + 1) as u8;
        dst[dst_begin..dst_end].copy_from_slice(block);
        out_idx += len + 1;
    }

    Ok(out_idx)
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
                input: vec![10, 11, 0, 12],
                output: vec![3, 10, 11, 2, 12],
            },
            Example {
                input: vec![0, 0, 1, 0],
                output: vec![1, 1, 2, 1, 1],
            },
            Example {
                input: vec![255, 0],
                output: vec![2, 255, 1],
            },
            Example {
                input: vec![1],
                output: vec![2, 1],
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

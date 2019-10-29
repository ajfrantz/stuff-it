#![no_main]
use libfuzzer_sys::fuzz_target;
use stuff_it::cobs::{encode, max_encoded_len};

fuzz_target!(|data: &[u8]| {
    let mut output = vec![0; max_encoded_len(data.len())];

    let encoded_len = encode(&data, &mut output).unwrap();
    let output = &output[..encoded_len];

    if !data.is_empty() {
        assert!(encoded_len > data.len());
    }

    assert!(!output.iter().any(|&b| b == 0));
});

#![no_main]
use libfuzzer_sys::fuzz_target;
use stuff_it::cobs::decode;

fuzz_target!(|data: &[u8]| {
    let mut buffer = data.to_vec();
    let _ = decode(&mut buffer);
});

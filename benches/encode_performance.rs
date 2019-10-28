use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use stuff_it::cobs::encode;

// This is more or less a direct reproduction of the reference implementation from the paper.
// In order to mimic C/C++, we do everything with _unchecked.  Don't use this.
fn reference_encoder(src: &[u8], dst: &mut [u8]) {
    unsafe {
        let length = src.len();
        let mut ptr = src.as_ptr();
        let end = ptr.add(length);

        let mut dst = dst.as_mut_ptr();
        let mut code_ptr = dst;
        dst = dst.add(1);
        let mut code = 0x01;

        while ptr < end {
            if *ptr == 0 {
                *code_ptr = code;
                code_ptr = dst;
                dst = dst.add(1);
                code = 0x01;
            } else {
                *dst = *ptr;
                dst = dst.add(1);
                code += 1;
                if code == 0xFF {
                    *code_ptr = code;
                    code_ptr = dst;
                    dst = dst.add(1);
                    code = 0x01;
                }
            }
            ptr = ptr.add(1);
        }

        *code_ptr = code;
    }
}

pub fn bench_encode(c: &mut Criterion) {
    let mut output_buf = vec![0; 2048];
    let mut group = c.benchmark_group("encode");

    for &input_len in &[1, 64, 256, 1024] {
        let input: Vec<_> = (1..input_len).map(|x| x as u8).collect();

        group.throughput(Throughput::Bytes(input_len));
        group.bench_with_input(BenchmarkId::new("[rust]", input_len), &input, |b, src| {
            b.iter(|| encode(&src, &mut output_buf))
        });
        group.bench_with_input(BenchmarkId::new("[unsafe]", input_len), &input, |b, src| {
            b.iter(|| reference_encoder(&src, &mut output_buf))
        });
    }
}

pub fn bench_encode_zeros(c: &mut Criterion) {
    let mut output_buf = vec![0; 2048];
    let mut group = c.benchmark_group("encode_zeros");

    for &input_len in &[1, 64, 256, 1024] {
        let mut input = Vec::new();
        input.resize(input_len, 0);

        group.throughput(Throughput::Bytes(input_len as u64));
        group.bench_with_input(BenchmarkId::new("[rust]", input_len), &input, |b, src| {
            b.iter(|| encode(&src, &mut output_buf))
        });
        group.bench_with_input(BenchmarkId::new("[unsafe]", input_len), &input, |b, src| {
            b.iter(|| reference_encoder(&src, &mut output_buf))
        });
    }
}

criterion_group!(benches, bench_encode, bench_encode_zeros);
criterion_main!(benches);

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use stuff_it::cobs::encode;

pub fn bench_encode(c: &mut Criterion) {
    let mut output_buf = vec![0; 2048];
    let mut group = c.benchmark_group("encode");

    for &input_len in &[1, 32, 64, 256, 512, 768, 1024] {
        let input: Vec<_> = (1..input_len).map(|x| x as u8).collect();

        group.throughput(Throughput::Bytes(input_len));
        group.bench_with_input(BenchmarkId::from_parameter(input_len), &input, |b, src| {
            b.iter(|| encode(&src, &mut output_buf))
        });
    }
}

pub fn bench_encode_zeros(c: &mut Criterion) {
    let mut output_buf = vec![0; 2048];
    let mut group = c.benchmark_group("encode_zeros");

    for &input_len in &[1, 32, 64, 256, 512, 768, 1024] {
        let mut input = Vec::new();
        input.resize(input_len, 0);

        group.throughput(Throughput::Bytes(input_len as u64));
        group.bench_with_input(BenchmarkId::from_parameter(input_len), &input, |b, src| {
            b.iter(|| encode(&src, &mut output_buf))
        });
    }
}

criterion_group!(benches, bench_encode, bench_encode_zeros);
criterion_main!(benches);

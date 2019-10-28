use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use stuff_it::cobs::{decode, encode};

const INPUT_SIZES: &[usize] = &[1, 32, 64, 256, 512, 768, 1024];

fn bench_encode(c: &mut Criterion, name: &str, input_gen: fn(usize) -> Vec<u8>) {
    let mut output_buf = vec![0; 2048];
    let mut group = c.benchmark_group(name);

    for &input_len in INPUT_SIZES {
        let input = input_gen(input_len);

        group.throughput(Throughput::Bytes(input_len as u64));
        group.bench_with_input(BenchmarkId::from_parameter(input_len), &input, |b, src| {
            b.iter(|| encode(&src, &mut output_buf).unwrap())
        });
    }
}

fn bench_encode_incrementing(c: &mut Criterion) {
    bench_encode(c, "encode/inc", |input_len| {
        (1..=input_len).map(|x| x as u8).collect()
    });
}

fn bench_encode_zeros(c: &mut Criterion) {
    bench_encode(c, "encode/zero", |input_len| {
        (1..=input_len).map(|_| 0u8).collect()
    });
}

fn bench_decode(c: &mut Criterion, name: &str, input_gen: fn(usize) -> Vec<u8>) {
    let mut buffer = vec![0; 2048];
    let mut group = c.benchmark_group(name);

    for &input_len in INPUT_SIZES {
        let input = input_gen(input_len);

        let mut encoded = vec![0; 2048];
        let end = encode(&input, &mut encoded).unwrap();
        encoded.resize(end, 0);

        group.throughput(Throughput::Bytes(input_len as u64));
        group.bench_with_input(
            BenchmarkId::from_parameter(input_len),
            &encoded,
            |b, input| {
                b.iter(|| {
                    buffer[..input.len()].copy_from_slice(&input);
                    let len = decode(&mut buffer[..input.len()]).unwrap().len();
                    assert!(len == input_len, "{} != {}", len, input_len);
                })
            },
        );
    }
}

fn bench_decode_incrementing(c: &mut Criterion) {
    bench_decode(c, "decode/inc", |input_len| {
        (1..=input_len).map(|x| x as u8).collect()
    });
}

fn bench_decode_zeros(c: &mut Criterion) {
    bench_decode(c, "decode/zero", |input_len| {
        (1..=input_len).map(|_| 0u8).collect()
    });
}

criterion_group!(
    benches,
    bench_encode_incrementing,
    bench_encode_zeros,
    bench_decode_incrementing,
    bench_decode_zeros
);
criterion_main!(benches);

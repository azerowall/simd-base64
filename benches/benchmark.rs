#![feature(portable_simd)]

use criterion::{BenchmarkId, Throughput};
use criterion::{criterion_group, criterion_main, Criterion, black_box};

use simd_base64::base64;
use simd_base64::base64_simd;

fn generate_base64_data(size: usize) -> Vec<u8> {
    let alphabet = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

    let mut count = 0;
    Vec::from_iter(std::iter::from_fn(|| {
        count += 1;
        if count == size {
            return None;
        }
        Some(alphabet[count % alphabet.len()])
    }))
}

fn generate_binary_data(size: usize) -> Vec<u8> {
    let mut count = 0;
    Vec::from_iter(std::iter::from_fn(|| {
        count += 1;
        if count == size {
            return None;
        }
        Some(count as u8)
    }))
}

fn bench_decode(c: &mut Criterion) {
    let mut group = c.benchmark_group("decode");

    for size in [100, 1000, 10_000] {
        let data = generate_base64_data(size);

        group.throughput(Throughput::Bytes(size as u64));

        group
            .bench_with_input(BenchmarkId::new("classic", size), &data, |g, input| {
                g.iter(|| {
                    base64::decode(&input, &mut Vec::new()).unwrap();
                })
            })
            .bench_with_input(BenchmarkId::new("simd_8", size), &data, |g, input| {
                g.iter(|| {
                    base64_simd::decode::<8>(&input, &mut Vec::new()).unwrap();
                })
            })
            .bench_with_input(BenchmarkId::new("simd_16", size), &data, |g, input| {
                g.iter(|| {
                    base64_simd::decode::<16>(&input, &mut Vec::new()).unwrap();
                })
            })
            .bench_with_input(BenchmarkId::new("simd_32", size), &data, |g, input| {
                g.iter(|| {
                    base64_simd::decode::<32>(&input, &mut Vec::new()).unwrap();
                })
            });
    }

    group.finish();
}

fn bench_encode(c: &mut Criterion) {

    let mut group = c.benchmark_group("encode");

    for size in [100, 1000, 10_000] {
        let data = generate_binary_data(size);

        group.throughput(Throughput::Bytes(size as u64));

        group
            .bench_with_input(BenchmarkId::new("classic", size), &data, |g, input| {
                g.iter(|| {
                    base64::encode(&input, &mut Vec::new())
                })
            })
            .bench_with_input(BenchmarkId::new("simd_4",  size),&data, |g, input| {
                g.iter(|| {
                    base64_simd::encode::<4>(&input, &mut Vec::new())
                })
            })
            .bench_with_input(BenchmarkId::new("simd_8", size), &data, |g, input| {
                g.iter(|| {
                    base64_simd::encode::<8>(&input, &mut Vec::new())
                })
            })
            .bench_with_input(BenchmarkId::new("simd_16", size), &data, |g, input| {
                g.iter(|| {
                    base64_simd::encode::<16>(&input, &mut Vec::new())
                })
            })
            .bench_with_input(BenchmarkId::new("simd_32", size), &data, |g, input| {
                g.iter(|| {
                    base64_simd::encode::<32>(&input, &mut Vec::new())
                })
            });
    }

    group.finish();
}

criterion_group!(
    decode,
    bench_decode,
);

criterion_group!(
    encode,
    bench_encode,
);

criterion_main!(
    decode,
    encode,
);
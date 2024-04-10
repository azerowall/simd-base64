#![feature(portable_simd)]

use std::simd::LaneCount;
use std::simd::SupportedLaneCount;

use criterion::{criterion_group, criterion_main, Criterion, black_box};

use simd_base64::base64;
use simd_base64::base64_simd;

const TEST_DATA_SIZE: usize = 1_000_000;

fn generate_base64_data() -> Vec<u8> {
    let alphabet = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

    let mut count = 0;
    black_box(Vec::from_iter(std::iter::from_fn(|| {
        count += 1;
        if count == TEST_DATA_SIZE {
            return None;
        }
        Some(alphabet[count % alphabet.len()])
    })))
}

fn generate_binary_data() -> Vec<u8> {
    let mut count = 0;
    black_box(Vec::from_iter(std::iter::from_fn(|| {
        count += 1;
        if count == TEST_DATA_SIZE {
            return None;
        }
        Some(count as u8)
    })))
}

fn bench_decode(c: &mut Criterion) {
    let data = generate_base64_data();

    c.bench_function("base64_decode", |b| {
        b.iter(|| {
            let mut out = Vec::new();
            base64::decode(&data, &mut out).unwrap();
        });
    });
}

fn bench_decode_simd<const N: usize>(c: &mut Criterion)
where
    LaneCount<N>: SupportedLaneCount
{
    let data = generate_base64_data();

    c.bench_function(&format!("base64_decode_simd{}", N), |b| {
        b.iter(|| {
            let mut out = Vec::new();
            base64_simd::decode::<N>(&data, &mut out).unwrap();
        });
    });
}

fn bench_encode(c: &mut Criterion) {
    let data = generate_binary_data();

    c.bench_function("base64_encode", |b| {
        b.iter(|| {
            let mut out = Vec::new();
            base64::encode(&data, &mut out);
        });
    });
}

fn bench_encode_simd<const N: usize>(c: &mut Criterion)
where
    LaneCount<N>: SupportedLaneCount
{
    let data = generate_binary_data();

    c.bench_function(&format!("base64_encode_simd{}", N), |b| {
        b.iter(|| {
            let mut out = Vec::new();
            base64_simd::encode::<N>(&data, &mut out);
        });
    });
}

criterion_group!(
    decode,
    bench_decode,
    bench_decode_simd::<8>,
    bench_decode_simd::<16>,
    bench_decode_simd::<32>,
);

criterion_group!(
    encode,
    bench_encode,
    bench_encode_simd::<4>,
    bench_encode_simd::<8>,
    bench_encode_simd::<16>,
    bench_encode_simd::<32>,
);

criterion_main!(
    decode,
    encode,
);
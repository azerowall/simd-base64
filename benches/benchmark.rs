#![feature(portable_simd)]

use std::simd::LaneCount;
use std::simd::SupportedLaneCount;

use criterion::{criterion_group, criterion_main, Criterion, black_box};

use simd_base64::base64;
use simd_base64::base64_simd;


fn bench_decode(c: &mut Criterion) {
    c.bench_function("base64_decode", |b| {
        let data = black_box(b"SGVsbG8sIHdvcmxkIQ==");
        b.iter(|| {
            let mut out = Vec::new();
            base64::decode(data, &mut out).unwrap();
        });
    });
}

fn bench_decode_simd<const N: usize>(c: &mut Criterion)
where
    LaneCount<N>: SupportedLaneCount
{
    c.bench_function(&format!("base64_decode_simd{}", N), |b| {
        let data = black_box(b"SGVsbG8sIHdvcmxkIQ==");
        b.iter(|| {
            let mut out = Vec::new();
            base64_simd::decode::<N>(data, &mut out).unwrap();
        });
    });
}

fn bench_encode(c: &mut Criterion) {
    c.bench_function("base64_encode", |b| {
        let data = black_box(b"Hello, world!");
        b.iter(|| {
            let mut out = Vec::new();
            base64::encode(data, &mut out);
        });
    });
}

fn bench_encode_simd<const N: usize>(c: &mut Criterion)
where
    LaneCount<N>: SupportedLaneCount
{
    c.bench_function(&format!("base64_encode_simd{}", N), |b| {
        let data = black_box(b"Hello, world!");
        b.iter(|| {
            let mut out = Vec::new();
            base64_simd::encode::<N>(data, &mut out);
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
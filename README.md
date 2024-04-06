# Simd base64

This is implementation of base64 algorithm with simd for fun and self education. Based on article https://mcyoung.xyz/2023/11/27/simd-base64/.

Here are two implementations of the Base64 algorithm: standard and with `std::simd` (SIMD portable library).

Requires nigtly, because `std::simd` is nightly.

# How to run benchmarks

`RUSTFLAGS="-Ctarget-cpu=native" cargo bench -Zbuild-std --target=<your_target>`

You can get your target by running comand `rustc -vV` - field `host`.

use std::{fmt, simd::{LaneCount, Simd, SimdElement, SupportedLaneCount}};



pub const SEXTET_UPPERS_OFFSET: u8 = 0;
pub const SEXTET_LOWERS_OFFSET: u8 = 26;
pub const SEXTET_DIGITS_OFFSET: u8 = 52;
pub const SEXTET_PLUS_OFFSET: u8 = 62;
pub const SEXTET_SLASH_OFFSET: u8 = 63;

// Removes '=' at the end of base64-encoded string
pub fn remove_trailing_eq(input: &[u8]) -> &[u8] {
    match input {
        [out @ .., b'=', b'='] => out,
        [out @ .., b'='] => out,
        out => out,
    }
}

// Adds '=' to the end of base64-encoded string
pub fn pad_with_trailing_eq(data_len: usize, out: &mut Vec<u8>) {
    match data_len % 3 {
        1 => {
            out.push(b'=');
            out.push(b'=');
        },
        2 => {
            out.push(b'=');
        },
        _ => {},
    }
}

// Functions for debug
// Return string representation of bits (bytes ordered in big endian)
#[allow(unused)]
pub fn bits(data: u32) -> String {
    let bytes = data.to_be_bytes();
    format!("{:0>8b} {:0>8b} {:0>8b} {:0>8b}", bytes[0], bytes[1], bytes[2], bytes[3])
}

#[allow(unused)]
pub fn bits_slice<T>(data: &[T]) -> String
where
    T: fmt::Binary
{
    data.iter().map(|b| format!("{b:0>8b}")).collect::<Vec<_>>().join(" ")
}

#[allow(unused)]
pub fn bits_simd<T, const N: usize>(data: Simd<T, N>) -> String
where
    T: SimdElement + fmt::Binary,
    LaneCount<N>: SupportedLaneCount,
{
    bits_slice(&data.to_array())
}
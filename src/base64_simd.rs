use core::simd::{Simd, num::SimdUint};
use std::simd::{cmp::{SimdPartialEq, SimdPartialOrd}, num::SimdInt, LaneCount, Mask, SimdElement, SupportedLaneCount};

use crate::common;
use crate::common::{
    SEXTET_UPPERS_OFFSET,
    SEXTET_LOWERS_OFFSET,
    SEXTET_DIGITS_OFFSET,
    SEXTET_PLUS_OFFSET,
    SEXTET_SLASH_OFFSET,
};

// branchless version of decoded_len() from base64.rs
pub fn decoded_len(encoded_len: usize) -> usize {
    // mod4 -> padding:
    // 0 => 0
    // 1 => 1
    // 2 => 1 
    // 3 => 2
    let mod4 = encoded_len % 4;
    let padding = mod4 - mod4 / 2;

    encoded_len / 4 * 3 + padding
}

// branchless version of encoded_len() from base64.rs
pub fn encoded_len(decoded_len: usize) -> usize {
    // mod3 -> padding:
    // 0 => 0
    // 1 => 2
    // 2 => 3
    let mod3 = decoded_len % 3;
    let padding = mod3 + (mod3 != 0) as usize;

    decoded_len / 3 * 4 + padding
}

// This function doesn't return Option<_> because it would lead
// to branching (match, if let)
#[inline]
fn decode_hot<const N: usize>(ascii: Simd<u8, N>) -> (Simd<u8, N>, bool)
where
    LaneCount<N>: SupportedLaneCount
{
    // Hash function:
    // A-Z = 0x41-0x5b => 4-5
    // a-z = 0x61-0x7b => 6-7
    // 0-9 = 0x30-0x3a => 3
    // +   = 0x2b      => 2
    // /   = 0x2f      => 1
    let hashes = (ascii >> Simd::splat(4))
        + ascii.simd_eq(Simd::splat(b'/')).to_int().cast::<u8>();

    // Even if this function is generic over N,
    // it has to be N > 4 (8, 16, ...) because
    // of the size of this array
    let offsets_for_hash = [
        0,
        b'/' as i8 - SEXTET_SLASH_OFFSET as i8,
        b'+' as i8 - SEXTET_PLUS_OFFSET as i8,
        b'0' as i8 - SEXTET_DIGITS_OFFSET as i8,
        b'A' as i8 - SEXTET_UPPERS_OFFSET as i8,
        b'A' as i8 - SEXTET_UPPERS_OFFSET as i8,
        b'a' as i8 - SEXTET_LOWERS_OFFSET as i8,
        b'a' as i8 - SEXTET_LOWERS_OFFSET as i8,
    ];
    let offsets_for_hash = repeated(&offsets_for_hash);

    // Use hashes as indicies to select appropriate offsets
    let offsets = offsets_for_hash.cast::<u8>().swizzle_dyn(hashes);
    let sextets = ascii - offsets;

    let ok = validate(ascii);

    // Pack 4 sextets into 3 bytes
    let shifts = [2, 4, 6, 8];
    let shifted = sextets.cast::<u16>() << repeated(&shifts);
    let lo = shifted.cast::<u8>();                      
    let hi = (shifted >> Simd::splat(8)).cast::<u8>();

    // low bits:  11111100 22220000 33000000 00000000
    // high bits: 00000000 00000022 00003333 00444444
    let packed_chunks = lo | hi.rotate_elements_left::<1>();

    // There are garbage value in every 4th lane after packing.
    // So clear every 4th lane.
    let indicies: [u8; N] = std::array::from_fn(|i| (i + i / 3) as u8);
    let output = packed_chunks.swizzle_dyn(Simd::from(indicies));

    (output, ok)
}

pub fn decode<const N: usize>(data: &[u8], out: &mut Vec<u8>) -> Result<(), &'static str>
where
    LaneCount<N>: SupportedLaneCount
{
    let data = common::remove_trailing_eq(data);

    // Calculate reservation size with extra, thus
    // we will be able to store the whole simd reg at once
    let final_size = decoded_len(data.len());
    let extra_size = N - 1;
    out.reserve(final_size + extra_size);

    let mut ptr = out.as_mut_ptr_range().end;
    let start = ptr;

    let mut error = false;

    let mut chunks = data.chunks_exact(N);

    // main loop
    for chunk in &mut chunks {
        let (sextets, ok) = decode_hot::<N>(Simd::from_slice(chunk));
        error |= !ok;

        let decoded = decoded_len(N);
        // Safety: there was allocated enough space
        unsafe {
            ptr.cast::<Simd<u8,N>>().write_unaligned(sextets);
            ptr = ptr.add(decoded);
        }
        
        // Safe variant:
        // out.extend_from_slice(&sextets.as_array()[..decoded]);
    }

    // remainder
    let rest = chunks.remainder();
    if !rest.is_empty() {
        let mut ascii = [b'A'; N];
        ascii[..rest.len()].copy_from_slice(rest);

        let (sextets, ok) = decode_hot::<N>(Simd::from(ascii));
        error |= !ok;

        let decoded = decoded_len(rest.len());
        // Safety: there was allocated enough space
        unsafe {
            ptr.cast::<Simd<u8,N>>().write_unaligned(sextets);
            ptr = ptr.add(decoded);
        }

        // Safe variant:
        // out.extend_from_slice(&sextets.as_array()[..decoded]);
    }

    if error {
        return Err("wrong base64");
    }
    
    // Safety:
    // - there was allocated enough space,
    // - all bytes are initialized
    // - any garbage from the last store (remainder) will be after vec.len()
    unsafe {
        let size = ptr.offset_from(start);
        // assert_eq!(size as usize, final_size);
        out.set_len(size as usize);
    }

    Ok(())
}

#[inline]
fn encode_hot<const N: usize>(bytes: Simd<u8, N>) -> Simd<u8, N>
where
    LaneCount<N>: SupportedLaneCount
{
    // Step 1: we need each 4th line empty
    // so make it by shifting bytes further.
    // aaaaaabb bbbbcccc ccdddddd eeeeeeff ffffgggg .. ->
    // aaaaaabb bbbbcccc ccdddddd ........ eeeeeeff ..

    let indicies: [u8; N] = std::array::from_fn(|i| [(i - i / 4) as u8, !0u8][((i + 1) % 4 == 0) as usize]);
    let bytes = bytes.swizzle_dyn(Simd::from(indicies));

    // Step 2: shift the bits so that each byte will be a sextet:
    // aaaaaabb bbbbcccc ccdddddd ........ ->
    // ..aaaaaa ..bbbbbb ..cccccc ..dddddd
    // To do that just inverse the operations from decode() func

    let mask: Simd<u8, N> = repeated(&[0b11111100, 0b11110000, 0b11000000, 0]);
    let lo = bytes & mask;
    let hi = (bytes & !mask).rotate_elements_right::<1>();
    let shifted = (hi.cast::<u16>() << Simd::splat(8)) | lo.cast::<u16>();
    let sextets = shifted >> repeated(&[2, 4, 6, 8]);
    let sextets = sextets.cast::<u8>();

    // Step 3: make ascii from sextets
    // Note: it would be nice to find a hash function like that one in decode() -
    // this would reduce needed amount of operations and reduce register pressure.

    let uppers = sextets.simd_lt(Simd::splat(SEXTET_LOWERS_OFFSET));
    let lowers = !uppers & sextets.simd_lt(Simd::splat(SEXTET_DIGITS_OFFSET));
    let digits = !uppers & !lowers & sextets.simd_lt(Simd::splat(SEXTET_PLUS_OFFSET));
    let pluses = sextets.simd_eq(Simd::splat(SEXTET_PLUS_OFFSET));
    let slashes = sextets.simd_eq(Simd::splat(SEXTET_SLASH_OFFSET));

    let asciis = sextets
        + masked_splat(uppers, (b'A' as i8 - SEXTET_UPPERS_OFFSET as i8) as u8)
        + masked_splat(lowers, (b'a' as i8 - SEXTET_LOWERS_OFFSET as i8) as u8)
        + masked_splat(digits, (b'0' as i8 - SEXTET_DIGITS_OFFSET as i8) as u8)
        + masked_splat(pluses, b'+')
        + masked_splat(slashes, b'+');

    asciis
}

pub fn encode<const N: usize>(data: &[u8], out: &mut Vec<u8>)
where
    LaneCount<N>: SupportedLaneCount
{
    let final_size = encoded_len(data.len());
    out.reserve(final_size + 2 /* padding */);

    let chunk_size = N - N / 4;
    let mut chunks = data.chunks_exact(chunk_size);

    // main loop
    for chunk in &mut chunks {
        let mut bytes = [0u8; N];
        bytes[..chunk.len()].copy_from_slice(chunk);

        let asciis = encode_hot(Simd::from(bytes));
        out.extend_from_slice(asciis.as_array());
    }

    // remainder
    let rem = chunks.remainder();
    if !rem.is_empty() {
        let mut bytes = [0u8; N];
        bytes[..rem.len()].copy_from_slice(rem);
        
        let asciis = encode_hot(Simd::from(bytes));
        let len = encoded_len(rem.len());
        out.extend_from_slice(&asciis.as_array()[..len]);
    }

    // padding
    common::pad_with_trailing_eq(data.len(), out);
}

// helpers

fn repeated<T, const N: usize>(input: &[T]) -> Simd<T, N>
where
    T: SimdElement,
    LaneCount<N>: SupportedLaneCount,
{
    let mut output = [input[0]; N];
    for i in 0..N {
        output[i] = input[i % input.len()];
    }
    Simd::from(output)
}

/// Resizes a vector by either truncation or padding with zeroes.
fn resize<T, const N: usize, const M: usize>(v: Simd<T, N>) -> Simd<T, M>
where
    T: SimdElement + Default,
    LaneCount<N>: SupportedLaneCount,
    LaneCount<M>: SupportedLaneCount,
{
    let len = usize::min(N, M);
    let mut out = Simd::default();
    out.as_mut_array()[..len].copy_from_slice(&v.as_array()[..len]);
    out
}

/// Creates a new `M`-byte vector by treating each element of `indices` as an
/// index into `table`, which is treated as being padded to infinite length
/// with zero.
fn shuffle<const N: usize, const M: usize>(
    table: Simd<u8, N>,
    indices: Simd<u8, M>,
) -> Simd<u8, M>
where
    LaneCount<N>: SupportedLaneCount,
    LaneCount<M>: SupportedLaneCount,
{
    if N < M {
        Simd::swizzle_dyn(resize(table), indices)
    } else {
        resize(Simd::swizzle_dyn(table, resize(indices)))
    }
}

fn validate<const N: usize>(ascii: Simd<u8, N>) -> bool
where
    LaneCount<N>: SupportedLaneCount,
{
    const LO_LUT: Simd<u8, 16> = Simd::from_array([
        0b10101, 0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b10001,
        0b10001, 0b10001, 0b10011, 0b11010, 0b11011, 0b11011, 0b11011, 0b11010,
    ]);

    const HI_LUT: Simd<u8, 16> = Simd::from_array([
        0b10000, 0b10000, 0b00001, 0b00010, 0b00100, 0b01000, 0b00100, 0b01000,
        0b10000, 0b10000, 0b10000, 0b10000, 0b10000, 0b10000, 0b10000, 0b10000,
    ]);

    let lo = shuffle::<16, N>(LO_LUT, ascii & Simd::splat(0x0f));
    let hi = shuffle::<16, N>(HI_LUT, ascii >> Simd::splat(4));
    let valid = (lo & hi).reduce_or() == 0;

    valid
}

fn masked_splat<const N: usize>(mask: Mask<i8, N>, value: u8) -> Simd<u8, N>
where
    LaneCount<N>: SupportedLaneCount
{
    mask.select(Simd::splat(value), Simd::splat(0))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hello_world() {
        let hello = b"Hello, world!";
        let hello_base64 = b"SGVsbG8sIHdvcmxkIQ==";

        encode_decode_with_expected::<8>(hello, Some(hello_base64));
        encode_decode_with_expected::<16>(hello, Some(hello_base64));
        encode_decode_with_expected::<32>(hello, Some(hello_base64));
    }

    #[test]
    fn test_invalid() {
        let mut result = Vec::new();
        let res = decode::<8>(b"invalid_base64_$", &mut result);
        assert!(res.is_err());
    }

    #[test]
    fn test_encode_decode() {
        let message = b"123456790";
        encode_decode_iters::<8>(message);
        encode_decode_iters::<16>(message);
        encode_decode_iters::<32>(message);
    }
    
    fn encode_decode_with_expected<const N: usize>(
        message: &[u8],
        encoded_expected: Option<&[u8]>
    )
    where
        LaneCount<N>: SupportedLaneCount
    {
        let mut buffer = Vec::new();

        buffer.clear();
        encode::<N>(message, &mut buffer);
        let encoded = buffer.clone();
        if let Some(encoded_expected) = encoded_expected {
            assert_eq!(encoded_expected, encoded);
        }

        buffer.clear();
        decode::<N>(&encoded, &mut buffer).unwrap();

        assert_eq!(message, buffer);
    }

    fn encode_decode<const N: usize>(message: &[u8])
    where
        LaneCount<N>: SupportedLaneCount
    {
        encode_decode_with_expected::<N>(message, None);
    }

    fn encode_decode_iters<const N: usize>(message: &[u8])
    where
        LaneCount<N>: SupportedLaneCount
    {
        for i in 0..message.len() {
            encode_decode::<N>(&message[..i]);
        }
    }
}
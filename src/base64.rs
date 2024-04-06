use crate::common;
use crate::common::{
    SEXTET_UPPERS_OFFSET,
    SEXTET_LOWERS_OFFSET,
    SEXTET_DIGITS_OFFSET,
    SEXTET_PLUS_OFFSET,
    SEXTET_SLASH_OFFSET,
};


fn decoded_len(encoded_len: usize) -> usize {
    let padding = match encoded_len % 4 {
        1 | 2 => 1,
        3 => 2,
        _ /* 0 */ => 0,
    };

    encoded_len / 4 * 3 + padding
}

pub fn decode(data: &[u8], out: &mut Vec<u8>) -> Result<(), String> {
    let data = common::remove_trailing_eq(data);

    let final_size = decoded_len(data.len());
    out.reserve(final_size);

    for chunk in data.chunks(4) {
        let mut bytes: u32 = 0;

        for byte in chunk {
            let sextet = match byte {
                b'A'..=b'Z' => byte - b'A' + SEXTET_UPPERS_OFFSET,
                b'a'..=b'z' => byte - b'a' + SEXTET_LOWERS_OFFSET,
                b'0'..=b'9' => byte - b'0' + SEXTET_DIGITS_OFFSET,
                b'+' => SEXTET_PLUS_OFFSET,
                b'/' => SEXTET_SLASH_OFFSET,
                _ => return Err(format!("{data:?} is not base64 because of char {byte}")),
            };

            bytes <<= 6;
            bytes |= sextet as u32;
        }

        // shift bits for the case chunk.len() < 4 and
        // plus one byte, so data will be at 3 high bytes
        bytes <<= (4 - chunk.len()) * 6 + 8;

        let decoded = decoded_len(chunk.len());
        out.extend_from_slice(&bytes.to_be_bytes()[..decoded]);
    }

    Ok(())
}

fn encoded_len(decoded_len: usize) -> usize {
    let padding = match decoded_len % 3 {
        1 => 2,
        2 => 3,
        _ /* 0 */ => 0,
    };
    decoded_len / 3 * 4 + padding
}

fn sextet_to_ascii(sextet: u8) -> u8 {
    let alphabet = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    alphabet[sextet as usize]
}

pub fn encode(data: &[u8], out: &mut Vec<u8>) {
    let final_size = encoded_len(data.len());
    out.reserve(final_size + 2 /* padding */);

    let mut chunks = data.chunks_exact(3);
    
    // main loop
    for chunk in &mut chunks {
        let bytes: u32 = ((chunk[0] as u32) << 16) | ((chunk[1] as u32) << 8) | (chunk[2] as u32);

        out.push(sextet_to_ascii((bytes >> 18) as u8 & 0b111111));
        out.push(sextet_to_ascii((bytes >> 12) as u8 & 0b111111));
        out.push(sextet_to_ascii((bytes >> 6) as u8 & 0b111111));
        out.push(sextet_to_ascii((bytes >> 0) as u8 & 0b111111));
    }

    // remainder + padding
    let rem = chunks.remainder();
    match rem.len() {
        1 => {
            let bytes = (rem[0] as u32) << 16;
            out.push(sextet_to_ascii((bytes >> 18) as u8 & 0b111111));
            out.push(sextet_to_ascii((bytes >> 12) as u8 & 0b111111));
            out.push(b'=');
            out.push(b'=');
        },
        2 => {
            let bytes = ((rem[0] as u32) << 16) | ((rem[1] as u32) << 8);
            out.push(sextet_to_ascii((bytes >> 18) as u8 & 0b111111));
            out.push(sextet_to_ascii((bytes >> 12) as u8 & 0b111111));
            out.push(sextet_to_ascii((bytes >> 6) as u8 & 0b111111));
            out.push(b'=');
        },
        _ => {},
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hello_world() {
        let hello = b"Hello, world!";
        let hello_base64 = b"SGVsbG8sIHdvcmxkIQ==";

        let mut result = Vec::new();

        encode(hello, &mut result);
        assert_eq!(result, hello_base64);

        result.clear();

        decode(hello_base64, &mut result).unwrap();
        assert_eq!(result, hello);
    }

    #[test]
    fn test_encode_decode() {
        let message = b"0123456789";
        let mut buffer = Vec::new();

        for i in 0..message.len() {
            let message = &message[..i];
            
            buffer.clear();
            encode(message, &mut buffer);
            let encoded = buffer.clone();
            buffer.clear();
            decode(&encoded, &mut buffer).unwrap();

            assert_eq!(message, buffer);
        }
    }
}
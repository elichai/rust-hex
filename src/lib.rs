// Copyright (c) 2013-2014 The Rust Project Developers.
// Copyright (c) 2015-2020 The rust-hex Developers.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.
//! Encoding and decoding hex strings.
//!
//! For most cases, you can simply use the [`decode`], [`encode`] and
//! [`encode_upper`] functions. If you need a bit more control, use the traits
//! [`ToHex`] and [`FromHex`] instead.
//!
//! # Example
//!
//! ```
//! # #[cfg(not(feature = "alloc"))]
//! # let mut output = [0; 0x18];
//! #
//! # #[cfg(not(feature = "alloc"))]
//! # let hex_string = hex::encode_to_slice(b"Hello world!", &mut output).unwrap();
//! #
//! # #[cfg(feature = "alloc")]
//! let hex_string = hex::encode("Hello world!");
//!
//! println!("{}", hex_string); // Prints "48656c6c6f20776f726c6421"
//!
//! # assert_eq!(hex_string, "48656c6c6f20776f726c6421");
//! ```

#![doc(html_root_url = "https://docs.rs/hex/0.5")]
#![cfg_attr(not(feature = "std"), no_std)]
#![cfg_attr(docsrs, feature(doc_cfg))]
#![allow(clippy::unreadable_literal)]

#[cfg(feature = "alloc")]
extern crate alloc;
#[cfg(feature = "alloc")]
use alloc::{string::String, vec, vec::Vec};

use core::{iter, u8};

mod error;
pub use crate::error::FromHexError;

#[cfg(feature = "serde")]
#[cfg_attr(docsrs, doc(cfg(feature = "serde")))]
pub mod serde;
#[cfg(feature = "serde")]
pub use crate::serde::deserialize;
#[cfg(all(feature = "alloc", feature = "serde"))]
pub use crate::serde::{serialize, serialize_upper};

/// Encoding values as hex string.
///
/// This trait is implemented for all `T` which implement `AsRef<[u8]>`. This
/// includes `String`, `str`, `Vec<u8>` and `[u8]`.
///
/// # Example
///
/// ```
/// use hex::ToHex;
///
/// println!("{}", "Hello world!".encode_hex::<String>());
/// # assert_eq!("Hello world!".encode_hex::<String>(), "48656c6c6f20776f726c6421".to_string());
/// ```
///
/// *Note*: instead of using this trait, you might want to use [`encode()`].
pub trait ToHex {
    /// Encode the hex strict representing `self` into the result. Lower case
    /// letters are used (e.g. `f9b4ca`)
    fn encode_hex<T: iter::FromIterator<char>>(&self) -> T;

    /// Encode the hex strict representing `self` into the result. Upper case
    /// letters are used (e.g. `F9B4CA`)
    fn encode_hex_upper<T: iter::FromIterator<char>>(&self) -> T;
}

const HEX_CHARS_LOWER: &[u8; 16] = b"0123456789abcdef";
const HEX_CHARS_UPPER: &[u8; 16] = b"0123456789ABCDEF";

struct BytesToHexChars<'a> {
    inner: ::core::slice::Iter<'a, u8>,
    table: &'static [u8; 16],
    next: Option<char>,
}

impl<'a> BytesToHexChars<'a> {
    #[inline(always)]
    fn new(inner: &'a [u8], table: &'static [u8; 16]) -> BytesToHexChars<'a> {
        BytesToHexChars {
            inner: inner.iter(),
            table,
            next: None,
        }
    }
}

impl<'a> Iterator for BytesToHexChars<'a> {
    type Item = char;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        match self.next.take() {
            Some(current) => Some(current),
            None => self.inner.next().map(|byte| {
                let current = self.table[(byte >> 4) as usize] as char;
                self.next = Some(self.table[(byte & 0x0F) as usize] as char);
                current
            }),
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let length = self.len();
        (length, Some(length))
    }
}

impl<'a> iter::ExactSizeIterator for BytesToHexChars<'a> {
    #[inline(always)]
    fn len(&self) -> usize {
        let mut length = self.inner.len() * 2;
        if self.next.is_some() {
            length += 1;
        }
        length
    }
}

fn encode_to_iter<T: iter::FromIterator<char>>(table: &'static [u8; 16], source: &[u8]) -> T {
    BytesToHexChars::new(source, table).collect()
}

impl<T: AsRef<[u8]>> ToHex for T {
    fn encode_hex<U: iter::FromIterator<char>>(&self) -> U {
        encode_to_iter(HEX_CHARS_LOWER, self.as_ref())
    }

    fn encode_hex_upper<U: iter::FromIterator<char>>(&self) -> U {
        encode_to_iter(HEX_CHARS_UPPER, self.as_ref())
    }
}

/// Types that can be decoded from a hex string.
///
/// This trait is implemented for `Vec<u8>` and small `u8`-arrays.
///
/// # Example
///
/// ```
/// use core::str;
/// use hex::FromHex;
///
/// let buffer = <[u8; 12]>::from_hex("48656c6c6f20776f726c6421")?;
/// let string = str::from_utf8(&buffer).expect("invalid buffer length");
///
/// println!("{}", string); // prints "Hello world!"
/// # assert_eq!("Hello world!", string);
/// # Ok::<(), hex::FromHexError>(())
/// ```
pub trait FromHex: Sized {
    type Error;

    /// Creates an instance of type `Self` from the given hex string, or fails
    /// with a custom error type.
    ///
    /// Both, upper and lower case characters are valid and can even be
    /// mixed (e.g. `f9b4ca`, `F9B4CA` and `f9B4Ca` are all valid strings).
    fn from_hex<T: AsRef<[u8]>>(hex: T) -> Result<Self, Self::Error>;
}

const __: u8 = u8::MAX;

// Lookup table for ascii to hex decoding.
#[rustfmt::skip]
static DECODE_TABLE: [u8; 256] = [
    //   1   2   3   4   5   6   7   8   9   a   b   c   d   e   f
    __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // 0
    __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // 1
    __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // 2
     0,  1,  2,  3,  4,  5,  6,  7,  8,  9, __, __, __, __, __, __, // 3
    __, 10, 11, 12, 13, 14, 15, __, __, __, __, __, __, __, __, __, // 4
    __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // 5
    __, 10, 11, 12, 13, 14, 15, __, __, __, __, __, __, __, __, __, // 6
    __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // 7
    __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // 8
    __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // 9
    __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // a
    __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // b
    __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // c
    __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // d
    __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // e
    __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // f
];

#[inline]
fn val(bytes: &[u8], idx: usize) -> Result<u8, FromHexError> {
    let upper = DECODE_TABLE[bytes[0] as usize];
    let lower = DECODE_TABLE[bytes[1] as usize];
    if upper == u8::MAX {
        return Err(FromHexError::InvalidHexCharacter {
            c: bytes[0] as char,
            index: idx,
        });
    }
    if lower == u8::MAX {
        return Err(FromHexError::InvalidHexCharacter {
            c: bytes[1] as char,
            index: idx + 1,
        });
    }
    Ok((upper << 4) | lower)
}

#[cfg(feature = "alloc")]
impl FromHex for Vec<u8> {
    type Error = FromHexError;

    fn from_hex<T: AsRef<[u8]>>(hex: T) -> Result<Self, Self::Error> {
        let hex = hex.as_ref();
        if hex.len() % 2 != 0 {
            return Err(FromHexError::OddLength);
        }

        let mut out = vec![0; hex.len() / 2];
        decode_to_slice(hex, &mut out)?;
        Ok(out)
    }
}

impl<const N: usize> FromHex for [u8; N] {
    type Error = FromHexError;

    fn from_hex<T: AsRef<[u8]>>(hex: T) -> Result<Self, Self::Error> {
        let mut out = [0_u8; N];
        decode_to_slice(hex, &mut out as &mut [u8])?;

        Ok(out)
    }
}

/// Encodes `data` as hex string using lowercase characters.
///
/// Lowercase characters are used (e.g. `f9b4ca`). The resulting string's
/// length is always even, each byte in `data` is always encoded using two hex
/// digits. Thus, the resulting string contains exactly twice as many bytes as
/// the input data.
///
/// # Example
///
/// ```
/// assert_eq!(hex::encode("Hello world!"), "48656c6c6f20776f726c6421");
/// assert_eq!(hex::encode(vec![1, 2, 3, 15, 16]), "0102030f10");
/// ```
#[must_use]
#[cfg(feature = "alloc")]
pub fn encode<T: AsRef<[u8]>>(data: T) -> String {
    let data = data.as_ref();
    let mut out = vec![0; data.len() * 2];
    encode_to_slice(data, &mut out).unwrap();
    String::from_utf8(out).unwrap()
}

/// Encodes `data` as hex string using lowercase characters, appending to target string.
///
/// This is otherwise the same as [`encode`].  One reason to use this function
/// is that if you are performing multiple encodings on distinct data in
/// a loop, this will allow reusing the allocation of a string.
///
/// Alternatively, this is also more efficient to use when you have an
/// existing string and just want to append to it.
///
/// # Example
///
/// ```
/// let mut s = "The hex encoding is: ".to_string();
/// hex::encode_to("Hello world!", &mut s);
/// assert_eq!(s, "The hex encoding is: 48656c6c6f20776f726c6421");
/// ```
#[cfg(feature = "alloc")]
pub fn encode_to<T: AsRef<[u8]>>(data: T, s: &mut String) {
    s.extend(BytesToHexChars::new(data.as_ref(), HEX_CHARS_LOWER))
}

/// Encodes `data` as hex string using uppercase characters.
///
/// Apart from the characters' casing, this works exactly like `encode()`.
///
/// # Example
///
/// ```
/// assert_eq!(hex::encode_upper("Hello world!"), "48656C6C6F20776F726C6421");
/// assert_eq!(hex::encode_upper(vec![1, 2, 3, 15, 16]), "0102030F10");
/// ```
#[must_use]
#[cfg(feature = "alloc")]
pub fn encode_upper<T: AsRef<[u8]>>(data: T) -> String {
    let data = data.as_ref();
    let mut out = vec![0; data.len() * 2];
    encode_to_slice_upper(data, &mut out).unwrap();
    String::from_utf8(out).unwrap()
}

/// Encodes `data` as hex string using uppercase characters, appending to target string.
///
/// This is the same as [`encode_to`], but uses uppercase characters.
///
/// # Example
///
/// ```
/// let mut s = "The hex encoding is: ".to_string();
/// hex::encode_upper_to("Hello world!", &mut s);
/// assert_eq!(s, "The hex encoding is: 48656C6C6F20776F726C6421");
/// ```
#[cfg(feature = "alloc")]
pub fn encode_upper_to<T: AsRef<[u8]>>(data: T, s: &mut String) {
    s.extend(BytesToHexChars::new(data.as_ref(), HEX_CHARS_UPPER))
}

/// Decodes a hex string into raw bytes.
///
/// Both, upper and lower case characters are valid in the input string and can
/// even be mixed (e.g. `f9b4ca`, `F9B4CA` and `f9B4Ca` are all valid strings).
///
/// # Example
///
/// ```
/// assert_eq!(
///     hex::decode("48656c6c6f20776f726c6421"),
///     Ok("Hello world!".to_owned().into_bytes())
/// );
///
/// assert_eq!(hex::decode("123"), Err(hex::FromHexError::OddLength));
/// assert!(hex::decode("foo").is_err());
/// ```
#[cfg(feature = "alloc")]
pub fn decode<T: AsRef<[u8]>>(data: T) -> Result<Vec<u8>, FromHexError> {
    FromHex::from_hex(data)
}

/// Decode a hex string into a mutable bytes slice.
///
/// Both, upper and lower case characters are valid in the input string and can
/// even be mixed (e.g. `f9b4ca`, `F9B4CA` and `f9B4Ca` are all valid strings).
///
/// # Example
///
/// ```
/// let mut bytes = [0u8; 4];
/// assert_eq!(hex::decode_to_slice("6b697769", &mut bytes as &mut [u8]), Ok(()));
/// assert_eq!(&bytes, b"kiwi");
/// ```
#[inline]
pub fn decode_to_slice<T: AsRef<[u8]>>(data: T, out: &mut [u8]) -> Result<(), FromHexError> {
    let data = data.as_ref();

    if data.len() % 2 != 0 {
        return Err(FromHexError::OddLength);
    }
    if data.len() / 2 != out.len() {
        return Err(FromHexError::InvalidStringLength);
    }

    for (i, (data, byte)) in data.chunks_exact(2).zip(out).enumerate() {
        *byte = val(data, 2 * i)?;
    }

    Ok(())
}

// the inverse of `val`.
#[inline(always)]
#[must_use]
fn byte2hex(byte: u8, table: &[u8; 16]) -> (u8, u8) {
    let high = table[((byte & 0xf0) >> 4) as usize];
    let low = table[(byte & 0x0f) as usize];

    (high, low)
}

#[inline(always)]
fn encode_to_slice_inner<'a>(
    input: &[u8],
    output: &'a mut [u8],
    table: &[u8; 16],
) -> Result<(), FromHexError> {
    if input.len() * 2 != output.len() {
        return Err(FromHexError::InvalidStringLength);
    }

    for (byte, output) in input.iter().zip(output.chunks_exact_mut(2)) {
        let (high, low) = byte2hex(*byte, table);
        output[0] = high;
        output[1] = low;
    }

    Ok(())
}

/// Encodes some bytes into a mutable slice of bytes using lowercase characters.
///
/// The output buffer, has to be able to hold exactly `input.len() * 2` bytes,
/// otherwise this function will return an error.
///
/// # Example
///
/// ```
/// # use hex::FromHexError;
/// # fn main() -> Result<(), FromHexError> {
/// let mut bytes = [0u8; 4 * 2];
///
/// let hex_str = hex::encode_to_slice(b"kiwi", &mut bytes)?;
/// assert_eq!(hex_str, "6b697769");
/// assert_eq!(&bytes, b"6b697769");
/// # Ok(())
/// # }
/// ```
///
/// If the buffer is too large, an error is returned:
///
/// ```
/// use hex::FromHexError;
/// # fn main() -> Result<(), FromHexError> {
/// let mut bytes = [0_u8; 5 * 2];
///
/// assert_eq!(hex::encode_to_slice(b"kiwi", &mut bytes), Err(FromHexError::InvalidStringLength));
///
/// // you can do this instead:
/// let hex_str = hex::encode_to_slice(b"kiwi", &mut bytes[..4 * 2])?;
/// assert_eq!(hex_str, "6b697769");
/// assert_eq!(&bytes, b"6b697769\0\0");
/// # Ok(())
/// # }
/// ```
pub fn encode_to_slice<T: AsRef<[u8]>>(input: T, output: &mut [u8]) -> Result<&mut str, FromHexError> {
    encode_to_slice_inner(input.as_ref(), output, HEX_CHARS_LOWER)?;
    if cfg!(debug_assertions) {
        Ok(core::str::from_utf8_mut(output).unwrap())
    } else {
        // Saftey: We just wrote valid utf8 hex string into the output
        Ok(unsafe { core::str::from_utf8_unchecked_mut(output) })
    }
}

/// Encodes some bytes into a mutable slice of bytes using uppercase characters.
///
/// The output buffer, has to be able to hold exactly `input.len() * 2` bytes,
/// otherwise this function will return an error.
///
/// # Example
///
/// ```
/// # use hex::FromHexError;
/// # fn main() -> Result<(), FromHexError> {
/// let mut bytes = [0u8; 4 * 2];
///
/// hex::encode_to_slice_upper(b"kiwi", &mut bytes)?;
/// assert_eq!(&bytes, b"6B697769");
/// # Ok(())
/// # }
/// ```
pub fn encode_to_slice_upper<T: AsRef<[u8]>>(
    input: T,
    output: &mut [u8],
) -> Result<&mut str, FromHexError> {
    encode_to_slice_inner(input.as_ref(), output, HEX_CHARS_UPPER)?;
    if cfg!(debug_assertions) {
        Ok(core::str::from_utf8_mut(output).unwrap())
    } else {
        // Saftey: We just wrote valid utf8 hex string into the output
        Ok(unsafe { core::str::from_utf8_unchecked_mut(output) })
    }
}

#[cfg(test)]
mod test {
    use super::*;
    #[cfg(feature = "alloc")]
    use alloc::string::ToString;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_encode_to_slice() {
        let mut output_1 = [0; 4 * 2];
        let encoded = encode_to_slice(b"kiwi", &mut output_1).unwrap();
        assert_eq!(encoded, "6b697769");
        assert_eq!(&output_1, b"6b697769");
        encode_to_slice_upper(b"kiwi", &mut output_1).unwrap();
        assert_eq!(&output_1, b"6B697769");

        let mut output_2 = [0; 5 * 2];
        let encoded = encode_to_slice(b"kiwis", &mut output_2).unwrap();
        assert_eq!(encoded, "6b69776973");
        assert_eq!(&output_2, b"6b69776973");
        encode_to_slice_upper(b"kiwis", &mut output_2).unwrap();
        assert_eq!(&output_2, b"6B69776973");

        let mut output_3 = [0; 100];

        assert_eq!(
            encode_to_slice(b"kiwis", &mut output_3),
            Err(FromHexError::InvalidStringLength)
        );
        assert_eq!(
            encode_to_slice_upper(b"kiwis", &mut output_3),
            Err(FromHexError::InvalidStringLength)
        );
    }

    #[test]
    fn test_decode_to_slice() {
        let mut output_1 = [0; 4];
        decode_to_slice(b"6b697769", &mut output_1).unwrap();
        assert_eq!(&output_1, b"kiwi");

        let mut output_2 = [0; 5];
        decode_to_slice(b"6b69776973", &mut output_2).unwrap();
        assert_eq!(&output_2, b"kiwis");

        let mut output_3 = [0; 4];

        assert_eq!(
            decode_to_slice(b"6", &mut output_3),
            Err(FromHexError::OddLength)
        );
    }

    #[test]
    #[cfg(feature = "alloc")]
    fn test_encode() {
        assert_eq!(encode("foobar"), "666f6f626172");
    }

    #[test]
    #[cfg(feature = "alloc")]
    fn test_decode() {
        assert_eq!(
            decode("666f6f626172"),
            Ok(String::from("foobar").into_bytes())
        );
    }

    #[test]
    #[cfg(feature = "alloc")]
    pub fn test_from_hex_okay_str() {
        assert_eq!(Vec::from_hex("666f6f626172").unwrap(), b"foobar");
        assert_eq!(Vec::from_hex("666F6F626172").unwrap(), b"foobar");
    }

    #[test]
    #[cfg(feature = "alloc")]
    pub fn test_from_hex_okay_bytes() {
        assert_eq!(Vec::from_hex(b"666f6f626172").unwrap(), b"foobar");
        assert_eq!(Vec::from_hex(b"666F6F626172").unwrap(), b"foobar");
    }

    #[test]
    #[cfg(feature = "alloc")]
    pub fn test_invalid_length() {
        assert_eq!(Vec::from_hex("1").unwrap_err(), FromHexError::OddLength);
        assert_eq!(
            Vec::from_hex("666f6f6261721").unwrap_err(),
            FromHexError::OddLength
        );
    }

    #[test]
    #[cfg(feature = "alloc")]
    pub fn test_invalid_char() {
        assert_eq!(
            Vec::from_hex("66ag").unwrap_err(),
            FromHexError::InvalidHexCharacter { c: 'g', index: 3 }
        );
    }

    #[test]
    #[cfg(feature = "alloc")]
    pub fn test_empty() {
        assert_eq!(Vec::from_hex("").unwrap(), b"");
    }

    #[test]
    #[cfg(feature = "alloc")]
    pub fn test_from_hex_whitespace() {
        assert_eq!(
            Vec::from_hex("666f 6f62617").unwrap_err(),
            FromHexError::InvalidHexCharacter { c: ' ', index: 4 }
        );
    }

    #[test]
    pub fn test_from_hex_array() {
        assert_eq!(
            <[u8; 6] as FromHex>::from_hex("666f6f626172"),
            Ok([0x66, 0x6f, 0x6f, 0x62, 0x61, 0x72])
        );

        assert_eq!(
            <[u8; 5] as FromHex>::from_hex("666f6f626172"),
            Err(FromHexError::InvalidStringLength)
        );
    }

    #[test]
    #[cfg(feature = "alloc")]
    fn test_to_hex() {
        assert_eq!(
            [0x66, 0x6f, 0x6f, 0x62, 0x61, 0x72].encode_hex::<String>(),
            "666f6f626172".to_string(),
        );

        assert_eq!(
            [0x66, 0x6f, 0x6f, 0x62, 0x61, 0x72].encode_hex_upper::<String>(),
            "666F6F626172".to_string(),
        );
    }
}

// SPDX-License-Identifier: MIT OR Apache-2.0

//! Intel HEX, the format avr-gcc's toolchain and the Arduino IDE hand a
//! compiled program over in (ADR 0017).
//!
//! avr8js has no loader of its own — its embedders bring one — so this one
//! is ours. A file is lines of `:LLAAAATT…CC`: a count of data bytes, a
//! 16-bit address, a record type, the data, and a checksum that makes the
//! line's bytes sum to zero. A program for the ATmega328P uses two types,
//! data (00) and end of file (01); the extended-address types (02, 04) that
//! a part with more than 64 KB of flash needs are read too, since they cost
//! nothing, and the start-address types (03, 05), which mean nothing to an
//! AVR, are skipped. Anything else, and any line that does not check out, is
//! an error naming the line: a program that loaded half-way would run, and
//! do something nobody wrote.

use std::fmt;

/// Why a HEX file was refused. Lines count from 1.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HexError {
    /// A line that does not start with `:`.
    MissingColon { line: usize },
    /// A character that is not a hexadecimal digit, or an odd number of them.
    BadDigits { line: usize },
    /// A line whose length disagrees with its byte count.
    BadLength { line: usize },
    /// A line whose bytes do not sum to zero.
    BadChecksum {
        line: usize,
        expected: u8,
        found: u8,
    },
    /// A record type this loader does not know.
    UnknownRecord { line: usize, kind: u8 },
    /// Data past the end of program memory.
    OutOfRange { line: usize, address: u32 },
    /// The file ended without an end-of-file record.
    MissingEnd,
}

impl fmt::Display for HexError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            HexError::MissingColon { line } => write!(f, "line {line}: a record starts with ':'"),
            HexError::BadDigits { line } => write!(f, "line {line}: not hexadecimal"),
            HexError::BadLength { line } => {
                write!(f, "line {line}: length does not match its byte count")
            }
            HexError::BadChecksum {
                line,
                expected,
                found,
            } => write!(
                f,
                "line {line}: checksum is {found:02X}, the bytes say {expected:02X}"
            ),
            HexError::UnknownRecord { line, kind } => {
                write!(f, "line {line}: unknown record type {kind:02X}")
            }
            HexError::OutOfRange { line, address } => {
                write!(
                    f,
                    "line {line}: address {address:#x} is past the end of flash"
                )
            }
            HexError::MissingEnd => write!(f, "no end-of-file record"),
        }
    }
}

impl std::error::Error for HexError {}

/// Loads `source` into `prog_mem`, little-endian within each word as the AVR
/// stores it. What the file does not mention is left as it was. Returns how
/// many bytes were loaded.
pub fn load_hex(source: &str, prog_mem: &mut [u16]) -> Result<usize, HexError> {
    let capacity = prog_mem.len() as u32 * 2;
    let mut base: u32 = 0;
    let mut loaded = 0;

    for (index, text) in source.lines().enumerate() {
        let line = index + 1;
        let text = text.trim();
        if text.is_empty() {
            continue;
        }
        let digits = text
            .strip_prefix(':')
            .ok_or(HexError::MissingColon { line })?;
        let bytes = decode_digits(digits).ok_or(HexError::BadDigits { line })?;
        if bytes.len() < 5 || bytes.len() != bytes[0] as usize + 5 {
            return Err(HexError::BadLength { line });
        }

        let (body, checksum) = bytes.split_at(bytes.len() - 1);
        let expected = body
            .iter()
            .fold(0u8, |sum, byte| sum.wrapping_add(*byte))
            .wrapping_neg();
        if expected != checksum[0] {
            return Err(HexError::BadChecksum {
                line,
                expected,
                found: checksum[0],
            });
        }

        let address = u16::from_be_bytes([body[1], body[2]]) as u32;
        let data = &body[4..];
        match body[3] {
            0x00 => {
                for (offset, &byte) in data.iter().enumerate() {
                    let at = base + address + offset as u32;
                    if at >= capacity {
                        return Err(HexError::OutOfRange { line, address: at });
                    }
                    let word = &mut prog_mem[(at >> 1) as usize];
                    let shift = (at & 1) * 8;
                    *word = (*word & !(0xff << shift)) | ((byte as u16) << shift);
                }
                loaded += data.len();
            }
            0x01 => return Ok(loaded),
            // Extended segment address: the base is sixteen times it.
            0x02 if data.len() == 2 => base = (u16::from_be_bytes([data[0], data[1]]) as u32) << 4,
            // Extended linear address: the upper sixteen bits.
            0x04 if data.len() == 2 => base = (u16::from_be_bytes([data[0], data[1]]) as u32) << 16,
            0x02 | 0x04 => return Err(HexError::BadLength { line }),
            // Start addresses: an AVR starts at its reset vector regardless.
            0x03 | 0x05 => {}
            kind => return Err(HexError::UnknownRecord { line, kind }),
        }
    }
    Err(HexError::MissingEnd)
}

fn decode_digits(digits: &str) -> Option<Vec<u8>> {
    if !digits.len().is_multiple_of(2) {
        return None;
    }
    (0..digits.len())
        .step_by(2)
        .map(|at| u8::from_str_radix(digits.get(at..at + 2)?, 16).ok())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// avr-objcopy's output for `ldi r16, 0xff; out 0x04, r16; rjmp .-2`,
    /// laid out as it lays out a real program.
    const BLINKISH: &str = "\
:060000000FEF04B9FFCF71
:00000001FF
";

    #[test]
    fn loads_data_records_little_endian_within_each_word() {
        let mut flash = vec![0; 16];
        assert_eq!(load_hex(BLINKISH, &mut flash), Ok(6));
        assert_eq!(&flash[..4], &[0xef0f, 0xb904, 0xcfff, 0]);
    }

    #[test]
    fn rejects_a_bad_checksum_and_names_the_line() {
        let mut flash = vec![0; 16];
        let bad = BLINKISH.replace("CF71", "CF72");
        assert_eq!(
            load_hex(&bad, &mut flash),
            Err(HexError::BadChecksum {
                line: 1,
                expected: 0x71,
                found: 0x72,
            })
        );
    }

    #[test]
    fn rejects_malformed_lines() {
        let mut flash = vec![0; 16];
        assert_eq!(
            load_hex("060000000FEF04B9FFCF71\n", &mut flash),
            Err(HexError::MissingColon { line: 1 })
        );
        assert_eq!(
            load_hex(":0600000G0FEF04B9FFCF71\n", &mut flash),
            Err(HexError::BadDigits { line: 1 })
        );
        assert_eq!(
            load_hex(":070000000FEF04B9FFCF4B\n", &mut flash),
            Err(HexError::BadLength { line: 1 })
        );
        assert_eq!(
            load_hex(":00000006FA\n", &mut flash),
            Err(HexError::UnknownRecord { line: 1, kind: 6 })
        );
    }

    #[test]
    fn needs_an_end_of_file_record() {
        let mut flash = vec![0; 16];
        assert_eq!(
            load_hex(":060000000FEF04B9FFCF71\n", &mut flash),
            Err(HexError::MissingEnd)
        );
    }

    #[test]
    fn refuses_data_past_the_end_of_flash() {
        let mut flash = vec![0; 2];
        assert_eq!(
            load_hex(BLINKISH, &mut flash),
            Err(HexError::OutOfRange {
                line: 1,
                address: 4
            })
        );
    }

    #[test]
    fn refuses_the_highest_extended_address_before_it_can_overflow() {
        // Base 0xFFFF0000, then two bytes at 0xFFFF: the second would be at
        // 2^32, but the first is already refused, so the sum never wraps.
        let mut flash = vec![0; 16];
        let source = "\
:02000004FFFFFC
:02FFFF00AABB9B
:00000001FF
";
        assert_eq!(
            load_hex(source, &mut flash),
            Err(HexError::OutOfRange {
                line: 2,
                address: 0xffff_ffff
            })
        );
    }

    #[test]
    fn follows_extended_addresses_and_skips_start_addresses() {
        let mut flash = vec![0; 0x10010];
        let source = "\
:020000040001F9
:02000000AABB99
:0400000300000000F9
:00000001FF
";
        assert_eq!(load_hex(source, &mut flash), Ok(2));
        assert_eq!(flash[0x8000], 0xbbaa);
    }

    #[test]
    fn ignores_blank_lines_and_crlf() {
        let mut flash = vec![0; 16];
        let source = BLINKISH.replace('\n', "\r\n") + "\r\n";
        assert_eq!(load_hex(&format!("\r\n{source}"), &mut flash), Ok(6));
    }
}

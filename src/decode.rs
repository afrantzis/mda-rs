// Copyright 2019 Alexandros Frantzis
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
//
// SPDX-License-Identifier: MPL-2.0

//! Base64 and quoted-printable decoding.

use crate::Result;

const PAD: u8 = 64; // The pseudo-index of the PAD character.
const INV: u8 = 99; // An invalid index.

static BASE64_INDICES: &'static [u8] = &[
     //   0    1    2    3    4    5    6    7    8    9    A    B    C    D    E    F
/* 0 */ INV, INV, INV, INV, INV, INV, INV, INV, INV, INV, INV, INV, INV, INV, INV, INV,
/* 1 */ INV, INV, INV, INV, INV, INV, INV, INV, INV, INV, INV, INV, INV, INV, INV, INV,
/* 2 */ INV, INV, INV, INV, INV, INV, INV, INV, INV, INV, INV,  62, INV, INV, INV,  63,
/* 3 */  52,  53,  54,  55,  56,  57,  58,  59,  60,  61, INV, INV, INV, PAD, INV, INV,
/* 4 */ INV,   0,   1,   2,   3,   4,   5,   6,   7,   8,   9,  10,  11,  12,  13,  14,
/* 5 */  15,  16,  17,  18,  19,  20,  21,  22,  23,  24,  25, INV, INV, INV, INV, INV,
/* 6 */ INV,  26,  27,  28,  29,  30,  31,  32,  33,  34,  35,  36,  37,  38,  39,  40,
/* 7 */  41,  42,  43,  44,  45,  46,  47,  48,  49,  50,  51, INV, INV, INV, INV, INV,
/* 8 */ INV, INV, INV, INV, INV, INV, INV, INV, INV, INV, INV, INV, INV, INV, INV, INV,
/* 9 */ INV, INV, INV, INV, INV, INV, INV, INV, INV, INV, INV, INV, INV, INV, INV, INV,
/* A */ INV, INV, INV, INV, INV, INV, INV, INV, INV, INV, INV, INV, INV, INV, INV, INV,
/* B */ INV, INV, INV, INV, INV, INV, INV, INV, INV, INV, INV, INV, INV, INV, INV, INV,
/* C */ INV, INV, INV, INV, INV, INV, INV, INV, INV, INV, INV, INV, INV, INV, INV, INV,
/* D */ INV, INV, INV, INV, INV, INV, INV, INV, INV, INV, INV, INV, INV, INV, INV, INV,
/* E */ INV, INV, INV, INV, INV, INV, INV, INV, INV, INV, INV, INV, INV, INV, INV, INV,
/* F */ INV, INV, INV, INV, INV, INV, INV, INV, INV, INV, INV, INV, INV, INV, INV, INV,
];

/// Decodes base64 encoded data, appending the decoded data to a Vec<u8>.
///
/// During decoding all line breaks and invalid characters are ignored.
/// If an error is encountered during decoding, the already decoded data in the
/// output buffer is left intact. It's up to the caller to deal with the partial
/// decoded data in case of failure.
pub fn base64_decode_into_buf(input: &[u8], output: &mut Vec<u8>) -> Result<()> {
    let mut num_chars = 0;
    let mut cur_triplet = 0;
    let mut valid_chars = 0;

    for c in input {
        let ci = BASE64_INDICES[*c as usize];
        match ci {
            // rfc2045: All line breaks or other characters not
            // found in Table 1 must be ignored by decoding software.
            INV => continue,
            _ if ci < PAD => valid_chars += 1,
            _ => {}
        }

        cur_triplet = cur_triplet << 6 | ((ci & 0x3f) as u32);
        num_chars += 1;

        if num_chars == 4 {
            match valid_chars {
                2 => output.push((cur_triplet >> 16) as u8),
                3 => output.extend(
                    &[(cur_triplet >> 16) as u8, (cur_triplet >> 8) as u8]
                ),
                4 => output.extend(
                    &[(cur_triplet >> 16) as u8,
                      (cur_triplet >> 8) as u8,
                      cur_triplet as u8
                    ]
                ),
                _ => return Err("Invalid base64 encoding".into()),
            }

            cur_triplet = 0;
            num_chars = 0;
            valid_chars = 0;
        }
    }

    // rfc2045: A full encoding quantum is always completed at the end of a body.
    if num_chars != 0 {
        return Err("Unpadded input".into());
    }

    Ok(())
}

/// Converts an ascii byte representing a hex digit to it's numerical value.
fn hexdigit_to_num(mut a: u8) -> Option<u8> {
    if a.is_ascii_digit() {
        return Some(a - b'0');
    }

    a.make_ascii_lowercase();

    if a >= b'a' && a <= b'f' {
        return Some(a - b'a' + 10);
    }

    None
}

/// Decodes quoted-printable encoded data, appending the decoding data to a
/// Vec<u8>.
///
/// During decoding all line breaks and invalid characters are ignored.
/// If an error is encountered during decoding, the already decoded data in the
/// output buffer is left intact. It's up to the caller to deal with the partial
/// decoded data in case of failure.
pub fn qp_decode_into_buf(input: &[u8], output: &mut Vec<u8>) -> Result<()> {
    let mut iter = input.iter().peekable();

    'outer: loop {
        loop {
            match iter.next() {
                Some(b'=') => break,
                Some(c) => output.push(*c),
                None => break 'outer,
            }
        }

        // At this point we have encountered a '=', so check
        // to see what follows.
        if let Some(&first) = iter.next() {
            // A CRLF/LF after '=' marks a line continuation, and
            // is effectively dropped.
            if first == b'\r' {
                if iter.peek() == Some(&&b'\n') {
                    iter.next();
                    continue;
                }
            } else if first == b'\n' {
                continue;
            } else if let Some(first_num) = hexdigit_to_num(first) {
                // A valid pair of hexdigits represent the raw byte value.
                if let Some(&&second) = iter.peek() {
                    if let Some(second_num) = hexdigit_to_num(second) {
                        output.push(first_num * 16 + second_num);
                        iter.next();
                        continue;
                    }
                }
            }

            // Emit the raw sequence if it's not one of the special
            // special cases checked above.
            output.extend(&[b'=', first]);
        } else {
            // Last character in the input was an '=', just emit it.
            output.push(b'=');
        }
    }


    Ok(())
}

#[cfg(test)]
mod test_base64 {
    use crate::decode::base64_decode_into_buf;

    #[test]
    fn decodes_full_length() {
        let mut decoded = Vec::new();
        assert!(base64_decode_into_buf("YWJj".as_bytes(), &mut decoded).is_ok());
        assert_eq!(decoded, &[b'a', b'b', b'c']);
    }

    #[test]
    fn decodes_with_two_padding() {
        let mut decoded = Vec::new();
        assert!(base64_decode_into_buf("YWJjZA==".as_bytes(), &mut decoded).is_ok());
        assert_eq!(decoded, &[b'a', b'b', b'c', b'd']);
    }

    #[test]
    fn decodes_with_one_padding() {
        let mut decoded = Vec::new();
        assert!(base64_decode_into_buf("YWJjZGU=".as_bytes(), &mut decoded).is_ok());
        assert_eq!(decoded, &[b'a', b'b', b'c', b'd', b'e']);
    }

    #[test]
    fn error_with_invalid_paddings() {
        let mut decoded = Vec::new();
        assert!(base64_decode_into_buf("YWJj====".as_bytes(), &mut decoded).is_err());
        assert!(base64_decode_into_buf("YWJjZ===".as_bytes(), &mut decoded).is_err());
        assert!(base64_decode_into_buf("====".as_bytes(), &mut decoded).is_err());
    }

    #[test]
    fn error_with_unpadded_input() {
        let mut decoded = Vec::new();
        assert!(base64_decode_into_buf("YWJjZA=".as_bytes(), &mut decoded).is_err());
    }
}

#[cfg(test)]
mod test_qp {
    use crate::decode::qp_decode_into_buf;

    #[test]
    fn decodes_byte() {
        let mut decoded = Vec::new();
        assert!(qp_decode_into_buf("a=62c=64".as_bytes(), &mut decoded).is_ok());
        assert_eq!(decoded, &[b'a', b'b', b'c', b'd']);
    }

    #[test]
    fn decodes_soft_break() {
        let mut decoded = Vec::new();
        assert!(qp_decode_into_buf("a=\r\nb=\nc".as_bytes(), &mut decoded).is_ok());
        assert_eq!(decoded, &[b'a', b'b', b'c']);
    }

    #[test]
    fn invalid_sequences_are_untouched() {
        let mut decoded = Vec::new();
        let invalid_sequence = "a=6t= c=".as_bytes();
        assert!(qp_decode_into_buf(invalid_sequence, &mut decoded).is_ok());
        assert_eq!(decoded, invalid_sequence);
    }
}

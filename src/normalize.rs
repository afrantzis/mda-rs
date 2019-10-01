// Copyright 2019 Alexandros Frantzis
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
//
// SPDX-License-Identifier: MPL-2.0

//! Normalization of email data for easier processing.
//!
//! Normalization includes:
//!
//! * Placing multi-line header fields on a single line
//! * Decoding base64 or quoted-printable encoded text data, including
//!   MIME encoded-words in the header.
//! * Converting all text data to UTF-8.

use ::regex::bytes::{RegexBuilder, Regex, Captures};
use std::collections::HashMap;
use std::iter::Peekable;
use memchr::{memchr, memchr_iter};
use charset::Charset;
use std::borrow::Cow;
use lazy_static::lazy_static;

use crate::decode::{base64_decode_into_buf, qp_decode_into_buf};

/// An element recognized by the [EmailParser](struct.EmailParser.html).
enum Element {
    HeaderField{data: Vec<u8>},
    Body{
        data: Vec<u8>,
        encoding: Option<String>,
        content_type: Option<String>,
        charset: Option<String>
    },
    Verbatim{data: Vec<u8>},
}

/// Information about a part in a multi-part email message.
/// The top-level is also considered a part.
struct Part {
    encoding: Option<String>,
    content_type: Option<String>,
    charset: Option<String>,
    subpart_boundary: Option<Vec<u8>>,
}

impl Part {
    fn new() -> Self {
        Part{
            encoding: None,
            content_type: None,
            charset: None,
            subpart_boundary: None,
        }
    }
}

/// Iterator for the lines contained in a slice of [u8].
pub struct SliceLines<'a> {
    buf: &'a [u8],
    last: usize,
}

impl<'a> Iterator for SliceLines<'a> {
    type Item = &'a [u8];

    fn next(&mut self) -> Option<&'a [u8]> {
        match memchr(b'\n', &self.buf[self.last..]) {
            Some(m) => {
                let line = &self.buf[self.last..=(self.last + m)];
                self.last = self.last + m + 1;
                Some(line)
            },
            None => {
                let line = &self.buf[self.last..];
                if line.is_empty() {
                    None
                } else {
                    self.last = self.buf.len();
                    Some(line)
                }
            }
        }
    }
}

/// A parser for the elements contained in an email.
///
/// The parsed elements are accessible by iterating over the parser.
///
/// Every line in the email is contained in a MIME part (which itself may be
/// nested in another part). The top level of the email is also considered
/// to be a part for convenience of processing.
struct EmailParser<'a> {
    lines: Peekable<SliceLines<'a>>,
    // The stack of nested parts the line we are processing is contained in.
    part_stack: Vec<Part>,
    // Whether we currently parsing header lines.
    in_header: bool,
    // The active multi-part boundary.
    active_boundary: Vec<u8>,
    content_encoding_regex: Regex,
    content_type_regex: Regex,
    boundary_regex: Regex,
}

impl<'a> EmailParser<'a> {
    fn new(buf: &'a [u8]) -> Self {
        let content_encoding_regex =
            RegexBuilder::new(r"Content-Transfer-Encoding:\s*([[:alnum:]-]+)")
                .case_insensitive(true)
                .build().unwrap();
        let content_type_regex =
            RegexBuilder::new(r#"^Content-Type:\s*([^;]+)\s*(?:;\s*charset\s*=\s*"?([[:alnum:]_:\-\.]+))?"?"#)
                .case_insensitive(true)
                .build().unwrap();

        let boundary_regex =
            RegexBuilder::new(r#"^Content-Type:\s*multipart/.*boundary\s*=\s*"?([[:alnum:]'_,/:=\(\)\+\-\.\?]+)"?"#)
                .case_insensitive(true)
                .build().unwrap();

        EmailParser{
            lines: SliceLines{buf, last: 0}.peekable(),
            // All emails have the top-level part.
            part_stack: vec![Part::new()],
            in_header: true,
            active_boundary: Vec::new(),
            content_encoding_regex: content_encoding_regex,
            content_type_regex: content_type_regex,
            boundary_regex: boundary_regex,
        }
    }

    // Returns the content type of the active part.
    fn active_content_type(&self) -> Option<String> {
        self.part_stack.last()?.content_type.clone()
    }

    // Returns the encoding of the active part.
    fn active_encoding(&self) -> Option<String> {
        self.part_stack.last()?.encoding.clone()
    }

    // Returns the charset of the active part.
    fn active_charset(&self) -> Option<String> {
        self.part_stack.last()?.charset.clone()
    }

    fn begin_part(&mut self) {
        let part = self.part_stack.last().unwrap();

        // We need to differentiate between the first and subsequent parts in a
        // multipart message. The first part creates a new subpart in the
        // part_stack...
        if part.subpart_boundary.as_ref().is_some() &&
           part.subpart_boundary.as_ref().unwrap() == &self.active_boundary {
            self.part_stack.push(Part::new())
        } else {
            // ...whereas subsequent sibling parts just replace the existing
            // part in the stack.
            let part = self.part_stack.last_mut().unwrap();
            *part = Part::new();
        }
    }

    fn end_part(&mut self) {
        self.part_stack.pop();
        if let Some(part) = self.part_stack.last_mut() {
            part.subpart_boundary = None;
        }
        for p in self.part_stack.iter().rev() {
            if let Some(b) = &p.subpart_boundary {
                self.active_boundary = b.clone();
            }
        }
    }

    fn update_active_part_from_header_field(&mut self, field: &[u8]) {
        let mut part = self.part_stack.last_mut().unwrap();

        if let Some(captures) = self.content_encoding_regex.captures(&field) {
            let enc_bytes = captures.get(1).unwrap().as_bytes();
            part.encoding = Some(std::str::from_utf8(&enc_bytes).unwrap().to_lowercase());
        } else if let Some(captures) = self.boundary_regex.captures(&field) {
            part.subpart_boundary = Some(captures.get(1).unwrap().as_bytes().to_vec());
            self.active_boundary = part.subpart_boundary.as_ref().unwrap().clone();
        }
        else if let Some(captures) = self.content_type_regex.captures(&field) {
            let type_bytes = captures.get(1).unwrap().as_bytes();
            part.content_type = Some(std::str::from_utf8(&type_bytes).unwrap().to_lowercase());
            if let Some(charset) = captures.get(2) {
                part.charset = Some(std::str::from_utf8(charset.as_bytes()).unwrap().to_lowercase());
            }
        }
    }
}

/// Removes newline characters from the end of a byte vector.
fn vec_trim_end_newline(line: &mut Vec<u8>) {
    while let Some(&b) = line.last() {
        if b != b'\n' && b != b'\r' {
            break;
        }
        line.pop();
    }
}

/// Returns a new slice not including any newline characters from the
/// end of an existing slice.
fn slice_trim_end_newline(mut line: &[u8]) -> &[u8] {
    while let Some(&b) = line.last() {
        if b != b'\n' && b != b'\r' {
            break;
        } 
        line = &line[..line.len()-1];
    }
    line
}

/// Returns whether a line of bytes is a multi-part boundary line for the
/// specified boundary string.
fn is_boundary_line(line: &[u8], boundary: &[u8]) -> bool {
    line.starts_with(b"--") &&
        !boundary.is_empty() &&
        line[2..].starts_with(&boundary)
}


impl Iterator for EmailParser<'_> {
    type Item = Element;

    fn next(&mut self) -> Option<Element> {
        let mut inprogress = Vec::new();
        let mut element = None;

        // Loop until we recognize an element (or reach end of input).
        loop {
            let line = match self.lines.next() {
                Some(l) => l,
                None => break,
            };

            if self.in_header {
                match line[0] {
                    // Empty lines denote the end of header.
                    b'\n' | b'\r' => {
                        self.in_header = false;
                        element = Some(Element::Verbatim{data: line.to_vec()});
                        break;
                    },
                    // Lines beginning with are continuation lines.
                    b' ' | b'\t' => {
                        vec_trim_end_newline(&mut inprogress);
                        inprogress.extend(line);
                    },
                    _ => inprogress = line.to_vec(),
                };

                // If the next line is not a continuation line, break
                // to emit the current header field.
                if let Some(next_line) = self.lines.peek() {
                    if next_line[0] != b' ' && next_line[0] != b'\t' {
                        break;
                    }
                }

                continue;
            }

            if is_boundary_line(&line, &self.active_boundary) {
                if slice_trim_end_newline(&line).ends_with(b"--") {
                    self.end_part();
                } else {
                    self.begin_part();
                    // After a boundary start line we expect a header.
                    self.in_header = true;
                }

                element = Some(Element::Verbatim{data: line.to_vec()});
                break;
            }

            // If we reached this point, this line is a body line. Append
            // it to the inprogress data.
            inprogress.extend(line);

            // If next line is a boundary line, break to emit the current
            // body.
            if let Some(next_line) = self.lines.peek() {
                if is_boundary_line(next_line, &self.active_boundary) {
                    break;
                }
            }
        }

        // Breaking out the loop happens in three cases:
        // 1. End of input
        // 2. We have recognized a verbatim element.
        // 3. We have inprogress data that we have recognized as a header field
        //    or body.

        // If we have inprogress data, emit it as header or body.
        if !inprogress.is_empty() {
            // We shouldn't have set an element at this point, since we have
            // inprogress data, and this would lead to loss of data.
            assert!(element.is_none());

            if self.in_header {
                element = Some(Element::HeaderField{data: inprogress});
            } else {
                element = Some(
                    Element::Body{
                        data: inprogress,
                        encoding: self.active_encoding(),
                        content_type: self.active_content_type(),
                        charset: self.active_charset(),
                    }
                );
            }
        }

        if let Some(Element::HeaderField{data: field}) = element.as_ref() {
            self.update_active_part_from_header_field(&field);
        }

        element
    }
}

/// Decodes a byte array slice with the specified content encoding and charset
/// to utf-8 byte data, appending to the specified Vec<u8>.
fn decode_text_data_to_buf(
    data: &[u8],
    encoding: Option<&str>,
    charset: Option<&str>,
    mut out: &mut Vec<u8>,
) {
    let should_decode = encoding.is_some();
    let mut should_convert_charset = true;
    let initial_len = out.len();

    if should_decode {
        let result = match encoding.unwrap().as_ref() {
            "base64" => base64_decode_into_buf(&data, &mut out),
            "quoted-printable" => qp_decode_into_buf(&data, &mut out),
            "8bit" | "binary" => { out.extend(data); Ok(()) },
            _ => Err("unknown encoding".into()),
        };

        if result.is_err() {
            out.resize(initial_len, 0);
            should_convert_charset = false;
        }
    }

    if out.len() == initial_len {
        out.extend(data);
    }

    if should_convert_charset {
        if let Some(chr) = Charset::for_label(charset.unwrap_or("us-ascii").as_bytes()) {
            let (cow, _, _) = chr.decode(&out[initial_len..]);
            if let Cow::Owned(c) = cow {
                out.resize(initial_len, 0);
                out.extend(c.bytes());
            }
        }
    }
}

/// Returns whether a byte array slice could contain an MIME encoded-word.
///
/// This function could return a false positive, but never a false negative.
fn maybe_contains_encoded_word(data: &[u8]) -> bool {
    for spacepos in memchr_iter(b'?', &data) {
        if spacepos + 1 < data.len() && data[spacepos + 1] == b'=' {
            return true;
        }
    }

    false
}

/// Decodes a MIME encoded-word represented as regex captures.
fn decode_encoded_word_from_captures(caps: &Captures) -> Vec<u8> {
    let charset = String::from_utf8_lossy(&caps[1]).to_lowercase();
    let encoding = match &caps[2] {
        b"q" | b"Q" => "quoted-printable",
        b"b" | b"B" => "base64",
        _ => "",
    };
    let mut data = Cow::from(&caps[3]);

    // Quoted-printable in encoded-words may use underscores for spaces.
    if encoding == "quoted-printable" {
        let space_positions: Vec<_> =  memchr_iter(b'_', &data).collect();
        for pos in space_positions {
            data.to_mut()[pos] = b' ';
        }
    }

    let mut decoded = Vec::new();
    decode_text_data_to_buf(&data, Some(encoding), Some(&charset), &mut decoded);
    decoded
}

/// Normalizes an email and parses header fields.
///
/// See module documentation about what is involved in normalization.
///
/// Returns the normalized data and a map of header field names to values.
pub fn normalize_email(data: &[u8]) -> (Vec<u8>, HashMap<String, Vec<String>>) {
    lazy_static! {
        static ref ENCODED_WORD_REGEX: Regex =
            RegexBuilder::new(r"=\?([^?]+)\?([^?]+)\?([^? \t]+)\?=")
                .case_insensitive(true)
                .build().unwrap();
        static ref ENCODED_WORD_WSP_REGEX: Regex =
            RegexBuilder::new(r"\?([^?]+)\?=\s*=\?([^?]+)\?")
                .case_insensitive(true)
                .build().unwrap();
    }
    let parser = EmailParser::new(&data);
    let mut normalized = Vec::new();
    let mut fields = HashMap::new();

    for element in parser {
        match element {
            Element::HeaderField{data} => {
                let initial_len = normalized.len();

                if maybe_contains_encoded_word(&data) {
                    // First remove whitespace between consecutive encoded-words
                    // as required by the RFC, then decode.
                    let data = ENCODED_WORD_WSP_REGEX.replace_all(
                        &data, "?$1?==?$2?".as_bytes());
                    let data = ENCODED_WORD_REGEX.replace_all(
                        &data, decode_encoded_word_from_captures);
                    normalized.extend(data.as_ref());
                } else {
                    normalized.extend(&data);
                }

                // Populate the fields map.
                let field_str = String::from_utf8_lossy(&normalized[initial_len..]);
                let field_str = field_str.trim();
                let mut split = field_str.splitn(2, ':');
                let name = split.next().map(|n| n.to_lowercase()).unwrap();
                let value = split.next().unwrap_or("").to_owned();
                fields.entry(name).or_insert(Vec::new()).push(value);
            },
            Element::Body{data, encoding, content_type, charset} => {
                // Only decode text content.
                match content_type {
                    Some(ref content_type) if !content_type.starts_with("text/") => {
                        normalized.extend(&data);
                    },
                    _ => {
                        decode_text_data_to_buf(
                            &data,
                            encoding.as_ref().map(String::as_str),
                            charset.as_ref().map(String::as_str),
                            &mut normalized);
                    }
                };
            },
            Element::Verbatim{data} => {
                normalized.extend(&data);
            },
        }
    }

    (normalized, fields)
}

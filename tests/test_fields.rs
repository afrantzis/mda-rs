// Copyright 2019 Alexandros Frantzis
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
//
// SPDX-License-Identifier: MPL-2.0

use mda::Email;

static TEST_EMAIL: &'static str = "Return-Path: <me@source.com>
Multi: multi1
To: Destination <someone.else@destination.com>
Cc: firstcc <firstcc@destination.com>,
 secondcc <secondcc@destination.com>,
\tthirsdcc <secondcc@destination.com>
Multi: multi2
Multi: multi3
 multi3.1

To: Body <body@destination.com>
Multi: multibody
BodyField: body
Body body body
";

static TEST_EMAIL_NO_BODY: &'static str = "Return-Path: <me@source.com>
Multi: multi1
To: Destination <someone.else@destination.com>
Cc: firstcc <firstcc@destination.com>,
 secondcc <secondcc@destination.com>,
    thirsdcc <secondcc@destination.com>
";

static TEST_EMAIL_CRLF: &'static str = "Return-Path: <me@source.com>\r
Multi: multi1\r
To: Destination <someone.else@destination.com>\r
Cc: firstcc <firstcc@destination.com>,\r
 secondcc <secondcc@destination.com>,\r
    thirsdcc <secondcc@destination.com>\r
Multi: multi2\r
Multi: multi3\r
 multi3.1
";

#[test]
fn parses_single_line_fields() {
    let email = Email::from_vec(TEST_EMAIL.to_string().into_bytes()).unwrap();
    assert_eq!(
        email.header_field("To").unwrap().trim(),
        "Destination <someone.else@destination.com>"
    );
    assert_eq!(
        email.header_field("Return-Path").unwrap().trim(),
        "<me@source.com>"
    );
}

#[test]
fn parses_multi_line_fields() {
    let email = Email::from_vec(TEST_EMAIL.to_string().into_bytes()).unwrap();
    assert_eq!(
        email.header_field("Cc").unwrap().trim(),
        "firstcc <firstcc@destination.com>, secondcc <secondcc@destination.com>,\t\
         thirsdcc <secondcc@destination.com>"
    );
}

#[test]
fn field_names_are_case_insensitive() {
    let email = Email::from_vec(TEST_EMAIL.to_string().into_bytes()).unwrap();

    assert_eq!(
        email.header_field("return-path").unwrap().trim(),
        "<me@source.com>"
    );
    assert_eq!(
        email.header_field("ReTuRn-PaTh").unwrap().trim(),
        "<me@source.com>"
    );
}

#[test]
fn non_existent_field_is_none() {
    let email = Email::from_vec(TEST_EMAIL.to_string().into_bytes()).unwrap();
    assert!(email.header_field("BodyField").is_none());
}

#[test]
fn fields_with_multiple_occurrences_return_all() {
    let email = Email::from_vec(TEST_EMAIL.to_string().into_bytes()).unwrap();

    let multi = email.header_field_all_occurrences("Multi").unwrap();

    assert_eq!(multi.len(), 3);
    assert_eq!(multi.iter().filter(|e| e.trim() == "multi1").count(), 1);
    assert_eq!(multi.iter().filter(|e| e.trim() == "multi2").count(), 1);
    assert_eq!(multi.iter().filter(|e| e.trim() == "multi3 multi3.1").count(), 1);
}

#[test]
fn field_with_multiple_occurrences_returns_first() {
    let email = Email::from_vec(TEST_EMAIL.to_string().into_bytes()).unwrap();

    assert_eq!(email.header_field("Multi").unwrap().trim(), "multi1");
}

#[test]
fn all_occurrences_of_non_existent_field_is_none() {
    let email = Email::from_vec(TEST_EMAIL.to_string().into_bytes()).unwrap();

    assert!(email.header_field_all_occurrences("BodyField").is_none())
}

#[test]
fn header_with_no_body_is_parsed_fully() {
    let email = Email::from_vec(TEST_EMAIL_NO_BODY.to_string().into_bytes()).unwrap();

    assert_eq!(
        email.header_field("Cc").unwrap().trim(),
        "firstcc <firstcc@destination.com>, secondcc <secondcc@destination.com>,    \
         thirsdcc <secondcc@destination.com>"
    );
}

#[test]
fn header_using_crlf() {
    let email = Email::from_vec(TEST_EMAIL_CRLF.to_string().into_bytes()).unwrap();

    assert_eq!(
        email.header_field("Cc").unwrap().trim(),
        "firstcc <firstcc@destination.com>, secondcc <secondcc@destination.com>,    \
         thirsdcc <secondcc@destination.com>"
    );
}

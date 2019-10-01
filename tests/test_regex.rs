// Copyright 2019 Alexandros Frantzis
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
//
// SPDX-License-Identifier: MPL-2.0

use mda::{Email, EmailRegex};

static TEST_EMAIL: &'static str = "Return-Path: <me@source.com>
To: Destination <someone.else@destination.com>
Cc: firstcc <firstcc@destination.com>,
 secondcc <secondcc@destination.com>,
    thirsdcc <thirdcc@destination.com>
X-Test-Field: name123=value456
Content-Type: text/plain; charset=utf-8

To: Body <body@destination.com>
Body body body
Σὰ βγεῖς στὸν πηγαιμὸ γιὰ τὴν Ἰθάκη,
νὰ εὔχεσαι νἆναι μακρὺς ὁ δρόμος,
γεμάτος περιπέτειες, γεμάτος γνώσεις.
";

#[test]
fn header_search() {
    let email = Email::from_vec(TEST_EMAIL.to_string().into_bytes()).unwrap();

    assert!(email.header().search(r"^(Cc|To).*someone\.else@destination\.com").unwrap());
    assert!(!email.header().search(r"^(Cc|To).*body@destination\.com") .unwrap());
}

#[test]
fn header_search_multiline() {
    let email = Email::from_vec(TEST_EMAIL.to_string().into_bytes()).unwrap();

    assert!(email.header().search(r"^Cc.*secondcc@destination\.com").unwrap());
    assert!(email.header().search(r"^Cc.*thirdcc@destination\.com").unwrap());
}

#[test]
fn body_search() {
    let email = Email::from_vec(TEST_EMAIL.to_string().into_bytes()).unwrap();

    assert!(email.body().search(r"^(Cc|To).*body@destination\.com").unwrap());
    assert!(!email.body().search(r"^(Cc|To).*someone\.else@destination\.com").unwrap());
}

#[test]
fn data_search() {
    let email = Email::from_vec(TEST_EMAIL.to_string().into_bytes()).unwrap();

    assert!(email.data().search(r"^(Cc|To).*firstcc@destination\.com").unwrap());
    assert!(email.data().search(r"^(Cc|To).*body@destination\.com").unwrap());
    assert!(!email.data().search(r"^(Cc|To).*unknown@destination\.com").unwrap());
}

#[test]
fn invalid_regex() {
    let email = Email::from_vec(TEST_EMAIL.to_string().into_bytes()).unwrap();

    assert!(email.body().search(r"^(Cc|To).*(body@destination\.com").is_err());
}

#[test]
fn header_search_set() {
    let email = Email::from_vec(TEST_EMAIL.to_string().into_bytes()).unwrap();

    let search =
        email.header().search_set(
            &[
                r"^(Cc|To).*someone\.else@destination\.com",
                r"^(Cc|To).*body@destination\.com",
            ]
        ).unwrap();

    let search: Vec<_> = search.into_iter().collect();
    assert_eq!(search, vec![0]);
}

#[test]
fn body_search_set() {
    let email = Email::from_vec(TEST_EMAIL.to_string().into_bytes()).unwrap();

    let search =
        email.body().search_set(
            &[
                r"^(Cc|To).*someone\.else@destination\.com",
                r"^(Cc|To).*body@destination\.com",
            ]
        ).unwrap();

    let search: Vec<_> = search.into_iter().collect();
    assert_eq!(search, vec![1]);
}

#[test]
fn data_search_set() {
    let email = Email::from_vec(TEST_EMAIL.to_string().into_bytes()).unwrap();

    let search =
        email.data().search_set(
            &[
                r"^(Cc|To).*someone\.else@destination\.com",
                r"^(Cc|To).*body@destination\.com",
            ]
        ).unwrap();

    let search: Vec<_> = search.into_iter().collect();
    assert_eq!(search, vec![0, 1]);
}

#[test]
fn search_set_invalid() {
    let email = Email::from_vec(TEST_EMAIL.to_string().into_bytes()).unwrap();

    let search =
        email.data().search_set(
            &[
                r"^((Cc|To).*someone\.else@destination\.com",
                r"^(Cc|To).*body@destination\.com",
            ]
        );

    assert!(search.is_err());
}

#[test]
fn unicode_support() {
    let email = Email::from_vec(TEST_EMAIL.to_string().into_bytes()).unwrap();

    assert!(email.body().search(r"νἆναι μακρὺς ὁ δρόμος").unwrap());
    assert!(email.body().search(r"νἆναι μακρὺς ὁ δρόμος").unwrap());

    assert_eq!(
        email.body().search_set(
            &[
                r"Τοὺς Λαιστρυγόνας καὶ τοὺς Κύκλωπας",
                r"νἆναι μακρὺς ὁ δρόμος",
            ]
        ).unwrap().into_iter().collect::<Vec<_>>(),
        vec![1]
    );

    assert_eq!(
        email.data().search_set(
            &[
                r"Τοὺς Λαιστρυγόνας καὶ τοὺς Κύκλωπας",
                r"νἆναι μακρὺς ὁ δρόμος",
            ]
        ).unwrap().into_iter().collect::<Vec<_>>(),
        vec![1]
    );
}

#[test]
fn captures() {
    let email = Email::from_vec(TEST_EMAIL.to_string().into_bytes()).unwrap();

    let header = email.header();
    let captures =
        header
            .search_with_captures(r"^X-Test-Field: *(?P<name>\w+)=(?P<value>\w+)")
            .unwrap()
            .unwrap();

    assert_eq!(captures.name("name").map(|m| m.as_bytes()), Some("name123".as_bytes()));
    assert_eq!(captures.name("value").map(|m| m.as_bytes()), Some("value456".as_bytes()));
}

#[test]
fn multiline_headers() {
    let email = Email::from_vec(TEST_EMAIL.to_string().into_bytes()).unwrap();

    let header = email.header();
    let captures =
        header
            .search_with_captures(r"^X-Test-Field: *(?P<name>\w+)=(?P<value>\w+)")
            .unwrap()
            .unwrap();

    assert_eq!(captures.name("name").map(|m| m.as_bytes()), Some("name123".as_bytes()));
    assert_eq!(captures.name("value").map(|m| m.as_bytes()), Some("value456".as_bytes()));
}

// Copyright 2019 Alexandros Frantzis
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
//
// SPDX-License-Identifier: MPL-2.0

use mda::{Email, EmailRegex};

static TEST_EMAIL_MULTIPART: &'static str = r#"Return-Path: <me@source.com>
To: =?iso-8859-1?q?=C0a_b=DF?= <someone.else1@destination.com>,
 =?utf-8?b?zqXOps6nzqjOqQo=?= <someone.else2@destination.com>,
Cc: =?iso-8859-1?q?=C0 b?= <someone.else3@destination.com>
Bcc: =?utf8?B?zpbOl86YCg=?= <someone.else4@destination.com>
Content-type: multipart/alternative; boundary="XtT01VFrJIenjlg+ZCXSSWq4"

--XtT01VFrJIenjlg+ZCXSSWq4
Content-Type: text/plain; charset="us-ascii"
Content-Transfer-Encoding: base64
X-header-field: =?UTF-8?B?zpHOks6TCg==?=

--XtT01VFrJIenjlg+ZCXSSWq4--
"#;

static TEST_EMAIL_INVALID_UTF8: &'static str =
    r#"Subject: =?utf-8?B?zojOus60zr/Pg863IGUtzrvOv86zzrHPgc65zrHPg868zr/P?="#;

static TEST_EMAIL_MULTI_ENC_WORD: &'static str = r#"Return-Path: <me@source.com>
Subject: =?utf-8?b?TXkgbXVsdGkgZW5jb2RlZC0=?=
 =?utf-8?b?d29yZCBzdWJqZWN0IGw=?=
	  =?utf-8?b?aW5l?=
"#;

#[test]
fn encoded_word_is_decoded() {
    let email = Email::from_vec(TEST_EMAIL_MULTIPART.to_string().into_bytes()).unwrap();

    assert!(email.data().search(r"Àa bß").unwrap());
    assert!(email.header_field("To").unwrap().contains(r"Àa bß"));
    assert!(!email.data().search(r"=C0a_b=DF").unwrap());
    assert!(!email.header_field("To").unwrap().contains(r"=C0a_b=DF"));

    assert!(email.data().search(r"ΥΦΧΨΩ").unwrap());
    assert!(email.header_field("To").unwrap().contains(r"ΥΦΧΨΩ"));
    assert!(!email.data().search(r"zqXOps6nzqjOqQo=").unwrap());
    assert!(!email.header_field("To").unwrap().contains(r"zqXOps6nzqjOqQo="));

    assert!(email.data().search(r"ΑΒΓ").unwrap());
    assert!(!email.data().search(r"zpHOks6TCg==").unwrap());
}

#[test]
fn invalid_encoded_word_is_not_decoded() {
    let email = Email::from_vec(TEST_EMAIL_MULTIPART.to_string().into_bytes()).unwrap();

    assert!(!email.data().search(r"À b").unwrap());
    assert!(email.data().search(r"=C0 b").unwrap());

    assert!(!email.data().search(r"ΖΗΘ").unwrap());
    assert!(email.data().search(r"zpbOl86YCg=").unwrap());
}

#[test]
fn invalid_charset_encoding_in_encoded_word_is_partially_decoded() {
    let email = Email::from_vec(TEST_EMAIL_INVALID_UTF8.to_string().into_bytes()).unwrap();

    assert!(email.data().search("Έκδοση e-λογαριασμο\u{FFFD}").unwrap());
    assert!(email.header_field("Subject").unwrap().contains("Έκδοση e-λογαριασμο\u{FFFD}"));
}

#[test]
fn multpile_encoded_words_are_concatenated() {
    let email = Email::from_vec(TEST_EMAIL_MULTI_ENC_WORD.to_string().into_bytes()).unwrap();

    assert!(email.data().search("My multi encoded-word subject line").unwrap());
    assert!(email.header_field("Subject").unwrap().contains("My multi encoded-word subject line"));
}

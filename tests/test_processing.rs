// Copyright 2019 Alexandros Frantzis
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
//
// SPDX-License-Identifier: MPL-2.0

use mda::Email;

static TEST_EMAIL: &'static str = "Return-Path: <me@source.com>
To: Destination <someone.else@destination.com>
Cc: firstcc <firstcc@destination.com>,
 secondcc <secondcc@destination.com>,
    thirsdcc <secondcc@destination.com>

To: Body <body@destination.com>
Body body body
";

#[test]
fn filtering_creates_new_email() {
    let email = Email::from_vec(TEST_EMAIL.to_string().into_bytes()).unwrap();

    let email = email.filter(&["sed", "s/destination.com/newdest.com/g"]).unwrap();

    assert_eq!(email.header_field("To").unwrap().trim(), "Destination <someone.else@newdest.com>");
}

#[test]
fn processing_returns_output() {
    let email = Email::from_vec(TEST_EMAIL.to_string().into_bytes()).unwrap();

    let output_dest = email.process(&["grep", "Destination"]).unwrap();
    let output_some = email.process(&["grep", "SomeInexistentString"]).unwrap();

    assert_eq!(output_dest.status.code().unwrap(), 0);
    assert_eq!(output_some.status.code().unwrap(), 1);
}

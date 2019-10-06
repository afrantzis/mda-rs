// Copyright 2019 Alexandros Frantzis
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
//
// SPDX-License-Identifier: MPL-2.0

use mda::{Email, EmailRegex};

static TEST_EMAIL_FAKE_BOUNDARY: &'static str = r#"Return-Path: <me@source.com>
To: Destination <someone.else@destination.com>
Content-type: multipart/alternative; boundary="QWFCYkN"

--QWFCYkN
Content-transfer-encoding: base64

--QWFCYkNj

--QWFCYkN
"#;

static TEST_EMAIL_BOUNDARY_BEGIN_AFTER_END: &'static str = r#"Return-Path: <me@source.com>
To: Destination <someone.else@destination.com>
Content-type: multipart/alternative; boundary="XtT01VFrJIenjlg+ZCXSSWq4"

--XtT01VFrJIenjlg+ZCXSSWq4--

--XtT01VFrJIenjlg+ZCXSSWq4
"#;

#[test]
fn only_exact_boundary_lines_are_parsed() {
    // The "--QWFCYkNj" line should be parsed as part of the body not as a boundary.
    let email =
        Email::from_vec(
            TEST_EMAIL_FAKE_BOUNDARY.to_string().into_bytes()
        ).unwrap();
    assert!(email.body().search("AaBbCc").unwrap());
}

#[test]
fn boundary_begin_after_end_is_parsed() {
    assert!(
        Email::from_vec(
            TEST_EMAIL_BOUNDARY_BEGIN_AFTER_END.to_string().into_bytes()
        ).is_ok()
    );
}

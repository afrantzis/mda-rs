// Copyright 2019 Alexandros Frantzis
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
//
// SPDX-License-Identifier: MPL-2.0

use mda::{Email, EmailRegex};

static TEST_EMAIL_ISO_BASE64: &'static str = r#"Return-Path: <me@source.com>
To: Destination <someone.else@destination.com>
Content-Type: text/plain; charset="iso-8859-7"
Content-Transfer-Encoding: base64

tuvr4SDm5/Tl3yDnIPj1994g8+/1LCDj6Scg3Ovr4SDq6+Hf5em3CvTv7SDd8OHp7e8g9O/1IMTe
7O/1IOrh6SD0+e0g0+/26fP0/u0sCvThIOT98+rv6+Eg6uHpIPQnIOHt5er03+zn9OEgxf3j5bcK
9OftIMHj7/HcLCD07yDI3eH08e8sIOrh6SD07/XyINP05fbc7e/18i4=
"#;

static TEST_EMAIL_ISO_8BIT: &'static [u8] = &[
    b'C', b'o', b'n', b't', b'e', b'n', b't', b'-', b'T', b'y', b'p', b'e',
    b':', b' ', b't', b'e', b'x', b't', b'/', b'p', b'l', b'a', b'i', b'n',
    b';', b' ', b'c', b'h', b'a', b'r', b's', b'e', b't', b'=', b'"', b'i',
    b's', b'o', b'-', b'8', b'8', b'5', b'9', b'-', b'7', b'"', b'\r', b'\n',
    b'C', b'o', b'n', b't', b'e', b'n', b't', b'-', b'T', b'r', b'a', b'n',
    b's', b'f', b'e', b'r', b'-', b'E', b'n', b'c', b'o', b'd', b'i', b'n',
    b'g', b':', b' ', b'8', b'b', b'i', b't', b'\r', b'\n',
    b'\r', b'\n',
    0xb6, 0xeb, 0xe1, 0x20, 0xe6, 0xe7, 0xf4, 0xe5, 0xdf, 0x20, 0xe7, 0x20,
    0xf8, 0xf5, 0xf7, 0xde, 0x20, 0xf3, 0xef, 0xf5, 0x2c, 0x20, 0xe3, 0xe9,
    0x27, 0x20, 0xdc, 0xeb, 0xe1, 0x20, 0xea, 0xeb, 0xe1, 0xdf, 0xe5, 0xe9,
    0xb7, 0x0a, 0xf4, 0xef, 0xed, 0x20, 0xdd, 0xf0, 0xe1, 0xe9, 0xed, 0xef,
    0x20, 0xf4, 0xef, 0xf5, 0x20, 0xc4, 0xde, 0xec, 0xef, 0xf5, 0x20, 0xea,
    0xe1, 0xe9, 0x20, 0xf4, 0xf9, 0xed, 0x20, 0xd3, 0xef, 0xf6, 0xe9, 0xf3,
    0xf4, 0xfe, 0xed, 0x2c, 0x0a, 0xf4, 0xe1, 0x20, 0xe4, 0xfd, 0xf3, 0xea,
    0xef, 0xeb, 0xe1, 0x20, 0xea, 0xe1, 0xe9, 0x20, 0xf4, 0x27, 0x20, 0xe1,
    0xed, 0xe5, 0xea, 0xf4, 0xdf, 0xec, 0xe7, 0xf4, 0xe1, 0x20, 0xc5, 0xfd,
    0xe3, 0xe5, 0xb7, 0x0a, 0xf4, 0xe7, 0xed, 0x20, 0xc1, 0xe3, 0xef, 0xf1,
    0xdc, 0x2c, 0x20, 0xf4, 0xef, 0x20, 0xc8, 0xdd, 0xe1, 0xf4, 0xf1, 0xef,
    0x2c, 0x20, 0xea, 0xe1, 0xe9, 0x20, 0xf4, 0xef, 0xf5, 0xf2, 0x20, 0xd3,
    0xf4, 0xe5, 0xf6, 0xdc, 0xed, 0xef, 0xf5, 0xf2, 0x2e
];

static TEST_EMAIL_MULTIPART_ISO: &'static str = r#"Return-Path: <me@source.com>
To: Destination <someone.else@destination.com>
Content-type: multipart/alternative; boundary="XtT01VFrJIenjlg+ZCXSSWq4"

--XtT01VFrJIenjlg+ZCXSSWq4
Content-Type: text/plain; charset="us-ascii"
Content-Transfer-Encoding: base64

Sample US-ASCII text.
--XtT01VFrJIenjlg+ZCXSSWq4
Content-type: multipart/alternative; boundary="2c+OeCbICgJrtINI5EFlsI6G"

--2c+OeCbICgJrtINI5EFlsI6G
Content-Type: text/plain; charset="utf-8"
Content-Transfer-Encoding: base64

zprOuSDhvILOvSDPgM+Ez4nPh865zrrhvbQgz4ThvbTOvSDOss+B4b+Hz4IsIOG8oSDhvLjOuM6s
zrrOtyDOtOG9ss69IM+D4b2yIM6zzq3Ou86xz4POtS4K4bycz4TPg865IM+Dzr/PhuG9uM+CIM+A
zr/hvbog4byUzrPOuc69zrXPgiwgzrzhvbIgz4TPjM+Dzrcgz4DOtc6vz4HOsSwK4bykzrTOtyDO
uOG9sCDPhOG9uCDOus6xz4TOrM67zrHOss61z4Ig4b6RIOG8uM64zqzOus61z4Igz4TOryDPg863
zrzOsc6vzr3Ov8+Fzr0uCg==
--2c+OeCbICgJrtINI5EFlsI6G
Content-Type: image/jpeg;
Content-Transfer-Encoding: base64

SSBhbSBzb3JyeSBEYXZlLCBJbSBhZnJhaWQgSSBjYW50IGRvIHRoYXQK

--2c+OeCbICgJrtINI5EFlsI6G--

--XtT01VFrJIenjlg+ZCXSSWq4
Content-Type: text/plain; charset="iso-8859-7"
Content-Transfer-Encoding: base64

tuvr4SDm5/Tl3yDnIPj1994g8+/1LCDj6Scg3Ovr4SDq6+Hf5em3CvTv7SDd8OHp7e8g9O/1IMTe
7O/1IOrh6SD0+e0g0+/26fP0/u0sCvThIOT98+rv6+Eg6uHpIPQnIOHt5er03+zn9OEgxf3j5bcK
9OftIMHj7/HcLCD07yDI3eH08e8sIOrh6SD07/XyINP05fbc7e/18i4=
--XtT01VFrJIenjlg+ZCXSSWq4--
"#;

#[test]
fn email_with_charset_is_decoded() {
    let email = Email::from_vec(TEST_EMAIL_ISO_BASE64.to_string().into_bytes()).unwrap();

    assert!(email.body().search(r"τα δύσκολα και τ' ανεκτίμητα Εύγε·").unwrap());
}

#[test]
fn email_with_charset_8bit_is_decoded() {
    let email = Email::from_vec(TEST_EMAIL_ISO_8BIT.to_vec()).unwrap();

    assert!(email.body().search(r"τα δύσκολα και τ' ανεκτίμητα Εύγε·").unwrap());
}

#[test]
fn email_part_with_charset_is_decoded() {
    let email = Email::from_vec(TEST_EMAIL_MULTIPART_ISO.as_bytes().to_vec()).unwrap();

    assert!(email.body().search(r"Sample US-ASCII text.").unwrap());
    assert!(email.body().search(r"τα δύσκολα και τ' ανεκτίμητα Εύγε·").unwrap());
}

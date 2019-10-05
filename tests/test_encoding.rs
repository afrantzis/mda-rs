// Copyright 2019 Alexandros Frantzis
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
//
// SPDX-License-Identifier: MPL-2.0

use mda::{Email, EmailRegex};

static TEST_EMAIL_BASE64: &'static str = r#"Return-Path: <me@source.com>
To: Destination <someone.else@destination.com>
Content-Type: text/plain; charset="utf-8"
Content-Transfer-Encoding: base64

VGhlIGFudGVjaGFwZWwgd2hlcmUgdGhlIHN0YXR1ZSBzdG9vZApPZiBOZXd0b24gd2l0aCBoaXMg
cHJpc20gYW5kIHNpbGVudCBmYWNlLApUaGUgbWFyYmxlIGluZGV4IG9mIGEgbWluZCBmb3IgZXZl
cgpWb3lhZ2luZyB0aHJvdWdoIHN0cmFuZ2Ugc2VhcyBvZiBUaG91Z2h0LCBhbG9uZS4gCg==
"#;

static TEST_EMAIL_MULTIPART: &'static str = r#"Return-Path: <me@source.com>
To: Destination <someone.else@destination.com>
Content-type: multipart/alternative; boundary="XtT01VFrJIenjlg+ZCXSSWq4"

--XtT01VFrJIenjlg+ZCXSSWq4
Content-Type: text/plain; charset="utf-8"
Content-Transfer-Encoding: base64

VGhlIGFudGVjaGFwZWwgd2hlcmUgdGhlIHN0YXR1ZSBzdG9vZApPZiBOZXd0b24gd2l0aCBoaXMg
cHJpc20gYW5kIHNpbGVudCBmYWNlLApUaGUgbWFyYmxlIGluZGV4IG9mIGEgbWluZCBmb3IgZXZl
cgpWb3lhZ2luZyB0aHJvdWdoIHN0cmFuZ2Ugc2VhcyBvZiBUaG91Z2h0LCBhbG9uZS4gCg==
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
Content-Type: text/plain; charset="utf-8"
Content-Transfer-Encoding: base64

T3VyIHBvc3R1cmluZ3MsIG91ciBpbWFnaW5lZCBzZWxmLWltcG9ydGFuY2UsIHRoZSBkZWx1c2lv
biB0aGF0IHdlIGhhdmUgc29tZSBwcml2aWxlZ2VkIHBvc2l0aW9uIGluIHRoZSBVbml2ZXJzZSwg
YXJlIGNoYWxsZW5nZWQgYnkgdGhpcyBwb2ludCBvZiBwYWxlIGxpZ2h0LiBPdXIgcGxhbmV0IGlz
IGEgbG9uZWx5IHNwZWNrIGluIHRoZSBncmVhdCBlbnZlbG9waW5nIGNvc21pYyBkYXJrLg==
--XtT01VFrJIenjlg+ZCXSSWq4--
"#;

static TEST_EMAIL_INVALID_BASE64: &'static str = r#"Return-Path: <me@source.com>
To: Destination <someone.else@destination.com>
Content-Type: text/plain; charset="utf-8"
Content-Transfer-Encoding: base64

VGhlIGFudGVjaGFwZWwgd2hlcmUgdGhlIHN0YXR1ZSBzdG9vZApPZiBOZXd0b24gd2l0aCBoaXMg
cHJpc20gYW5kIHNpbGVudCBmYWNlLApUaGUgbWFyYmxlIGluZGV4IG9mIGEgbWluZCBmb3IgZXZl
cgpWb3lhZ2luZyB0aHJvdWdoIHN0cmFuZ2Ugc2VhcyBvZiBUaG91Z2h0LCBhbG9uZS4gCg====
"#;

static TEST_EMAIL_QP: &'static str = r#"Return-Path: <me@source.com>
To: Destination <someone.else@destination.com>
Content-Type: text/plain; charset="utf-8"
Content-Transfer-Encoding: quoted-printable

=54=68=65=20=61=6E=74=65=63=68=61=70=65=6C=20=77=68=65=72=65=20=74=68=
=65=20=73=74=61=74=75=65=20=73=74=6F=6F=64
=4F=66=20=4E=65=77=74=6F=6E=20=77=69=74=68=20=68=69=73=20=70=72=69=73=
=6D=20=61=6E=64=20=73=69=6C=65=6E=74=20=66=61=63=65=2C
=54=68=65=20=6D=61=72=62=6C=65=20=69=6E=64=65=78=20=6F=66=20=61=20=6D=
=69=6E=64=20=66=6F=72=20=65=76=65=72
=56=6F=79=61=67=69=6E=67=20=74=68=72=6F=75=67=68=20=73=74=72=61=6E=67=
=65=20=73=65=61=73=20=6F=66=20=54=68=6F=75=67=68=74=2C=20=61=6C=6F=6E=
=65=2E=20
"#;

#[test]
fn base64_email_is_decoded() {
    let email = Email::from_vec(TEST_EMAIL_BASE64.to_string().into_bytes()).unwrap();

    assert!(email.body().search(r"a\smind\sfor\sever\svoyaging").unwrap());
}

#[test]
fn base64_parts_are_decoded() {
    let email = Email::from_vec(TEST_EMAIL_MULTIPART.to_string().into_bytes()).unwrap();

    // First level part.
    assert!(email.body().search(r"a\smind\sfor\sever\svoyaging").unwrap());
    // Second level nested part.
    assert!(email.body().search(r"ἤδη θὰ τὸ κατάλαβες ᾑ Ἰθάκες τί σημαίνουν").unwrap());
    // First level part after end of previous nested subparts.
    assert!(email.body().search(r"are challenged by this point of pale light").unwrap());
}

#[test]
fn base64_boundaries_remain_on_their_own_line() {
    let email = Email::from_vec(TEST_EMAIL_MULTIPART.to_string().into_bytes()).unwrap();

    assert!(!email.data().search(r"[^\n]--XtT01VFrJIenjlg\+ZCXSSWq4").unwrap());
    assert!(!email.data().search(r"[^\n]--2c\+OeCbICgJrtINI5EFlsI6G").unwrap());
}

#[test]
fn non_text_base64_is_not_decoded() {
    let email = Email::from_vec(TEST_EMAIL_MULTIPART.to_string().into_bytes()).unwrap();

    assert!(!email.body().search(r"I am sorry Dave").unwrap());
}

#[test]
fn invalid_base64_is_not_decoded() {
    let email = Email::from_vec(TEST_EMAIL_INVALID_BASE64.to_string().into_bytes()).unwrap();

    assert!(!email.body().search(r"a\smind\sfor\sever\svoyaging").unwrap());
    assert!(email.body().search(r"4gCg=").unwrap());
}

#[test]
fn qp_email_is_decoded() {
    let email = Email::from_vec(TEST_EMAIL_QP.to_string().into_bytes()).unwrap();

    assert!(email.body().search(r"a\smind\sfor\sever\svoyaging").unwrap());
}

#[test]
fn raw_data_is_not_decoded() {
    let email = Email::from_vec(TEST_EMAIL_MULTIPART.to_string().into_bytes()).unwrap();

    assert!(email.raw_data().search(r"vZiBUaG91Z2h0LCBhbG9uZS4gCg==").unwrap());
    assert!(!email.raw_data().search(r"ἤδη θὰ τὸ κατάλαβες ᾑ Ἰθάκες τί σημαίνουν").unwrap());
}

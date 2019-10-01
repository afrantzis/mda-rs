// Copyright 2019 Alexandros Frantzis
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
//
// SPDX-License-Identifier: MPL-2.0

use mda::Email;
use tempfile;
use std::fs;
use std::os::unix::fs as unix_fs;

#[test]
fn creates_maildir_dir_structure() {
    let tmpdir = tempfile::tempdir().unwrap();

    let email = Email::from_vec(Vec::new()).unwrap();
    email.deliver_to_maildir(tmpdir.path()).unwrap();

    let entries: Vec<_> = fs::read_dir(tmpdir.path()).unwrap().collect();

    let dir_named = |x,e: &fs::DirEntry| e.path() == tmpdir.path().join(x) &&
                                         e.metadata().unwrap().is_dir();

    assert_eq!(entries.len(), 3);
    assert_eq!(entries.iter().filter(|e| dir_named("new", e.as_ref().unwrap())).count(), 1);
    assert_eq!(entries.iter().filter(|e| dir_named("tmp", e.as_ref().unwrap())).count(), 1);
    assert_eq!(entries.iter().filter(|e| dir_named("cur", e.as_ref().unwrap())).count(), 1);
}

#[test]
fn delivers_to_maildir_new() {
    let tmpdir = tempfile::tempdir().unwrap();
    let data = [1, 3, 5, 7, 11];

    let email = Email::from_vec(data.to_vec()).unwrap();
    email.deliver_to_maildir(tmpdir.path()).unwrap();

    let new_entries: Vec<_> = fs::read_dir(tmpdir.path().join("new")).unwrap().collect();
    let tmp_entries: Vec<_> = fs::read_dir(tmpdir.path().join("tmp")).unwrap().collect();
    let cur_entries: Vec<_> = fs::read_dir(tmpdir.path().join("cur")).unwrap().collect();

    assert_eq!(new_entries.len(), 1);

    let file_contents = fs::read(new_entries[0].as_ref().unwrap().path()).unwrap();
    assert_eq!(file_contents, &data);

    assert_eq!(tmp_entries.len(), 0);
    assert_eq!(cur_entries.len(), 0);
}

#[test]
fn keeps_old_maildir_data() {
    let tmpdir = tempfile::tempdir().unwrap();

    let data1 = [1, 3, 5, 7, 11];
    let email1 = Email::from_vec(data1.to_vec()).unwrap();
    let path1 = email1.deliver_to_maildir(tmpdir.path()).unwrap();

    let data2 = [2, 4, 6, 8, 12];
    let email2 = Email::from_vec(data2.to_vec()).unwrap();
    let path2 = email2.deliver_to_maildir(tmpdir.path()).unwrap();

    let new_entries: Vec<_> = fs::read_dir(tmpdir.path().join("new")).unwrap().collect();

    assert_eq!(new_entries.len(), 2);
    assert_eq!(new_entries.iter().filter(|e| e.as_ref().unwrap().path() == path1).count(), 1);
    assert_eq!(new_entries.iter().filter(|e| e.as_ref().unwrap().path() == path2).count(), 1);

    assert_eq!(fs::read(path1).unwrap(), &data1);
    assert_eq!(fs::read(path2).unwrap(), &data2);
}

#[test]
fn deals_with_soft_link_path() {
    let tmpdir = tempfile::tempdir().unwrap();
    let subdir = tmpdir.path().join("subdir");
    let symlink = tmpdir.path().join("symlink");

    fs::create_dir(&subdir).unwrap();
    unix_fs::symlink(&subdir, &symlink).unwrap();

    let email = Email::from_vec(Vec::new()).unwrap();
    email.deliver_to_maildir(&symlink).unwrap();
}

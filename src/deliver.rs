// Copyright 2019 Alexandros Frantzis
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
//
// SPDX-License-Identifier: MPL-2.0

//! Email delivery functionality.

use std::fs::{self, File};
use std::io::ErrorKind;
use std::io::prelude::*;
use std::os::unix::prelude::*;
use std::path::{PathBuf, Path};
use std::process;
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::{DeliveryDurability, Result};

use gethostname::gethostname;
use libc;

/// A generator for likely unique maildir email filenames.
///
/// Using it as an iterator gets a filename that can be used in a maildir
/// and is likely to be unique.
pub struct EmailFilenameGenerator {
    count: usize,
    max_seen_unix_time: u64,
    hostname: String,
}

impl EmailFilenameGenerator {
    pub fn new() -> Self {
        // From https://cr.yp.to/proto/maildir.html:
        // "To deal with invalid host names, replace / with \057 and : with \072"
        let hostname =
            gethostname()
                .to_string_lossy()
                .into_owned()
                .replace("/", r"\057")
                .replace(":", r"\072");

        EmailFilenameGenerator{
            count: 0,
            max_seen_unix_time: 0,
            hostname: hostname,
        }
    }
}

impl Iterator for EmailFilenameGenerator {
    type Item = String;

    fn next(&mut self) -> Option<String> {
        let unix_time = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
        let pid = process::id();

        if self.max_seen_unix_time < unix_time {
            self.max_seen_unix_time = unix_time;
            self.count = 0;
        } else {
            self.count += 1;
        }

        Some(format!("{}.{}_{}.{}", unix_time, pid, self.count, self.hostname))
    }
}

/// A representation of a maildir.
pub struct Maildir {
    root: PathBuf,
    email_filename_gen: Arc<Mutex<EmailFilenameGenerator>>,
}

impl Maildir {
    /// Opens, or creates if it doesn't a exist, a maildir directory structure
    /// at the specified path.
    pub fn open_or_create(
        mailbox: &Path,
        email_filename_gen: Arc<Mutex<EmailFilenameGenerator>>
    ) -> Result<Self> {
        let root = PathBuf::from(mailbox);
        for s in &["tmp", "new", "cur"] {
            let path = root.join(&s);
            fs::create_dir_all(&path)?;
        }

        Ok(Maildir{root, email_filename_gen})
    }

    /// Delivers an email to the maildir by creating a new file with the email data,
    /// and using the specified DeliveryDurability method.
    pub fn deliver(
        &self,
        data: &[u8],
        delivery_durability: DeliveryDurability
    ) -> Result<PathBuf> {
        loop {
            let tmp_dir = self.root.join("tmp");
            let new_dir = self.root.join("new");

            let tmp_email = self.write_email_to_dir(data, &tmp_dir)?;
            let new_email = new_dir.join(
                tmp_email.file_name().ok_or("")?.to_str().ok_or("")?);

            let result = fs::hard_link(&tmp_email, &new_email);
            fs::remove_file(&tmp_email)?;

            match result {
                Ok(_) => {
                    if delivery_durability == DeliveryDurability::FileAndDirSync {
                        File::open(&new_dir)?.sync_all()?;
                        File::open(&tmp_dir)?.sync_all()?;
                    }
                    return Ok(new_email);
                },
                Err(ref err) if err.kind() == ErrorKind::AlreadyExists => {},
                Err(err)  => return Err(err.into()),
            }
        }
    }

    /// Delivers an email to the maildir by hard-linking with an existing file,
    /// and using the specified DeliveryDurability method.
    pub fn deliver_with_hard_link(
        &self,
        src: &Path,
        delivery_durability: DeliveryDurability
    ) -> Result<PathBuf> {
        loop {
            let new_dir = self.root.join("new");
            let new_email = new_dir.join(self.next_email_filename_candidate()?);

            match fs::hard_link(&src, &new_email) {
                Ok(_) => {
                    if delivery_durability == DeliveryDurability::FileAndDirSync {
                        File::open(&new_dir)?.sync_all()?;
                    }
                    return Ok(new_email);
                },
                Err(ref err) if err.kind() == ErrorKind::AlreadyExists => {},
                Err(err)  => return Err(err.into()),
            }
        }
    }

    /// Writes email data to a new file in the specified directory.
    fn write_email_to_dir(&self, data: &[u8], dir: &Path) -> Result<PathBuf> {
        loop {
            let email = dir.join(self.next_email_filename_candidate()?);
            let result = fs::OpenOptions::new()
                        .create_new(true)
                        .write(true)
                        .custom_flags(libc::O_SYNC)
                        .open(&email);

            match result {
                Ok(mut f) => {
                    f.write_all(&data)?;
                    return Ok(email);
                },
                Err(ref err) if err.kind() == ErrorKind::AlreadyExists => {},
                Err(err)  => return Err(err.into()),
            }
        }
    }

    /// Gets the next email filename candidate from the EmailFilenameGenerator.
    fn next_email_filename_candidate(&self) -> Result<String> {
        let mut gen = self.email_filename_gen.lock().map_err(|_| "")?;
        gen.next().ok_or("".into())
    }
}

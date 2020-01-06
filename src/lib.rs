// Copyright 2019 Alexandros Frantzis
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
//
// SPDX-License-Identifier: MPL-2.0

//! The mda crate provides a library for writing custom Mail Deliver Agents. It
//! supports local delivery to maildirs, access to normalized email byte
//! data for easier processing, and access to individual header fields.
//!
//! Email data normalization involves ensuring header fields are in single
//! lines, decoding text parts of the message that use some kind of transfer
//! encoding (e.g., base64), and converting all text to UTF-8.  The original
//! (non-normalized) email data is used during delivery.
//!
//! This crate also exposes convenience methods for regular expression searching
//! and processing/filtering of emails.
//!
//! # Email construction
//!
//! The [Email struct](struct.Email.html) is the basic abstraction of the `mda`
//! crate. To construct an Email use the
//! [Email::from_stdin](struct.Email.html#method.from_stdin) or
//! [Email::from_vec](struct.Email.html#method.from_vec) method.
//!
//! ```no_run
//! use mda::Email;
//! let email = Email::from_stdin()?;
//! let email = Email::from_vec(vec![97, 98, 99])?;
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```
//!
//! # Email delivery
//!
//! Use the
//! [Email::deliver_to_maildir](struct.Email.html#method.deliver_to_maildir)
//! method to deliver the email to local maildir directories. Note that
//! the original (non-normalized) email data is used during delivery.
//!
//! ```no_run
//! use mda::Email;
//! let email = Email::from_stdin()?;
//! email.deliver_to_maildir("/my/maildir/path")?;
//! email.deliver_to_maildir("/my/other/maildir/path")?;
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```
//!
//! # Accessing email header fields
//!
//! Use the [Email::header_field](struct.Email.html#method.header_field) and
//! [Email::header_field_all_occurrences](struct.Email.html#method.header_field_all_occurrences)
//! methods to access the email header fields. Any MIME encoded words in the
//! header field values are decoded and the field value is converted to UTF-8.
//!
//! ```no_run
//! use mda::Email;
//! let email = Email::from_stdin()?;
//! let to = email.header_field("To").unwrap_or("");
//! if to.contains("me@example.com") {
//!     email.deliver_to_maildir("/my/maildir/path")?;
//! }
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```
//!
//! # Searching with regular expressions
//!
//! The [EmailRegex](trait.EmailRegex.html) trait provides convenience methods
//! for searching the header, the body or the whole email with regular
//! expressions. The convenience functions use case-insensitive, multi-line
//! search (`^` and `$` match beginning and end of lines).  If the above don't
//! match your needs, or you require additional functionality, you can perform
//! manual regex search using the email data.
//!
//! ```no_run
//! use mda::{Email, EmailRegex};
//! let email = Email::from_stdin()?;
//! if email.header().search(r"^To:.*me@example.com")? {
//!     email.deliver_to_maildir("/my/maildir/path")?;
//! }
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```
//!
//! # Processing and filtering the email with external programs
//!
//! Use the [Email::filter](struct.Email.html#method.filter) and
//! [Email::from_stdin_filtered](struct.Email.html#method.from_stdin_filtered)
//! methods to filter the email, in both cases creating a new email.
//!
//! ```no_run
//! use mda::Email;
//! // Filtering directly from stdin is more efficient.
//! let email = Email::from_stdin_filtered(&["bogofilter", "-ep"])?;
//! let bogosity = email.header_field("X-Bogosity").unwrap_or("");
//! if bogosity.contains("Spam, tests=bogofilter") {
//!     email.deliver_to_maildir("/my/spam/path")?;
//! }
//! // We can also filter at any other time.
//! let email = email.filter(&["bogofilter", "-ep"])?;
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```
//!
//! To perform more general processing use the
//! [Email::process](struct.Email.html#method.process)
//! method:
//!
//! ```no_run
//! use mda::Email;
//! let email = Email::from_stdin()?;
//! let output = email.process(&["bogofilter"])?;
//! if let Some(0) = output.status.code() {
//!     email.deliver_to_maildir("/my/spam/path")?;
//! }
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```
//!
//! # Access to byte data
//!
//! Use the [Email::header](struct.Email.html#method.header),
//! [Email::body](struct.Email.html#method.body),
//! [Email::data](struct.Email.html#method.data) methods to access the
//! normalized byte data of the header, body and whole email respectively.
//!
//! Normalization involves ensuring header fields are in single lines, decoding
//! text parts of the message that use some kind of transfer encoding (e.g.,
//! base64), and converting all text to UTF-8 character encoding.
//!
//! If for some reason you need access to non-normalized data use
//! [Email::raw_data](struct.Email.html#method.raw_data).
//!
//! ```no_run
//! use std::str;
//! use mda::Email;
//! let email = Email::from_stdin()?;
//! let body_str = String::from_utf8_lossy(email.header());
//!
//! if body_str.contains("FREE BEER") {
//!     email.deliver_to_maildir("/my/spam/path")?;
//! }
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```
//!
//! # Decide delivery durability vs speed trade-off
//!
//! Use the [Email::set_delivery_durability](struct.Email.html#method.set_delivery_durability)
//! to decide which [DeliveryDurability](enum.DeliveryDurability.html) method to use.
//! By default the most durable (but also slower) method is used.
//!
//! ```no_run
//! use mda::{Email, DeliveryDurability};
//! let mut email = Email::from_stdin()?;
//! email.set_delivery_durability(DeliveryDurability::FileSyncOnly);
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```

mod deliver;
mod regex;
mod processing;
mod normalize;
mod decode;

use std::io;
use std::io::prelude::*;
use std::path::{PathBuf, Path};
use std::sync:: {Arc, Mutex, RwLock};
use std::collections::HashMap;

use deliver::{Maildir, EmailFilenameGenerator};
use normalize::normalize_email;

pub use crate::regex::EmailRegex;

pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

fn find_empty_line(data: &[u8]) -> Option<usize> {
    data.windows(2).position(|w| w[0]== b'\n' && (w[1] == b'\n' || w[1] == b'\r'))
}

/// The method to use to try to guarantee durable email delivery.
#[derive(PartialEq, Copy, Clone)]
pub enum DeliveryDurability {
    /// Perform both file and directory syncing during delivery.
    /// This is the default delivery durability method.
    FileAndDirSync,
    /// Perform only file sync during delivery. This method is
    /// potentially much faster, and is used by many existing
    /// MDAs, but, depending on the used filesystem, may not
    /// provide the required delivery durability guarantees.
    FileSyncOnly,
}

/// A representation of an email.
pub struct Email {
    data: Vec<u8>,
    normalized_data: Vec<u8>,
    body_index: usize,
    deliver_path: RwLock<Option<PathBuf>>,
    fields: HashMap<String, Vec<String>>,
    email_filename_gen: Arc<Mutex<EmailFilenameGenerator>>,
    delivery_durability: DeliveryDurability,
}

impl Email {
    /// Creates an `Email` by reading data from stdin.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use mda::Email;
    /// let email = Email::from_stdin()?;
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn from_stdin() -> Result<Self> {
        let stdin = io::stdin();
        let mut data = Vec::new();
        stdin.lock().read_to_end(&mut data)?;
        Email::from_vec(data)
    }

    /// Creates an `Email` by using data passed in a `Vec<u8>`.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use mda::Email;
    /// let email = Email::from_vec(vec![1, 2, 3])?;
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn from_vec(data: Vec<u8>) -> Result<Self> {
        let (normalized_data, fields) = normalize_email(&data);
        let body_index = find_empty_line(&normalized_data).unwrap_or(normalized_data.len());
        let email_filename_gen = Arc::new(Mutex::new(EmailFilenameGenerator::new()));

        Ok(
            Email{
                data: data,
                normalized_data: normalized_data,
                body_index: body_index,
                deliver_path: RwLock::new(None),
                fields: fields,
                email_filename_gen: email_filename_gen,
                delivery_durability: DeliveryDurability::FileAndDirSync,
            }
        )
    }

    /// Sets the durability method for delivery of this email.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use mda::{DeliveryDurability, Email};
    /// let mut email = Email::from_stdin()?;
    /// email.set_delivery_durability(DeliveryDurability::FileSyncOnly);
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn set_delivery_durability(&mut self, delivery_durability: DeliveryDurability) {
        self.delivery_durability = delivery_durability;
    }

    /// Returns the value of a header field, if present. If a field occurs
    /// multiple times, the value of the first occurrence is returned.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use mda::Email;
    /// let email = Email::from_stdin()?;
    /// let to = email.header_field("To").unwrap_or("");
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn header_field(&self, name: &str) -> Option<&str> {
        self.fields.get(&name.to_lowercase()).map(|v| v[0].as_str())
    }

    /// Returns all names of header fields found in the Email
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use mda::Email;
    /// let email = Email::from_stdin()?;
    /// for name in email.header_field_names() {
    ///     println!("{}: {:?}", name, email.header_field(name));
    /// }
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn header_field_names(&self) -> Vec<&str> {
        self.fields.keys().map(::std::ops::Deref::deref).collect()
    }

    /// Returns the values from all occurrences of a header field, if present.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use mda::Email;
    /// let email = Email::from_stdin()?;
    /// if let Some(all_received) = email.header_field_all_occurrences("Received") {
    ///     // process all_received
    /// }
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn header_field_all_occurrences(&self, name: &str) -> Option<&Vec<String>> {
        self.fields.get(&name.to_lowercase()).map(|v| v)
    }

    /// Delivers the email to the specified maildir. If the maildir isn't
    /// present it is created.
    ///
    /// The first delivery of an email involves writing the email data to
    /// the target file, whereas subsequent deliveries try to use a hard link
    /// to the first delivery, falling back to a normal write if needed.
    ///
    /// The email is delivered durably by syncing both the file and the
    /// associated directories (`DeliveryDurability::FileAndDirSync`),
    /// unless a different durability method is specified with
    /// `set_delivery_durability`.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use mda::Email;
    /// let email = Email::from_stdin()?;
    /// email.deliver_to_maildir("/path/to/maildir/")?;
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn deliver_to_maildir(&self, path: impl AsRef<Path>) -> Result<PathBuf> {
        self.deliver_to_maildir_path(path.as_ref())
    }

    fn deliver_to_maildir_path(&self, path: &Path) -> Result<PathBuf> {
        let maildir = Maildir::open_or_create(&path, self.email_filename_gen.clone())?;

        if let Some(deliver_path) = self.deliver_path.read().unwrap().as_ref() {
            let email_path_result =
                maildir.deliver_with_hard_link(
                    deliver_path,
                    self.delivery_durability);

            if email_path_result.is_ok() {
                return email_path_result;
            }
        }

        let email_path = maildir.deliver(&self.data, self.delivery_durability)?;

        *self.deliver_path.write().unwrap() = Some(email_path.clone());

        Ok(email_path)
    }

    /// Returns whether the email has been delivered to at least one maildir.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use mda::Email;
    /// let email = Email::from_stdin()?;
    /// if !email.has_been_delivered() {
    ///     email.deliver_to_maildir("/fallback/maildir/")?;
    /// }
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn has_been_delivered(&self) -> bool {
        self.deliver_path.read().unwrap().is_some()
    }

    /// Provides access to the normalized email byte data.
    pub fn data(&self) -> &[u8] {
        &self.normalized_data
    }

    /// Provides access to the normalized email header byte data.
    pub fn header(&self) -> &[u8] {
        &self.normalized_data[..self.body_index]
    }

    /// Provides access to the normalized email body byte data.
    pub fn body(&self) -> &[u8] {
        &self.normalized_data[self.body_index..]
    }

    /// Provides access to the raw (non-normalized) email byte data.
    pub fn raw_data(&self) -> &[u8] {
        &self.data
    }
}

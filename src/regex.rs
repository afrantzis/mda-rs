// Copyright 2019 Alexandros Frantzis
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
//
// SPDX-License-Identifier: MPL-2.0

//! Convenience functionality for regex searches of email data.

use std::str;

use regex::bytes::{RegexBuilder, RegexSetBuilder, SetMatches, Captures};

use crate::Result;

/// Trait providing convenience methods for regular expression searching
/// in emails. The trait methods can be use with the byte data returned by
/// the `Email::header`, `Email::body` and `Email::data` methods.
///
/// This trait treats and searches the email contents as bytes. The regular
/// expression parsing is configured for case-insensitive and multi-line
/// search (i.e., `^` and `$` match beginning and end of lines respectively).
///
/// In addition to the single regular expression searching, a method for
/// matching regular expression sets is provided. This can be more
/// efficient than matching multiple regular expressions independently.
///
/// All the trait methods will fail if the regular expression is
/// invalid, or the searched email data isn't valid utf-8.
pub trait EmailRegex {
    /// Returns whether the contents match a regular expression.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use mda::{Email, EmailRegex};
    /// let email = Email::from_stdin()?;
    /// if email.header().search(r"^To:.*me@example.com")? {
    ///     email.deliver_to_maildir("/my/maildir/path")?;
    /// }
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    fn search(&self, regex: &str) -> Result<bool>;

    /// Returns the capture groups matched from a regular expression.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use std::path::Path;
    /// use mda::{Email, EmailRegex};
    /// let email = Email::from_stdin()?;
    /// if let Some(captures) = email.header().search_with_captures(r"^X-Product: name=(\w+)")? {
    ///     let name = std::str::from_utf8(captures.get(1).unwrap().as_bytes()).unwrap();
    ///     email.deliver_to_maildir(Path::new("/my/maildir/").join(name))?;
    /// }
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    fn search_with_captures(&self, regex: &str) -> Result<Option<Captures>>;

    /// Returns the matches from a set of regular expression. This can be
    /// more efficient than matching multiple regular expressions independently.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use mda::{Email, EmailRegex};
    /// let email = Email::from_stdin()?;
    /// let matched_sets = email.header().search_set(
    ///     &[
    ///         r"^To: confidential <confidential@example.com>",
    ///         r"^X-Confidential: true",
    ///     ]
    /// )?;
    /// if matched_sets.matched_any() {
    ///     email.deliver_to_maildir("/my/mail/confidential/")?;
    /// }
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    fn search_set(&self, regex_set: &[&str]) -> Result<SetMatches>;
}

impl EmailRegex for &[u8] {
    fn search(&self, regex: &str) -> Result<bool> {
        Ok(
            RegexBuilder::new(regex)
                .multi_line(true)
                .case_insensitive(true)
                .build()?
                .is_match(self)
        )
    }

    fn search_with_captures(&self, regex: &str) -> Result<Option<Captures>> {
        Ok(
            RegexBuilder::new(regex)
                .multi_line(true)
                .case_insensitive(true)
                .build()?
                .captures(self)
        )
    }

    fn search_set(&self, regex_set: &[&str]) -> Result<SetMatches> {
        Ok(
            RegexSetBuilder::new(regex_set)
                .multi_line(true)
                .case_insensitive(true)
                .build()?
                .matches(self)
        )
    }
}

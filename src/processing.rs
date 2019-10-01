// Copyright 2019 Alexandros Frantzis
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
//
// SPDX-License-Identifier: MPL-2.0

//! Email processing and filtering.

use std::io::Write;
use std::process::{Command, Output, Stdio};

use crate::{Email, Result};

impl Email {
    /// Filters the contents of the email using an external command,
    /// returning a new email with the filtered contents.
    ///
    /// The command is expected to be provided as a `&str` array, with the
    /// first element being the command name and the remaining elements the
    /// command arguments.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use mda::Email;
    /// let email = Email::from_stdin()?;
    /// let email = email.filter(&["bogofilter", "-ep"])?;
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn filter(&self, cmd: &[&str]) -> Result<Email> {
        Email::from_vec(self.process(cmd)?.stdout)
    }

    /// Process the contents of the email using an external command,
    /// returning a `std::process::Output` for the executed command.
    ///
    /// The command is expected to be provided as a `&str` array, with the
    /// first element being the command name and the remaining elements the
    /// command arguments.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use mda::Email;
    /// let email = Email::from_stdin()?;
    /// let output = email.process(&["bogofilter"])?;
    /// if let Some(0) = output.status.code() {
    ///     email.deliver_to_maildir("/my/spam/path")?;
    /// }
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn process(&self, cmd: &[&str]) -> Result<Output> {
        let mut child =
            Command::new(cmd[0])
                .args(&cmd[1..])
                .stdin(Stdio::piped())
                .stdout(Stdio::piped())
                .spawn()?;

        child.stdin
            .as_mut()
            .ok_or("Failed to write to stdin")?
            .write_all(&self.data)?;

        Ok(child.wait_with_output()?)
    }

    /// Creates an `Email` by filtering the contents from stdin.
    ///
    /// This can be more efficient than creating an `Email` from stdin and
    /// filtering separately, since it can avoid an extra data copy.
    ///
    /// The command is expected to be provided as a `&str` array, with the
    /// first element being the command name and the remaining elements the
    /// command arguments.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use mda::Email;
    /// let email = Email::from_stdin_filtered(&["bogofilter", "-ep"])?;
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn from_stdin_filtered(cmd: &[&str]) -> Result<Self> {
        let output =
            Command::new(cmd[0])
                .args(&cmd[1..])
                .stdin(Stdio::inherit())
                .output()?;

        Email::from_vec(output.stdout)
    }
}

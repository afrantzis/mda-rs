// Copyright 2019 Alexandros Frantzis
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
//
// SPDX-License-Identifier: MPL-2.0

//! Writes out the normalized form of an email.

use std::io::{self, Write};
use mda::{Email, Result};

fn main() -> Result<()> {
    let email = Email::from_stdin()?;
    io::stdout().lock().write_all(email.data())?;
    Ok(())
}

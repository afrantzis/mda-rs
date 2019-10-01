// Copyright 2019 Alexandros Frantzis
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
//
// SPDX-License-Identifier: MPL-2.0

//! An example of a custom MDA.

use std::path::PathBuf;

use mda::{Email, EmailRegex, Result, DeliveryDurability};

fn main() -> Result<()> {
    // Just some random path to make it highly unlikely that this example will
    // indvertently mess up something.
    let root = PathBuf::from("/tmp/my-personal-mail-96f29eb6375cfa37");

    // If we are sure bogofilter is available, the below can be better written as:
    // let mut email = Email::from_stdin_filtered(&["/usr/bin/bogofilter", "-ep"])?;
    let mut email = Email::from_stdin()?;
    if let Ok(new_email) = email.filter(&["/usr/bin/bogofilter", "-ep"]) {
        email = new_email;
    }

    // Quicker (but possibly less durable) delivery.
    email.set_delivery_durability(DeliveryDurability::FileSyncOnly);

    let from = email.header_field("From").unwrap_or("");
    let bogosity = email.header_field("X-Bogosity").unwrap_or("");

    if bogosity.contains("Spam, tests=bogofilter") ||
       from.contains("@banneddomain.com") {
        email.deliver_to_maildir(root.join("spam"))?;
        return Ok(());
    }

    let cc = email.header_field("Cc").unwrap_or("");
    let to = email.header_field("To").unwrap_or("");

    if to.contains("myworkemail@example.com") ||
       cc.contains("myworkemail@example.com") {
        if email.body().search("URGENCY RATING: (CRITICAL|URGENT)")? {
            email.deliver_to_maildir(root.join("inbox/myemail/urgent"))?;
        } else {
            email.deliver_to_maildir(root.join("inbox/myemail/normal"))?;
        }
        return Ok(());
    }

    email.deliver_to_maildir(root.join("inbox/unsorted"))?;

    Ok(())
}

// Copyright 2022 System76 <info@system76.com>
// SPDX-License-Identifier: MPL-2.0

use std::ffi::OsStr;

pub(crate) fn xdg_open<Url: AsRef<OsStr> + Send + 'static>(url: Url) {
    std::thread::spawn(move || {
        let _ = std::process::Command::new("xdg-open")
            .arg(url.as_ref())
            .status();
    });
}

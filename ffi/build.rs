// Copyright 2022 System76 <info@system76.com>
// SPDX-License-Identifier: MPL-2.0

use std::{env, fs::File, io::Write, path::PathBuf};

fn main() {
    cdylib_link_lines::metabuild();

    let target_dir = PathBuf::from("../target");

    let pkg_config = format!(
        include_str!("../data/pop_analytics_panel.pc.in"),
        name = "pop_analytics_panel",
        description = env::var("CARGO_PKG_DESCRIPTION").unwrap(),
        version = env::var("CARGO_PKG_VERSION").unwrap()
    );

    File::create(target_dir.join("pop_analytics_panel.pc.stub"))
        .expect("failed to create pc.stub")
        .write_all(pkg_config.as_bytes())
        .expect("failed to write pc.stub");
}

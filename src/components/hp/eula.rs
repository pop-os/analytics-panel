// Copyright 2022 System76 <info@system76.com>
// SPDX-License-Identifier: MPL-2.0

use crate::fl;

use gtk::prelude::*;

use std::{fs::OpenOptions, io::Write, os::unix::fs::OpenOptionsExt, process::Command};

// Create TCO certified desktop file on supported hardware
fn tco_certified() -> Result<(), String> {
    let desktop_dir = match dirs::desktop_dir() {
        Some(some) => some,
        None => return Err(String::from("failed to get desktop directory")),
    };

    let shortcut_path = desktop_dir.join("hp-tco-certified.desktop");
    let shortcut_data = include_bytes!("../../../data/hp-tco-certified.desktop");
    OpenOptions::new()
        .create(true)
        .write(true)
        .mode(0o775)
        .open(&shortcut_path)
        .map_err(|err| format!("failed to open {}: {}", shortcut_path.display(), err))?
        .write_all(shortcut_data)
        .map_err(|err| format!("failed to write {}: {}", shortcut_path.display(), err))?;

    let status = Command::new("gio")
        .arg("set")
        .arg(&shortcut_path)
        .arg("metadata::trusted")
        .arg("true")
        .status()
        .map_err(|err| {
            format!(
                "failed to set {} as trusted: {}",
                shortcut_path.display(),
                err
            )
        })?;
    if !status.success() {
        return Err(format!(
            "failed to set {} as trusted: {}",
            shortcut_path.display(),
            status
        ));
    }

    Ok(())
}

pub struct Model {}

#[derive(relm_derive::Msg)]
pub enum Message {}

#[relm_derive::widget]
impl relm::Widget for Widget {
    fn init_view(&mut self) {
        self.widgets.title.style_context().add_class("h1");

        self.widgets
            .title
            .set_markup(&format!("<b>{}</b>", fl!("eula")));

        if let Some(buffer) = self.widgets.text.buffer() {
            let eula_markup = include_str!("../../../data/hp-eula.md");
            buffer.insert_markup(&mut buffer.start_iter(), eula_markup);
        }

        match tco_certified() {
            Ok(()) => (),
            Err(err) => {
                eprintln!("failed to create TCO certified link: {}", err);
            }
        }
    }

    fn model(_relm: &relm::Relm<Self>, _: ()) -> Model {
        Model {}
    }

    fn update(&mut self, _message: Message) {}

    relm::view! {
        gtk::Box {
            halign: gtk::Align::Center,
            margin_bottom: 36,
            margin_top: 36,
            orientation: gtk::Orientation::Vertical,
            spacing: 24,

            #[name="title"]
            gtk::Label {},

            gtk::Label {
                label: &fl!("eula-prompt"),
                line_wrap: true,
                max_width_chars: 80,
            },

            gtk::ScrolledWindow {
                hscrollbar_policy: gtk::PolicyType::Never,
                min_content_height: 400,
                vscrollbar_policy: gtk::PolicyType::Always,

                #[name="text"]
                gtk::TextView {
                    cursor_visible: false,
                    editable: false,
                    wrap_mode: gtk::WrapMode::Word,
                    // These are actually setting paddings, not margins
                    left_margin: 24,
                    right_margin: 24,
                    top_margin: 16,
                },
            },
        }
    }
}

// Copyright 2022 System76 <info@system76.com>
// SPDX-License-Identifier: MPL-2.0

pub mod dialog;
pub mod panel;
pub mod summary;

use gtk::prelude::*;
use relm::StreamHandle;

pub fn message_dialog(
    window: &gtk::Window,
    stream: StreamHandle<panel::Message>,
    width: i32,
) -> relm::Component<dialog::Widget> {
    let dialog = gtk::MessageDialog::builder()
        .transient_for(window)
        .modal(true)
        .decorated(true)
        .resizable(false)
        .default_width(width)
        .build();

    // NOTE: Hack to get a dialog without padding around the edge of the window.
    for child in dialog.children() {
        if child.style_context().has_class("dialog-vbox") {
            if let Ok(vbox) = child.downcast::<gtk::Box>() {
                for child in vbox.children() {
                    if child.style_context().has_class("horizontal") {
                        vbox.remove(&child);
                    }
                }
            }
        }
    }

    let component = relm::init::<dialog::Widget>((dialog.clone(), stream)).unwrap();

    dialog.content_area().add(component.widget());

    component
}

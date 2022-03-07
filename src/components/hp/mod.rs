// Copyright 2022 System76 <info@system76.com>
// SPDX-License-Identifier: MPL-2.0

pub mod dialog;
pub mod panel;
pub mod summary;

pub use self::panel::Widget as Panel;
pub use self::summary::Widget as Summary;

use gtk::prelude::*;
use relm::StreamHandle;

use panel::Message as PanelMessage;
use relm::Sender;
use std::fs;
use std::time::Duration;

pub enum Status {
    Success,
    Failed,
}

fn analytics_dir() -> Option<std::path::PathBuf> {
    if let Some(home_dir) = dirs::home_dir() {
        return Some(home_dir.join(".hp-analytics"));
    }

    None
}

/// Check if analytics data is held remotely.
async fn check(sender: Sender<PanelMessage>) -> Status {
    Status::Success
}

/// Delete analytics data currently held remotely.
async fn delete(sender: Sender<PanelMessage>) {
    if let Status::Failed = check(sender.clone()).await {
        return;
    }

    println!("TODO: deleting analytics data");
}

/// Check if analytics data exists that can be deleted.
async fn delete_requested(sender: Sender<PanelMessage>) {
    if let Status::Failed = check(sender.clone()).await {
        return;
    }

    let _ = sender.send(PanelMessage::DeleteDialog);
}

/// Download analytics data currently held remotely.
async fn download(sender: Sender<PanelMessage>) {
    if let Status::Failed = check(sender.clone()).await {
        return;
    }

    println!("TODO: downloading analytics data");

    if let Some(analytics_dir) = analytics_dir() {
        let _ = fs::create_dir_all(&analytics_dir);

        let mut p = 0.0;

        for _ in 0..10 {
            p += 0.1;
            async_std::task::sleep(Duration::from_millis(500)).await;
            let _ = sender.send(PanelMessage::DownloadProgress(p));
        }
    }

    let _ = sender.send(PanelMessage::DownloadComplete);
}

/// Toggle collection of analytics data.
async fn toggle(enable: bool) {
    println!("TODO: analytics toggled");
}

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

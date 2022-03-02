// Copyright 2022 System76 <info@system76.com>
// SPDX-License-Identifier: MPL-2.0

#[macro_use]
extern crate cascade;

pub mod components;

mod clamp;
mod localize;
mod misc;

pub use self::localize::localizer;

use components::panel::Message as PanelMessage;
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

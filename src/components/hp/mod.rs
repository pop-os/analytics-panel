// Copyright 2022 System76 <info@system76.com>
// SPDX-License-Identifier: MPL-2.0

pub mod dialog;
pub mod eula;
pub mod initial_setup;
pub mod panel;
pub mod summary;

pub use self::eula::Widget as Eula;
pub use self::initial_setup::Widget as InitialSetup;
pub use self::panel::Widget as Panel;
pub use self::summary::Widget as Summary;

use flate2::read::GzDecoder;
use gtk::prelude::*;
use relm::StreamHandle;

use panel::Message as PanelMessage;
use relm::Sender;
use std::collections::HashMap;
use std::fs;
use std::io::{self, Read};
use std::str::FromStr;

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
async fn delete(sender: Sender<PanelMessage>) -> Result<(), hp_vendor_client::Error> {
    if let Status::Failed = check(sender.clone()).await {
        return Ok(());
    }

    let pool = glib::ThreadPool::shared(None).unwrap();
    pool.push_future(move || hp_vendor_client::delete_and_disable())
        .unwrap()
        .await
}

/// Check if analytics data exists that can be deleted.
async fn delete_requested(sender: Sender<PanelMessage>) {
    if let Status::Failed = check(sender.clone()).await {
        return;
    }

    let _ = sender.send(PanelMessage::DeleteDialog);
}

struct ReadCounter<T: Read, F: FnMut(usize)>(T, F);

impl<T: Read, F: FnMut(usize)> Read for ReadCounter<T, F> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let count = self.0.read(buf)?;
        (self.1)(count);
        Ok(count)
    }
}

/// Download analytics data currently held remotely.
async fn download(sender: Sender<PanelMessage>) -> Result<(), hp_vendor_client::Error> {
    if let Status::Failed = check(sender.clone()).await {
        return Ok(());
    }

    let pool = glib::ThreadPool::shared(None).unwrap();
    pool.push_future(move || {
        let _ = sender.send(PanelMessage::DownloadProgress(0.));

        if let Some(analytics_dir) = analytics_dir() {
            let _ = fs::create_dir_all(&analytics_dir);

            let file = fs::File::create(analytics_dir.join("data.json"))?;

            let mut download = hp_vendor_client::download(hp_vendor_client::DownloadFormat::GZip)?;
            let length = download.length();

            // Keep track of how many bytes have been read
            let mut bytes = 0;
            let sender = &sender;
            let read_counter = ReadCounter(&mut download, move |count| {
                bytes += count;
                let _ = sender.send(PanelMessage::DownloadProgress(bytes as f32 / length as f32));
            });

            // Decompress and pretty print
            let json: serde_json::Value = serde_json::from_reader(GzDecoder::new(read_counter))?;
            serde_json::to_writer_pretty(file, &json)?;

            download.wait()?
        }

        let _ = sender.send(PanelMessage::DownloadComplete);

        Ok(())
    })
    .unwrap()
    .await
}

pub fn purpose_for_locale(
    mut purposes: HashMap<String, hp_vendor_client::DataCollectionPurpose>,
) -> (String, String, hp_vendor_client::DataCollectionPurpose) {
    let locale = locale_config::Locale::current();
    let mut region = None;
    for i in locale.tags_for("messages") {
        if let Ok(identifier) = unic_langid::LanguageIdentifier::from_str(&i.to_string()) {
            let language = identifier.language.as_str();
            if region.is_none() {
                if let Some(new_region) = identifier.region {
                    region = Some(new_region.as_str().to_owned());
                }
            }
            if let Some(purpose) = purposes.remove(language) {
                // Is this a reasonable default?
                let region = region.unwrap_or_else(String::new);
                return (language.to_string(), region, purpose);
            }
        }
    }
    // Assume `en` is always a valid locale, and use as fallback
    let region = region.unwrap_or_else(String::new);
    ("en".to_string(), region, purposes.remove("en").unwrap())
}

pub struct PurposeAndOpt {
    language: String,
    region: String,
    purpose: hp_vendor_client::DataCollectionPurpose,
    opted: bool,
}

async fn purpose_and_opted(fetch: bool) -> Result<PurposeAndOpt, hp_vendor_client::Error> {
    let pool = glib::ThreadPool::shared(None).unwrap();
    pool.push_future(move || {
        let res = hp_vendor_client::purposes(fetch)?;
        let opted = res.consent.is_some();
        let (language, region, purpose) = purpose_for_locale(res.purposes);
        Ok(PurposeAndOpt {
            language,
            region,
            purpose,
            opted,
        })
    })
    .unwrap()
    .await
}

/// Toggle collection of analytics data.
async fn toggle(
    purpose: (String, String, String, String),
    enable: bool,
) -> Result<(), hp_vendor_client::Error> {
    let pool = glib::ThreadPool::shared(None).unwrap();
    pool.push_future(move || {
        if enable {
            let (language, region, purpose_id, version) = purpose;
            hp_vendor_client::consent(&language, &region, &purpose_id, &version)
        } else {
            hp_vendor_client::disable()
        }
    })
    .unwrap()
    .await
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

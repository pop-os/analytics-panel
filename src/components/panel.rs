// Copyright 2022 System76 <info@system76.com>
// SPDX-License-Identifier: MPL-2.0

use crate::fl;
use gtk::prelude::*;
use relm::Channel;

use super::dialog::{self, Variant};

pub struct Model {
    _channel: Channel<Message>,
    dialog: relm::Component<super::dialog::Widget>,
    operation: Option<Operation>,
    background: relm::Sender<Message>,
}

#[derive(Debug)]
pub enum Operation {
    DeleteData,
    DownloadData,
}

#[derive(relm_derive::Msg)]
pub enum Message {
    Delete,
    DeleteDialog,
    DeleteRequest,
    DisplaySample,
    Download,
    DownloadComplete,
    DownloadProgress(f32),
    OpenDataDir,
    OpenWebpage(&'static str),
    Toggle,
    TryAgain,
}

impl Widget {
    fn delete_data(&self) {
        let tx = self.model.background.clone();
        glib::MainContext::default().spawn_local(crate::delete(tx));
    }

    fn delete_data_request(&self) {
        let tx = self.model.background.clone();
        glib::MainContext::default().spawn_local(crate::delete_requested(tx));
    }

    fn download_data(&self) {
        let tx = self.model.background.clone();
        glib::MainContext::default().spawn_local(crate::download(tx));
    }

    fn toggle_analytics(&mut self, enable: bool) {
        let tx = self.model.background.clone();
        glib::MainContext::default().spawn_local(crate::toggle(tx, enable));
    }
}

#[relm_derive::widget]
impl relm::Widget for Widget {
    fn init_view(&mut self) {
        cascade! {
            gtk::SizeGroup::new(gtk::SizeGroupMode::Both);
            ..add_widget(&self.widgets.download_button);
            ..add_widget(&self.widgets.delete_button);
        };
    }

    fn update(&mut self, message: Message) {
        match message {
            Message::Toggle => self.toggle_analytics(self.widgets.toggle.is_active()),

            Message::Delete => {
                self.delete_data();
            }

            Message::DeleteRequest => {
                self.model.operation = Some(Operation::DeleteData);
                self.delete_data_request();
            }

            Message::DeleteDialog => {
                self.model
                    .dialog
                    .stream()
                    .emit(dialog::Message::Update(Variant::DeleteData));
                self.model.dialog.widget().show();
            }

            Message::Download => {
                self.widgets.download_button.set_sensitive(false);
                self.model.operation = Some(Operation::DownloadData);
                self.download_data();
            }

            Message::DownloadProgress(progress) => {
                self.widgets
                    .download_progress_bar
                    .set_fraction(progress as f64);
                self.widgets
                    .download_stack
                    .set_visible_child(&self.widgets.download_progress);
            }

            Message::DownloadComplete => {
                self.model.operation = None;
                self.model
                    .dialog
                    .emit(dialog::Message::Update(dialog::Variant::DataDownloaded));
                self.widgets
                    .download_stack
                    .set_visible_child(&self.widgets.download_label);
            }

            Message::DisplaySample => {}

            Message::OpenDataDir => {
                if let Some(analytics_dir) = crate::analytics_dir() {
                    crate::misc::xdg_open(analytics_dir);
                }
            }

            Message::OpenWebpage(url) => {
                crate::misc::xdg_open(url);
            }

            Message::TryAgain => match self.model.operation {
                Some(Operation::DeleteData) => self.delete_data(),
                Some(Operation::DownloadData) => self.download_data(),
                None => (),
            },
        }
    }

    fn model(relm: &relm::Relm<Self>, window: gtk::Window) -> Model {
        let stream = relm.stream().clone();

        let (_channel, sender) = relm::Channel::new(move |message| {
            stream.emit(message);
        });

        Model {
            _channel,
            dialog: super::message_dialog(&window, relm.stream().clone(), 480),
            operation: None,
            background: sender,
        }
    }

    relm::view! {
        gtk::Box {
            margin_bottom: 48,
            margin_top: 48,
            halign: gtk::Align::Center,
            orientation: gtk::Orientation::Vertical,

            gtk::Label {
                xalign: 0.0,
                label: &fl!("hp-analytics-description"),
                margin_bottom: 24,
            },

            gtk::LinkButton {
                halign: gtk::Align::Start,
                label: &fl!("data-sample"),
                margin_bottom: 24,
                activate_link => (Message::DisplaySample, gtk::Inhibit(false)),
            },

            gtk::LinkButton {
                halign: gtk::Align::Start,
                label: &fl!("hp-privacy-policy"),
                activate_link => (Message::OpenWebpage(""), gtk::Inhibit(false)),
            },

            gtk::LinkButton {
                halign: gtk::Align::Start,
                label: &fl!("pop-privacy-policy"),
                activate_link => (Message::OpenWebpage(""), gtk::Inhibit(false)),
                margin_bottom: 24,
            },

            gtk::ListBox {
                selection_mode: gtk::SelectionMode::None,

                InfoBox {
                    gtk::Box {
                        orientation: gtk::Orientation::Vertical,
                        halign: gtk::Align::Start,
                        hexpand: true,
                        valign: gtk::Align::Center,

                        gtk::Label {
                            ellipsize: gtk::pango::EllipsizeMode::End,
                            label: &fl!("hp-analytics-toggle-header"),
                            xalign: 0.0,
                        },

                        gtk::Label {
                            ellipsize: gtk::pango::EllipsizeMode::End,
                            label: &fl!("hp-analytics-toggle-description"),
                            xalign: 0.0,
                        }
                    },

                    #[name="toggle"]
                    gtk::Switch {
                        valign: gtk::Align::Center,
                        changed_active => Message::Toggle
                    }
                },

                InfoBox {
                    gtk::Label {
                        halign: gtk::Align::Start,
                        hexpand: true,
                        valign: gtk::Align::Center,
                        ellipsize: gtk::pango::EllipsizeMode::End,
                        label: &fl!("delete-data-option"),
                        xalign: 0.0,
                    },

                    #[name="delete_button"]
                    gtk::Button {
                        label: &fl!("delete"),
                        clicked => Message::DeleteRequest,
                    }
                },

                InfoBox {
                    #[name="download_stack"]
                    gtk::Stack {
                        halign: gtk::Align::Start,
                        hexpand: true,
                        valign: gtk::Align::Center,

                        #[name="download_label"]
                        gtk::Label {
                            ellipsize: gtk::pango::EllipsizeMode::End,
                            label: &fl!("download-option"),
                            xalign: 0.0,
                        },

                        #[name="download_progress"]
                        gtk::Box {
                            orientation: gtk::Orientation::Vertical,

                            gtk::Label {
                                ellipsize: gtk::pango::EllipsizeMode::End,
                                label: &fl!("download-option-downloading"),
                                xalign: 0.0,
                            },

                            #[name="download_progress_bar"]
                            gtk::ProgressBar {}
                        }
                    },

                    #[name="download_button"]
                    gtk::Button {
                        label: &fl!("download"),
                        clicked => Message::Download
                    }
                }
            }
        }
    }
}

#[relm_derive::widget]
impl relm::Widget for InfoBox {
    fn model(_: ()) {}

    fn update(&mut self, _: ()) {}

    relm::view! {
        #[container]
        gtk::Box {
            orientation: gtk::Orientation::Horizontal,
            margin_start: 20,
            margin_end: 20,
            margin_top: 8,
            margin_bottom: 8,
            spacing: 24
        }
    }
}

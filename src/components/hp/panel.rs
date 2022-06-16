// Copyright 2022 System76 <info@system76.com>
// SPDX-License-Identifier: MPL-2.0

use crate::fl;
use gtk::prelude::*;
use relm::Channel;
use std::{future::Future, process::Command};

use super::dialog::{self, Variant};

const MODEL: &str = "HP Dev One";

fn header_func(row: &gtk::ListBoxRow, before: Option<&gtk::ListBoxRow>) {
    if before.is_none() {
        row.set_header(None::<&gtk::Widget>)
    } else if row.header().is_none() {
        row.set_header(Some(&cascade! {
            gtk::Separator::new(gtk::Orientation::Horizontal);
            ..show();
        }));
    }
}

pub struct Model {
    _channel: Channel<Message>,
    dialog: relm::Component<super::dialog::Widget>,
    summary: relm::Component<super::summary::Widget>,
    operation: Option<Operation>,
    background: relm::Sender<Message>,
    purpose: (String, String, String, String),
    purpose_statement: String,
    relm: relm::Relm<Widget>,
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
    DeleteComplete(bool),
    Download,
    DownloadComplete(bool),
    DownloadProgress(f32),
    OpenDataDir,
    OpenSupportPanel,
    PurposeAndOpt(super::PurposeAndOpt),
    Toggle,
    ToggleSensitive,
    TryAgain,
}

impl Widget {
    fn spawn_and_handle_err<F: Future<Output = Result<(), hp_vendor_client::Error>> + 'static>(
        &self,
        fut: F,
    ) {
        let stream = self.model.dialog.stream();
        glib::MainContext::default().spawn_local(async move {
            if let Err(err) = fut.await {
                if let hp_vendor_client::Error::Api(err) = &err {
                    if err.code == 404 && &err.endpoint == "DataDownload"
                        || &err.endpoint == "DataDelete"
                    {
                        stream.emit(dialog::Message::Update(dialog::Variant::NoDataFound));
                        return;
                    } else if err.code == 404 && &err.endpoint == "Token" {
                        stream.emit(dialog::Message::Update(dialog::Variant::NotEnrolled));
                        return;
                    } else if err.code == 400 && &err.endpoint == "Token" {
                        stream.emit(dialog::Message::Update(dialog::Variant::DeviceIdInvalid));
                        return;
                    }
                } else if let hp_vendor_client::Error::Reqwest(_message) = &err {
                    stream.emit(dialog::Message::Update(dialog::Variant::NoInternet));
                    return;
                }

                stream.emit(dialog::Message::Update(dialog::Variant::Error(err)));
            }
        });
    }

    fn delete_data(&self) {
        let tx = self.model.background.clone();
        self.spawn_and_handle_err(async move {
            let res = super::delete().await;
            tx.send(Message::DeleteComplete(res.is_ok()));
            res
        });
    }

    fn download_data(&self) {
        let tx = self.model.background.clone();
        self.spawn_and_handle_err(async move {
            let res = super::download(tx.clone()).await;
            let _ = tx.send(Message::DownloadComplete(res.is_ok()));
            res
        });
    }

    fn toggle_analytics(&mut self, purpose: (String, String, String, String), enable: bool) {
        self.widgets.toggle.set_sensitive(false);
        let tx = self.model.background.clone();
        self.spawn_and_handle_err(async move {
            let res = super::toggle(purpose, enable).await;
            tx.send(Message::ToggleSensitive);
            res
        });
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

        self.widgets.content.add(self.model.summary.widget());
        self.widgets
            .content
            .reorder_child(self.model.summary.widget(), 0);

        self.widgets.list_box.style_context().add_class("frame");

        let tx = self.model.background.clone();
        glib::MainContext::default().spawn_local(async move {
            if let Ok(purpose_and_opt) = super::purpose_and_opted(false).await {
                tx.send(Message::PurposeAndOpt(purpose_and_opt));
            } else {
                // TODO? Shouldn't happen if hp-vendor installed correctly
            }
        });
    }

    fn update(&mut self, message: Message) {
        match message {
            Message::Toggle => {
                self.toggle_analytics(self.model.purpose.clone(), self.widgets.toggle.is_active())
            }

            Message::ToggleSensitive => {
                self.widgets.toggle.set_sensitive(true);
            }

            Message::Delete => {
                self.widgets.delete_button.set_sensitive(false);
                self.delete_data();
            }

            Message::DeleteComplete(success) => {
                self.model.operation = None;
                self.widgets.delete_button.set_sensitive(true);
                if success {
                    let _lock = self.model.relm.stream().lock();
                    self.widgets.toggle.set_active(false);
                }
            }

            Message::DeleteDialog => {
                self.model.operation = Some(Operation::DeleteData);
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

            Message::DownloadComplete(success) => {
                self.model.operation = None;
                self.widgets.download_button.set_sensitive(true);
                if success {
                    self.model
                        .dialog
                        .emit(dialog::Message::Update(dialog::Variant::DataDownloaded));
                }
                self.widgets
                    .download_stack
                    .set_visible_child(&self.widgets.download_label);
            }

            Message::OpenDataDir => {
                if let Some(analytics_dir) = super::analytics_dir() {
                    crate::misc::xdg_open(analytics_dir);
                }
            }

            Message::OpenSupportPanel => {
                let _ = Command::new("gnome-control-center").arg("support").spawn();
            }

            Message::PurposeAndOpt(super::PurposeAndOpt {
                language,
                region,
                purpose,
                opted,
            }) => {
                {
                    let _lock = self.model.relm.stream().lock();
                    self.widgets.toggle.set_active(opted);
                }
                self.widgets.toggle.set_sensitive(true);
                self.model.purpose = (language, region, purpose.purpose_id, purpose.version);
                self.widgets.purpose_statement.set_text(&purpose.statement);
                self.model.purpose_statement = purpose.statement;
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

        let (language, region, purpose) =
            hp_vendor_client::purpose_for_locale(hp_vendor_client::static_purposes());

        Model {
            _channel,
            dialog: super::message_dialog(&window, relm.stream().clone(), 480),
            summary: relm::create_component::<super::summary::Widget>(()),
            operation: None,
            background: sender,
            purpose: (language, region, purpose.purpose_id, purpose.version),
            purpose_statement: purpose.statement,
            relm: relm.clone(),
        }
    }

    relm::view! {
        gtk::ScrolledWindow {
            hscrollbar_policy: gtk::PolicyType::Never,

            libhandy::Clamp {
                margin_top: 36,
                margin_bottom: 36,
                margin_start: 12,
                margin_end: 12,

                #[name="content"]
                gtk::Box {
                    spacing: 6,
                    halign: gtk::Align::Center,
                    orientation: gtk::Orientation::Vertical,

                    #[name="list_box"]
                    gtk::ListBox {
                        selection_mode: gtk::SelectionMode::None,
                        header_func: Some(Box::new(header_func)),

                        InfoBox {
                            gtk::Box {
                                orientation: gtk::Orientation::Vertical,
                                halign: gtk::Align::Start,
                                hexpand: true,
                                valign: gtk::Align::Center,

                                #[name="purpose_statement"]
                                gtk::Label {
                                    line_wrap: true,
                                    label: &self.model.purpose_statement,
                                    xalign: 0.0,
                                },
                            },

                            #[name="toggle"]
                            gtk::Switch {
                                valign: gtk::Align::Center,
                                sensitive: false,
                                changed_active => Message::Toggle
                            }
                        },

                        InfoBox {
                            gtk::Label {
                                halign: gtk::Align::Start,
                                hexpand: true,
                                valign: gtk::Align::Center,
                                line_wrap: true,
                                label: &fl!("delete-data-option", model=MODEL),
                                xalign: 0.0,
                            },

                            #[name="delete_button"]
                            gtk::Button {
                                label: &fl!("delete"),
                                clicked => Message::DeleteDialog,
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
                                    line_wrap: true,
                                    label: &fl!("download-option", model=MODEL),
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
    }
}

#[relm_derive::widget(pub)]
impl relm::Widget for InfoBox {
    fn model(_: ()) {}

    fn update(&mut self, _: ()) {}

    relm::view! {
        gtk::ListBoxRow {
            activatable: false,
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
}

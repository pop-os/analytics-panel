// Copyright 2022 System76 <info@system76.com>
// SPDX-License-Identifier: MPL-2.0

use crate::fl;
use gtk::prelude::*;
use relm::Channel;
use std::future::Future;

use super::dialog::{self, Variant};

pub struct Model {
    _channel: Channel<Message>,
    dialog: relm::Component<super::dialog::Widget>,
    summary: relm::Component<super::summary::Widget>,
    operation: Option<Operation>,
    background: relm::Sender<Message>,
    purpose: Option<(String, String, String, String)>,
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
    DeleteRequest,
    Download,
    DownloadComplete,
    DownloadProgress(f32),
    OpenDataDir,
    PurposeAndOpt(super::PurposeAndOpt),
    Toggle,
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
                stream.emit(dialog::Message::Update(dialog::Variant::Error(err)));
            }
        });
    }

    fn delete_data(&self) {
        let tx = self.model.background.clone();
        self.spawn_and_handle_err(super::delete(tx));
    }

    fn delete_data_request(&self) {
        let tx = self.model.background.clone();
        glib::MainContext::default().spawn_local(super::delete_requested(tx));
    }

    fn download_data(&self) {
        let tx = self.model.background.clone();
        self.spawn_and_handle_err(super::download(tx));
    }

    fn toggle_analytics(&mut self, purpose: (String, String, String, String), enable: bool) {
        self.spawn_and_handle_err(super::toggle(purpose, enable));
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

        use crate::clamp::BinClamp;
        self.widgets.root.bin_clamp(300, 600, 80);
        self.widgets.content.add(self.model.summary.widget());
        self.widgets
            .content
            .reorder_child(self.model.summary.widget(), 0);

        let tx = self.model.background.clone();
        glib::MainContext::default().spawn_local(async move {
            if let Ok(purpose_and_opt) = super::purpose_and_opted(true).await {
                tx.send(Message::PurposeAndOpt(purpose_and_opt));
            } else {
                // TODO? Shouldn't happen if hp-vendor installed correctly
            }
        });
    }

    fn update(&mut self, message: Message) {
        match message {
            Message::Toggle => {
                if let Some(purpose) = self.model.purpose.clone() {
                    self.toggle_analytics(purpose, self.widgets.toggle.is_active())
                }
            }

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

            Message::OpenDataDir => {
                if let Some(analytics_dir) = super::analytics_dir() {
                    crate::misc::xdg_open(analytics_dir);
                }
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
                self.model.purpose = Some((language, region, purpose.purpose_id, purpose.version));
                self.model
                    .summary
                    .emit(super::summary::Message::PurposeStatement(purpose.statement));
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
            summary: relm::create_component::<super::summary::Widget>(false),
            operation: None,
            background: sender,
            purpose: None,
            relm: relm.clone(),
        }
    }

    relm::view! {
        #[name="root"]
        gtk::ScrolledWindow {
            hscrollbar_policy: gtk::PolicyType::Never,

            #[name="content"]
            gtk::Box {
                margin_bottom: 48,
                margin_top: 48,
                halign: gtk::Align::Center,
                orientation: gtk::Orientation::Vertical,

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
                            sensitive: false,
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
}

#[relm_derive::widget(pub)]
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

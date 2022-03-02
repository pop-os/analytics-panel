// Copyright 2022 System76 <info@system76.com>
// SPDX-License-Identifier: MPL-2.0

use super::panel;
use crate::fl;

use concat_in_place::strcat;
use gtk::prelude::*;
use relm::StreamHandle;

pub struct AnalyticsDialog(gtk::MessageDialog);

pub struct Model {
    dialog: Option<gtk::MessageDialog>,
    variant: Option<Variant>,
    sender: StreamHandle<panel::Message>,
}

#[derive(relm_derive::Msg)]
pub enum Message {
    Accept,
    Close,
    Hide,
    Update(Variant),
}

#[derive(Debug)]
pub enum Variant {
    DataDownloaded,
    Deleted,
    DeleteData,
    NoDataFound,
    NoInternet,
}

impl Widget {
    fn close(&mut self) {
        if let Some(dialog) = self.model.dialog.take() {
            dialog.close();
        }
    }

    fn hide(&self) {
        if let Some(ref dialog) = self.model.dialog {
            dialog.hide()
        }
    }

    fn configure_view(&self, header: &str, description: &str, close: &str, accept: Option<&str>) {
        self.widgets.header.set_label(&strcat!("<b>" header "</b>"));
        self.widgets.description.set_label(description);
        self.widgets.close_button.set_label(close);

        match accept {
            Some(label) => {
                self.widgets.accept_button.show();
                self.widgets.accept_button.set_label(label);
            }
            None => self.widgets.accept_button.hide(),
        }
    }

    fn data_deleted_view(&self) {
        self.configure_view(
            &fl!("data-deleted-header"),
            &fl!("data-deleted-description"),
            &fl!("close"),
            None,
        );
    }

    fn data_downloaded_view(&self) {
        self.configure_view(
            &fl!("data-downloaded-header"),
            &fl!("data-downloaded-description"),
            &fl!("cancel"),
            Some(&fl!("open-folder")),
        );
    }

    fn delete_data_confirmation_view(&self) {
        self.configure_view(
            &fl!("delete-data-header"),
            &fl!("delete-data-description"),
            &fl!("cancel"),
            Some(&fl!("delete")),
        );
    }

    fn no_data_found_view(&self) {
        self.configure_view(
            &fl!("no-data-header"),
            &fl!("no-data-description"),
            &fl!("close"),
            None,
        );
    }

    fn no_internet_view(&self) {
        self.configure_view(
            &fl!("no-internet-header"),
            &fl!("no-internet-description"),
            &fl!("close"),
            Some(&fl!("try-again")),
        );
    }

    fn send(&self, message: panel::Message) {
        let _ = self.model.sender.emit(message);
    }

    fn update(&mut self, variant: Variant) {
        if let Some(ref dialog) = self.model.dialog {
            match variant {
                Variant::Deleted => self.data_deleted_view(),
                Variant::DeleteData => self.delete_data_confirmation_view(),
                Variant::NoDataFound => self.no_data_found_view(),
                Variant::NoInternet => self.no_internet_view(),
                Variant::DataDownloaded => self.data_downloaded_view(),
            }

            self.model.variant = Some(variant);
            dialog.show();
        }
    }
}

#[relm_derive::widget]
impl relm::Widget for Widget {
    fn update(&mut self, message: Message) {
        match message {
            Message::Accept => {
                match self.model.variant {
                    Some(Variant::DeleteData) => self.send(panel::Message::Delete),
                    Some(Variant::NoInternet) => self.send(panel::Message::TryAgain),
                    Some(Variant::DataDownloaded) => self.send(panel::Message::OpenDataDir),
                    _ => (),
                }

                self.hide()
            }

            Message::Update(variant) => self.update(variant),
            Message::Hide => self.hide(),
            Message::Close => self.close(),
        }
    }

    fn init_view(&mut self) {
        self.widgets.header.style_context().add_class("h2");
    }

    fn model(
        _: &relm::Relm<Self>,
        (dialog, sender): (gtk::MessageDialog, StreamHandle<panel::Message>),
    ) -> Model {
        Model {
            dialog: Some(dialog),
            variant: None,
            sender,
        }
    }

    relm::view! {
        gtk::Box {
            orientation: gtk::Orientation::Vertical,
            spacing: 24,
            margin_top: 8,

            #[name="header"]
            gtk::Label {
                halign: gtk::Align::Center,
                use_markup: true,
            },

            #[name="description"]
            gtk::Label {
                line_wrap: true,
            },

            gtk::ButtonBox {
                hexpand: true,
                homogeneous: true,
                layout_style: gtk::ButtonBoxStyle::Expand,
                orientation: gtk::Orientation::Horizontal,
                valign: gtk::Align::End,

                #[name="close_button"]
                gtk::Button {
                    gtk::Label {
                        label: &fl!("close"),
                        margin_top: 16,
                        margin_bottom: 16,
                    },
                    sensitive: true,
                    clicked => Message::Hide,
                },

                #[name="accept_button"]
                gtk::Button {
                    gtk::Label {
                        margin_top: 16,
                        margin_bottom: 16,
                    },
                    sensitive: true,
                    clicked => Message::Accept
                }
            }
        }
    }
}

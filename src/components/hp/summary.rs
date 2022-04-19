use crate::fl;

use concat_in_place::strcat;
use gtk::prelude::*;

pub struct Model {
    background: relm::Sender<Message>,
    show_toggle: bool,
    purpose: Option<(String, String, String, String)>,
    relm: relm::Relm<Widget>,
}

#[derive(relm_derive::Msg)]
pub enum Message {
    DisplaySample,
    OpenWebpage(&'static str),
    PurposeAndOpt(super::PurposeAndOpt),
    PurposeStatement(String),
    Toggle,
}

#[relm_derive::widget]
impl relm::Widget for Widget {
    fn init_view(&mut self) {
        let provider = gtk::CssProvider::new();
        provider
            .load_from_data(b".analytics-link { padding-left: 0 }")
            .unwrap();

        gtk::StyleContext::add_provider_for_screen(
            &gtk::gdk::Screen::default().unwrap(),
            &provider,
            gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
        );

        if !self.model.show_toggle {
            self.root().remove(&self.widgets.toggle_box);
        }

        self.widgets
            .link1
            .style_context()
            .add_class("analytics-link");

        self.widgets
            .link2
            .style_context()
            .add_class("analytics-link");

        self.widgets
            .link3
            .style_context()
            .add_class("analytics-link");

        if self.model.show_toggle {
            let tx = self.model.background.clone();
            glib::MainContext::default().spawn_local(async move {
                if let Ok(purpose_and_opt) = super::purpose_and_opted(false).await {
                    tx.send(Message::PurposeAndOpt(purpose_and_opt));
                } else {
                    // TODO? Shouldn't happen if hp-vendor installed correctly
                }
            });
        }
    }

    fn model(relm: &relm::Relm<Self>, show_toggle: bool) -> Model {
        let stream = relm.stream().clone();

        let (_channel, sender) = relm::Channel::new(move |message| {
            stream.emit(message);
        });

        Model {
            background: sender,
            show_toggle,
            purpose: None,
            relm: relm.clone(),
        }
    }

    fn update(&mut self, message: Message) {
        match message {
            Message::DisplaySample => {}

            Message::OpenWebpage(url) => {
                crate::misc::xdg_open(url);
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
                self.widgets.toggle.set_active(opted);
                self.widgets.toggle.set_sensitive(true);
                self.widgets.purpose_statement.set_text(&purpose.statement);
                self.model.purpose = Some((language, region, purpose.purpose_id, purpose.version));
            }

            Message::PurposeStatement(statement) => {
                self.widgets.purpose_statement.set_text(&statement);
            }

            Message::Toggle => {
                let enable = self.widgets.toggle.is_active();
                if let Some(purpose) = self.model.purpose.clone() {
                    glib::MainContext::default().spawn_local(async move {
                        super::toggle(purpose, enable).await;
                    });
                }
            }
        }
    }

    relm::view! {
        gtk::Box {
            orientation: gtk::Orientation::Vertical,

            #[name="toggle_box"]
            gtk::Box {
                hexpand: true,
                margin_bottom: 24,

                gtk::Label {
                    ellipsize: gtk::pango::EllipsizeMode::End,
                    halign: gtk::Align::Start,
                    hexpand: true,
                    label: &*strcat!("<b>" fl!("hp-analytics-toggle-header").as_str() "</b>"),
                    use_markup: true,
                    valign: gtk::Align::Center,
                    xalign: 0.0,
                },

                #[name="toggle"]
                gtk::Switch {
                    changed_active => Message::Toggle,
                    valign: gtk::Align::Center,
                }
            },

            gtk::Label {
                xalign: 0.0,
                label: &fl!("hp-analytics-description"),
                margin_bottom: 24,
            },

            #[name="link1"]
            gtk::LinkButton {
                halign: gtk::Align::Start,
                label: &fl!("see-data-sample"),
                margin_bottom: 24,
                activate_link => (Message::DisplaySample, gtk::Inhibit(false)),
            },

            #[name="link2"]
            gtk::LinkButton {
                halign: gtk::Align::Start,
                label: &fl!("hp-privacy-policy"),
                activate_link => (Message::OpenWebpage(""), gtk::Inhibit(false)),
            },

            #[name="link3"]
            gtk::LinkButton {
                halign: gtk::Align::Start,
                label: &fl!("pop-privacy-policy"),
                activate_link => (Message::OpenWebpage(""), gtk::Inhibit(false)),
                margin_bottom: 24,
            },

            #[name="purpose_statement"]
            gtk::Label {
            },
        }
    }
}

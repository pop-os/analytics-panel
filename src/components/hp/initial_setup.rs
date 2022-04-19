use crate::fl;

use concat_in_place::strcat;
use gtk::prelude::*;

pub struct Model {
    callback: Box<dyn Fn(bool)>,
    background: relm::Sender<Message>,
    purpose: Option<(String, String, String, String)>,
    relm: relm::Relm<Widget>,
}

#[derive(relm_derive::Msg)]
pub enum Message {
    Agree,
    Decline,
    DisplaySample,
    OpenWebpage(&'static str),
    PurposeAndOpt(super::PurposeAndOpt),
    PurposeStatement(String),
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

        self.widgets.title.style_context().add_class("h1");

        self.widgets.title.set_markup(&format!("<b>{}</b>", fl!("hp-dev-one-analytics")));

        self.widgets
            .link
            .style_context()
            .add_class("analytics-link");

        self.widgets.sample_title.set_markup(&format!("<b>{}</b>", fl!("data-sample")));

        self.widgets
            .agree
            .style_context()
            .add_class(&gtk::STYLE_CLASS_SUGGESTED_ACTION);

        self.widgets
            .settings_notice
            .style_context()
            .add_class("dim-label");

        let tx = self.model.background.clone();
        glib::MainContext::default().spawn_local(async move {
            if let Ok(purpose_and_opt) = super::purpose_and_opted(false).await {
                tx.send(Message::PurposeAndOpt(purpose_and_opt));
            } else {
                // TODO? Shouldn't happen if hp-vendor installed correctly
            }
        });
    }

    fn model(relm: &relm::Relm<Self>, callback: Box<dyn Fn(bool)>) -> Model {
        let stream = relm.stream().clone();

        let (_channel, sender) = relm::Channel::new(move |message| {
            stream.emit(message);
        });

        Model {
            callback,
            background: sender,
            purpose: None,
            relm: relm.clone(),
        }
    }

    fn update(&mut self, message: Message) {
        match message {
            Message::Agree => {
                if let Some(purpose) = self.model.purpose.clone() {
                    glib::MainContext::default().spawn_local(async move {
                        super::toggle(purpose, true).await;
                    });
                }
                (self.model.callback)(true);
            }

            Message::Decline => {
                if let Some(purpose) = self.model.purpose.clone() {
                    glib::MainContext::default().spawn_local(async move {
                        super::toggle(purpose, false).await;
                    });
                }
                (self.model.callback)(false);
            }

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
                }
                self.widgets.purpose_statement.set_text(&purpose.statement);
                self.model.purpose = Some((language, region, purpose.purpose_id, purpose.version));
            }

            Message::PurposeStatement(statement) => {
                self.widgets.purpose_statement.set_text(&statement);
            }
        }
    }

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
                xalign: 0.0,
                label: &fl!("hp-analytics-description"),
            },

            #[name="link"]
            gtk::LinkButton {
                halign: gtk::Align::Start,
                label: &fl!("hp-privacy-statement"),
                //TODO: ensure path to PDF is correct when prefix is changed
                activate_link => (Message::OpenWebpage("file:///usr/share/pop_analytics_panel/hp-privacy-statement-2021.pdf"), gtk::Inhibit(false)),
            },

            #[name="sample_title"]
            gtk::Label {
                halign: gtk::Align::Start,
            },

            gtk::ScrolledWindow {
                hscrollbar_policy: gtk::PolicyType::Never,
                min_content_height: 200,
                vscrollbar_policy: gtk::PolicyType::Always,

                #[name="sample_text"]
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

            gtk::Separator {},

            #[name="purpose_statement"]
            gtk::Label {
                label: &fl!("hp-analytics-prompt"),
            },

            gtk::Box {
                halign: gtk::Align::End,
                orientation: gtk::Orientation::Horizontal,
                spacing: 8,

                #[name="decline"]
                gtk::Button {
                    label: &fl!("decline-and-continue"),
                    clicked => Message::Decline,
                },

                #[name="agree"]
                gtk::Button {
                    label: &fl!("agree-and-continue"),
                    clicked => Message::Agree,
                },
            },

            #[name="settings_notice"]
            gtk::Label {
                label: &fl!("settings-notice"),
            }
        }
    }
}

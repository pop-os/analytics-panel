use crate::fl;

use gtk::prelude::*;
use std::rc::Rc;

pub struct Model {
    callback: Rc<dyn Fn(bool)>,
    purpose: (String, String, String, String),
    purpose_statement: String,
}

#[derive(relm_derive::Msg)]
pub enum Message {
    Agree,
    Decline,
    DisplaySample,
    OpenWebpage(&'static str),
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

        self.widgets
            .title
            .set_markup(&format!("<b>{}</b>", fl!("hp-dev-one-analytics")));

        self.widgets
            .link
            .style_context()
            .add_class("analytics-link");

        self.widgets
            .sample_title
            .set_markup(&format!("<b>{}</b>", fl!("data-sample")));

        self.widgets
            .agree
            .style_context()
            .add_class(&gtk::STYLE_CLASS_SUGGESTED_ACTION);

        self.widgets
            .settings_notice
            .style_context()
            .add_class("dim-label");
    }

    fn model(relm: &relm::Relm<Self>, callback: Box<dyn Fn(bool)>) -> Model {
        let stream = relm.stream().clone();

        let (_channel, _sender) = relm::Channel::new(move |message| {
            stream.emit(message);
        });

        let (language, region, purpose) =
            super::purpose_for_locale(hp_vendor_client::static_purposes());

        Model {
            callback: Rc::from(callback),
            purpose: (language, region, purpose.purpose_id, purpose.version),
            purpose_statement: purpose.statement,
        }
    }

    fn update(&mut self, message: Message) {
        match message {
            Message::Agree => {
                let purpose = self.model.purpose.clone();
                let callback = self.model.callback.clone();
                glib::MainContext::default().spawn_local(async move {
                    let _value = super::toggle(purpose, true).await;
                    (callback)(true);
                });
            }

            Message::Decline => {
                let purpose = self.model.purpose.clone();
                let callback = self.model.callback.clone();
                glib::MainContext::default().spawn_local(async move {
                    let _value = super::toggle(purpose, false).await;
                    (callback)(false);
                });
            }

            Message::DisplaySample => {}

            Message::OpenWebpage(url) => {
                crate::misc::xdg_open(url);
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
                line_wrap: true,
                label: &fl!("hp-analytics-description"),
            },

            #[name="link"]
            gtk::LinkButton {
                halign: gtk::Align::Start,
                label: &fl!("hp-privacy-statement"),
                activate_link => (Message::OpenWebpage("https://www.hp.com/us-en/privacy/privacy-central.html"), gtk::Inhibit(false)),
            },

            #[name="sample_title"]
            gtk::Label {
                halign: gtk::Align::Start,
            },

            gtk::ScrolledWindow {
                hscrollbar_policy: gtk::PolicyType::Never,
                min_content_height: 200,
                vscrollbar_policy: gtk::PolicyType::Always,

                // #[name="sample_text"]
                gtk::TextView {
                    buffer: Some(&super::sample_buffer()),
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

            // #[name="purpose_statement"]
            gtk::Label {
                label: &self.model.purpose_statement,
            },

            gtk::Box {
                halign: gtk::Align::End,
                orientation: gtk::Orientation::Horizontal,
                spacing: 8,

                // #[name="decline"]
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

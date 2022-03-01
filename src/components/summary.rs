use crate::fl;
use gtk::prelude::*;

pub struct Model {}

#[derive(relm_derive::Msg)]
pub enum Message {
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
    }

    fn model(relm: &relm::Relm<Self>, args: ()) -> Model {
        Model {}
    }

    fn update(&mut self, message: Message) {
        match message {
            Message::DisplaySample => {}

            Message::OpenWebpage(url) => {
                crate::misc::xdg_open(url);
            }
        }
    }

    relm::view! {
        gtk::Box {
            orientation: gtk::Orientation::Vertical,

            gtk::Label {
                xalign: 0.0,
                label: &fl!("hp-analytics-description"),
                margin_bottom: 24,
            },

            #[name="link1"]
            gtk::LinkButton {
                halign: gtk::Align::Start,
                label: &fl!("data-sample"),
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
        }
    }
}

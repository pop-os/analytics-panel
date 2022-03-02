use crate::fl;

use concat_in_place::strcat;
use gtk::prelude::*;

pub struct Model {
    show_toggle: bool,
}

#[derive(relm_derive::Msg)]
pub enum Message {
    DisplaySample,
    OpenWebpage(&'static str),
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
    }

    fn model(_: &relm::Relm<Self>, show_toggle: bool) -> Model {
        Model { show_toggle }
    }

    fn update(&mut self, message: Message) {
        match message {
            Message::DisplaySample => {}

            Message::OpenWebpage(url) => {
                crate::misc::xdg_open(url);
            }

            Message::Toggle => {
                let enable = self.widgets.toggle.is_active();
                glib::MainContext::default().spawn_local(crate::toggle(enable));
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

// Copyright 2022 System76 <info@system76.com>
// SPDX-License-Identifier: MPL-2.0

use crate::fl;

use concat_in_place::strcat;
use gtk::prelude::*;

pub struct Model {}

#[derive(relm_derive::Msg)]
pub enum Message {}

#[relm_derive::widget]
impl relm::Widget for Widget {
    fn init_view(&mut self) {
        self.widgets.title.style_context().add_class("h1");

        self.widgets.title.set_markup(&format!("<b>{}</b>", fl!("eula")));

        self.widgets.text.buffer().map(|buffer| {
            let eula_markup = include_str!("../../../data/hp-eula.md");
            buffer.insert_markup(&mut buffer.start_iter(), eula_markup);
        });
    }

    fn model(_relm: &relm::Relm<Self>, _: ()) -> Model {
        Model {}
    }

    fn update(&mut self, _message: Message) {}

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
                label: &fl!("eula-prompt"),
                line_wrap: true,
                max_width_chars: 80,
            },

            gtk::ScrolledWindow {
                hscrollbar_policy: gtk::PolicyType::Never,
                min_content_height: 400,
                vscrollbar_policy: gtk::PolicyType::Always,

                #[name="text"]
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
        }
    }
}

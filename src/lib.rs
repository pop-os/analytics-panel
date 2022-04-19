// Copyright 2022 System76 <info@system76.com>
// SPDX-License-Identifier: MPL-2.0

#[macro_use]
extern crate cascade;

pub mod components;

mod clamp;
mod localize;
mod misc;

pub use self::localize::localizer;

use gtk::prelude::*;

pub fn attach_eula(container: &gtk::Container) {
    let component = relm::create_component::<components::hp::Eula>(());

    container.add(component.widget());
    container.connect_destroy(move |_| {
        let _relm_handle = &component;
    });
}

pub fn attach_panel(container: &gtk::Container, window: gtk::Window) {
    let panel = relm::create_component::<components::hp::Panel>(window);

    container.add(panel.widget());
    container.connect_destroy(move |_| {
        let _panel = &panel;
    });
}

pub fn attach_summary(container: &gtk::Container) {
    let component = relm::create_component::<components::hp::Summary>(true);

    container.add(component.widget());
    container.connect_destroy(move |_| {
        let _relm_handle = &component;
    });
}

pub fn should_show() -> bool {
    hp_vendor_client::has_hp_vendor() && hp_vendor_client::supported_hardware().is_ok()
}

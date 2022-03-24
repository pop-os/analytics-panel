// Copyright 2022 System76 <info@system76.com>
// SPDX-License-Identifier: MPL-2.0

#![allow(clippy::not_unsafe_ptr_arg_deref)]

use glib::translate::FromGlibPtrNone;
use gtk_sys::{GtkContainer, GtkWindow};
use i18n_embed::DesktopLanguageRequester;
use pop_analytics_panel::*;

#[no_mangle]
pub extern "C" fn pop_analytics_panel_init() {
    let localizer = localizer();
    let requested_languages = DesktopLanguageRequester::requested_languages();

    if let Err(error) = localizer.select(&requested_languages) {
        eprintln!(
            "Error while loading languages for pop_upgrade_gtk {}",
            error
        );
    }
}

#[no_mangle]
pub extern "C" fn pop_analytics_panel_attach(
    c_container: *mut GtkContainer,
    c_window: *mut GtkWindow,
) {
    let container;
    let window;

    unsafe {
        gtk::set_initialized();
        container = gtk::Container::from_glib_none(c_container);
        window = gtk::Window::from_glib_none(c_window);
    };

    attach_panel(&container, window);
}

#[no_mangle]
pub extern "C" fn pop_analytics_panel_summary_attach(c_container: *mut GtkContainer) {
    let container;

    unsafe {
        gtk::set_initialized();
        container = gtk::Container::from_glib_none(c_container);
    }

    attach_summary(&container);
}

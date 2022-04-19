// Copyright 2022 System76 <info@system76.com>
// SPDX-License-Identifier: MPL-2.0

use gtk::prelude::*;
use i18n_embed::DesktopLanguageRequester;
use pop_analytics_panel::*;

fn main() -> anyhow::Result<()> {
    gtk()
}

#[allow(unused)]
fn gtk() -> anyhow::Result<()> {
    let localizer = localizer();
    let requested_languages = DesktopLanguageRequester::requested_languages();

    if let Err(error) = localizer.select(&requested_languages) {
        eprintln!(
            "Error while loading languages for pop_upgrade_gtk {}",
            error
        );
    }

    let app = gtk::Application::builder().build();

    app.connect_activate(|app| {
        let window = gtk::ApplicationWindow::new(app);

        attach_summary(window.upcast_ref());

        window.show_all();

        window.connect_delete_event(move |_, _| {
            gtk::main_quit();
            gtk::Inhibit(false)
        });
    });

    app.run();

    Ok(())
}

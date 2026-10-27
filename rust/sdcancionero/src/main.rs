// This file is part of Campfire.
// Copyright (c) Pandula Péter 2017-2026.
// https://github.com/pandulapeter/campfire
//
// This Source Code Form is subject to the terms of the Mozilla Public License, v. 2.0.
// If a copy of the MPL was not distributed with this file, You can obtain one at
// https://mozilla.org/MPL/2.0/.

//! SDCancionero for Windows computers of 32 bits.

// No console window behind the app in a release build.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod app;
mod audio;
mod diagnostics;
mod diagram;
mod editor;
mod icons;
mod layout;
mod listen;
mod media;
mod settings;
mod song_list;
mod theme;
mod tools;
mod viewer;

use eframe::egui;

/// The icon of the window, from the picture of the app.
fn window_icon() -> Option<egui::IconData> {
    let picture = image::load_from_memory(include_bytes!("../assets/icon.png")).ok()?.into_rgba8();
    let (width, height) = picture.dimensions();
    Some(egui::IconData { rgba: picture.into_raw(), width, height })
}

fn main() -> eframe::Result {
    let mut viewport = egui::ViewportBuilder::default().with_title("SDCancionero");
    if let Some(icon) = window_icon() {
        viewport = viewport.with_icon(std::sync::Arc::new(icon));
    }
    let options = eframe::NativeOptions {
        viewport: viewport.with_inner_size([1100.0, 720.0]).with_min_inner_size([520.0, 420.0]),
        ..Default::default()
    };
    eframe::run_native("SDCancionero", options, Box::new(|creation_context| Ok(Box::new(app::App::new(&creation_context.egui_ctx)))))
}

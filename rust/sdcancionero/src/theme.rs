// This file is part of Campfire.
// Copyright (c) Pandula Péter 2017-2026.
// https://github.com/pandulapeter/campfire
//
// This Source Code Form is subject to the terms of the Mozilla Public License, v. 2.0.
// If a copy of the MPL was not distributed with this file, You can obtain one at
// https://mozilla.org/MPL/2.0/.

//! The fonts and the colors: the interface is set in its bundled Inter and JetBrains Mono, and its brand colors are purple and
//! orange. The player may pick another face for the songs and for the editor from the ones the computer has.

use std::path::PathBuf;
use std::sync::Arc;

use eframe::egui::{self, Color32, FontData, FontDefinitions, FontFamily};

/// The bold face of the interface font, for headings and chords.
pub fn bold() -> FontFamily {
    FontFamily::Name("bold".into())
}

/// The face songs are shown in.
pub fn view_family() -> FontFamily {
    FontFamily::Name("view".into())
}

/// The face the editor is written in.
pub fn editor_family() -> FontFamily {
    FontFamily::Name("editor".into())
}

pub const BUNDLED_VIEW_FONT: &str = "Inter";
pub const BUNDLED_EDITOR_FONT: &str = "JetBrains Mono";

/// Faces of Windows that read well in a song, by name and file.
const SYSTEM_VIEW_FONTS: [(&str, &str); 8] = [
    ("Segoe UI", "segoeui.ttf"),
    ("Arial", "arial.ttf"),
    ("Calibri", "calibri.ttf"),
    ("Verdana", "verdana.ttf"),
    ("Tahoma", "tahoma.ttf"),
    ("Trebuchet MS", "trebuc.ttf"),
    ("Georgia", "georgia.ttf"),
    ("Times New Roman", "times.ttf"),
];

/// Faces of Windows with letters of one width, which is what a chord sheet needs to stay aligned.
const SYSTEM_EDITOR_FONTS: [(&str, &str); 4] = [("Consolas", "consola.ttf"), ("Courier New", "cour.ttf"), ("Lucida Console", "lucon.ttf"), ("Cascadia Mono", "CascadiaMono.ttf")];

fn fonts_directory() -> PathBuf {
    let windows = std::env::var_os("WINDIR").map(PathBuf::from).unwrap_or_else(|| PathBuf::from(r"C:\Windows"));
    windows.join("Fonts")
}

fn system_font_bytes(catalog: &[(&str, &str)], name: &str) -> Option<Vec<u8>> {
    let (_, file) = catalog.iter().find(|(candidate, _)| *candidate == name)?;
    std::fs::read(fonts_directory().join(file)).ok()
}

fn available(catalog: &[(&'static str, &str)]) -> Vec<&'static str> {
    catalog.iter().filter(|(_, file)| fonts_directory().join(file).exists()).map(|(name, _)| *name).collect()
}

/// The faces the player may show songs in: the bundled one first, then the ones with letters of one width, which keep the words
/// of a chord sheet exactly where the file has them, then the others.
pub fn view_fonts() -> Vec<&'static str> {
    std::iter::once(BUNDLED_VIEW_FONT)
        .chain(std::iter::once(BUNDLED_EDITOR_FONT))
        .chain(available(&SYSTEM_EDITOR_FONTS))
        .chain(available(&SYSTEM_VIEW_FONTS))
        .collect()
}

/// The faces the player may write in: the bundled one first.
pub fn editor_fonts() -> Vec<&'static str> {
    std::iter::once(BUNDLED_EDITOR_FONT).chain(available(&SYSTEM_EDITOR_FONTS)).collect()
}

/// Installs the fonts: the bundled ones, and the faces picked for the songs and the editor in front of them. What a picked face
/// lacks (the `♯` and `♭` signs) is taken from the bundled ones behind it.
pub fn install_fonts(context: &egui::Context, view_font: &str, editor_font: &str) {
    let mut fonts = FontDefinitions::default();
    fonts.font_data.insert("inter".into(), Arc::new(FontData::from_static(include_bytes!("../assets/inter_regular.ttf"))));
    fonts.font_data.insert("inter_bold".into(), Arc::new(FontData::from_static(include_bytes!("../assets/inter_bold.ttf"))));
    fonts.font_data.insert("mono".into(), Arc::new(FontData::from_static(include_bytes!("../assets/jetbrains_mono_regular.ttf"))));
    // The bundled faces come first and the built-in ones stay behind them for what they lack (symbols, emoji).
    fonts.families.entry(FontFamily::Proportional).or_default().insert(0, "inter".into());
    fonts.families.entry(FontFamily::Monospace).or_default().insert(0, "mono".into());
    fonts.families.insert(bold(), vec!["inter_bold".into(), "inter".into()]);
    let mut view = vec!["inter".to_string()];
    if view_font == BUNDLED_EDITOR_FONT {
        view.insert(0, "mono".into());
    } else if let Some(bytes) = system_font_bytes(&SYSTEM_VIEW_FONTS, view_font).or_else(|| system_font_bytes(&SYSTEM_EDITOR_FONTS, view_font)) {
        fonts.font_data.insert("view_font".into(), Arc::new(FontData::from_owned(bytes)));
        view.insert(0, "view_font".into());
    }
    fonts.families.insert(view_family(), view);
    let mut editor = vec!["mono".to_string()];
    if let Some(bytes) = system_font_bytes(&SYSTEM_EDITOR_FONTS, editor_font) {
        fonts.font_data.insert("editor_font".into(), Arc::new(FontData::from_owned(bytes)));
        editor.insert(0, "editor_font".into());
    }
    fonts.families.insert(editor_family(), editor);
    context.set_fonts(fonts);
}

pub const PURPLE: Color32 = Color32::from_rgb(0x8E, 0x5B, 0xD6);
pub const ORANGE: Color32 = Color32::from_rgb(0xF2, 0x8C, 0x28);

/// The colors a song is drawn in, for the light and the dark theme.
pub struct SongColors {
    pub text: Color32,
    pub chord: Color32,
    pub heading: Color32,
    pub comment: Color32,
    pub comment_box: Color32,
    pub muted: Color32,
}

impl SongColors {
    pub fn of(is_dark: bool) -> SongColors {
        if is_dark {
            SongColors {
                text: Color32::from_gray(0xE6),
                chord: Color32::from_rgb(0xFF, 0xA8, 0x4D),
                heading: Color32::from_rgb(0xB9, 0x92, 0xF0),
                comment: Color32::from_gray(0xA8),
                comment_box: Color32::from_gray(0x2E),
                muted: Color32::from_gray(0x80),
            }
        } else {
            SongColors {
                text: Color32::from_gray(0x1A),
                chord: Color32::from_rgb(0xC2, 0x5E, 0x00),
                heading: Color32::from_rgb(0x6B, 0x3F, 0xB0),
                comment: Color32::from_gray(0x60),
                comment_box: Color32::from_gray(0xEC),
                muted: Color32::from_gray(0x90),
            }
        }
    }
}

pub fn apply_visuals(context: &egui::Context, is_dark: bool) {
    let mut visuals = if is_dark { egui::Visuals::dark() } else { egui::Visuals::light() };
    visuals.selection.bg_fill = PURPLE;
    visuals.hyperlink_color = PURPLE;
    context.set_visuals(visuals);
}

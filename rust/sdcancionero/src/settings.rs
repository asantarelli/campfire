// This file is part of Campfire.
// Copyright (c) Pandula Péter 2017-2026.
// https://github.com/pandulapeter/campfire
//
// This Source Code Form is subject to the terms of the Mozilla Public License, v. 2.0.
// If a copy of the MPL was not distributed with this file, You can obtain one at
// https://mozilla.org/MPL/2.0/.

//! What the player chose, kept in a small JSON document next to the library.

use std::path::{Path, PathBuf};

use chordpro::notation::ChordNotation;
use serde_json::{Map, Value};

/// The notations a chart can be shown in. The numberings (Nashville, Roman) need the key of every song and are left out.
pub const NOTATIONS: [(ChordNotation, &str); 3] =
    [(ChordNotation::Standard, "Inglesa (C D E)"), (ChordNotation::German, "Alemana (H)"), (ChordNotation::Latin, "Latina (Do Re Mi)")];

/// The instrument the chord diagrams are drawn for.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Instrument {
    Guitar,
    Ukulele,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Settings {
    /// How many columns a song is spread over, as many as fit.
    pub columns: usize,
    pub text_size: f32,
    pub notation: ChordNotation,
    pub accidental_signs: bool,
    pub show_chords: bool,
    /// `None` leaves the spelling of a transposed song to the song's own key.
    pub prefer_flats: Option<bool>,
    /// Pixels a second the autoscroll moves a song by.
    pub autoscroll_speed: f32,
    pub is_dark: bool,
    pub pin_chord_diagrams: bool,
    pub instrument: Instrument,
    /// The face songs are shown in, by name.
    pub view_font: String,
    /// The face the editor is written in, by name.
    pub editor_font: String,
    pub editor_text_size: f32,
}

impl Default for Settings {
    fn default() -> Self {
        Settings {
            columns: 1,
            text_size: 18.0,
            notation: ChordNotation::Standard,
            accidental_signs: true,
            show_chords: true,
            prefer_flats: None,
            autoscroll_speed: 30.0,
            is_dark: true,
            pin_chord_diagrams: true,
            instrument: Instrument::Guitar,
            view_font: crate::theme::BUNDLED_VIEW_FONT.to_string(),
            editor_font: crate::theme::BUNDLED_EDITOR_FONT.to_string(),
            editor_text_size: 16.0,
        }
    }
}

pub const MIN_TEXT_SIZE: f32 = 12.0;
pub const MAX_TEXT_SIZE: f32 = 48.0;
pub const MAX_COLUMNS: usize = 4;
pub const MIN_AUTOSCROLL_SPEED: f32 = 5.0;
pub const MAX_AUTOSCROLL_SPEED: f32 = 200.0;

fn notation_key(notation: ChordNotation) -> &'static str {
    match notation {
        ChordNotation::German => "german",
        ChordNotation::Latin => "latin",
        _ => "standard",
    }
}

impl Settings {
    /// Where the settings of the library at `root` are kept: next to it, never inside it, since the library is what gets
    /// exported and synced.
    pub fn path_for(root: &Path) -> PathBuf {
        root.parent().unwrap_or(root).join("settings.json")
    }

    pub fn load(path: &Path) -> Settings {
        std::fs::read_to_string(path).ok().and_then(|text| Self::decode(&text)).unwrap_or_default()
    }

    pub fn save(&self, path: &Path) -> std::io::Result<()> {
        std::fs::write(path, self.encode())
    }

    /// Reads a document; every field that is missing or wrong keeps its default.
    pub fn decode(text: &str) -> Option<Settings> {
        let Value::Object(object) = serde_json::from_str::<Value>(text).ok()? else {
            return None;
        };
        let defaults = Settings::default();
        let boolean = |key: &str, default: bool| object.get(key).and_then(Value::as_bool).unwrap_or(default);
        let number = |key: &str, default: f32| object.get(key).and_then(Value::as_f64).map(|value| value as f32).unwrap_or(default);
        Some(Settings {
            columns: object.get("columns").and_then(Value::as_u64).map(|columns| (columns as usize).clamp(1, MAX_COLUMNS)).unwrap_or(defaults.columns),
            text_size: number("textSize", defaults.text_size).clamp(MIN_TEXT_SIZE, MAX_TEXT_SIZE),
            notation: match object.get("notation").and_then(Value::as_str) {
                Some("german") => ChordNotation::German,
                Some("latin") => ChordNotation::Latin,
                _ => ChordNotation::Standard,
            },
            accidental_signs: boolean("accidentalSigns", defaults.accidental_signs),
            show_chords: boolean("showChords", defaults.show_chords),
            prefer_flats: object.get("preferFlats").and_then(Value::as_bool),
            autoscroll_speed: number("autoscrollSpeed", defaults.autoscroll_speed).clamp(MIN_AUTOSCROLL_SPEED, MAX_AUTOSCROLL_SPEED),
            is_dark: boolean("isDark", defaults.is_dark),
            pin_chord_diagrams: boolean("pinChordDiagrams", defaults.pin_chord_diagrams),
            instrument: match object.get("instrument").and_then(Value::as_str) {
                Some("ukulele") => Instrument::Ukulele,
                _ => Instrument::Guitar,
            },
            view_font: object.get("viewFont").and_then(Value::as_str).map(str::to_string).unwrap_or(defaults.view_font),
            editor_font: object.get("editorFont").and_then(Value::as_str).map(str::to_string).unwrap_or(defaults.editor_font),
            editor_text_size: number("editorTextSize", defaults.editor_text_size).clamp(MIN_TEXT_SIZE, MAX_TEXT_SIZE),
        })
    }

    pub fn encode(&self) -> String {
        let mut object = Map::new();
        object.insert("columns".into(), Value::from(self.columns));
        object.insert("textSize".into(), Value::from(self.text_size as f64));
        object.insert("notation".into(), Value::from(notation_key(self.notation)));
        object.insert("accidentalSigns".into(), Value::from(self.accidental_signs));
        object.insert("showChords".into(), Value::from(self.show_chords));
        if let Some(flats) = self.prefer_flats {
            object.insert("preferFlats".into(), Value::from(flats));
        }
        object.insert("autoscrollSpeed".into(), Value::from(self.autoscroll_speed as f64));
        object.insert("isDark".into(), Value::from(self.is_dark));
        object.insert("pinChordDiagrams".into(), Value::from(self.pin_chord_diagrams));
        object.insert("viewFont".into(), Value::from(self.view_font.clone()));
        object.insert("editorFont".into(), Value::from(self.editor_font.clone()));
        object.insert("editorTextSize".into(), Value::from(self.editor_text_size as f64));
        object.insert("instrument".into(), Value::from(if self.instrument == Instrument::Ukulele { "ukulele" } else { "guitar" }));
        serde_json::to_string_pretty(&Value::Object(object)).expect("a JSON value always serializes")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn settings_survive_a_round_trip() {
        let settings = Settings {
            columns: 3,
            text_size: 22.0,
            notation: ChordNotation::Latin,
            accidental_signs: false,
            show_chords: false,
            prefer_flats: Some(true),
            autoscroll_speed: 55.0,
            is_dark: false,
            pin_chord_diagrams: true,
            instrument: Instrument::Ukulele,
            view_font: "Georgia".into(),
            editor_font: "Consolas".into(),
            editor_text_size: 20.0,
        };
        assert_eq!(Settings::decode(&settings.encode()), Some(settings));
    }

    #[test]
    fn what_is_wrong_keeps_its_default_and_what_is_out_of_range_is_held_in_it() {
        let settings = Settings::decode(r#"{"columns": 99, "textSize": "big", "notation": "klingon", "autoscrollSpeed": 1}"#).unwrap();
        assert_eq!(settings.columns, MAX_COLUMNS);
        assert_eq!(settings.text_size, Settings::default().text_size);
        assert_eq!(settings.notation, ChordNotation::Standard);
        assert_eq!(settings.autoscroll_speed, MIN_AUTOSCROLL_SPEED);
        assert_eq!(Settings::decode("not json"), None);
    }

    #[test]
    fn the_file_sits_next_to_the_library() {
        assert_eq!(Settings::path_for(Path::new("a/b/library")), Path::new("a/b/settings.json"));
    }
}

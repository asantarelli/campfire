// This file is part of Campfire.
// Copyright (c) Pandula Péter 2017-2026.
// https://github.com/pandulapeter/campfire
//
// This Source Code Form is subject to the terms of the Mozilla Public License, v. 2.0.
// If a copy of the MPL was not distributed with this file, You can obtain one at
// https://mozilla.org/MPL/2.0/.

//! The `*.setlist.json` document, as far as this version knows it.
//!
//! Every field is defaulted so that a hand-edited or older document still loads. Every other member of the document and of
//! each song is kept and written back after the known ones, so a field added by another version, or by hand, outlives this
//! version changing the setlist: the file is shared with those versions through an export. Written pretty printed with four
//! spaces and with every field at its default left out, which is byte for byte what the Kotlin app writes.

use serde_json::{ser::PrettyFormatter, Map, Number, Serializer, Value};

/// A setlist: a named, ordered list of songs, each with the transposition it is played at.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Setlist {
    pub title: String,
    pub description: String,
    /// An ISO date (`2026-09-28`), kept as text so that one that does not read as a date loses only itself.
    pub date: Option<String>,
    pub is_countdown_shown: bool,
    /// The place in the list older versions gave a setlist, which the date took over. Read and kept as written.
    pub priority: i64,
    pub is_archived: bool,
    pub songs: Vec<SetlistSong>,
    /// What the file held besides the fields above.
    pub unknown_fields: Map<String, Value>,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct SetlistSong {
    /// The file name of the song, which is its identity.
    pub file: String,
    pub transposition: i64,
    /// Beats per minute, `None` for the song's own tempo.
    pub tempo: Option<i64>,
    /// The fret the song is capoed at, `None` for the song's own.
    pub capo: Option<i64>,
    pub unknown_fields: Map<String, Value>,
}

const DOCUMENT_KEYS: [&str; 7] = ["title", "description", "date", "isCountdownShown", "priority", "isArchived", "songs"];
const SONG_KEYS: [&str; 4] = ["file", "transposition", "tempo", "capo"];
const TEMPO_RANGE: std::ops::RangeInclusive<i64> = 30..=300;
const CAPO_RANGE: std::ops::RangeInclusive<i64> = 0..=12;

impl Setlist {
    /// Reads a document; `None` where the text is not a setlist (not JSON, or not an object).
    pub fn decode(text: &str) -> Option<Setlist> {
        let Value::Object(object) = serde_json::from_str::<Value>(text).ok()? else {
            return None;
        };
        let text_of = |key: &str| object.get(key).and_then(Value::as_str).map(str::to_string);
        let songs = object
            .get("songs")
            .and_then(Value::as_array)
            .map(|songs| songs.iter().map(SetlistSong::from_value).collect())
            .unwrap_or_default();
        Some(Setlist {
            title: text_of("title").unwrap_or_default(),
            description: text_of("description").unwrap_or_default(),
            date: text_of("date"),
            is_countdown_shown: object.get("isCountdownShown").and_then(Value::as_bool).unwrap_or(false),
            priority: object.get("priority").and_then(Value::as_i64).unwrap_or(0),
            is_archived: object.get("isArchived").and_then(Value::as_bool).unwrap_or(false),
            songs,
            unknown_fields: object.into_iter().filter(|(key, _)| !DOCUMENT_KEYS.contains(&key.as_str())).collect(),
        })
    }

    /// The document as the app writes it.
    pub fn encode(&self) -> String {
        let mut object = Map::new();
        if !self.title.is_empty() {
            object.insert("title".into(), Value::String(self.title.clone()));
        }
        if !self.description.is_empty() {
            object.insert("description".into(), Value::String(self.description.clone()));
        }
        if let Some(date) = &self.date {
            object.insert("date".into(), Value::String(date.clone()));
        }
        if self.is_countdown_shown {
            object.insert("isCountdownShown".into(), Value::Bool(true));
        }
        if self.priority != 0 {
            object.insert("priority".into(), Value::Number(Number::from(self.priority)));
        }
        if self.is_archived {
            object.insert("isArchived".into(), Value::Bool(true));
        }
        if !self.songs.is_empty() {
            object.insert("songs".into(), Value::Array(self.songs.iter().map(SetlistSong::to_value).collect()));
        }
        object.extend(self.unknown_fields.clone());
        let mut bytes = Vec::new();
        let mut serializer = Serializer::with_formatter(&mut bytes, PrettyFormatter::with_indent(b"    "));
        serde::Serialize::serialize(&Value::Object(object), &mut serializer).expect("a JSON value always serializes");
        String::from_utf8(bytes).expect("JSON is UTF-8")
    }

    /// The same setlist with `file` out of it, which is how a deleted song leaves the lists that named it.
    pub fn without_song(&self, file: &str) -> Setlist {
        Setlist { songs: self.songs.iter().filter(|song| song.file != file).cloned().collect(), ..self.clone() }
    }

    /// The same setlist with every entry for `from` naming `to`, which is how a renamed song stays in its lists.
    pub fn with_song_renamed(&self, from: &str, to: &str) -> Setlist {
        Setlist {
            songs: self.songs.iter().map(|song| if song.file == from { SetlistSong { file: to.to_string(), ..song.clone() } } else { song.clone() }).collect(),
            ..self.clone()
        }
    }
}

impl SetlistSong {
    fn from_value(value: &Value) -> SetlistSong {
        let Value::Object(object) = value else {
            return SetlistSong::default();
        };
        let whole_number = |key: &str, range: &std::ops::RangeInclusive<i64>| {
            object.get(key).and_then(Value::as_i64).filter(|number| range.contains(number))
        };
        SetlistSong {
            file: object.get("file").and_then(Value::as_str).unwrap_or("").to_string(),
            transposition: object.get("transposition").and_then(Value::as_i64).unwrap_or(0),
            tempo: whole_number("tempo", &TEMPO_RANGE),
            capo: whole_number("capo", &CAPO_RANGE),
            unknown_fields: object.iter().filter(|(key, _)| !SONG_KEYS.contains(&key.as_str())).map(|(k, v)| (k.clone(), v.clone())).collect(),
        }
    }

    fn to_value(&self) -> Value {
        let mut object = Map::new();
        if !self.file.is_empty() {
            object.insert("file".into(), Value::String(self.file.clone()));
        }
        if self.transposition != 0 {
            object.insert("transposition".into(), Value::Number(Number::from(self.transposition)));
        }
        if let Some(tempo) = self.tempo {
            object.insert("tempo".into(), Value::Number(Number::from(tempo)));
        }
        if let Some(capo) = self.capo {
            object.insert("capo".into(), Value::Number(Number::from(capo)));
        }
        object.extend(self.unknown_fields.clone());
        Value::Object(object)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const DOCUMENT: &str = r#"{
    "title": "Sábado",
    "description": "Misa de la tarde",
    "date": "2026-09-28",
    "isCountdownShown": true,
    "songs": [
        {
            "file": "a.cho",
            "transposition": 2,
            "tempo": 96,
            "capo": 0,
            "note": "slow"
        },
        {
            "file": "b.cho"
        }
    ],
    "futureField": {
        "x": 1
    }
}"#;

    #[test]
    fn a_document_survives_a_round_trip_byte_for_byte() {
        let setlist = Setlist::decode(DOCUMENT).expect("a setlist");
        assert_eq!(setlist.title, "Sábado");
        assert_eq!(setlist.songs.len(), 2);
        assert_eq!(setlist.songs[0].transposition, 2);
        assert_eq!(setlist.songs[0].tempo, Some(96));
        assert_eq!(setlist.songs[0].capo, Some(0));
        assert_eq!(setlist.songs[1].tempo, None);
        assert_eq!(setlist.encode(), DOCUMENT);
    }

    #[test]
    fn a_hand_edited_document_loses_only_what_it_got_wrong() {
        let setlist = Setlist::decode(r#"{"title": "T", "date": 20260928, "songs": [{"file": "a.cho", "tempo": "fast", "capo": 99}, 5], "priority": "x"}"#)
            .expect("a setlist");
        assert_eq!(setlist.title, "T");
        assert_eq!(setlist.date, None);
        assert_eq!(setlist.songs[0].tempo, None);
        assert_eq!(setlist.songs[0].capo, None);
        assert_eq!(setlist.songs[1], SetlistSong::default());
        assert_eq!(setlist.priority, 0);
    }

    #[test]
    fn what_is_no_setlist_is_refused() {
        for text in ["", "[]", "not json", "42", "\"x\""] {
            assert_eq!(Setlist::decode(text), None, "{text:?}");
        }
    }

    #[test]
    fn an_empty_setlist_is_an_empty_object() {
        assert_eq!(Setlist::default().encode(), "{}");
    }

    #[test]
    fn a_song_leaves_and_is_renamed_in_a_list() {
        let setlist = Setlist::decode(DOCUMENT).unwrap();
        assert_eq!(setlist.without_song("a.cho").songs.len(), 1);
        let renamed = setlist.with_song_renamed("a.cho", "z.cho");
        assert_eq!(renamed.songs[0].file, "z.cho");
        assert_eq!(renamed.songs[0].tempo, Some(96));
    }
}

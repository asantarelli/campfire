// This file is part of Campfire.
// Copyright (c) Pandula Péter 2017-2026.
// https://github.com/pandulapeter/campfire
//
// This Source Code Form is subject to the terms of the Mozilla Public License, v. 2.0.
// If a copy of the MPL was not distributed with this file, You can obtain one at
// https://mozilla.org/MPL/2.0/.

//! What the last paste was, kept in a file next to the settings: the text that arrived, the characters in it that are not letters,
//! digits or the usual punctuation, and what it was converted to. A paste that does not convert as expected can then be looked at
//! exactly as the page copied it, which no chat message shows.

use std::collections::BTreeMap;

/// Writes `ultimo-pegado.txt` next to the settings. A failure to write is of no concern to the paste itself.
pub fn record_paste(raw: &str, converted: &str) {
    let path = library::default_root().parent().map(|parent| parent.join("ultimo-pegado.txt"));
    let Some(path) = path else {
        return;
    };
    let mut unusual: BTreeMap<char, usize> = BTreeMap::new();
    for character in raw.chars() {
        if !(character.is_alphanumeric() || character.is_ascii_graphic() || matches!(character, ' ' | '\n')) {
            *unusual.entry(character).or_default() += 1;
        }
    }
    let list: Vec<String> = unusual.iter().map(|(character, count)| format!("U+{:04X} x{count}", *character as u32)).collect();
    let report = format!(
        "== Texto pegado ({} caracteres) ==\n{raw}\n\n== Caracteres no habituales ==\n{}\n\n== Convertido ==\n{converted}\n",
        raw.chars().count(),
        if list.is_empty() { "ninguno".to_string() } else { list.join(", ") }
    );
    let _ = std::fs::write(path, report);
}

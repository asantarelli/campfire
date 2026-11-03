// This file is part of Campfire.
// Copyright (c) Pandula Péter 2017-2026.
// https://github.com/pandulapeter/campfire
//
// This Source Code Form is subject to the terms of the Mozilla Public License, v. 2.0.
// If a copy of the MPL was not distributed with this file, You can obtain one at
// https://mozilla.org/MPL/2.0/.

//! The names of the files of a library. The file name is a song's and a setlist's identity: nothing is overwritten
//! implicitly, and a collision gets `_2`, `_3`…
//!
//! A port of the Latin part of the Kotlin `LibraryFiles.normalizedName` and `FileNames.uniqueName`.

use std::collections::HashSet;

pub const SONG_EXTENSION: &str = ".cho";
pub const SETLIST_EXTENSION: &str = ".setlist.json";

/// Every extension a ChordPro file may carry; all of them name the same thing.
pub const SONG_EXTENSIONS: [&str; 6] = [".cho", ".chopro", ".chordpro", ".crd", ".chord", ".pro"];

const NAME_SEPARATOR: char = '_';
const FALLBACK_NAME: &str = "song";
const MAX_NAME_BYTES: usize = 120;
const APOSTROPHES: [char; 5] = ['\'', '’', '‘', '`', '´'];

/// Whether `name` is a file some tool wrote for itself rather than one somebody put there: macOS leaves an AppleDouble
/// `._name.cho` next to every file it copies, and a `.DS_Store` in every folder it has shown.
pub fn is_hidden_file_name(name: &str) -> bool {
    name.starts_with('.')
}

/// Whether a file called `name` in the songs folder is a song.
pub fn is_song_file_name(name: &str) -> bool {
    let lower = name.to_lowercase();
    !is_hidden_file_name(name) && SONG_EXTENSIONS.iter().any(|extension| lower.ends_with(extension))
}

/// Whether `name` is a Guitar Pro score, which an import turns into a song that holds it (see `guitarpro::song`).
pub fn is_score_file_name(name: &str) -> bool {
    let lower = name.to_lowercase();
    !is_hidden_file_name(name) && [".gp3", ".gp4", ".gp5"].iter().any(|extension| lower.ends_with(extension))
}

/// The same question about the setlists folder.
pub fn is_setlist_file_name(name: &str) -> bool {
    !is_hidden_file_name(name) && name.to_lowercase().ends_with(SETLIST_EXTENSION)
}

/// Whether `name` is a zip archive by its name.
pub fn is_archive_file_name(name: &str) -> bool {
    name.to_lowercase().ends_with(".zip")
}

/// Whether two names are one file: a name differing only by case is the same one to sync and to the other devices.
pub fn identity_key(name: &str) -> String {
    name.to_lowercase()
}

/// `base` as the app names a file: lowercase words joined by underscores, Latin accents folded, other scripts kept,
/// at most 120 bytes, apostrophes dropped and `&` and `+` spelled out. The rule is idempotent.
pub fn normalized_name(base: &str) -> String {
    let mut folded = String::new();
    for character in base.to_lowercase().chars() {
        if APOSTROPHES.contains(&character) || ('\u{300}'..='\u{36f}').contains(&character) {
            continue;
        }
        if character == '&' || character == '+' {
            folded.push(NAME_SEPARATOR);
            folded.push_str("and");
            folded.push(NAME_SEPARATOR);
            continue;
        }
        match fold_accent(character) {
            Some(plain) => folded.push_str(plain),
            None if character.is_ascii_alphanumeric() => folded.push(character),
            // Another script's letters are kept, whatever they are.
            None if character.is_alphanumeric() && !is_latin(character) => folded.push(character),
            None => folded.push(NAME_SEPARATOR),
        }
    }
    let words: Vec<&str> = folded.split(NAME_SEPARATOR).filter(|word| !word.is_empty()).map(abbreviation).collect();
    // Capped by whole words rather than by characters: a word cut short can become a different word on the next pass.
    let mut name = String::new();
    for word in &words {
        let added = if name.is_empty() { 0 } else { 1 } + word.len();
        if name.len() + added > MAX_NAME_BYTES {
            break;
        }
        if !name.is_empty() {
            name.push(NAME_SEPARATOR);
        }
        name.push_str(word);
    }
    if name.is_empty() {
        if let Some(first) = words.first() {
            name = first.chars().scan(0, |bytes, c| {
                *bytes += c.len_utf8();
                (*bytes <= MAX_NAME_BYTES).then_some(c)
            }).collect();
        }
    }
    if name.is_empty() {
        FALLBACK_NAME.to_string()
    } else {
        name
    }
}

/// How the library files a credit: `featuring` is `ft`, `with` stays.
fn abbreviation(word: &str) -> &str {
    match word {
        "feat" | "featuring" | "ft" => "ft",
        other => other,
    }
}

fn is_latin(character: char) -> bool {
    character.is_ascii_alphabetic() || ('\u{c0}'..='\u{24f}').contains(&character) || ('\u{1e00}'..='\u{1eff}').contains(&character)
}

/// The plain letters of a precomposed Latin letter, `None` for every other character.
fn fold_accent(character: char) -> Option<&'static str> {
    Some(match character {
        'à' | 'á' | 'â' | 'ã' | 'ä' | 'å' | 'ā' | 'ă' | 'ą' => "a",
        'ç' | 'ć' | 'ĉ' | 'ċ' | 'č' => "c",
        'ď' | 'đ' | 'ð' => "d",
        'è' | 'é' | 'ê' | 'ë' | 'ē' | 'ĕ' | 'ė' | 'ę' | 'ě' | 'ə' | 'ɛ' => "e",
        'ĝ' | 'ğ' | 'ġ' | 'ģ' => "g",
        'ĥ' | 'ħ' => "h",
        'ì' | 'í' | 'î' | 'ï' | 'ĩ' | 'ī' | 'ĭ' | 'į' | 'ı' => "i",
        'ĵ' => "j",
        'ķ' => "k",
        'ĺ' | 'ļ' | 'ľ' | 'ŀ' | 'ł' => "l",
        'ñ' | 'ń' | 'ņ' | 'ň' | 'ŋ' => "n",
        'ò' | 'ó' | 'ô' | 'õ' | 'ö' | 'ø' | 'ō' | 'ŏ' | 'ő' | 'ɔ' => "o",
        'ŕ' | 'ŗ' | 'ř' => "r",
        'ś' | 'ŝ' | 'ş' | 'š' => "s",
        'ţ' | 'ť' | 'ŧ' => "t",
        'ù' | 'ú' | 'û' | 'ü' | 'ũ' | 'ū' | 'ŭ' | 'ů' | 'ű' | 'ų' => "u",
        'ŵ' => "w",
        'ý' | 'ÿ' | 'ŷ' => "y",
        'ź' | 'ż' | 'ž' => "z",
        'ß' => "ss",
        'æ' => "ae",
        'œ' => "oe",
        'þ' => "th",
        'ĳ' => "ij",
        _ => return None,
    })
}

/// The file name a new song is given: `artist-title.cho`, both halves normalized, or the title alone where there is no
/// artist.
pub fn song_file_name(artist: Option<&str>, title: &str) -> String {
    let title = normalized_name(title);
    match artist.map(str::trim).filter(|artist| !artist.is_empty()) {
        Some(artist) => format!("{}-{title}{SONG_EXTENSION}", normalized_name(artist)),
        None => format!("{title}{SONG_EXTENSION}"),
    }
}

/// The file name a new setlist is given from its title.
pub fn setlist_file_name(title: &str) -> String {
    format!("{}{SETLIST_EXTENSION}", normalized_name(title))
}

/// `wanted` where no listed file has it, else `wanted` with `_2`, `_3`… before its extension. A name is taken where a listed
/// file has it under any case.
pub fn unique_name(taken: &HashSet<String>, wanted: &str) -> String {
    if !taken.contains(&identity_key(wanted)) {
        return wanted.to_string();
    }
    let extension_start = [SETLIST_EXTENSION]
        .iter()
        .find(|extension| wanted.to_lowercase().ends_with(*extension))
        .map(|extension| wanted.len() - extension.len())
        .or_else(|| wanted.rfind('.').filter(|&index| index > 0))
        .unwrap_or(wanted.len());
    let (stem, extension) = wanted.split_at(extension_start);
    (2..)
        .map(|number| format!("{stem}_{number}{extension}"))
        .find(|candidate| !taken.contains(&identity_key(candidate)))
        .expect("an unbounded count finds a free name")
}

/// Whether `name` can be written as a file name on Windows: no path, no reserved character, no trailing space or dot.
pub fn can_hold_file_name(name: &str) -> bool {
    !name.is_empty()
        && name != "."
        && name != ".."
        && !name.chars().any(|c| c.is_control() || matches!(c, '/' | '\\' | '?' | ':' | '*' | '"' | '<' | '>' | '|'))
        && !name.ends_with(' ')
        && !name.ends_with('.')
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_name_is_lowercase_words_with_their_accents_folded() {
        assert_eq!(normalized_name("Canción de Cuna"), "cancion_de_cuna");
        assert_eq!(normalized_name("Años y años"), "anos_y_anos");
        assert_eq!(normalized_name("Don't Stop Believin'"), "dont_stop_believin");
        assert_eq!(normalized_name("Simon & Garfunkel"), "simon_and_garfunkel");
        assert_eq!(normalized_name("Artist feat. Other"), "artist_ft_other");
        assert_eq!(normalized_name("  --  "), "song");
        assert_eq!(normalized_name("Straße"), "strasse");
    }

    #[test]
    fn other_scripts_are_kept() {
        assert_eq!(normalized_name("Песня о друге"), "песня_о_друге");
    }

    #[test]
    fn the_rule_is_idempotent() {
        for text in ["Canción de Cuna", "Simon & Garfunkel", "Artist feat. Other (live)", "ÑANDÚ", "x".repeat(300).as_str(), "a b c"] {
            let once = normalized_name(text);
            assert_eq!(normalized_name(&once), once, "{text}");
        }
    }

    #[test]
    fn a_name_is_capped_by_whole_words() {
        let long = vec!["palabra"; 40].join(" ");
        let name = normalized_name(&long);
        assert!(name.len() <= MAX_NAME_BYTES);
        assert!(name.split('_').all(|word| word == "palabra"));
    }

    #[test]
    fn a_song_is_named_by_its_artist_and_title() {
        assert_eq!(song_file_name(Some("Mercedes Sosa"), "Gracias a la Vida"), "mercedes_sosa-gracias_a_la_vida.cho");
        assert_eq!(song_file_name(None, "Sin Artista"), "sin_artista.cho");
        assert_eq!(song_file_name(Some("  "), "Sin Artista"), "sin_artista.cho");
        assert_eq!(setlist_file_name("Sábado en la Iglesia"), "sabado_en_la_iglesia.setlist.json");
    }

    #[test]
    fn a_collision_is_numbered_before_the_extension() {
        let taken: HashSet<String> = ["a.cho", "a_2.cho", "b.setlist.json"].iter().map(|name| identity_key(name)).collect();
        assert_eq!(unique_name(&taken, "free.cho"), "free.cho");
        assert_eq!(unique_name(&taken, "a.cho"), "a_3.cho");
        assert_eq!(unique_name(&taken, "A.CHO"), "A_3.CHO");
        assert_eq!(unique_name(&taken, "b.setlist.json"), "b_2.setlist.json");
    }

    #[test]
    fn extensions_say_what_a_file_is() {
        assert!(is_song_file_name("x.cho"));
        assert!(is_song_file_name("X.ChoPro"));
        assert!(!is_song_file_name("._x.cho"));
        assert!(!is_song_file_name("x.txt"));
        assert!(is_setlist_file_name("x.setlist.json"));
        assert!(!is_setlist_file_name("x.json"));
        assert!(is_archive_file_name("backup.ZIP"));
    }

    #[test]
    fn windows_refuses_some_names() {
        for name in ["", ".", "..", "a/b", "a:b", "a?", "a ", "a.", "tab\there"] {
            assert!(!can_hold_file_name(name), "{name:?}");
        }
        assert!(can_hold_file_name("cancion-de-cuna.cho"));
    }
}

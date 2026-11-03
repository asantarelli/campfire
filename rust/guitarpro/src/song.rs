// This file is part of Campfire.
// Copyright (c) Pandula Péter 2017-2026.
// https://github.com/pandulapeter/campfire
//
// This Source Code Form is subject to the terms of the Mozilla Public License, v. 2.0.
// If a copy of the MPL was not distributed with this file, You can obtain one at
// https://mozilla.org/MPL/2.0/.

//! A Guitar Pro score as a ChordPro song, which is how a library holds one: the file itself, in `{meta}` lines, under a title and an
//! artist the player can change like any song's. Every other ChordPro program keeps custom metadata and ignores it, and everything the
//! library does to a song file (naming, tags, export, sync) then works on a score with nothing of its own. The Kotlin app writes and reads
//! the same lines, so a library is the same in both.
//!
//! ```text
//! {title: Exercises}
//! {artist: Somebody}
//! {meta: score guitarpro}
//! {meta: score-data UEsDBBQ...}
//! ```

use std::collections::BTreeMap;

/// The `{meta}` key of the marker line, which says the song is a score; its value names the format.
pub const MARKER_NAME: &str = "score";
/// The `{meta}` key of every line of the file's data.
pub const DATA_NAME: &str = "score-data";

const FORMAT: &str = "guitarpro";
/// How many characters of Base64 a line holds, so that no line is long enough to trouble an editor.
const LINE_LENGTH: usize = 76;
const ALPHABET: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

/// The song text holding the score of `bytes`, titled by the file and credited to the artist it names, or `fallback_title` for a title
/// it names none; `None` where the bytes are not a score that can be read, which is not worth keeping.
pub fn of(bytes: &[u8], fallback_title: &str) -> Option<String> {
    let score = crate::read(bytes).ok()?;
    let title = if score.title.trim().is_empty() { fallback_title } else { &score.title };
    Some(encode(title, &score.artist, bytes))
}

/// The song text of `bytes` under `title` and `artist`, either of which may be blank, whatever the bytes are.
pub fn encode(title: &str, artist: &str, bytes: &[u8]) -> String {
    let (title, artist) = (directive_text(title), directive_text(artist));
    let mut text = String::new();
    if !title.is_empty() {
        text.push_str(&format!("{{title: {title}}}\n"));
    }
    if !artist.is_empty() {
        text.push_str(&format!("{{artist: {artist}}}\n"));
    }
    text.push_str(&format!("{{meta: {MARKER_NAME} {FORMAT}}}\n"));
    let data = base64_encode(bytes);
    for line in data.as_bytes().chunks(LINE_LENGTH) {
        text.push_str(&format!("{{meta: {DATA_NAME} {}}}\n", std::str::from_utf8(line).unwrap_or_default()));
    }
    text
}

/// Whether the custom metadata of a song says it is a score.
pub fn is_score(custom: &BTreeMap<String, Vec<String>>) -> bool {
    custom.get(MARKER_NAME).is_some_and(|values| !values.is_empty())
}

/// The file a song text holds, from the custom metadata of its header; `None` for a song that holds none.
pub fn bytes_of(custom: &BTreeMap<String, Vec<String>>) -> Option<Vec<u8>> {
    if !is_score(custom) {
        return None;
    }
    let data: String = custom.get(DATA_NAME)?.iter().map(|line| line.trim()).collect();
    if data.is_empty() {
        return None;
    }
    base64_decode(&data)
}

/// The song text with the `{title}` and the `{artist}` it names changed to `title` and `artist` (a blank one is left out), every other
/// line as it was: the score inside is never touched.
pub fn with_title_and_artist(text: &str, title: &str, artist: &str) -> String {
    let (title, artist) = (directive_text(title), directive_text(artist));
    let is_named = |line: &str, names: &[&str]| {
        let inner = line.trim().strip_prefix('{').and_then(|rest| rest.strip_suffix('}')).unwrap_or("");
        let name = inner.split(':').next().unwrap_or("").trim().to_lowercase();
        inner.contains(':') && names.contains(&name.as_str())
    };
    let mut result = String::new();
    if !title.is_empty() {
        result.push_str(&format!("{{title: {title}}}
"));
    }
    if !artist.is_empty() {
        result.push_str(&format!("{{artist: {artist}}}
"));
    }
    for line in text.lines().filter(|line| !is_named(line, &["title", "t", "artist"])) {
        result.push_str(line);
        result.push('\n');
    }
    result
}

/// A value a directive can hold: on one line, and with nothing that would close the directive early.
fn directive_text(text: &str) -> String {
    text.chars().map(|character| if character == '{' || character == '}' || character.is_control() { ' ' } else { character }).collect::<String>().trim().to_string()
}

fn base64_encode(bytes: &[u8]) -> String {
    let mut text = String::with_capacity(bytes.len().div_ceil(3) * 4);
    for chunk in bytes.chunks(3) {
        let block = (chunk[0] as u32) << 16 | (*chunk.get(1).unwrap_or(&0) as u32) << 8 | *chunk.get(2).unwrap_or(&0) as u32;
        for position in 0..4 {
            if position <= chunk.len() {
                text.push(ALPHABET[(block >> (18 - 6 * position) & 0x3F) as usize] as char);
            } else {
                text.push('=');
            }
        }
    }
    text
}

fn base64_decode(text: &str) -> Option<Vec<u8>> {
    let digits: Vec<u32> = text.trim_end_matches('=').bytes().map(|byte| ALPHABET.iter().position(|&known| known == byte).map(|index| index as u32)).collect::<Option<_>>()?;
    let mut bytes = Vec::with_capacity(digits.len() * 3 / 4);
    for chunk in digits.chunks(4) {
        if chunk.len() == 1 {
            return None;
        }
        let block = chunk.iter().enumerate().fold(0u32, |block, (position, digit)| block | digit << (18 - 6 * position));
        for position in 0..chunk.len() - 1 {
            bytes.push((block >> (16 - 8 * position)) as u8);
        }
    }
    Some(bytes)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn custom_of(text: &str) -> BTreeMap<String, Vec<String>> {
        let mut custom: BTreeMap<String, Vec<String>> = BTreeMap::new();
        for line in text.lines() {
            if let Some(rest) = line.strip_prefix("{meta: ").and_then(|rest| rest.strip_suffix('}')) {
                let (name, value) = rest.split_once(' ').unwrap_or((rest, ""));
                custom.entry(name.to_string()).or_default().push(value.to_string());
            }
        }
        custom
    }

    #[test]
    fn the_file_comes_back_out_of_the_song_it_was_put_in() {
        for length in [0, 1, 2, 3, 4, 5, 76, 5000] {
            let bytes: Vec<u8> = (0..length).map(|index: u32| (index * 31 + index / 7) as u8).collect();
            let text = encode("Exercises", "Somebody", &bytes);
            let custom = custom_of(&text);
            assert!(is_score(&custom));
            if length > 0 {
                assert_eq!(bytes_of(&custom), Some(bytes));
            }
        }
    }

    #[test]
    fn the_text_is_the_one_kotlin_writes() {
        // "Man" is TWFu, and the last group of "Ma" is padded.
        assert_eq!(encode("T", "", b"Man"), "{title: T}\n{meta: score guitarpro}\n{meta: score-data TWFu}\n");
        assert_eq!(base64_encode(b"Ma"), "TWE=");
        assert_eq!(base64_decode("TWE="), Some(b"Ma".to_vec()));
    }

    #[test]
    fn a_title_or_an_artist_that_would_close_the_directive_is_cleaned() {
        let text = encode("A {sharp} } day", "B\nC", b"x");
        assert!(text.starts_with("{title: A  sharp    day}\n{artist: B C}\n"));
    }

    #[test]
    fn the_title_and_the_artist_change_and_the_score_does_not() {
        let text = encode("Old", "Someone", b"score bytes");
        let changed = with_title_and_artist(&text, "New {title}", "");
        assert!(changed.starts_with("{title: New  title}
{meta: score guitarpro}
"));
        assert!(!changed.contains("Someone") && !changed.contains("Old"));
        assert_eq!(bytes_of(&custom_of(&changed)), Some(b"score bytes".to_vec()));
    }

    #[test]
    fn a_song_that_holds_no_score_says_so() {
        assert!(!is_score(&BTreeMap::new()));
        assert_eq!(bytes_of(&BTreeMap::new()), None);
    }

    #[test]
    fn only_a_score_that_reads_is_turned_into_a_song() {
        assert_eq!(of(b"not a score", "x"), None);
        let bytes = std::fs::read(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/song.gp5")).unwrap();
        let text = of(&bytes, "fallback").unwrap();
        assert!(text.starts_with("{title: Práctica 002}\n{artist: Yo}\n"));
        assert_eq!(bytes_of(&custom_of(&text)), Some(bytes));
    }
}

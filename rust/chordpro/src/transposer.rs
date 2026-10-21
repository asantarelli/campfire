// This file is part of Campfire.
// Copyright (c) Pandula Péter 2017-2026.
// https://github.com/pandulapeter/campfire
//
// This Source Code Form is subject to the terms of the Mozilla Public License, v. 2.0.
// If a copy of the MPL was not distributed with this file, You can obtain one at
// https://mozilla.org/MPL/2.0/.

//! Moves chords up or down by a number of semitones. The names come first; the song and the raw text follow with the model.

use std::collections::HashMap;

use crate::chord_names;
use crate::model::{Block, Song};
use crate::notation;
use crate::parser;
use crate::rewriter;
use crate::text_rewriter;

const NOTE_COUNT: i32 = 12;
const SHARP_NAMES: [&str; 12] = ["C", "C#", "D", "D#", "E", "F", "F#", "G", "G#", "A", "A#", "B"];
const FLAT_NAMES: [&str; 12] = ["C", "Db", "D", "Eb", "E", "F", "Gb", "G", "Ab", "A", "Bb", "B"];
const FLAT_MAJOR_KEYS: [&str; 6] = ["C", "F", "Bb", "Eb", "Ab", "Db"];
const FLAT_MINOR_KEYS: [&str; 6] = ["Dm", "Gm", "Cm", "Fm", "Bbm", "Ebm"];
const MINOR_KEY_WORDS: [&str; 6] = ["minor", "min", "moll", "menor", "mineur", "minore"];
const KEY_WORDS: [&str; 13] = ["major", "minor", "maj", "min", "dur", "dúr", "moll", "mayor", "menor", "majeur", "mineur", "maggiore", "minore"];

/// Transposes a chord name, keeping an optional chord's parentheses and its bass note.
///
/// A word that is not a chord name is returned as it was. Brackets are how a chart marks its parts as often as they hold
/// chords — `[Intro]`, `[Break]`, `[Chorus 2x]` — and every one of those that starts with a note letter would otherwise be
/// moved, with the `b` after an `E` eaten as a flat sign.
pub fn transpose_chord(name: &str, semitones: i32, prefer_flats: bool) -> String {
    if !chord_names::is_chord_name(name) {
        return name.to_string();
    }
    chord_names::rewrite_notes(name, &|note| transpose_note(note, semitones, prefer_flats))
}

fn note_index(letter: Option<&char>) -> Option<i32> {
    Some(match letter? {
        'C' => 0,
        'D' => 2,
        'E' => 4,
        'F' => 5,
        'G' => 7,
        'A' => 9,
        'B' | 'H' => 11,
        _ => return None,
    })
}

fn accidental(sign: Option<&char>) -> Option<i32> {
    match sign? {
        '#' | '♯' => Some(1),
        'b' | '♭' => Some(-1),
        _ => None,
    }
}

fn transpose_note(part: &str, semitones: i32, prefer_flats: bool) -> String {
    let chars: Vec<char> = part.chars().collect();
    let Some(note_index) = note_index(chars.first()) else {
        return part.to_string();
    };
    let accidental = accidental(chars.get(1));
    let suffix_start = if accidental.is_none() { 1 } else { 2 };
    let transposed = (note_index + accidental.unwrap_or(0) + semitones).rem_euclid(NOTE_COUNT) as usize;
    let names = if prefer_flats { &FLAT_NAMES } else { &SHARP_NAMES };
    format!("{}{}", names[transposed], chars[suffix_start..].iter().collect::<String>())
}

/// A key: its tonic as a pitch class and whether it is minor.
struct Key {
    tonic: i32,
    is_minor: bool,
}

impl Key {
    fn prefers_flats(&self) -> bool {
        let name = FLAT_NAMES[self.tonic as usize];
        if self.is_minor {
            FLAT_MINOR_KEYS.contains(&format!("{name}m").as_str())
        } else {
            FLAT_MAJOR_KEYS.contains(&name)
        }
    }

    fn transposed_by(&self, semitones: i32) -> Key {
        Key { tonic: (self.tonic + semitones).rem_euclid(NOTE_COUNT), is_minor: self.is_minor }
    }
}

/// The word a key spelled out in words names its mode with, in lowercase: `minor` for `A minor`, `moll` for `a-moll`.
fn key_word_of(suffix: &str) -> String {
    suffix.trim_start_matches(|c: char| c.is_whitespace() || c == '-').to_lowercase()
}

fn key_of(name: &str) -> Option<Key> {
    let root_note = chord_names::notes(name.trim()).into_iter().next()?;
    let chars: Vec<char> = root_note.chars().collect();
    let note_index = note_index(chars.first())?;
    let accidental = accidental(chars.get(1));
    let suffix: String = chars[if accidental.is_none() { 1 } else { 2 }..].iter().collect::<String>();
    let suffix = suffix.trim_start().to_string();
    let word = key_word_of(&suffix);
    let is_minor = if KEY_WORDS.contains(&word.as_str()) {
        MINOR_KEY_WORDS.contains(&word.as_str())
    } else {
        suffix.starts_with('m') && !suffix.to_lowercase().starts_with("maj")
    };
    Some(Key { tonic: (note_index + accidental.unwrap_or(0)).rem_euclid(NOTE_COUNT), is_minor })
}

fn is_written_in_flats(names: &[String]) -> bool {
    let mut flats = 0;
    let mut sharps = 0;
    for note in names.iter().filter(|name| chord_names::is_chord_name(name)).flat_map(|name| chord_names::notes(name)) {
        let chars: Vec<char> = note.chars().collect();
        if note_index(chars.first()).is_some() {
            match accidental(chars.get(1)) {
                Some(1) => sharps += 1,
                Some(-1) => flats += 1,
                _ => {}
            }
        }
    }
    flats > sharps
}

/// Whether flats should be preferred for the key a song arrives in after moving it by `semitones`: the key it names, else
/// the one its first chord names, else whichever accidental its chords are written with most.
pub fn prefers_flats(key: Option<&str>, written_chord_names: &[String], semitones: i32) -> bool {
    let key = key.and_then(key_of).or_else(|| written_chord_names.iter().find(|name| chord_names::is_chord_name(name)).and_then(|name| key_of(name)));
    match key {
        Some(key) => key.transposed_by(semitones).prefers_flats(),
        None => is_written_in_flats(written_chord_names),
    }
}

/// Transposes every chord in the model (lyrics chords, grid chords, the brackets of comments and labels; never annotations)
/// and the key, each stretch of the song by `semitones` plus the offset of the `{transpose}` in force where it stands.
///
/// `prefer_flats` forces the spelling of the accidentals; `None` leaves it to [`prefers_flats`], which is what the song itself
/// asks for. Forced, it is worth doing even for no semitones at all: respelling the chords of a song that is not transposed is
/// exactly what a reader who always wants flats (or sharps) is asking for.
pub fn transpose_song(song: &Song, semitones: i32, prefer_flats: Option<bool>) -> Song {
    let is_modulated = song.blocks.iter().any(|block| matches!(block, Block::Transpose { .. }));
    if semitones == 0 && prefer_flats.is_none() && !is_modulated {
        return song.clone();
    }
    let names = rewriter::written_chord_names(song);
    let key = song.metadata.key.as_deref();
    // The spelling of every stretch is decided once: it asks for the chord names of the whole song.
    let mut flats: HashMap<i32, bool> = HashMap::new();
    let offsets = std::iter::once(0).chain(song.blocks.iter().filter_map(|block| match block {
        Block::Transpose { semitones, .. } => Some(*semitones),
        _ => None,
    }));
    for offset in offsets {
        let shift = semitones + offset;
        let is_moved = shift != 0 || prefer_flats.is_some();
        flats.entry(offset).or_insert_with(|| is_moved && prefer_flats.unwrap_or_else(|| prefers_flats(key, &names, shift)));
    }
    rewriter::rewrite_chords(song, &|offset, name| {
        let shift = semitones + offset;
        if shift == 0 && prefer_flats.is_none() {
            return name.to_string();
        }
        transpose_chord(name, shift, flats.get(&offset).copied().unwrap_or(false))
    })
}

/// `rename` applied to a lowercase minor (`a` for `Am`, `f#7` for `F#m7`) as the chord it stands for, the result written the same
/// way again, so that a chart written in lowercase stays written in lowercase after it is moved.
fn keeping_lowercase_minors<'a>(rename: impl Fn(&str) -> String + 'a) -> impl Fn(&str) -> String + 'a {
    move |name| match chord_names::lowercase_minor_expanded(name) {
        Some(expanded) => chord_names::lowercase_minor_folded(&rename(&expanded)),
        None => rename(name),
    }
}

/// Transposes raw ChordPro text in place, keeping its formatting and its original notation: a German chart stays German as long
/// as its chords can be, a chart in lowercase minors stays so. `{transpose}` is left alone, since each one is relative to the
/// song as written.
pub fn transpose_text(text: &str, semitones: i32, prefer_flats: Option<bool>) -> String {
    if semitones == 0 && prefer_flats.is_none() {
        return text.to_string();
    }
    let written = parser::parse_as_written(text);
    let names = rewriter::written_chord_names(&written);
    let is_german = names.iter().any(|name| notation::is_german_name(name));
    let standard_of = |name: &str| {
        let expanded = chord_names::lowercase_minor_expanded(name).unwrap_or_else(|| name.to_string());
        if is_german {
            notation::from_german(&expanded)
        } else {
            expanded
        }
    };
    let standard_names: Vec<String> = names.iter().map(|name| standard_of(name)).collect();
    let key = written.metadata.key.as_deref().map(standard_of);
    let flats = prefer_flats.unwrap_or_else(|| prefers_flats(key.as_deref(), &standard_names, semitones));
    let transpose_name = |name: &str| transpose_chord(name, semitones, flats);
    if !is_german {
        let rename = keeping_lowercase_minors(transpose_name);
        return text_rewriter::rewrite_chord_names_in_text(text, &rename);
    }
    let stays_german = standard_names.iter().any(|name| notation::is_german_name(&notation::to_german(&transpose_name(name))));
    let rename = keeping_lowercase_minors(move |name: &str| {
        let transposed = transpose_name(&notation::from_german(name));
        if stays_german {
            notation::to_german(&transposed)
        } else {
            transposed
        }
    });
    text_rewriter::rewrite_chord_names_in_text(text, &rename)
}

/// Whether a `{key}` value names a key: a note with anything after it (`D dorian`, `Dm (capo 2)`), but not `S`.
pub fn is_key(key: &str) -> bool {
    let text = key.trim();
    let note_end = text.char_indices().skip(1).find(|(_, c)| c.is_whitespace() || *c == '-' || *c == '(').map_or(text.len(), |(index, _)| index);
    let note = &text[..note_end];
    chord_names::is_chord_name(note) || chord_names::lowercase_minor_expanded(note).is_some() || chord_names::latin_expanded(note).is_some()
}

/// Whether [`transpose_text`] moves a song's `{key}` of `key`, so that a stepper offering to move a song with no chords offers it
/// only where the key would change: `D dorian` stays as written.
pub fn is_movable_key(key: &str) -> bool {
    let line = format!("{{key: {key}}}");
    transpose_text(&line, 1, None) != line
}

#[cfg(test)]
mod tests {
    use super::*;

    fn names(names: &[&str]) -> Vec<String> {
        names.iter().map(|name| name.to_string()).collect()
    }

    #[test]
    fn a_chord_moves_with_its_bass_and_its_parentheses() {
        assert_eq!(transpose_chord("C", 2, false), "D");
        assert_eq!(transpose_chord("Am7", 3, false), "Cm7");
        assert_eq!(transpose_chord("D/F#", 2, false), "E/G#");
        assert_eq!(transpose_chord("D/F#", 2, true), "E/Ab");
        assert_eq!(transpose_chord("(Bm7/F#)", 1, false), "(Cm7/G)");
        assert_eq!(transpose_chord("C", -1, false), "B");
        assert_eq!(transpose_chord("C", 12, false), "C");
        assert_eq!(transpose_chord("D/f#", 2, false), "E/g#");
    }

    #[test]
    fn a_word_that_is_no_chord_stays() {
        for name in ["Chorus 2x", "Bridge", "N.C.", "Break"] {
            assert_eq!(transpose_chord(name, 2, false), name);
        }
    }

    #[test]
    fn flat_keys_ask_for_flats() {
        assert!(prefers_flats(Some("F"), &[], 0));
        assert!(prefers_flats(Some("Dm"), &[], 0));
        assert!(!prefers_flats(Some("G"), &[], 0));
        assert!(!prefers_flats(Some("Em"), &[], 0));
        assert!(prefers_flats(Some("C"), &[], 5));
        assert!(!prefers_flats(Some("F"), &[], 2));
        assert!(prefers_flats(Some("A minor"), &[], 5));
        assert!(prefers_flats(Some("A-moll"), &[], 5));
    }

    #[test]
    fn without_a_key_the_first_chord_decides_then_the_accidentals() {
        assert!(prefers_flats(None, &names(&["F", "Bb", "C"]), 0));
        assert!(!prefers_flats(None, &names(&["G", "D", "Em"]), 0));
        assert!(prefers_flats(None, &names(&["N.C.", "Gm", "Cm"]), 0));
    }

    #[test]
    fn a_text_is_moved_where_the_chords_are_and_nowhere_else() {
        let text = "{title: Am I Blue}\n{key: Am}\n\n[Am]Hello [G]world\n{c: Intro [Am] [F]}\n";
        assert_eq!(transpose_text(text, 2, None), "{title: Am I Blue}\n{key: Bm}\n\n[Bm]Hello [A]world\n{c: Intro [Bm] [G]}\n");
        assert_eq!(transpose_text(text, 0, None), text);
        assert_eq!(transpose_text("[C]x\r\n[G]y\r\n", 1, Some(true)), "[Db]x\r\n[Ab]y\r\n");
    }

    #[test]
    fn the_spelling_follows_the_key_it_arrives_in() {
        assert_eq!(transpose_text("{key: C}\n[C]x [F]y\n", 5, None), "{key: F}\n[F]x [Bb]y\n");
        assert_eq!(transpose_text("{key: C}\n[C]x [F]y\n", 2, None), "{key: D}\n[D]x [G]y\n");
        assert_eq!(transpose_text("[C]x\n", 1, Some(false)), "[C#]x\n");
    }

    #[test]
    fn a_lowercase_minor_stays_lowercase() {
        assert_eq!(transpose_text("[a]x [e]y [C]z\n", 2, None), "[b]x [f#]y [D]z\n");
    }

    #[test]
    fn a_german_chart_stays_german() {
        // `H` is B natural: a semitone up is C, and the minor E is F minor.
        assert_eq!(transpose_text("[H]x [Em]y\n", 1, Some(false)), "[C]x [Fm]y\n");
        assert_eq!(transpose_text("[G]x [H7]y\n", 2, Some(false)), "[A]x [C#7]y\n");
    }

    #[test]
    fn a_key_in_words_moves_with_its_note_only_where_it_can() {
        assert!(is_movable_key("Bb major"));
        assert!(is_movable_key("F#m"));
        assert!(!is_movable_key("D dorian"));
        assert!(is_key("D dorian"));
        assert!(is_key("Dm (capo 2)"));
        assert!(!is_key("S"));
        assert!(!is_key("capo 2"));
    }
}

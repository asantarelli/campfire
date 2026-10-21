// This file is part of Campfire.
// Copyright (c) Pandula Péter 2017-2026.
// https://github.com/pandulapeter/campfire
//
// This Source Code Form is subject to the terms of the Mozilla Public License, v. 2.0.
// If a copy of the MPL was not distributed with this file, You can obtain one at
// https://mozilla.org/MPL/2.0/.

//! Converts chord names between the standard notation, which every file is stored in, and the notation a reader prefers.
//!
//! German notation writes the note the standard notation calls `B` as `H`, and the note it calls `Bb` as `B`. Latin
//! notation names the seven notes `Do Re Mi Fa Sol La Si`. Since no Latin name is a standard one, a Latin chord is read as
//! the chord it names wherever it stands. Only the conversion of single names lives here; the conversion of a whole text
//! comes with the song model.

use crate::chord_names;

/// How the notes of a chord are named.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChordNotation {
    /// `C D E F G A B`, the one every file and the model are in.
    Standard,
    /// `H` for `B` and `B` for `Bb`.
    German,
    /// `Do Re Mi Fa Sol La Si`.
    Latin,
    /// The steps of the song's key as numbers, for showing only.
    Nashville,
    /// The steps of the song's key as Roman numerals, for showing only.
    Roman,
}

impl ChordNotation {
    /// Whether the chords are counted from the song's key rather than named.
    pub fn is_numbering(self) -> bool {
        matches!(self, ChordNotation::Nashville | ChordNotation::Roman)
    }
}

const SHARP_SIGN: char = '♯';
const FLAT_SIGN: char = '♭';

/// One chord name in the standard notation as `notation` writes it where it is no step of a key: in a numbering the
/// letters it is written in, which is what a note on its own, or a chord with no song around it, is named by.
pub fn shown_name(name: &str, notation: ChordNotation) -> String {
    match notation {
        ChordNotation::Standard | ChordNotation::Nashville | ChordNotation::Roman => name.to_string(),
        ChordNotation::German => to_german(name),
        ChordNotation::Latin => to_latin(name),
    }
}

/// The chord `name` stands for, typed in `notation`, named the way the app names it in `notation`. `None` where `name`
/// is no chord.
pub fn canonical_name(name: &str, notation: ChordNotation) -> Option<String> {
    standard_name(name, notation).map(|standard| shown_name(&standard, notation))
}

/// The chord `name` stands for, typed in `notation`, named in the standard notation, which is what a file stores: its root
/// and its bass in capitals, a lowercase minor spelled out, the parentheses and the `♯` and `♭` signs dropped. `None`
/// where `name` is no chord.
pub fn standard_name(name: &str, notation: ChordNotation) -> Option<String> {
    let standard = read(name.trim(), notation == ChordNotation::German);
    if !chord_names::is_chord_name(&standard) {
        return None;
    }
    let notes = chord_names::notes(&standard);
    let bass = notes.get(1).map(|note| {
        let mut chars = note.chars();
        let first: String = chars.next().into_iter().flat_map(char::to_uppercase).collect();
        format!("{first}{}", chars.as_str())
    });
    let parts: Vec<&str> = std::iter::once(notes[0].as_str()).chain(bass.as_deref()).collect();
    Some(parts.join("/"))
}

/// The standard chord `name` stands for as a file or a field writes it: a Latin name as the chord it names, a lowercase
/// minor spelled out, a German one read as German where `is_german`, and the accidentals in ASCII. The expansion comes
/// before the German reading, so that a German `b` (B flat minor) becomes `Bm` and then `Bbm`. Anything that is no chord is
/// returned with its accidentals folded and nothing else changed.
pub fn read(name: &str, is_german: bool) -> String {
    if let Some(latin) = chord_names::latin_expanded(name) {
        return with_ascii_accidentals(&latin);
    }
    let expanded = chord_names::lowercase_minor_expanded(name).unwrap_or_else(|| name.to_string());
    with_ascii_accidentals(&if is_german { from_german(&expanded) } else { expanded })
}

/// Rewrites one chord name, its bass note included, in German notation. A word that is not a chord is returned as it was,
/// which is what keeps this from turning prose starting with a `B` into prose starting with an `H`.
pub fn to_german(name: &str) -> String {
    if !chord_names::is_chord_name(name) {
        return name.to_string();
    }
    chord_names::rewrite_notes(name, &note_to_german)
}

/// The same in Latin notation: `Am` is `Lam`, `Bb7` is `Sib7`, `D/f#` is `Re/fa#`.
pub fn to_latin(name: &str) -> String {
    if !chord_names::is_chord_name(name) {
        return name.to_string();
    }
    chord_names::rewrite_notes(name, &|note| {
        let mut chars = note.chars();
        match chars.next().and_then(latin_name) {
            Some(latin) => format!("{latin}{}", chars.as_str()),
            None => note.to_string(),
        }
    })
}

/// Whether `name` is a real chord that uses German notation's `H`, at its root or bass, a lowercase `h` minor or bass note
/// included.
pub fn is_german_name(name: &str) -> bool {
    let chord = chord_names::lowercase_minor_expanded(name).unwrap_or_else(|| name.to_string());
    chord_names::is_chord_name(&chord) && chord_names::notes(&chord).iter().any(|note| note.to_uppercase().starts_with('H'))
}

/// Reads a German-notated chord into the notation the rest of the app works in.
pub fn from_german(name: &str) -> String {
    if !chord_names::is_chord_name(name) {
        return name.to_string();
    }
    chord_names::rewrite_notes(name, &note_from_german)
}

/// A chord name with musical accidental signs folded to the ASCII spelling the app writes.
pub fn with_ascii_accidentals(name: &str) -> String {
    name.replace(SHARP_SIGN, "#").replace(FLAT_SIGN, "b")
}

/// One chord name as a screen or a page shows it: every sharp and flat as the musical sign `♯` or `♭` rather than the `#`
/// and `b` a file holds, so that `Bbm7` reads `B♭m7`. Only for showing. A `b` that is a note (the lowercase bass of `D/b`)
/// stays, and a word that is no chord is returned as it is.
pub fn with_accidental_signs(name: &str) -> String {
    if !chord_names::is_chord_name(name) && chord_names::latin_expanded(name).is_none() {
        return name.to_string();
    }
    // A root is a capital (`H` included) or a Latin name, so a lowercase `b` is a note only where a bass is written.
    let chars: Vec<char> = name.chars().collect();
    chars
        .iter()
        .enumerate()
        .map(|(index, &char)| match char {
            '#' => SHARP_SIGN,
            'b' if index > 0 && chars[index - 1] != '/' => FLAT_SIGN,
            other => other,
        })
        .collect()
}

fn latin_name(letter: char) -> Option<&'static str> {
    Some(match letter {
        'C' => "Do",
        'D' => "Re",
        'E' => "Mi",
        'F' => "Fa",
        'G' => "Sol",
        'A' => "La",
        'B' => "Si",
        _ => return None,
    })
}

fn note_to_german(note: &str) -> String {
    // `H` is already the German name of the note, and `Hb` an unusual but unambiguous way of writing the flat one.
    let chars: Vec<char> = note.chars().collect();
    match chars.first() {
        Some('B' | 'H') => {
            if matches!(chars.get(1).copied(), Some('b' | '♭')) {
                // Bb -> B: the flat is in the letter now, so its sign goes away with it.
                format!("B{}", chars[2..].iter().collect::<String>())
            } else {
                // B -> H, and B# -> H#, which the classical German names would spell His.
                format!("H{}", chars[1..].iter().collect::<String>())
            }
        }
        _ => note.to_string(),
    }
}

fn note_from_german(note: &str) -> String {
    let chars: Vec<char> = note.chars().collect();
    match chars.first() {
        Some('H') => format!("B{}", chars[1..].iter().collect::<String>()),
        Some('B') => {
            if matches!(chars.get(1).copied(), Some('#' | 'b' | '♯' | '♭')) {
                note.to_string()
            } else {
                format!("Bb{}", chars[1..].iter().collect::<String>())
            }
        }
        _ => note.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn german_writes_h_for_b_and_b_for_b_flat() {
        assert_eq!(to_german("B"), "H");
        assert_eq!(to_german("Bb"), "B");
        assert_eq!(to_german("Bbm7/F"), "Bm7/F");
        assert_eq!(to_german("F/B"), "F/H");
        assert_eq!(to_german("Bridge"), "Bridge");
        assert_eq!(from_german("H7"), "B7");
        assert_eq!(from_german("B"), "Bb");
        assert_eq!(from_german("Hm/F#"), "Bm/F#");
    }

    #[test]
    fn latin_names_the_seven_notes() {
        assert_eq!(to_latin("Am"), "Lam");
        assert_eq!(to_latin("Bb7"), "Sib7");
        assert_eq!(to_latin("D/f#"), "Re/fa#");
        assert_eq!(to_latin("N.C."), "N.C.");
    }

    #[test]
    fn a_name_typed_in_a_notation_is_named_in_the_standard_one() {
        assert_eq!(standard_name("(a)", ChordNotation::Standard).as_deref(), Some("Am"));
        assert_eq!(standard_name("c7", ChordNotation::Standard).as_deref(), Some("Cm7"));
        assert_eq!(standard_name("D/f#", ChordNotation::Standard).as_deref(), Some("D/F#"));
        assert_eq!(standard_name("Sib", ChordNotation::Standard).as_deref(), Some("Bb"));
        assert_eq!(standard_name("B", ChordNotation::German).as_deref(), Some("Bb"));
        assert_eq!(standard_name("H", ChordNotation::German).as_deref(), Some("B"));
        assert_eq!(standard_name("Bridge", ChordNotation::Standard), None);
        assert_eq!(canonical_name("Sib", ChordNotation::Latin).as_deref(), Some("Sib"));
    }

    #[test]
    fn accidental_signs_are_only_for_showing() {
        assert_eq!(with_accidental_signs("Bbm7"), "B♭m7");
        assert_eq!(with_accidental_signs("F#m/C#"), "F♯m/C♯");
        assert_eq!(with_accidental_signs("D/b"), "D/b");
        assert_eq!(with_accidental_signs("N.C."), "N.C.");
        assert_eq!(with_ascii_accidentals("B♭m7/F♯"), "Bbm7/F#");
    }
}

// This file is part of Campfire.
// Copyright (c) Pandula Péter 2017-2026.
// https://github.com/pandulapeter/campfire
//
// This Source Code Form is subject to the terms of the Mozilla Public License, v. 2.0.
// If a copy of the MPL was not distributed with this file, You can obtain one at
// https://mozilla.org/MPL/2.0/.

//! Brings a song whose file was written in another notation into the one the app works in: English note names and ASCII
//! accidentals.

use crate::chord_names;
use crate::model::Song;
use crate::notation::{self, ChordNotation};
use crate::rewriter;

/// `song`, its file written in `notation`, in the app's own notation. A file in German notation says so by using an `H`
/// chord anywhere; lowercase minors (`a`) are spelled out and Latin names named in English, whatever the notation is said to be.
pub(crate) fn normalized(song: Song, notation: ChordNotation) -> Song {
    let names = rewriter::written_chord_names(&song);
    let german = notation == ChordNotation::German || names.iter().any(|name| notation::is_german_name(name));
    let has_lowercase_minors = names.iter().any(|name| chord_names::lowercase_minor_expanded(name).is_some());
    let has_latin_name = names.iter().any(|name| chord_names::latin_expanded(name).is_some());
    let has_accidental_signs = names.iter().any(|name| name.contains('♯') || name.contains('♭'));
    if !german && !has_lowercase_minors && !has_latin_name && !has_accidental_signs {
        return song;
    }
    rewriter::rewrite_chords(&song, &|_, name| notation::read(name, german))
}

#[cfg(test)]
mod tests {
    use crate::model::{Block, Line};
    use crate::notation::ChordNotation;
    use crate::parser;

    fn chord_names(text: &str, notation: ChordNotation) -> Vec<String> {
        let song = parser::parse(text, notation);
        let mut names = Vec::new();
        for block in &song.blocks {
            if let Block::Section(section) = block {
                for line in &section.lines {
                    if let Line::Lyrics(lyrics) = line {
                        names.extend(lyrics.chords.iter().map(|chord| chord.name.clone()));
                    }
                }
            }
        }
        names
    }

    #[test]
    fn a_standard_file_is_left_as_it_is() {
        assert_eq!(chord_names("[Am]a [Bb]b [F#m7/C#]c [N.C.]d\n", ChordNotation::Standard), ["Am", "Bb", "F#m7/C#", "N.C."]);
    }

    #[test]
    fn an_h_chord_makes_a_german_file() {
        assert_eq!(chord_names("[H]a [B]b [Hm]c [Fis]d\n", ChordNotation::Standard), ["B", "Bb", "Bm", "Fis"]);
    }

    #[test]
    fn lowercase_minors_latin_names_and_accidental_signs_are_brought_in() {
        assert_eq!(chord_names("[a]x [Sol]y [Lam]z [Sib7]w\n", ChordNotation::Standard), ["Am", "G", "Am", "Bb7"]);
        assert_eq!(chord_names("[B♭]x [F♯m]y\n", ChordNotation::Standard), ["Bb", "F#m"]);
    }

    #[test]
    fn a_text_typed_in_german_is_read_as_german() {
        assert_eq!(chord_names("[B]x [E]y\n", ChordNotation::German), ["Bb", "E"]);
    }

    #[test]
    fn annotations_and_comments_do_not_vote() {
        assert_eq!(chord_names("[*Hold]x [Bb]y\n{c: Hallelujah [H]}\n", ChordNotation::Standard), ["Hold", "Bb"]);
    }
}

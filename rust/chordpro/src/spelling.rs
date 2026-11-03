// This file is part of Campfire.
// Copyright (c) Pandula Péter 2017-2026.
// https://github.com/pandulapeter/campfire
//
// This Source Code Form is subject to the terms of the Mozilla Public License, v. 2.0.
// If a copy of the MPL was not distributed with this file, You can obtain one at
// https://mozilla.org/MPL/2.0/.

//! The notes of a chord as the chord names them: spelled by their place above its root, so that the minor third of a G is a B♭ and
//! the third of a D an F♯, and numbered by that place.

const ROOT_NAMES: [&str; 12] = ["C", "D♭", "D", "E♭", "E", "F", "F♯", "G", "A♭", "A", "B♭", "B"];
const SHARP_NAMES: [&str; 12] = ["C", "C♯", "D", "D♯", "E", "F", "F♯", "G", "G♯", "A", "A♯", "B"];
const LETTERS: [char; 7] = ['C', 'D', 'E', 'F', 'G', 'A', 'B'];
const NATURAL_PITCH_CLASSES: [i32; 7] = [0, 2, 4, 5, 7, 9, 11];

/// How many letters above the root's the note a number of semitones above it is spelled, and its place in the chord.
const STEPS: [(usize, &str); 12] = [(0, "R"), (1, "♭2"), (1, "2"), (2, "♭3"), (2, "3"), (3, "4"), (4, "♭5"), (4, "5"), (4, "♯5"), (5, "6"), (6, "♭7"), (6, "7")];

/// The name of the note of `pitch_class` as the chord on `root` spells it. The root is spelled the usual way (D♭ and not C♯,
/// F♯ and not G♭). Without a root (a shape of the song's own name) the notes are spelled with sharps.
pub fn note_label(pitch_class: i32, root: Option<i32>) -> String {
    let pitch = pitch_class.rem_euclid(12);
    let Some(root) = root.map(|root| root.rem_euclid(12)) else {
        return SHARP_NAMES[pitch as usize].to_string();
    };
    let interval = (pitch - root).rem_euclid(12) as usize;
    let root_letter = ROOT_NAMES[root as usize].chars().next().unwrap_or('C');
    let root_index = LETTERS.iter().position(|&letter| letter == root_letter).unwrap_or(0);
    let letter_index = (root_index + STEPS[interval].0) % LETTERS.len();
    let offset = (pitch - NATURAL_PITCH_CLASSES[letter_index] + 6).rem_euclid(12) - 6;
    let accidental = match offset {
        1 => "♯",
        -1 => "♭",
        2 => "𝄪",
        -2 => "𝄫",
        _ => "",
    };
    format!("{}{accidental}", LETTERS[letter_index])
}

/// The place of the note of `pitch_class` in the chord on `root`: `R`, `♭3`, `5`, `♭7`.
pub fn degree_label(pitch_class: i32, root: Option<i32>) -> Option<&'static str> {
    let root = root?.rem_euclid(12);
    Some(STEPS[(pitch_class - root).rem_euclid(12) as usize].1)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn notes(root: i32, pitch_classes: &[i32]) -> Vec<String> {
        pitch_classes.iter().map(|&pitch| note_label(pitch, Some(root))).collect()
    }

    #[test]
    fn the_notes_of_a_chord_are_spelled_by_their_place_above_its_root() {
        assert_eq!(notes(0, &[0, 4, 7]), ["C", "E", "G"]);
        assert_eq!(notes(7, &[7, 10, 2]), ["G", "B♭", "D"]);
        assert_eq!(notes(2, &[2, 6, 9]), ["D", "F♯", "A"]);
        assert_eq!(notes(10, &[10, 2, 5]), ["B♭", "D", "F"]);
        assert_eq!(notes(6, &[6, 10, 1]), ["F♯", "A♯", "C♯"]);
        assert_eq!(notes(8, &[8, 0, 3]), ["A♭", "C", "E♭"]);
    }

    #[test]
    fn a_shape_with_no_root_spells_its_notes_with_sharps_and_numbers_none() {
        assert_eq!(note_label(1, None), "C♯");
        assert_eq!(note_label(10, None), "A♯");
        assert_eq!(degree_label(4, None), None);
    }

    #[test]
    fn the_places_in_a_chord_are_named_as_a_player_counts_them() {
        let places: Vec<_> = [0, 3, 7, 10, 4, 11, 9].iter().map(|&pitch| degree_label(pitch, Some(0))).collect();
        assert_eq!(places, [Some("R"), Some("♭3"), Some("5"), Some("♭7"), Some("3"), Some("7"), Some("6")]);
        assert_eq!(degree_label(9, Some(9)), Some("R"));
        assert_eq!(degree_label(4, Some(9)), Some("5"));
    }
}

// This file is part of Campfire.
// Copyright (c) Pandula Péter 2017-2026.
// https://github.com/pandulapeter/campfire
//
// This Source Code Form is subject to the terms of the Mozilla Public License, v. 2.0.
// If a copy of the MPL was not distributed with this file, You can obtain one at
// https://mozilla.org/MPL/2.0/.

//! Scales: the notes of each from a root, spelled the way a chart spells them, and where they lie on a fretted neck.

use crate::voicings::Tuning;

/// A scale: its name and the semitones of its notes above the root.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Scale {
    pub name: &'static str,
    pub intervals: &'static [i32],
}

pub const SCALES: [Scale; 12] = [
    Scale { name: "Mayor", intervals: &[0, 2, 4, 5, 7, 9, 11] },
    Scale { name: "Menor natural", intervals: &[0, 2, 3, 5, 7, 8, 10] },
    Scale { name: "Menor armónica", intervals: &[0, 2, 3, 5, 7, 8, 11] },
    Scale { name: "Menor melódica", intervals: &[0, 2, 3, 5, 7, 9, 11] },
    Scale { name: "Pentatónica mayor", intervals: &[0, 2, 4, 7, 9] },
    Scale { name: "Pentatónica menor", intervals: &[0, 3, 5, 7, 10] },
    Scale { name: "Blues", intervals: &[0, 3, 5, 6, 7, 10] },
    Scale { name: "Dórica", intervals: &[0, 2, 3, 5, 7, 9, 10] },
    Scale { name: "Frigia", intervals: &[0, 1, 3, 5, 7, 8, 10] },
    Scale { name: "Lidia", intervals: &[0, 2, 4, 6, 7, 9, 11] },
    Scale { name: "Mixolidia", intervals: &[0, 2, 4, 5, 7, 9, 10] },
    Scale { name: "Locria", intervals: &[0, 1, 3, 5, 6, 8, 10] },
];

const LETTERS: [char; 7] = ['C', 'D', 'E', 'F', 'G', 'A', 'B'];
const LETTER_PITCH: [i32; 7] = [0, 2, 4, 5, 7, 9, 11];

/// The pitch class of a root written as `C`, `F#` or `Bb`.
pub fn root_pitch_class(root: &str) -> Option<i32> {
    crate::chord::pitch_class(root)
}

/// The notes of `scale` from `root` (as written, `Eb`), one letter to a note for a scale of seven, so that D major is
/// `D E F# G A B C#` and F major has its `Bb`; a scale of five or six notes is spelled with the root's own accidentals.
pub fn note_names(root: &str, scale: &Scale) -> Option<Vec<String>> {
    let root_class = root_pitch_class(root)?;
    let root_letter = LETTERS.iter().position(|&letter| root.starts_with(letter))?;
    let uses_flats = root.contains('b') || root.contains('♭');
    Some(
        scale
            .intervals
            .iter()
            .enumerate()
            .map(|(degree, interval)| {
                let class = (root_class + interval).rem_euclid(12);
                if scale.intervals.len() == 7 {
                    let letter = (root_letter + degree) % 7;
                    match (class - LETTER_PITCH[letter]).rem_euclid(12) {
                        0 => LETTERS[letter].to_string(),
                        1 => format!("{}#", LETTERS[letter]),
                        11 => format!("{}b", LETTERS[letter]),
                        // A double accidental is spelled as the note it sounds like.
                        _ => fallback(class, uses_flats),
                    }
                } else {
                    fallback(class, uses_flats)
                }
            })
            .collect(),
    )
}

fn fallback(class: i32, uses_flats: bool) -> String {
    (if uses_flats { crate::chord::FLAT_NAMES } else { crate::chord::SHARP_NAMES })[class as usize].to_string()
}

/// A chord a scale stacks, with the Roman numeral of the step it stands on.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScaleChord {
    pub degree: String,
    pub name: String,
}

const ROMAN: [&str; 7] = ["I", "II", "III", "IV", "V", "VI", "VII"];

/// The chord every step of a scale of seven stacks in thirds: triads, or sevenths where `with_seventh`. A step whose stack is no
/// chord a chart names is left out. `None` for a scale that does not have seven notes.
pub fn scale_chords(root: &str, scale: &Scale, with_seventh: bool) -> Option<Vec<ScaleChord>> {
    if scale.intervals.len() != 7 {
        return None;
    }
    let names = note_names(root, scale)?;
    let above = |from: usize, steps: usize| (scale.intervals[(from + steps) % 7] - scale.intervals[from]).rem_euclid(12);
    let mut chords = Vec::new();
    for degree in 0..7 {
        let (third, fifth, seventh) = (above(degree, 2), above(degree, 4), above(degree, 6));
        let (suffix, numeral) = if with_seventh {
            match (third, fifth, seventh) {
                (4, 7, 11) => ("maj7", ROMAN[degree].to_string() + "maj7"),
                (4, 7, 10) => ("7", ROMAN[degree].to_string() + "7"),
                (3, 7, 10) => ("m7", ROMAN[degree].to_lowercase() + "7"),
                (3, 6, 10) => ("m7b5", ROMAN[degree].to_lowercase() + "ø7"),
                (3, 6, 9) => ("dim7", ROMAN[degree].to_lowercase() + "°7"),
                (3, 7, 11) => ("m(maj7)", ROMAN[degree].to_lowercase() + "maj7"),
                (4, 8, 11) => ("maj7#5", ROMAN[degree].to_string() + "maj7+"),
                (4, 8, 10) => ("7#5", ROMAN[degree].to_string() + "7+"),
                _ => continue,
            }
        } else {
            match (third, fifth) {
                (4, 7) => ("", ROMAN[degree].to_string()),
                (3, 7) => ("m", ROMAN[degree].to_lowercase()),
                (3, 6) => ("dim", ROMAN[degree].to_lowercase() + "°"),
                (4, 8) => ("aug", ROMAN[degree].to_string() + "+"),
                _ => continue,
            }
        };
        chords.push(ScaleChord { degree: numeral, name: format!("{}{suffix}", names[degree]) });
    }
    Some(chords)
}

/// Where the notes of the scale are on the neck: for each string (from the lowest) and each fret from 0 to `frets`, whether the note
/// there is one of the scale, and whether it is the root.
pub fn on_fretboard(root: &str, scale: &Scale, tuning: &Tuning, frets: u8) -> Option<Vec<Vec<(bool, bool)>>> {
    let root_class = root_pitch_class(root)?;
    let classes: Vec<i32> = scale.intervals.iter().map(|interval| (root_class + interval).rem_euclid(12)).collect();
    Some(
        tuning
            .strings
            .iter()
            .map(|string| (0..=frets).map(|fret| {
                let class = (string + fret as i32).rem_euclid(12);
                (classes.contains(&class), class == root_class)
            }).collect())
            .collect(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn notes(root: &str, scale: usize) -> String {
        note_names(root, &SCALES[scale]).unwrap().join(" ")
    }

    #[test]
    fn a_scale_of_seven_is_spelled_one_letter_to_a_note() {
        assert_eq!(notes("C", 0), "C D E F G A B");
        assert_eq!(notes("D", 0), "D E F# G A B C#");
        assert_eq!(notes("F", 0), "F G A Bb C D E");
        assert_eq!(notes("Bb", 0), "Bb C D Eb F G A");
        assert_eq!(notes("A", 1), "A B C D E F G");
        assert_eq!(notes("A", 2), "A B C D E F G#");
        assert_eq!(notes("E", 7), "E F# G A B C# D");
    }

    #[test]
    fn a_pentatonic_and_the_blues_take_the_roots_accidentals() {
        assert_eq!(notes("A", 5), "A C D E G");
        assert_eq!(notes("C", 4), "C D E G A");
        assert_eq!(notes("Eb", 5), "Eb Gb Ab Bb Db");
        assert_eq!(notes("E", 6), "E G A A# B D");
    }

    #[test]
    fn a_scale_stacks_a_chord_on_every_step() {
        let names = |root: &str, scale: usize, seventh: bool| scale_chords(root, &SCALES[scale], seventh).unwrap().iter().map(|chord| chord.name.clone()).collect::<Vec<_>>().join(" ");
        assert_eq!(names("C", 0, false), "C Dm Em F G Am Bdim");
        assert_eq!(names("C", 0, true), "Cmaj7 Dm7 Em7 Fmaj7 G7 Am7 Bm7b5");
        assert_eq!(names("A", 1, false), "Am Bdim C Dm Em F G");
        assert_eq!(names("D", 0, false), "D Em F#m G A Bm C#dim");
        let degrees: Vec<String> = scale_chords("C", &SCALES[0], false).unwrap().into_iter().map(|chord| chord.degree).collect();
        assert_eq!(degrees.join(" "), "I ii iii IV V vi vii°");
        assert_eq!(scale_chords("C", &SCALES[4], false), None);
    }

    #[test]
    fn a_root_that_is_no_note_has_no_scale() {
        assert_eq!(note_names("X", &SCALES[0]), None);
    }

    #[test]
    fn the_neck_shows_where_the_notes_are() {
        let neck = on_fretboard("C", &SCALES[0], &Tuning::guitar(), 12).unwrap();
        assert_eq!(neck.len(), 6);
        assert_eq!(neck[0].len(), 13);
        // Low E string: E (open) F G A B C at the 8th fret is the root.
        assert_eq!(neck[0][0], (true, false));
        assert_eq!(neck[0][1], (true, false));
        assert_eq!(neck[0][2], (false, false));
        assert_eq!(neck[0][8], (true, true));
        // The A string's third fret is the root too.
        assert_eq!(neck[1][3], (true, true));
    }
}

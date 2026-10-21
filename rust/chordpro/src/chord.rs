// This file is part of Campfire.
// Copyright (c) Pandula Péter 2017-2026.
// https://github.com/pandulapeter/campfire
//
// This Source Code Form is subject to the terms of the Mozilla Public License, v. 2.0.
// If a copy of the MPL was not distributed with this file, You can obtain one at
// https://mozilla.org/MPL/2.0/.

//! What a chord name means: the notes it is made of.
//!
//! A port of the Kotlin `ChordProChords` and `Chord`. A name is read by the same walk that recognizes it
//! ([`chord_names::read`]), so a name is read exactly where it is recognized.

use std::collections::BTreeSet;

use crate::chord_names::{self, ChordNameReader};
use crate::notation::{self, ChordNotation};

pub(crate) const SHARP_NAMES: [&str; 12] = ["C", "C#", "D", "D#", "E", "F", "F#", "G", "G#", "A", "A#", "B"];
pub(crate) const FLAT_NAMES: [&str; 12] = ["C", "Db", "D", "Eb", "E", "F", "Gb", "G", "Ab", "A", "Bb", "B"];
const LETTERS: [char; 7] = ['C', 'D', 'E', 'F', 'G', 'A', 'B'];
const WHITE_KEY_ENHARMONICS: [&str; 4] = ["Cb", "Fb", "E#", "B#"];

const MAJOR_SECOND: i32 = 2;
const MINOR_THIRD: i32 = 3;
const MAJOR_THIRD: i32 = 4;
const PERFECT_FOURTH: i32 = 5;
const DIMINISHED_FIFTH: i32 = 6;
const PERFECT_FIFTH: i32 = 7;
const AUGMENTED_FIFTH: i32 = 8;
const MINOR_SIXTH: i32 = 8;
const MAJOR_SIXTH: i32 = 9;
const DIMINISHED_SEVENTH: i32 = 9;
const MINOR_SEVENTH: i32 = 10;
const MAJOR_SEVENTH: i32 = 11;

/// The notes a chord name stands for, as pitch classes: what is played, whatever the name's spelling. `C#m7`, `Dbm7`
/// and `C#min7` are one [`Chord`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Chord {
    /// The root's pitch class, 0 for C up to 11 for B.
    pub root: i32,
    /// The notes in semitones above the root, folded into one octave, sorted and each once; the root's `0` is among
    /// them unless the name leaves it out (`no1`).
    pub intervals: Vec<i32>,
    /// The pitch class of the note after the slash of a slash chord, or `None` where there is none or it is the root.
    pub bass: Option<i32>,
}

impl Chord {
    /// The chord as one string, the same for every spelling of it: the root as a sharp, the intervals and the bass
    /// (`C#:0.3.7.10`, `D:0.4.7/F#`). What a player's choice of shape is stored under, so it has to stay stable.
    pub fn id(&self) -> String {
        let intervals: Vec<String> = self.intervals.iter().map(i32::to_string).collect();
        let bass = self.bass.map(|bass| format!("/{}", SHARP_NAMES[bass as usize])).unwrap_or_default();
        format!("{}:{}{}", SHARP_NAMES[self.root as usize], intervals.join("."), bass)
    }

    /// Every pitch class the chord sounds, the bass included.
    pub fn pitch_classes(&self) -> BTreeSet<i32> {
        self.intervals.iter().map(|interval| (self.root + interval) % 12).chain(self.bass).collect()
    }
}

/// The chord `name` stands for, as shown in `notation`, or `None` where it is no chord name: a word in brackets,
/// `N.C.`, an annotation. A parenthesized chord, a lowercase minor (`a`, `f#7`), a Latin name and the `♯` and `♭` signs
/// are read as well, since this is handed what a page draws.
pub fn parse(name: &str, notation: ChordNotation) -> Option<Chord> {
    let mut reader = Reader::new();
    let read = notation::read(name, notation == ChordNotation::German);
    chord_names::read(&read, &mut reader).then(|| reader.chord())
}

/// The notes of `chord` as names in `notation`, spelled with flats or sharps as `prefer_flats` says: the bass of a
/// slash chord first where it is not one of the chord's own notes, then the chord's from its root up.
pub fn note_names(chord: &Chord, notation: ChordNotation, prefer_flats: bool) -> Vec<String> {
    let pitch_classes: Vec<i32> = chord.intervals.iter().map(|interval| (chord.root + interval) % 12).collect();
    let bass = chord.bass.filter(|bass| !pitch_classes.contains(bass));
    let names = if prefer_flats { &FLAT_NAMES } else { &SHARP_NAMES };
    bass.iter()
        .chain(pitch_classes.iter())
        .map(|&pitch_class| notation::shown_name(names[pitch_class as usize], notation))
        .collect()
}

/// The notes of the chord `name` stands for, as [`note_names`] lists them, each spelled from the letter of the root as
/// `name` writes it by the degree it is: `Cm` is `C Eb G` and `F7` is `F A C Eb`, as a chart spells them. A note that
/// would need a double sharp or flat, or would be one of `Cb`, `Fb`, `E#`, `B#`, is named as [`note_names`] would with
/// the root's accidental, and the bass of a slash chord as the name writes it. `None` where `name` is no chord.
pub fn spelled_note_names(name: &str, notation: ChordNotation) -> Option<Vec<String>> {
    let mut reader = Reader::new();
    // The standard notation always, since in German the standard `B` this is handed would be read as B flat.
    if !chord_names::read(&notation::read(name, false), &mut reader) {
        return None;
    }
    let chord = reader.chord();
    let root_name = reader.root_name.as_deref().map(standard_spelling);
    let root_letter = root_name.as_deref().and_then(|root| LETTERS.iter().position(|&letter| root.starts_with(letter)));
    let (Some(root_name), Some(root_letter)) = (root_name, root_letter) else {
        return Some(note_names(&chord, notation, false));
    };
    let fallback = if root_name.chars().nth(1) == Some('b') { &FLAT_NAMES } else { &SHARP_NAMES };
    let pitch_classes: Vec<i32> = chord.intervals.iter().map(|interval| (chord.root + interval) % 12).collect();
    let bass = chord
        .bass
        .filter(|bass| !pitch_classes.contains(bass))
        .map(|bass| reader.bass_name.as_deref().map(standard_spelling).unwrap_or_else(|| fallback[bass as usize].to_string()));
    let notes = chord.intervals.iter().map(|&interval| {
        let letter = LETTERS[(root_letter + letter_step(interval, &chord.intervals)) % LETTERS.len()];
        let pitch_class = (chord.root + interval) % 12;
        let spelled = match (pitch_class - letter_index(letter)).rem_euclid(12) {
            0 => Some(letter.to_string()),
            1 => Some(format!("{letter}#")),
            11 => Some(format!("{letter}b")),
            _ => None,
        };
        spelled
            .filter(|spelled| !WHITE_KEY_ENHARMONICS.contains(&spelled.as_str()))
            .unwrap_or_else(|| fallback[pitch_class as usize].to_string())
    });
    Some(bass.into_iter().chain(notes).map(|note| notation::shown_name(&note, notation)).collect())
}

/// How many letters above the root the note `interval` semitones above it is written: a chart's degrees, with the
/// neighbors of a note the chord also holds telling a sharp ninth from a minor third, a sharp eleventh from a flat fifth
/// and a flat thirteenth from a sharp fifth, and a diminished seventh's `bb7` written as the sixth, as charts do.
fn letter_step(interval: i32, intervals: &[i32]) -> usize {
    match interval {
        0 => 0,
        1 | 2 => 1,
        3 => {
            if intervals.contains(&4) {
                1
            } else {
                2
            }
        }
        4 => 2,
        5 => 3,
        6 => {
            if intervals.contains(&7) {
                3
            } else {
                4
            }
        }
        7 => 4,
        8 => {
            if intervals.contains(&7) {
                5
            } else {
                4
            }
        }
        9 => 5,
        _ => 6,
    }
}

fn letter_index(letter: char) -> i32 {
    match letter {
        'C' => 0,
        'D' => 2,
        'E' => 4,
        'F' => 5,
        'G' => 7,
        'A' => 9,
        _ => 11,
    }
}

/// A note as written in the standard notation, with a capital letter and `#` or `b` for its accidental.
fn standard_spelling(note: &str) -> String {
    let mut chars = note.chars();
    let letter: String = chars.next().into_iter().flat_map(char::to_uppercase).collect();
    let accidental = match chars.next() {
        Some('#' | '♯') => "#",
        Some('b' | '♭') => "b",
        _ => "",
    };
    format!("{letter}{accidental}")
}

/// The pitch class of a note as written: a capital or lowercase letter, `H` as `B`, and an accidental or none.
pub fn pitch_class(note: &str) -> Option<i32> {
    let mut chars = note.chars();
    let letter: i32 = match chars.next()?.to_ascii_uppercase() {
        'C' => 0,
        'D' => 2,
        'E' => 4,
        'F' => 5,
        'G' => 7,
        'A' => 9,
        'B' | 'H' => 11,
        _ => return None,
    };
    let accidental: i32 = match chars.next() {
        Some('#' | '♯') => 1,
        Some('b' | '♭') => -1,
        _ => 0,
    };
    Some((letter + accidental).rem_euclid(12))
}

/// The interval a scale degree names above the root, folded into the octave, or `None` for no degree a chord has.
fn degree_interval(degree: &str) -> Option<i32> {
    match degree.parse::<u32>().ok()? {
        1 | 8 => Some(0),
        2 | 9 => Some(2),
        3 | 10 => Some(4),
        4 | 11 => Some(5),
        5 | 12 => Some(7),
        6 | 13 => Some(9),
        7 => Some(10),
        _ => None,
    }
}

/// Collects the notes while [`chord_names::read`] walks a name already in the standard notation.
struct Reader {
    root: i32,
    bass: Option<i32>,
    third: Option<i32>,
    suspension: Option<i32>,
    fifth: Option<i32>,
    seventh: Option<i32>,
    is_major_seventh: bool,
    is_diminished: bool,
    has_root: bool,
    added: BTreeSet<i32>,
    quality: Option<String>,
    /// The root as the name writes it, which [`spelled_note_names`] spells the other notes from.
    root_name: Option<String>,
    /// The bass of a slash chord as the name writes it.
    bass_name: Option<String>,
}

impl Reader {
    fn new() -> Self {
        Reader {
            root: 0,
            bass: None,
            third: Some(MAJOR_THIRD),
            suspension: None,
            fifth: Some(PERFECT_FIFTH),
            seventh: None,
            is_major_seventh: false,
            is_diminished: false,
            has_root: true,
            added: BTreeSet::new(),
            quality: None,
            root_name: None,
            bass_name: None,
        }
    }

    fn chord(&self) -> Chord {
        let mut intervals = BTreeSet::new();
        if self.has_root {
            intervals.insert(0);
        }
        intervals.extend(self.suspension.or(self.third));
        intervals.extend(self.fifth);
        intervals.extend(self.seventh);
        intervals.extend(self.added.iter().copied());
        Chord { root: self.root, intervals: intervals.into_iter().collect(), bass: self.bass.filter(|&bass| bass != self.root) }
    }

    /// A number that stacks up to itself: `9` and `11` bring the seventh and the ninth under them, `13` the seventh.
    fn extend(&mut self, number: &str) {
        match number {
            "6" => {
                self.added.insert(MAJOR_SIXTH);
            }
            "69" => self.added.extend([MAJOR_SIXTH, MAJOR_SECOND]),
            "7" => self.seventh = Some(self.seventh_of_quality()),
            "9" => {
                self.seventh = Some(self.seventh_of_quality());
                self.added.insert(MAJOR_SECOND);
            }
            "11" => {
                self.seventh = Some(self.seventh_of_quality());
                self.added.extend([MAJOR_SECOND, PERFECT_FOURTH]);
                if self.third == Some(MAJOR_THIRD) {
                    self.third = None;
                }
            }
            "13" => {
                self.seventh = Some(self.seventh_of_quality());
                self.added.insert(MAJOR_SIXTH);
            }
            _ => self.add(number),
        }
    }

    fn add(&mut self, degree: &str) {
        if let Some(interval) = degree_interval(degree) {
            self.added.insert(interval);
        }
    }

    fn seventh_of_quality(&self) -> i32 {
        if self.is_major_seventh {
            MAJOR_SEVENTH
        } else if self.is_diminished {
            DIMINISHED_SEVENTH
        } else {
            self.seventh.unwrap_or(MINOR_SEVENTH)
        }
    }
}

impl ChordNameReader for Reader {
    fn root(&mut self, note: &str) {
        self.root_name = Some(note.to_string());
        self.root = pitch_class(note).unwrap_or(0);
    }

    fn quality(&mut self, quality: &str) {
        self.quality = Some(quality.to_string());
        match quality {
            "m" | "min" | "mi" | "-" => self.third = Some(MINOR_THIRD),
            "maj" | "Maj" | "M" => self.is_major_seventh = true,
            "Δ" | "∆" => {
                self.is_major_seventh = true;
                self.seventh = Some(MAJOR_SEVENTH);
            }
            "dim" | "°" => {
                self.third = Some(MINOR_THIRD);
                self.fifth = Some(DIMINISHED_FIFTH);
                self.is_diminished = true;
            }
            "ø" => {
                self.third = Some(MINOR_THIRD);
                self.fifth = Some(DIMINISHED_FIFTH);
                self.seventh = Some(MINOR_SEVENTH);
            }
            "aug" | "+" => self.fifth = Some(AUGMENTED_FIFTH),
            "sus" => self.suspension = Some(PERFECT_FOURTH),
            _ => {}
        }
    }

    fn number(&mut self, number: &str) {
        match self.quality.clone().as_deref() {
            Some("sus") => {
                if number == "2" {
                    self.suspension = Some(MAJOR_SECOND);
                } else if number != "4" {
                    self.extend(number);
                }
            }
            Some("add") => self.add(number),
            Some("ø") => {
                if number != "7" {
                    self.extend(number);
                }
            }
            quality => match number {
                "2" => {
                    self.added.insert(MAJOR_SECOND);
                }
                "4" => self.suspension = Some(PERFECT_FOURTH),
                "5" => match quality {
                    None => self.third = None,
                    // `-` is the flat sign wherever a `5` follows it (`C7-5`), so `C-5` is the major triad with a flat
                    // fifth rather than a minor one with its fifth named twice.
                    Some("-") => {
                        self.third = Some(MAJOR_THIRD);
                        self.fifth = Some(DIMINISHED_FIFTH);
                    }
                    _ => {}
                },
                _ => self.extend(number),
            },
        }
    }

    fn altered(&mut self) {
        if self.seventh.is_none() {
            self.seventh = Some(MINOR_SEVENTH);
        }
        self.fifth = None;
        self.added.extend([MINOR_THIRD, MINOR_SIXTH]);
    }

    fn added(&mut self, degree: &str) {
        match degree {
            "7" => {
                if self.seventh.is_none() {
                    self.seventh = Some(self.seventh_of_quality());
                }
            }
            _ => self.add(degree),
        }
    }

    fn alteration(&mut self, word: &str, degree: &str) {
        match word {
            "maj" | "Maj" | "M" => {
                self.is_major_seventh = true;
                if degree.is_empty() {
                    self.seventh = Some(MAJOR_SEVENTH);
                } else {
                    self.extend(degree);
                }
            }
            "min" | "mi" => self.third = Some(MINOR_THIRD),
            "dim" => {
                self.fifth = Some(DIMINISHED_FIFTH);
                if degree == "7" {
                    self.seventh = Some(DIMINISHED_SEVENTH);
                }
            }
            "aug" => self.fifth = Some(AUGMENTED_FIFTH),
            "sus" => self.suspension = Some(if degree == "2" { MAJOR_SECOND } else { PERFECT_FOURTH }),
            "add" => self.add(degree),
            "#" | "♯" | "+" => match degree {
                "" | "5" => self.fifth = Some(AUGMENTED_FIFTH),
                _ => {
                    if let Some(interval) = degree_interval(degree) {
                        self.added.insert((interval + 1) % 12);
                    }
                }
            },
            "b" | "♭" | "-" => match degree {
                "5" => self.fifth = Some(DIMINISHED_FIFTH),
                "" => {}
                _ => {
                    if let Some(interval) = degree_interval(degree) {
                        self.added.insert((interval + 11) % 12);
                    }
                }
            },
            _ => {}
        }
    }

    fn omitted(&mut self, degree: &str) {
        match degree {
            "1" | "8" => self.has_root = false,
            "3" => {
                self.third = None;
                self.suspension = None;
            }
            "5" => self.fifth = None,
            _ => {
                if let Some(interval) = degree_interval(degree) {
                    self.added.remove(&interval);
                }
            }
        }
    }

    fn bass(&mut self, note: &str) {
        self.bass_name = Some(note.to_string());
        self.bass = pitch_class(note);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn chord(name: &str) -> Chord {
        parse(name, ChordNotation::Standard).unwrap_or_else(|| panic!("{name} is a chord"))
    }

    #[test]
    fn the_id_is_the_same_for_every_spelling() {
        assert_eq!(chord("C#m7").id(), "C#:0.3.7.10");
        assert_eq!(chord("Dbm7").id(), "C#:0.3.7.10");
        assert_eq!(chord("C#min7").id(), "C#:0.3.7.10");
        assert_eq!(chord("D/F#").id(), "D:0.4.7/F#");
    }

    #[test]
    fn a_word_is_no_chord() {
        for name in ["N.C.", "Bridge", "[Intro]", ""] {
            assert_eq!(parse(name, ChordNotation::Standard), None, "{name}");
        }
    }
}

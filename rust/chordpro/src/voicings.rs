// This file is part of Campfire.
// Copyright (c) Pandula Péter 2017-2026.
// https://github.com/pandulapeter/campfire
//
// This Source Code Form is subject to the terms of the Mozilla Public License, v. 2.0.
// If a copy of the MPL was not distributed with this file, You can obtain one at
// https://mozilla.org/MPL/2.0/.

//! How a chord is held on a fretted instrument: the shapes that sound exactly the chord, found by search.
//!
//! The search looks at every window of four frets of the neck (and the open strings), keeps the shapes whose strings all sound a
//! note of the chord and that hold every note the chord cannot do without, and puts the most comfortable first: open shapes and
//! low ones, few fingers, no muted string between two sounding ones, the root in the bass.

use crate::chord::Chord;

/// An instrument's strings, from the lowest sounding to the highest, as MIDI note numbers.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Tuning {
    pub name: &'static str,
    pub strings: Vec<i32>,
}

impl Tuning {
    pub fn guitar() -> Tuning {
        Tuning { name: "Guitarra", strings: vec![40, 45, 50, 55, 59, 64] }
    }

    pub fn ukulele() -> Tuning {
        Tuning { name: "Ukelele", strings: vec![67, 60, 64, 69] }
    }
}

/// How one string is played: muted, or at a fret (0 is open).
pub type Fret = Option<u8>;

/// A shape of a chord on a tuning.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Voicing {
    /// One entry per string, from the lowest.
    pub frets: Vec<Fret>,
}

impl Voicing {
    /// The lowest fret that is pressed, `None` for a shape of open strings alone.
    pub fn lowest_pressed(&self) -> Option<u8> {
        self.frets.iter().flatten().copied().filter(|&fret| fret > 0).min()
    }

    pub fn highest_pressed(&self) -> Option<u8> {
        self.frets.iter().flatten().copied().filter(|&fret| fret > 0).max()
    }

    /// The fret the diagram starts at: 1 for a shape that reaches the nut, else the lowest pressed fret.
    pub fn base_fret(&self) -> u8 {
        match self.highest_pressed() {
            Some(highest) if highest > FRETS_SHOWN => self.lowest_pressed().unwrap_or(1),
            _ => 1,
        }
    }

    /// `x32010` for the shape of a C major on a guitar: the frets from the lowest string, `x` for a muted one.
    pub fn text(&self) -> String {
        self.frets.iter().map(|fret| fret.map_or("x".to_string(), |fret| fret.to_string())).collect::<Vec<_>>().join(if self.frets.iter().flatten().any(|&fret| fret > 9) { "." } else { "" })
    }

    /// How many fingers the shape takes: every pressed fret is a finger, except that a barre across the lowest fret counts once.
    pub fn fingers(&self) -> usize {
        let Some(lowest) = self.lowest_pressed() else {
            return 0;
        };
        let at_lowest: Vec<usize> = self.frets.iter().enumerate().filter(|(_, fret)| **fret == Some(lowest)).map(|(index, _)| index).collect();
        let others = self.frets.iter().flatten().filter(|&&fret| fret > lowest).count();
        // A barre is the lowest fret held on two strings or more with nothing open between them.
        let is_barre = at_lowest.len() >= 2 && {
            let (first, last) = (at_lowest[0], at_lowest[at_lowest.len() - 1]);
            (first..=last).all(|string| self.frets[string].is_some_and(|fret| fret >= lowest))
        };
        others + if is_barre { 1 } else { at_lowest.len() }
    }
}

/// How many frets a diagram shows.
pub const FRETS_SHOWN: u8 = 5;
const MAX_FRET: u8 = 12;
const SPAN: u8 = 3;
const MAX_FINGERS: usize = 4;

/// The pitch class a string sounds at a fret.
fn pitch_class(string: i32, fret: u8) -> i32 {
    (string + fret as i32).rem_euclid(12)
}

/// The shapes of `chord` on `tuning`, the most comfortable first, at most `limit` of them.
pub fn voicings(chord: &Chord, tuning: &Tuning, limit: usize) -> Vec<Voicing> {
    let chord_classes: Vec<i32> = chord.intervals.iter().map(|interval| (chord.root + interval).rem_euclid(12)).collect();
    // The notes the chord cannot do without: all of them, except the fifth of a chord with five notes or more and the root where
    // a bass says another note is lowest.
    let mut required: Vec<i32> = chord_classes.clone();
    if chord.intervals.len() >= 5 || (chord.intervals.len() >= 4 && tuning.strings.len() <= 4) {
        required.retain(|class| *class != (chord.root + 7).rem_euclid(12));
    }
    if chord.intervals.len() >= 6 {
        required.retain(|class| *class != chord.root || chord.bass.is_some());
    }
    let allowed: Vec<i32> = chord_classes.iter().copied().chain(chord.bass).collect();
    let bass = chord.bass.unwrap_or(chord.root);
    // The root is wanted in the bass on an instrument that has one; a ukulele has no bass to speak of unless the chord names one.
    let is_bass_asked = chord.bass.is_some() || tuning.strings.len() >= 5;
    let mut found: Vec<(i64, Voicing)> = Vec::new();
    let strings = tuning.strings.len();
    for window in 0..=(MAX_FRET - SPAN) {
        let mut frets: Vec<Fret> = vec![None; strings];
        search(0, window, tuning, &allowed, &mut frets, &mut |frets| {
            let voicing = Voicing { frets: frets.to_vec() };
            if let Some(score) = score(&voicing, tuning, &required, bass, is_bass_asked, window) {
                found.push((score, voicing));
            }
        });
    }
    found.sort_by(|a, b| a.0.cmp(&b.0).then_with(|| a.1.frets.cmp(&b.1.frets)));
    let mut unique: Vec<Voicing> = Vec::new();
    for (_, voicing) in found {
        if !unique.contains(&voicing) {
            unique.push(voicing);
        }
        if unique.len() == limit {
            break;
        }
    }
    unique
}

/// Every way of playing the strings from `string` up with the frets of one window: muted, open, or one of its four frets.
fn search(string: usize, window: u8, tuning: &Tuning, allowed: &[i32], frets: &mut Vec<Fret>, found: &mut dyn FnMut(&[Fret])) {
    if string == tuning.strings.len() {
        found(frets);
        return;
    }
    let options = std::iter::once(None).chain(std::iter::once(Some(0))).chain((window.max(1)..=window.max(1) + SPAN).map(Some));
    for option in options {
        if let Some(fret) = option {
            if !allowed.contains(&pitch_class(tuning.strings[string], fret)) {
                continue;
            }
        }
        frets[string] = option;
        search(string + 1, window, tuning, allowed, frets, found);
    }
    frets[string] = None;
}

/// How uncomfortable a shape is, lower being better; `None` for one that is no shape of the chord.
fn score(voicing: &Voicing, tuning: &Tuning, required: &[i32], bass: i32, is_bass_asked: bool, window: u8) -> Option<i64> {
    let sounding: Vec<(usize, u8)> = voicing.frets.iter().enumerate().filter_map(|(string, fret)| fret.map(|fret| (string, fret))).collect();
    let classes: Vec<i32> = sounding.iter().map(|&(string, fret)| pitch_class(tuning.strings[string], fret)).collect();
    // Enough strings to make a chord of it, and every note it cannot do without.
    if sounding.len() < 3.min(tuning.strings.len()) || !required.iter().all(|class| classes.contains(class)) {
        return None;
    }
    // A shape that sits in one window only: the window it is found in is the one its lowest pressed fret opens, so the same shape
    // is not found twice from windows that both hold it.
    if let Some(lowest) = voicing.lowest_pressed() {
        if lowest != window.max(1) && !(window == 0 && lowest == 1) {
            return None;
        }
    } else if window != 0 {
        return None;
    }
    if voicing.fingers() > MAX_FINGERS {
        return None;
    }
    // No muted string between two that sound, which a hand cannot do on most of the neck.
    let first = sounding.first()?.0;
    let last = sounding.last()?.0;
    let holes = (first..=last).filter(|&string| voicing.frets[string].is_none()).count();
    if holes > 0 {
        return None;
    }
    // The lowest note is the lowest in pitch, which is not the lowest string on an instrument that is tuned re-entrant (a ukulele).
    let lowest_class = sounding.iter().map(|&(string, fret)| tuning.strings[string] + fret as i32).min().map(|note| note.rem_euclid(12))?;
    let mut score: i64 = 0;
    score += if lowest_class == bass || !is_bass_asked { 0 } else { 40 };
    score += 3 * (tuning.strings.len() - sounding.len()) as i64;
    score += 4 * voicing.fingers() as i64;
    score += 6 * voicing.lowest_pressed().map_or(0, |fret| fret as i64 - 1);
    score -= 3 * voicing.frets.iter().filter(|fret| **fret == Some(0)).count() as i64;
    // Every note of the chord is there, which the open shapes with a doubled root and a missing fifth are not.
    score += 2 * required.iter().filter(|class| !classes.contains(class)).count() as i64;
    Some(score)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::chord::parse;
    use crate::notation::ChordNotation;

    fn shapes(name: &str, tuning: &Tuning, limit: usize) -> Vec<String> {
        let chord = parse(name, ChordNotation::Standard).unwrap_or_else(|| panic!("{name} is a chord"));
        voicings(&chord, tuning, limit).iter().map(Voicing::text).collect()
    }

    #[test]
    fn the_open_chords_of_a_guitar_are_found_first() {
        let guitar = Tuning::guitar();
        for (name, expected) in [("C", "x32010"), ("Am", "x02210"), ("E", "022100"), ("Em", "022000"), ("A", "x02220"), ("D", "xx0232"), ("Dm", "xx0231")] {
            let found = shapes(name, &guitar, 3);
            assert!(found.contains(&expected.to_string()), "{name}: {found:?}");
        }
        assert_eq!(shapes("G", &guitar, 1).len(), 1);
    }

    #[test]
    fn every_shape_sounds_only_notes_of_the_chord() {
        let guitar = Tuning::guitar();
        for name in ["C", "Am7", "F#m", "Bb", "G7", "Dsus4", "Cmaj7", "E7", "Fm", "Bdim", "Caug", "C/E", "D/F#", "Am/G", "C9", "Gadd9"] {
            let chord = parse(name, ChordNotation::Standard).unwrap();
            let classes = chord.pitch_classes();
            let found = voicings(&chord, &guitar, 6);
            assert!(!found.is_empty(), "{name}");
            for voicing in found {
                assert!(voicing.frets.len() == 6);
                for (string, fret) in guitar.strings.iter().zip(&voicing.frets) {
                    if let Some(fret) = fret {
                        assert!(classes.contains(&pitch_class(*string, *fret)), "{name}: {}", voicing.text());
                    }
                }
            }
        }
    }

    #[test]
    fn a_slash_chord_has_its_bass_lowest() {
        let guitar = Tuning::guitar();
        let chord = parse("C/E", ChordNotation::Standard).unwrap();
        let first = &voicings(&chord, &guitar, 1)[0];
        let lowest = first.frets.iter().enumerate().find_map(|(string, fret)| fret.map(|fret| pitch_class(guitar.strings[string], fret))).unwrap();
        assert_eq!(lowest, 4);
    }

    #[test]
    fn the_ukulele_is_held_with_four_strings() {
        let ukulele = Tuning::ukulele();
        let found = shapes("C", &ukulele, 3);
        assert!(found.contains(&"0003".to_string()), "{found:?}");
        let found = shapes("Am", &ukulele, 3);
        assert!(found.contains(&"2000".to_string()), "{found:?}");
        let found = shapes("F", &ukulele, 3);
        assert!(found.contains(&"2010".to_string()), "{found:?}");
    }

    #[test]
    fn a_barre_chord_is_found_where_there_is_no_open_one() {
        let found = shapes("F", &Tuning::guitar(), 4);
        assert!(found.contains(&"133211".to_string()), "{found:?}");
        let found = shapes("Bb", &Tuning::guitar(), 4);
        assert!(found.contains(&"x13331".to_string()), "{found:?}");
    }

    #[test]
    fn a_shape_is_described_by_its_frets() {
        let c = Voicing { frets: vec![None, Some(3), Some(2), Some(0), Some(1), Some(0)] };
        assert_eq!(c.text(), "x32010");
        assert_eq!(c.base_fret(), 1);
        assert_eq!(c.fingers(), 3);
        let f = Voicing { frets: vec![Some(1), Some(3), Some(3), Some(2), Some(1), Some(1)] };
        assert_eq!(f.fingers(), 4);
        let high = Voicing { frets: vec![Some(8), Some(10), Some(10), Some(9), Some(8), Some(8)] };
        assert_eq!(high.base_fret(), 8);
        assert_eq!(high.text(), "8.10.10.9.8.8");
    }

    #[test]
    fn no_shape_holds_more_than_four_fingers_or_a_span_over_four_frets() {
        let guitar = Tuning::guitar();
        for name in ["C", "G7", "Fmaj7", "Bm7b5", "C13", "F#m7"] {
            let chord = parse(name, ChordNotation::Standard).unwrap();
            for voicing in voicings(&chord, &guitar, 10) {
                assert!(voicing.fingers() <= MAX_FINGERS, "{name} {}", voicing.text());
                if let (Some(low), Some(high)) = (voicing.lowest_pressed(), voicing.highest_pressed()) {
                    assert!(high - low <= SPAN, "{name} {}", voicing.text());
                }
            }
        }
    }
}

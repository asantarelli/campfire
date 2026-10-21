// This file is part of Campfire.
// Copyright (c) Pandula Péter 2017-2026.
// https://github.com/pandulapeter/campfire
//
// This Source Code Form is subject to the terms of the Mozilla Public License, v. 2.0.
// If a copy of the MPL was not distributed with this file, You can obtain one at
// https://mozilla.org/MPL/2.0/.

//! The tables of the Kotlin `ChordProChordsTest`, generated from it: they are the specification of this port.

use chordpro::chord::{note_names, parse, spelled_note_names, Chord};
use chordpro::chord_names::{is_chord_name, lowercase_minor_expanded};
use chordpro::notation::ChordNotation::{German, Latin, Standard};

#[test]
fn every_usual_name_is_read_as_the_notes_chord_charts_agree_on() {
    let table: &[(&str, &str)] = &[
        ("C", "C E G"),
        ("Cm", "C D# G"),
        ("Cmin", "C D# G"),
        ("Cmi", "C D# G"),
        ("C-", "C D# G"),
        ("C5", "C G"),
        ("C+5", "C E G#"),
        ("Caug5", "C E G#"),
        ("Cdim5", "C D# F#"),
        ("C°5", "C D# F#"),
        ("Cm5", "C D# G"),
        ("C-5", "C E F#"),
        ("C2", "C D E G"),
        ("C4", "C F G"),
        ("Csus", "C F G"),
        ("Csus4", "C F G"),
        ("Csus2", "C D G"),
        ("C7sus4", "C F G A#"),
        ("C7sus2", "C D G A#"),
        ("C9sus4", "C D F G A#"),
        ("Cadd9", "C D E G"),
        ("Cadd2", "C D E G"),
        ("Cadd4", "C E F G"),
        ("Cadd11", "C E F G"),
        ("C(add9)", "C D E G"),
        ("Cm(add9)", "C D D# G"),
        ("C6", "C E G A"),
        ("Cm6", "C D# G A"),
        ("C69", "C D E G A"),
        ("C6/9", "C D E G A"),
        ("Cm6/9", "C D D# G A"),
        ("C7", "C E G A#"),
        ("Cmaj7", "C E G B"),
        ("CMaj7", "C E G B"),
        ("CM7", "C E G B"),
        ("CΔ", "C E G B"),
        ("CΔ7", "C E G B"),
        ("C∆7", "C E G B"),
        ("Cmaj", "C E G"),
        ("CM", "C E G"),
        ("Cm7", "C D# G A#"),
        ("Cmi7", "C D# G A#"),
        ("C-7", "C D# G A#"),
        ("CmM7", "C D# G B"),
        ("Cmmaj7", "C D# G B"),
        ("Cm(maj7)", "C D# G B"),
        ("C9", "C D E G A#"),
        ("Cmaj9", "C D E G B"),
        ("Cm9", "C D D# G A#"),
        ("Cadd9", "C D E G"),
        ("C11", "C D F G A#"),
        ("Cm11", "C D D# F G A#"),
        ("C13", "C E G A A#"),
        ("Cm13", "C D# G A A#"),
        ("Cmaj13", "C E G A B"),
        ("Cdim", "C D# F#"),
        ("C°", "C D# F#"),
        ("Cdim7", "C D# F# A"),
        ("C°7", "C D# F# A"),
        ("Cø", "C D# F# A#"),
        ("Cø7", "C D# F# A#"),
        ("Cm7b5", "C D# F# A#"),
        ("Cm7-5", "C D# F# A#"),
        ("Caug", "C E G#"),
        ("C+", "C E G#"),
        ("C+7", "C E G# A#"),
        ("C7+", "C E G# A#"),
        ("C7#5", "C E G# A#"),
        ("C7+5", "C E G# A#"),
        ("Caugmaj7", "C E G# B"),
        ("C7b5", "C E F# A#"),
        ("C7b9", "C C# E G A#"),
        ("C7(b9)", "C C# E G A#"),
        ("C7-9", "C C# E G A#"),
        ("C7#9", "C D# E G A#"),
        ("C7(#9, b13)", "C D# E G G# A#"),
        ("Cmaj7#11", "C E F# G B"),
        ("C7b13", "C E G G# A#"),
        ("C7alt", "C D# E G# A#"),
        ("C7(9)", "C D E G A#"),
        ("C(no3)", "C G"),
        ("C7(omit5)", "C E A#"),
    ];
    for (name, notes) in table {
        let chord = parse(name, Standard).unwrap_or_else(|| panic!("{name} is a chord"));
        assert_eq!(&note_names(&chord, Standard, false).join(" "), notes, "{name}");
    }
}

#[test]
fn every_root_is_read_with_its_accidental() {
    let roots: &[(&str, i32)] = &[
        ("C", 0),
        ("C#", 1),
        ("Db", 1),
        ("D", 2),
        ("D#", 3),
        ("Eb", 3),
        ("E", 4),
        ("Fb", 4),
        ("E#", 5),
        ("F", 5),
        ("F#", 6),
        ("Gb", 6),
        ("G", 7),
        ("G#", 8),
        ("Ab", 8),
        ("A", 9),
        ("A#", 10),
        ("Bb", 10),
        ("B", 11),
        ("Cb", 11),
        ("H", 11),
        ("C♯", 1),
        ("B♭", 10),
    ];
    for (name, root) in roots {
        assert_eq!(parse(name, Standard).map(|chord| chord.root), Some(*root), "{name}");
    }
}

#[test]
fn the_bass_of_a_slash_chord_is_kept_apart_from_the_chord() {
    assert_eq!(parse("D/F#", Standard), Some(Chord { root: 2, intervals: vec![0, 4, 7], bass: Some(6) }));
    assert_eq!(parse("D/f#", Standard), Some(Chord { root: 2, intervals: vec![0, 4, 7], bass: Some(6) }));
    assert_eq!(parse("Am7/G", Standard), Some(Chord { root: 9, intervals: vec![0, 3, 7, 10], bass: Some(7) }));
    assert_eq!(parse("C/C", Standard), Some(Chord { root: 0, intervals: vec![0, 4, 7], bass: None }));
    assert_eq!(note_names(&parse("D/F#", Standard).unwrap(), Standard, false), ["D", "F#", "A"]);
    assert_eq!(note_names(&parse("C/Bb", Standard).unwrap(), Standard, false), ["A#", "C", "E", "G"]);
}

#[test]
fn spellings_of_one_chord_share_an_id() {
    let groups: &[&[&str]] = &[
        &["C#m7", "Dbm7", "C#min7", "C#mi7", "C#-7", "(C#m7)", "C♯m7", "D♭m7", "c#7"],
        &["Bb", "A#", "B♭"],
        &["D/F#", "D/Gb", "D/f#"],
        &["Cmaj7", "CM7", "CΔ", "CΔ7"],
        &["Cm7b5", "Cø", "Cø7", "Cm7-5"],
        &["Caug", "C+", "C+5", "Caug5"],
        &["Cdim", "C°", "Cdim5"],
    ];
    for names in groups {
        let mut ids: Vec<String> = names.iter().map(|name| parse(name, Standard).unwrap_or_else(|| panic!("{name}")).id()).collect();
        ids.dedup();
        assert_eq!(ids.len(), 1, "{names:?}");
    }
    assert_eq!(parse("Dbm7", Standard).unwrap().id(), "C#:0.3.7.10");
    assert_eq!(parse("D/Gb", Standard).unwrap().id(), "D:0.4.7/F#");
}

#[test]
fn notes_are_spelled_by_their_degree_from_the_root() {
    let table: &[(&str, &str)] = &[
        ("Cm", "C Eb G"),
        ("C7", "C E G Bb"),
        ("F7", "F A C Eb"),
        ("Fm", "F Ab C"),
        ("D", "D F# A"),
        ("F#m", "F# A C#"),
        ("Bb", "Bb D F"),
        ("Caug", "C E G#"),
        ("Cdim7", "C Eb Gb A"),
        ("C7(#9)", "C D# E G Bb"),
        ("Cm(b6)", "C Eb G Ab"),
        ("D/F#", "D F# A"),
        ("C/Bb", "Bb C E G"),
        ("Gbm", "Gb A Db"),
        ("C#", "C# F G#"),
    ];
    for (name, notes) in table {
        assert_eq!(spelled_note_names(name, Standard).map(|notes| notes.join(" ")).as_deref(), Some(*notes), "{name}");
    }
    assert_eq!(spelled_note_names("Bb", German).unwrap().join(" "), "B D F");
    assert_eq!(spelled_note_names("B", German).unwrap().join(" "), "H D# F#");
    assert_eq!(spelled_note_names("Am", Latin).unwrap().join(" "), "La Do Mi");
    assert_eq!(spelled_note_names("N.C.", Standard), None);
}

#[test]
fn german_names_are_read_in_german_notation() {
    assert_eq!(parse("H7", German).map(|chord| chord.root), Some(11));
    assert_eq!(parse("B", German).map(|chord| chord.root), Some(10));
    assert_eq!(parse("b", German).map(|chord| chord.root), Some(10));
    assert_eq!(parse("b", German), parse("Bbm", Standard));
    assert_eq!(parse("h", German), parse("Bm", Standard));
    assert_eq!(parse("G/H", German), parse("G/B", Standard));
    assert_eq!(parse("H", Standard).map(|chord| chord.root), Some(11));
    assert_eq!(note_names(&parse("B", Standard).unwrap(), German, false), ["H", "D#", "F#"]);
    assert_eq!(note_names(&parse("Bb", Standard).unwrap(), German, true), ["B", "D", "F"]);
}

#[test]
fn flats_are_spelled_where_they_are_preferred() {
    assert_eq!(note_names(&parse("Cm7", Standard).unwrap(), Standard, true), ["C", "Eb", "G", "Bb"]);
}

#[test]
fn a_name_is_read_exactly_where_it_is_recognized() {
    let roots = ["C", "C#", "Db", "H", "Bb", "(A", "f#", "a"];
    let middles = ["", "m", "maj", "M", "min", "dim", "°", "ø", "aug", "+", "-", "sus", "add", "Δ", "mi", "x", "alt"];
    let numbers = ["", "2", "4", "5", "6", "69", "7", "9", "11", "13", "1", "٣"];
    let tails = ["", "b5", "#5", "(b9)", "(#9, b13)", "/G", "/f#", "/9", "add9", "alt", "(no3)", "-5", "+", "/", "(", "()", "sus4", ")", "/E/G", "q"];
    for root in roots {
        for middle in middles {
            for number in numbers {
                for tail in tails {
                    let closing = if root.starts_with('(') { ")" } else { "" };
                    let name = format!("{root}{middle}{number}{tail}{closing}");
                    let expanded = lowercase_minor_expanded(&name).unwrap_or_else(|| name.clone());
                    assert_eq!(is_chord_name(&expanded), parse(&name, Standard).is_some(), "{name}");
                }
            }
        }
    }
}

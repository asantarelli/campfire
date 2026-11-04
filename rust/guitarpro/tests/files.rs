// This file is part of Campfire.
// Copyright (c) Pandula Péter 2017-2026.
// https://github.com/pandulapeter/campfire
//
// This Source Code Form is subject to the terms of the Mozilla Public License, v. 2.0.
// If a copy of the MPL was not distributed with this file, You can obtain one at
// https://mozilla.org/MPL/2.0/.

//! The files PyGuitarPro wrote (see `fixtures/generate.py`), read here and compared with what PyGuitarPro itself made of them.

use guitarpro::model::{BeatStatus, NoteKind, Score};
use serde_json::Value;

fn expected_of(name: &str) -> Value {
    let text = std::fs::read_to_string(format!("{}/tests/fixtures/{name}.json", env!("CARGO_MANIFEST_DIR"))).unwrap();
    serde_json::from_str(&text).unwrap()
}

fn score_of(name: &str) -> Score {
    let bytes = std::fs::read(format!("{}/tests/fixtures/{name}", env!("CARGO_MANIFEST_DIR"))).unwrap();
    guitarpro::read(&bytes).unwrap_or_else(|error| panic!("{name}: {error}"))
}

fn check(name: &str) {
    let score = score_of(name);
    let expected = expected_of(name);
    assert_eq!(score.title, expected["title"].as_str().unwrap(), "{name}");
    assert_eq!(score.artist, expected["artist"].as_str().unwrap(), "{name}");
    assert_eq!(score.album, expected["album"].as_str().unwrap(), "{name}");
    assert_eq!(score.tempo as i64, expected["tempo"].as_i64().unwrap(), "{name}");
    let headers = expected["headers"].as_array().unwrap();
    assert_eq!(score.headers.len(), headers.len(), "{name}");
    for (index, (header, expected)) in score.headers.iter().zip(headers).enumerate() {
        let label = format!("{name} measure {}", index + 1);
        assert_eq!(header.numerator as i64, expected["numerator"].as_i64().unwrap(), "{label}");
        assert_eq!(header.denominator as i64, expected["denominator"].as_i64().unwrap(), "{label}");
        assert_eq!(header.is_repeat_open, expected["repeat_open"].as_bool().unwrap(), "{label}");
        assert_eq!(header.repeat_close as i64, expected["repeat_close"].as_i64().unwrap(), "{label}");
        assert_eq!(header.marker.as_deref(), expected["marker"].as_str(), "{label}");
        assert_eq!(header.has_double_bar, expected["double_bar"].as_bool().unwrap(), "{label}");
    }
    let tracks = expected["tracks"].as_array().unwrap();
    assert_eq!(score.tracks.len(), tracks.len(), "{name}");
    for (track, expected) in score.tracks.iter().zip(tracks) {
        assert_eq!(track.name, expected["name"].as_str().unwrap(), "{name}");
        let strings: Vec<i64> = expected["strings"].as_array().unwrap().iter().map(|value| value.as_i64().unwrap()).collect();
        assert_eq!(track.strings.iter().map(|&value| value as i64).collect::<Vec<_>>(), strings, "{name}");
        assert_eq!(track.capo as i64, expected["capo"].as_i64().unwrap(), "{name}");
        let measures = expected["measures"].as_array().unwrap();
        assert_eq!(track.measures.len(), measures.len(), "{name}");
        for (index, (measure, expected)) in track.measures.iter().zip(measures).enumerate() {
            let label = format!("{name} {} measure {}", track.name, index + 1);
            let beats = expected.as_array().unwrap();
            let first_voice = &measure.voices[0];
            assert_eq!(first_voice.len(), beats.len(), "{label}");
            for (position, (beat, expected)) in first_voice.iter().zip(beats).enumerate() {
                let label = format!("{label} beat {}", position + 1);
                let values = expected.as_array().unwrap();
                assert_eq!(beat.start as i64, values[0].as_i64().unwrap(), "{label} start");
                assert_eq!(beat.duration.value as i64, values[1].as_i64().unwrap(), "{label} value");
                assert_eq!(beat.duration.is_dotted, values[2].as_bool().unwrap(), "{label} dotted");
                assert_eq!(beat.duration.enters as i64, values[3].as_i64().unwrap(), "{label} enters");
                assert_eq!(beat.duration.times as i64, values[4].as_i64().unwrap(), "{label} times");
                let status = match beat.status {
                    BeatStatus::Empty => 0,
                    BeatStatus::Normal => 1,
                    BeatStatus::Rest => 2,
                };
                assert_eq!(status, values[5].as_i64().unwrap(), "{label} status");
                match (&beat.chord_shape, &values[7]) {
                    (Some(shape), Value::Object(chord)) => {
                        assert_eq!(shape.first_fret as i64, chord["first_fret"].as_i64().unwrap(), "{label} chord first fret");
                        let frets: Vec<i64> = chord["frets"].as_array().unwrap().iter().map(|fret| fret.as_i64().unwrap()).collect();
                        assert_eq!(shape.frets.iter().map(|&fret| fret as i64).collect::<Vec<_>>(), frets, "{label} chord frets");
                        assert_eq!(beat.chord.as_deref().unwrap_or(""), chord["name"].as_str().unwrap(), "{label} chord name");
                    }
                    (None, Value::Null) => {}
                    (shape, expected) => panic!("{label}: chord {shape:?} against {expected}"),
                }
                let notes = values[6].as_array().unwrap();
                assert_eq!(beat.notes.len(), notes.len(), "{label} notes");
                for (note, expected) in beat.notes.iter().zip(notes) {
                    let expected = expected.as_array().unwrap();
                    assert_eq!(note.string as i64, expected[0].as_i64().unwrap(), "{label} string");
                    assert_eq!(note.fret as i64, expected[1].as_i64().unwrap(), "{label} fret");
                    let kind = match note.kind {
                        NoteKind::Normal => 1,
                        NoteKind::Tie => 2,
                        NoteKind::Dead => 3,
                    };
                    assert_eq!(kind, expected[2].as_i64().unwrap(), "{label} kind");
                }
            }
        }
    }
}

#[test]
fn a_guitar_pro_3_file_is_read_as_pyguitarpro_reads_it() {
    check("song.gp3");
}

#[test]
fn a_guitar_pro_4_file_is_read_as_pyguitarpro_reads_it() {
    check("song.gp4");
}

#[test]
fn a_guitar_pro_5_file_is_read_as_pyguitarpro_reads_it() {
    check("song.gp5");
}

#[test]
fn a_guitar_pro_5_1_file_is_read_as_pyguitarpro_reads_it() {
    check("song51.gp5");
}

#[test]
fn the_effects_of_the_notes_and_the_text_of_the_beats_are_kept() {
    for name in ["song.gp3", "song.gp4", "song.gp5", "song51.gp5"] {
        let score = score_of(name);
        let guitar = &score.tracks[0];
        let first = &guitar.measures[0].voices[0];
        assert_eq!(first[0].text.as_deref(), Some("Verso"), "{name}");
        assert!(guitar.measures[0].voices[0][2].notes[0].effects.hammer, "{name} hammer");
        assert!(guitar.measures[1].voices[0][4].notes.iter().any(|note| note.effects.slide.is_some()), "{name} slide");
        assert!(guitar.measures[3].voices[0][0].notes[0].effects.bend.is_some(), "{name} bend");
        assert!(guitar.measures[3].voices[0][1].notes[0].effects.harmonic, "{name} harmonic");
        assert_eq!(score.headers[0].marker.as_deref(), Some("Intro"), "{name}");
        assert_eq!(guitar.capo, 2, "{name}");
    }
}

#[test]
fn what_is_not_a_guitar_pro_file_is_refused() {
    assert!(matches!(guitarpro::read(b"not a score at all, but a text"), Err(guitarpro::Error::NotGuitarPro)));
    assert!(guitarpro::read(&[]).is_err());
    let bytes = std::fs::read(format!("{}/tests/fixtures/song.gp5", env!("CARGO_MANIFEST_DIR"))).unwrap();
    assert!(matches!(guitarpro::read(&bytes[..bytes.len() / 2]), Err(guitarpro::Error::UnexpectedEnd)));
}

#[test]
fn a_file_whose_last_measure_lacks_the_closing_byte_is_read() {
    // Guitar Pro 5 files from other editors end without the byte that says where the line breaks, and Guitar Pro reads past it.
    let mut bytes = std::fs::read(format!("{}/tests/fixtures/song.gp5", env!("CARGO_MANIFEST_DIR"))).unwrap();
    let complete = guitarpro::read(&bytes).unwrap();
    bytes.pop();
    let truncated = guitarpro::read(&bytes).unwrap();
    assert_eq!(truncated.headers.len(), complete.headers.len());
}

#[test]
fn a_chord_diagram_with_no_name_is_drawn_over_its_beat() {
    use guitarpro::draw::Draw;
    use guitarpro::layout::{layout, Options};
    for name in ["song.gp3", "song.gp4", "song.gp5", "song51.gp5"] {
        let score = score_of(name);
        let page = layout(&score, 0, &Options { width: 60.0, ..Options::default() });
        let texts: Vec<&str> = page.systems.iter().flat_map(|system| &system.draws).filter_map(|draw| if let Draw::Text { text, .. } = draw { Some(text.as_str()) } else { None }).collect();
        // The first fret of the diagram that starts up the neck, and the name of the one that has one.
        assert!(texts.contains(&"7"), "{name}");
        assert!(texts.contains(&"Am"), "{name}");
    }
}

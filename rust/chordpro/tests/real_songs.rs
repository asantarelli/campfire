// This file is part of Campfire.
// Copyright (c) Pandula Péter 2017-2026.
// https://github.com/pandulapeter/campfire
//
// This Source Code Form is subject to the terms of the Mozilla Public License, v. 2.0.
// If a copy of the MPL was not distributed with this file, You can obtain one at
// https://mozilla.org/MPL/2.0/.

//! The `.cho` files the Kotlin app ships and tests with, read by the Rust parser: nothing may fail, and a song moved up and
//! back comes home.

use std::path::{Path, PathBuf};

use chordpro::model::{Block, GridToken, Line, Song};
use chordpro::notation::ChordNotation;
use chordpro::parser;
use chordpro::transposer;

fn repository_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("..").join("..")
}

fn cho_files() -> Vec<PathBuf> {
    let root = repository_root();
    let mut found = Vec::new();
    let mut pending = vec![root.join("presentation"), root.join("tools"), root.join("data")];
    while let Some(directory) = pending.pop() {
        let Ok(entries) = std::fs::read_dir(&directory) else {
            continue;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            let name = path.file_name().and_then(|name| name.to_str()).unwrap_or("");
            if path.is_dir() {
                if name != "build" && name != ".gradle" && name != "node_modules" {
                    pending.push(path);
                }
            } else if name.ends_with(".cho") {
                found.push(path);
            }
        }
    }
    found.sort();
    found
}

fn chord_names(song: &Song) -> Vec<String> {
    let mut names = Vec::new();
    let mut collect = |blocks: &[Block]| {
        for block in blocks {
            if let Block::Section(section) = block {
                for line in &section.lines {
                    match line {
                        Line::Lyrics(lyrics) => names.extend(lyrics.chords.iter().filter(|chord| !chord.is_annotation).map(|chord| chord.name.clone())),
                        Line::Grid(grid) => {
                            names.extend(grid.tokens.iter().filter_map(|token| if let GridToken::Chord(name) = token { Some(name.clone()) } else { None }))
                        }
                        _ => {}
                    }
                }
            }
        }
    };
    collect(&song.blocks);
    names
}

#[test]
fn every_shipped_file_is_read() {
    let files = cho_files();
    if files.is_empty() {
        eprintln!("no .cho files found next to the Rust workspace, skipping");
        return;
    }
    for path in files {
        let text = std::fs::read_to_string(&path).unwrap_or_else(|error| panic!("{}: {error}", path.display()));
        let song = parser::parse(&text, ChordNotation::Standard);
        let moved = transposer::transpose_song(&song, 2, Some(false));
        assert_eq!(moved.blocks.len(), song.blocks.len(), "{}", path.display());
        assert_eq!(chord_names(&moved).len(), chord_names(&song).len(), "{}", path.display());
    }
}

#[test]
fn the_demo_song_is_a_song() {
    let path = repository_root().join("presentation/src/commonMain/composeResources/files/demo/traditional_american-house_of_the_rising_sun.cho");
    let Ok(text) = std::fs::read_to_string(&path) else {
        return;
    };
    let song = parser::parse(&text, ChordNotation::Standard);
    assert!(song.metadata.title.is_some(), "a title");
    assert!(song.has_chords());
    let names = chord_names(&song);
    assert!(names.iter().any(|name| name == "Am"), "{names:?}");
    // Up a tone and back down a tone, spelled with sharps, comes home for every chord that has no flat spelling.
    let there_and_back = transposer::transpose_song(&transposer::transpose_song(&song, 2, Some(false)), -2, Some(false));
    assert_eq!(chord_names(&there_and_back), names);
    // The chords the diagrams are drawn for: each once, in the order they are played, the key aside.
    let chords = chordpro::chords_in(&song);
    assert!(chords.iter().any(|name| name == "Am"), "{chords:?}");
    assert_eq!(chords.iter().collect::<std::collections::HashSet<_>>().len(), chords.len());
}

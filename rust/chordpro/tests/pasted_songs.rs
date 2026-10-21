// This file is part of Campfire.
// Copyright (c) Pandula Péter 2017-2026.
// https://github.com/pandulapeter/campfire
//
// This Source Code Form is subject to the terms of the Mozilla Public License, v. 2.0.
// If a copy of the MPL was not distributed with this file, You can obtain one at
// https://mozilla.org/MPL/2.0/.

//! Chord sheets as they are copied from the web, which have to come out as ChordPro.

use chordpro::convert::prepare_pasted_song;
use chordpro::model::{Block, Line};
use chordpro::notation::ChordNotation;
use chordpro::parser;

const MIL_HORAS: &str = "Mil Horas\nLos Abuelos de La Nada\n\nTono: Gm\n\n[Intro] Gm7\n\n[Primera Parte]\n\n       Cm7     Dm7          Gm7\nHace frío y estoy lejos de casa\n       Cm7\nHace tiempo que estoy\n    Dm7              Gm7\nSentado sobre esta piedra\n          Cm7\nYo me pregunto\n         Dm7            C7\n\"¿Para qué sirven las guerras?\"\n\n[Pre-Estribillo]\n\n            C7\nTengo un cohete en el pantalón\n\nVos estás tan fría\n\nComo la nieve a mi alrededor\n Gm7\nVos estás tan blanca\n           Eb7M\nQue ya no sé qué hacer\n\n[Estribillo]\n\nGm7\n    La otra noche te esperé\n                    Cm7\nBajo la lluvia dos horas\n     Dm7            Eb7M\nMil horas, como un perro\nGm7\nY cuando llegaste\n                           Cm7\nMe miraste y me dijiste: \"Loco\n         Dm7              Eb7M  Eb7M/D\nEstás mojado, ya no te quiero\"\n\n[Solo] Gm7  Cm7  Dm7  Eb  Eb7M/D\n       Gm7  Cm7  Dm7  Eb  Eb7M/D\n\n[Final] Gm7  Cm7  Dm7  Eb7M\n        Gm7  Cm7  Dm7\n        Eb7M  Dm7  Gm7\n        Eb7M  Dm7  Cm7  Gm7\n";

#[test]
fn mil_horas_becomes_a_song() {
    let song = prepare_pasted_song(MIL_HORAS);
    println!("{song}");
    let parsed = parser::parse(&song, ChordNotation::Standard);
    assert_eq!(parsed.metadata.title.as_deref(), Some("Mil Horas"));
    assert_eq!(parsed.metadata.artist.as_deref(), Some("Los Abuelos de La Nada"));
    assert_eq!(parsed.metadata.key.as_deref(), Some("Gm"));
    let lines_with_chords = parsed
        .blocks
        .iter()
        .filter_map(|block| if let Block::Section(section) = block { Some(section) } else { None })
        .flat_map(|section| section.lines.iter())
        .filter(|line| matches!(line, Line::Lyrics(lyrics) if !lyrics.chords.is_empty()))
        .count();
    assert!(lines_with_chords >= 20, "{lines_with_chords}\n{song}");
    // The chord over "frío" is set at that word.
    assert!(song.contains("Hace [Cm7]frío"), "{song}");
    // No row of chords is left among the words.
    assert!(!song.contains("Gm7  Cm7"), "{song}");
}

/// The same song as it came, repeats and all, and as a table cell of a page copies it: the whole text in quotation marks, and the
/// ones inside it doubled.
#[test]
fn the_whole_of_mil_horas_is_recognized_as_it_is_and_as_a_quoted_cell() {
    let full = include_str!("mil_horas_full.txt");
    for pasted in [full.to_string(), format!("\"{}\"", full.trim_end().replace('"', "\"\""))] {
        let song = prepare_pasted_song(&pasted);
        assert!(song.starts_with("{title: Mil Horas}
{artist: Los Abuelos de La Nada}
{key: Gm}
"), "{song}");
        assert!(song.contains("{start_of_chorus: Estribillo}"), "{song}");
        assert!(!song.contains("Gm7  Cm7"), "{song}");
        assert!(song.matches("[Gm7]").count() >= 10, "{song}");
    }
}

// This file is part of Campfire.
// Copyright (c) Pandula Péter 2017-2026.
// https://github.com/pandulapeter/campfire
//
// This Source Code Form is subject to the terms of the Mozilla Public License, v. 2.0.
// If a copy of the MPL was not distributed with this file, You can obtain one at
// https://mozilla.org/MPL/2.0/.

//! The shapes a song defines for its chords with `{define}`: `{define: Am base-fret 1 frets x 0 2 2 1 0 fingers 0 0 2 3 1 0}`.

use crate::model::ChordDefinition;
use crate::voicings::Voicing;

/// The words a definition is made of, which end the list of numbers before them.
const KEYWORDS: [&str; 8] = ["base-fret", "frets", "fingers", "keys", "copy", "copyall", "display", "format"];

fn is_keyword(word: &str) -> bool {
    KEYWORDS.contains(&word.to_ascii_lowercase().as_str())
}

/// What `{define: <value>}` says, or `None` for a definition that gives no frets (a keyboard's keys, a copy of another chord).
///
/// The frets are counted from the fret the diagram starts at, `base-fret` (1 at the nut), the way ChordPro writes them; they are
/// answered counted from the nut, which is how a diagram is drawn.
pub fn parse(value: &str) -> Option<ChordDefinition> {
    let mut words = value.split_whitespace().peekable();
    let name = words.next()?.to_string();
    let mut base_fret: u8 = 1;
    let mut written_frets: Option<Vec<Option<u8>>> = None;
    let mut fingers: Option<Vec<u8>> = None;
    while let Some(keyword) = words.next() {
        match keyword.to_ascii_lowercase().as_str() {
            "base-fret" => {
                if let Some(number) = words.peek().and_then(|word| word.parse::<u8>().ok()).filter(|&number| number >= 1) {
                    base_fret = number;
                    words.next();
                }
            }
            "frets" => {
                let mut frets = Vec::new();
                while let Some(word) = words.next_if(|word| !is_keyword(word)) {
                    frets.push(word.parse::<u8>().ok());
                }
                written_frets = Some(frets);
            }
            "fingers" => {
                let mut values = Vec::new();
                while let Some(word) = words.next_if(|word| !is_keyword(word)) {
                    values.push(word.parse::<u8>().unwrap_or(0));
                }
                fingers = Some(values);
            }
            _ => {}
        }
    }
    let written_frets = written_frets.filter(|frets| !frets.is_empty())?;
    let frets = written_frets.into_iter().map(|fret| fret.map(|fret| if fret == 0 { 0 } else { base_fret.saturating_add(fret - 1) })).collect();
    Some(ChordDefinition { name, base_fret, frets, fingers })
}

/// The shape of `definition` as a diagram draws it.
pub fn voicing_of(definition: &ChordDefinition) -> Voicing {
    Voicing { frets: definition.frets.clone(), fingers: definition.fingers.clone().filter(|fingers| fingers.len() == definition.frets.len()) }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_definition_is_read_with_its_frets_counted_from_the_nut() {
        let definition = parse("Am base-fret 1 frets x 0 2 2 1 0 fingers 0 0 2 3 1 0").unwrap();
        assert_eq!(definition.name, "Am");
        assert_eq!(definition.frets, [None, Some(0), Some(2), Some(2), Some(1), Some(0)]);
        assert_eq!(definition.fingers, Some(vec![0, 0, 2, 3, 1, 0]));
        let high = parse("Dv1 base-fret 5 frets x 1 3 3 3 1").unwrap();
        assert_eq!(high.frets, [None, Some(5), Some(7), Some(7), Some(7), Some(5)]);
        assert_eq!(high.base_fret, 5);
        assert_eq!(high.fingers, None);
    }

    #[test]
    fn the_base_fret_is_optional_and_a_muted_string_can_be_written_in_the_other_ways() {
        assert_eq!(parse("G frets 3 2 0 0 0 N").unwrap().frets, [Some(3), Some(2), Some(0), Some(0), Some(0), None]);
        assert_eq!(parse("G frets 3 2 0 0 0 -1").unwrap().frets.last(), Some(&None));
    }

    #[test]
    fn a_definition_with_no_frets_is_no_shape_of_a_fretted_instrument() {
        assert_eq!(parse("C keys 0 4 7"), None);
        assert_eq!(parse("C copy G"), None);
        assert_eq!(parse(""), None);
    }

    #[test]
    fn the_fingers_are_kept_only_for_as_many_strings_as_there_are() {
        let definition = parse("X frets 0 0 0 0 fingers 1 2").unwrap();
        assert_eq!(voicing_of(&definition).fingers, None);
        let whole = parse("X frets 0 0 0 0 fingers 1 2 3 4").unwrap();
        assert_eq!(voicing_of(&whole).fingers, Some(vec![1, 2, 3, 4]));
    }

    #[test]
    fn a_song_keeps_its_definitions_and_draws_a_diagram_of_each_chord_it_plays_or_defines() {
        let text = "{title: T}
{define: Dv1 base-fret 1 frets x 5 4 2 3 x fingers 0 1 2 4 3 0}
{define: Solo base-fret 1 frets 0 0 0 0 0 0}

[Dv1] [G]la
";
        let song = crate::parser::parse(text, crate::notation::ChordNotation::Standard);
        assert_eq!(song.metadata.definitions.len(), 2);
        assert_eq!(crate::diagram_chords_in(&song), ["Dv1", "G", "Solo"]);
        // The last definition of a chord, on as many strings, is the one that counts.
        let again = crate::parser::parse("{define: X frets 0 0 0 0 0 0}
{define: X frets 1 1 1 1 1 1}
[X]la
", crate::notation::ChordNotation::Standard);
        assert_eq!(again.metadata.definitions.len(), 1);
        assert_eq!(again.metadata.definitions[0].frets, [Some(1); 6]);
    }
}

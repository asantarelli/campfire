// This file is part of Campfire.
// Copyright (c) Pandula Péter 2017-2026.
// https://github.com/pandulapeter/campfire
//
// This Source Code Form is subject to the terms of the Mozilla Public License, v. 2.0.
// If a copy of the MPL was not distributed with this file, You can obtain one at
// https://mozilla.org/MPL/2.0/.

//! Rewrites every chord name of a parsed song: the chords over its lyrics, the cells of its grids, the brackets of its
//! comments and labels (where an intro is written down as a row of chords) and its key.
//!
//! A port of the part of the Kotlin `ChordProChordRewriter` that works on the model. The chord rows above a tab are left as
//! written.

use crate::chord_names;
use crate::directives;
use crate::model::{Block, Comment, Grid, GridToken, Line, LyricChord, Lyrics, Section, Song, Tab};
use crate::tokens;

/// Renames a chord name in the stretch of the song that a `{transpose}` moved by the given offset, 0 before the first.
pub(crate) type Rename<'a> = &'a dyn Fn(i32, &str) -> String;

const KEY_WORDS: [&str; 13] = ["major", "minor", "maj", "min", "dur", "dúr", "moll", "mayor", "menor", "majeur", "mineur", "maggiore", "minore"];

/// `song` with `rename` applied to every chord it names, by the offset of the `{transpose}` in force where the chord stands.
pub(crate) fn rewrite_chords(song: &Song, rename: Rename) -> Song {
    let mut offset = 0;
    let blocks = song.blocks.iter().map(|block| rewrite_block(block, &mut offset, rename)).collect();
    let mut metadata = song.metadata.clone();
    metadata.key = metadata.key.as_deref().map(|key| rename_key(key, &|note| rename(0, note)));
    Song { metadata, blocks }
}

fn rewrite_block(block: &Block, offset: &mut i32, rename: Rename) -> Block {
    let at = *offset;
    match block {
        Block::Section(section) => Block::Section(Section {
            label: section.label.as_deref().map(|label| rewrite_brackets(label, &|name| rename(at, name))),
            lines: section.lines.iter().map(|line| rewrite_line(line, at, rename)).collect(),
            ..section.clone()
        }),
        // A recalled chorus is read where the recall stands, and holds no `{transpose}` of its own.
        Block::ChorusRecall { label, blocks } => Block::ChorusRecall {
            label: label.as_deref().map(|label| rewrite_brackets(label, &|name| rename(at, name))),
            blocks: blocks.iter().map(|inner| rewrite_block(inner, &mut at.clone(), rename)).collect(),
        },
        Block::Comment(comment) => Block::Comment(Comment { text: rewrite_brackets(&comment.text, &|name| rename(at, name)), ..comment.clone() }),
        Block::Transpose { semitones, key } => {
            *offset = *semitones;
            let semitones = *semitones;
            Block::Transpose { semitones, key: key.as_deref().map(|key| rename_key(key, &|note| rename(semitones, note))) }
        }
        Block::Break => Block::Break,
    }
}

fn rewrite_line(line: &Line, offset: i32, rename: Rename) -> Line {
    let rename_name = |name: &str| rename(offset, name);
    match line {
        Line::Lyrics(lyrics) => Line::Lyrics(Lyrics {
            text: lyrics.text.clone(),
            chords: lyrics
                .chords
                .iter()
                .map(|chord| if chord.is_annotation { chord.clone() } else { LyricChord { name: rename_name(&chord.name), ..chord.clone() } })
                .collect(),
        }),
        Line::Grid(grid) => Line::Grid(Grid {
            tokens: grid
                .tokens
                .iter()
                .map(|token| match token {
                    GridToken::Chord(cell) => GridToken::Chord(tokens::cell_chords(cell).into_iter().map(&rename_name).collect::<Vec<_>>().join("~")),
                    other => other.clone(),
                })
                .collect(),
            label: grid.label.as_deref().map(|label| rewrite_brackets(label, &rename_name)),
        }),
        Line::Tab(tab) => Line::Tab(Tab { label: tab.label.as_deref().map(|label| rewrite_brackets(label, &rename_name)), ..tab.clone() }),
        Line::Blank => Line::Blank,
    }
}

/// `text` with `rename` applied to the chord inside every bracket of it, annotations (`[*softly]`) left alone and every
/// other character where it was.
pub(crate) fn rewrite_brackets(text: &str, rename: &dyn Fn(&str) -> String) -> String {
    let chars: Vec<char> = text.chars().collect();
    let mut result = String::new();
    let mut consumed_until = 0;
    for bracket in directives::brackets(text) {
        result.extend(&chars[consumed_until..=bracket.open]);
        let content = &bracket.content;
        let trimmed = content.trim();
        if trimmed.is_empty() || trimmed.starts_with('*') {
            result.push_str(content);
        } else {
            let leading: String = content.chars().take_while(|c| c.is_whitespace()).collect();
            let trailing: String = content.chars().rev().take_while(|c| c.is_whitespace()).collect::<Vec<_>>().into_iter().rev().collect();
            result.push_str(&format!("{leading}{}{trailing}", rename(trimmed)));
        }
        result.push(']');
        consumed_until = bracket.close + 1;
    }
    result.extend(&chars[consumed_until..]);
    result
}

/// `key` with `rename` applied to its note: the key as a chord name (`F#m`), or a note with its mode spelled out after it
/// (`Bb major`, `A-moll`). Anything else (`D dorian`, `capo 2`) is left as written.
pub(crate) fn rename_key(key: &str, rename: &dyn Fn(&str) -> String) -> String {
    let trimmed = key.trim();
    if chord_names::is_chord_name(trimmed) {
        return key.replacen(trimmed, &rename(trimmed), 1);
    }
    let chars: Vec<char> = trimmed.chars().collect();
    let Some(first) = chars.first().filter(|c| ('A'..='H').contains(c)) else {
        return key.to_string();
    };
    let _ = first;
    let note_length = if chars.get(1).is_some_and(|c| chord_names::ACCIDENTALS.contains(*c)) { 2 } else { 1 };
    let rest: String = chars[note_length..].iter().collect();
    let word = rest.trim_start_matches(|c: char| c.is_whitespace() || c == '-').to_lowercase();
    if !rest.starts_with(|c: char| c.is_whitespace() || c == '-') || !KEY_WORDS.contains(&word.as_str()) {
        return key.to_string();
    }
    let note: String = chars[..note_length].iter().collect();
    key.replacen(trimmed, &format!("{}{rest}", rename(&note)), 1)
}

/// The chords a song plays, each once, in the order they first appear: the chords over its lyrics and the cells of its grids.
/// The key is no chord anybody plays, and an annotation is no chord.
pub fn chords_in(song: &Song) -> Vec<String> {
    let mut names = written_chord_names(song);
    if song.metadata.key.is_some() {
        names.remove(0);
    }
    let mut seen = std::collections::HashSet::new();
    names.retain(|name| chord_names::is_chord_name(name) && seen.insert(name.clone()));
    names
}

/// The chord names a song writes, in the order it writes them: its key, then the chords over its lyrics and the cells of its
/// grids. What the notation of a file is judged by, and the spelling of a transposition.
pub(crate) fn written_chord_names(song: &Song) -> Vec<String> {
    let mut names: Vec<String> = song.metadata.key.iter().cloned().collect();
    for block in &song.blocks {
        let Block::Section(section) = block else {
            continue;
        };
        for line in &section.lines {
            match line {
                Line::Lyrics(lyrics) => names.extend(lyrics.chords.iter().filter(|chord| !chord.is_annotation).map(|chord| chord.name.clone())),
                Line::Grid(grid) => {
                    for token in &grid.tokens {
                        if let GridToken::Chord(cell) = token {
                            names.extend(tokens::cell_chords(cell).into_iter().map(str::to_string));
                        }
                    }
                }
                Line::Tab(_) | Line::Blank => {}
            }
        }
    }
    names
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_the_brackets_of_a_text_are_renamed() {
        let upper = |name: &str| name.to_uppercase();
        assert_eq!(rewrite_brackets("Intro [am] [ g ] and [*soft] [] x", &upper), "Intro [AM] [ G ] and [*soft] [] x");
        assert_eq!(rewrite_brackets("no brackets", &upper), "no brackets");
        assert_eq!(rewrite_brackets("[am", &upper), "[am");
    }

    #[test]
    fn a_key_is_renamed_as_a_chord_or_as_a_note_with_its_mode() {
        let mark = |note: &str| format!("<{note}>");
        assert_eq!(rename_key("F#m", &mark), "<F#m>");
        assert_eq!(rename_key("Bb major", &mark), "<Bb> major");
        assert_eq!(rename_key("A-moll", &mark), "<A>-moll");
        assert_eq!(rename_key("D dorian", &mark), "D dorian");
        assert_eq!(rename_key("capo 2", &mark), "capo 2");
    }
}

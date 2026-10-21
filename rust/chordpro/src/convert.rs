// This file is part of Campfire.
// Copyright (c) Pandula Péter 2017-2026.
// https://github.com/pandulapeter/campfire
//
// This Source Code Form is subject to the terms of the Mozilla Public License, v. 2.0.
// If a copy of the MPL was not distributed with this file, You can obtain one at
// https://mozilla.org/MPL/2.0/.

//! Turns a chord sheet as it is copied from a web page or a document (the chords on a row of their own above the words they
//! fall on) into ChordPro: the chords set inside the words, the headings (`Verse 1`, `Chorus`) as sections.
//!
//! A smaller sibling of the Kotlin `ChordSheetConverter`: no PDF or Word reading, no title guessing from page furniture. Text with
//! no row of chords is not a chord sheet, and is left alone (`None`) rather than guessed at.

use crate::chord_names;
use crate::directives;

/// The words that head a part of a song, with the environment each opens.
const HEADINGS: [(&str, &str); 16] = [
    ("intro", "intro"),
    ("verse", "verse"),
    ("estrofa", "verse"),
    ("verso", "verse"),
    ("pre-chorus", "pre-chorus"),
    ("pre-estribillo", "pre-chorus"),
    ("prechorus", "pre-chorus"),
    ("chorus", "chorus"),
    ("estribillo", "chorus"),
    ("coro", "chorus"),
    ("refrain", "chorus"),
    ("bridge", "bridge"),
    ("puente", "bridge"),
    ("solo", "solo"),
    ("outro", "outro"),
    ("final", "outro"),
];

/// A word that stands in a row of chords without being one.
fn is_row_filler(word: &str) -> bool {
    matches!(word, "N.C." | "NC" | "|" | "/" | "-" | "%") || (word.starts_with('x') && word[1..].chars().all(|c| c.is_ascii_digit()) && word.len() > 1)
}

/// The chords of `line` and the character column each starts at, `None` where the line is not a row of chords.
fn chord_row(line: &str) -> Option<Vec<(usize, String)>> {
    let chars: Vec<char> = line.chars().collect();
    let mut found = Vec::new();
    let mut index = 0;
    let mut real = 0;
    while index < chars.len() {
        if chars[index].is_whitespace() {
            index += 1;
            continue;
        }
        let start = index;
        while index < chars.len() && !chars[index].is_whitespace() {
            index += 1;
        }
        let word: String = chars[start..index].iter().collect();
        let name = word.trim_matches(|c| c == '(' || c == ')').to_string();
        if chord_names::is_chord_name(&name) {
            real += 1;
            found.push((start, name));
        } else if !is_row_filler(&word) {
            return None;
        }
    }
    (real > 0).then_some(found)
}

/// The environment and the label a heading line opens, `None` for a line that is not one: a heading word, in brackets or
/// followed by a colon or a number, alone on its line.
fn heading(line: &str) -> Option<(&'static str, String)> {
    let trimmed = line.trim();
    if trimmed.is_empty() || trimmed.chars().count() > 32 {
        return None;
    }
    let is_bracketed = trimmed.starts_with('[') && trimmed.ends_with(']');
    let label = trimmed.trim_start_matches('[').trim_end_matches(']').trim_end_matches(':').trim();
    let word: String = label.chars().take_while(|c| c.is_alphabetic() || *c == '-').collect::<String>().to_lowercase();
    let rest = label[word.len()..].trim();
    let environment = HEADINGS.iter().find(|(name, _)| *name == word)?.1;
    // `Verse 1`, `Chorus x2`: a number or a repeat after the word is still the heading.
    let is_plain = rest.is_empty() || rest.chars().all(|c| c.is_ascii_digit() || c == 'x' || c == ' ');
    (is_plain && (is_bracketed || trimmed.ends_with(':') || !rest.is_empty() || word.len() > 4)).then(|| (environment, label.to_string()))
}

/// `lyrics` with the chords of `row` set into it at the columns they stood at.
fn set_chords(lyrics: &str, row: &[(usize, String)]) -> String {
    let mut chars: Vec<char> = lyrics.chars().collect();
    for (column, name) in row.iter().rev() {
        if *column > chars.len() {
            chars.resize(*column, ' ');
        }
        let bracketed: Vec<char> = format!("[{name}]").chars().collect();
        let at = (*column).min(chars.len());
        chars.splice(at..at, bracketed);
    }
    chars.into_iter().collect::<String>().trim_end().to_string()
}

/// Whether `text` is a chord sheet at all: at least one row of two chords or more, which a sung line never is.
pub fn is_chord_sheet(text: &str) -> bool {
    !text.contains('{') && directives::split_lines(text).iter().any(|line| chord_row(line).is_some_and(|row| row.len() >= 2))
}

/// The ChordPro of a pasted chord sheet: its body, with no title or credit guessed, since the song it goes into has a header of
/// its own. `None` where the text is not a chord sheet, and is to be pasted as it is.
pub fn convert_fragment(text: &str) -> Option<String> {
    if !is_chord_sheet(text) {
        return None;
    }
    let lines = directives::split_lines(text);
    let mut output: Vec<String> = Vec::new();
    let mut open_environment: Option<&'static str> = None;
    let mut index = 0;
    let close = |output: &mut Vec<String>, open_environment: &mut Option<&'static str>| {
        if let Some(environment) = open_environment.take() {
            while output.last().is_some_and(|line| line.trim().is_empty()) {
                output.pop();
            }
            output.push(format!("{{end_of_{environment}}}"));
            output.push(String::new());
        }
    };
    while index < lines.len() {
        let line = &lines[index];
        if let Some((environment, label)) = heading(line) {
            close(&mut output, &mut open_environment);
            output.push(format!("{{start_of_{environment}: {label}}}"));
            open_environment = Some(environment);
            index += 1;
            continue;
        }
        if let Some(row) = chord_row(line) {
            let next = lines.get(index + 1);
            let lyrics = next.filter(|next| !next.trim().is_empty() && chord_row(next).is_none() && heading(next).is_none());
            match lyrics {
                Some(lyrics) => {
                    output.push(set_chords(lyrics, &row));
                    index += 2;
                }
                None => {
                    // A row with no words under it (an intro, a solo) is a line of chords of its own.
                    output.push(row.iter().map(|(_, name)| format!("[{name}]")).collect::<Vec<_>>().join(" "));
                    index += 1;
                }
            }
            continue;
        }
        output.push(line.trim_end().to_string());
        index += 1;
    }
    close(&mut output, &mut open_environment);
    while output.last().is_some_and(|line| line.trim().is_empty()) {
        output.pop();
    }
    Some(output.join("\n"))
}

/// The ChordPro of a whole pasted chord sheet as a new song: the body as [`convert_fragment`] writes it under a header with a
/// title, which is the first line where that is a short line of words, else left for the player to write.
pub fn convert_song(text: &str) -> Option<String> {
    let body = convert_fragment(text)?;
    let lines: Vec<&str> = body.lines().collect();
    let first = lines.iter().position(|line| !line.trim().is_empty())?;
    let candidate = lines[first].trim();
    let is_title = !candidate.starts_with('{') && !candidate.contains('[') && candidate.chars().count() <= 60 && lines.get(first + 1).is_some_and(|next| next.trim().is_empty());
    if is_title {
        let rest = lines[first + 1..].join("\n");
        Some(format!("{{title: {candidate}}}\n{{artist: }}\n{{key: }}\n\n{}", rest.trim_start_matches('\n')))
    } else {
        Some(format!("{{title: }}\n{{artist: }}\n{{key: }}\n\n{body}"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn chords_are_set_into_the_words_they_stood_over() {
        let sheet = "Am        G\nHello darkness my old friend\n";
        assert_eq!(convert_fragment(sheet).as_deref(), Some("[Am]Hello dark[G]ness my old friend"));
        // Columns measured as written: the chord is set before the character it stood over.
        let sheet = "C    G\nHola amigos\n";
        assert_eq!(convert_fragment(sheet).as_deref(), Some("[C]Hola [G]amigos"));
    }

    #[test]
    fn a_chord_past_the_end_of_the_words_is_set_at_their_end() {
        let sheet = "C   G   D\nHi\n";
        assert_eq!(convert_fragment(sheet).as_deref(), Some("[C]Hi  [G]    [D]"));
    }

    #[test]
    fn a_row_with_no_words_is_a_line_of_chords() {
        let sheet = "C  G  Am  F\n\nC    G\nla la\n";
        let converted = convert_fragment(sheet).unwrap();
        assert!(converted.contains("[C] [G] [Am] [F]"), "{converted}");
        assert!(converted.contains("[C]la la[G]"), "{converted}");
    }

    #[test]
    fn headings_become_sections() {
        let sheet = "Verse 1\nC    G\nHola amigos\n\nChorus:\nAm   F\nLa la la la\n\nBridge\nC G Am F\n";
        let converted = convert_fragment(sheet).unwrap();
        assert_eq!(
            converted,
            "{start_of_verse: Verse 1}\n[C]Hola [G]amigos\n{end_of_verse}\n\n{start_of_chorus: Chorus}\n[Am]La la[F] la la\n{end_of_chorus}\n\n{start_of_bridge: Bridge}\n[C] [G] [Am] [F]\n{end_of_bridge}"
        );
    }

    #[test]
    fn prose_is_no_chord_sheet() {
        for text in ["", "Just words\nand more words\n", "A\nLa la la\n", "Time after time\n", "{title: X}\n[Am]x [G]y\n"] {
            assert_eq!(convert_fragment(text), None, "{text:?}");
        }
        assert!(is_chord_sheet("Am G\nwords"));
    }

    #[test]
    fn a_pasted_song_gets_a_header() {
        let sheet = "Sound of Silence\n\nAm   G\nHello darkness\n";
        let song = convert_song(sheet).unwrap();
        assert!(song.starts_with("{title: Sound of Silence}\n{artist: }\n{key: }\n\n"), "{song}");
        assert!(song.ends_with("[Am]Hello[G] darkness"), "{song}");
        let untitled = convert_song("Am   G\nHello darkness\n").unwrap();
        assert!(untitled.starts_with("{title: }\n"));
    }

    #[test]
    fn the_converted_text_reads_back_as_the_chords_that_were_set() {
        let converted = convert_fragment("Am        G\nHello darkness my old friend\n").unwrap();
        let song = crate::parser::parse(&converted, crate::notation::ChordNotation::Standard);
        assert!(song.has_chords());
    }
}

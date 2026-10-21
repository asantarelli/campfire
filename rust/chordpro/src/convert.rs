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

/// Whether `text` is already ChordPro: a line that is a directive ChordPro defines, or two lines with chords in brackets.
pub fn looks_like_chordpro(text: &str) -> bool {
    let lines = directives::split_lines(text);
    if lines.iter().any(|line| directives::is_known_directive(line.trim())) {
        return true;
    }
    let with_chords = lines.iter().filter(|line| directives::brackets(line).iter().any(|bracket| chord_names::is_chord_name(bracket.content.trim()))).count();
    with_chords >= 2
}

/// The directive a line of a chord sheet that says something about the song stands for (`Tono: G` is `{key: G}`), `None` for any
/// other line.
fn field_directive(line: &str) -> Option<String> {
    const FIELDS: [(&str, &str); 14] = [
        ("title", "title"),
        ("titulo", "title"),
        ("título", "title"),
        ("artist", "artist"),
        ("artista", "artist"),
        ("key", "key"),
        ("tono", "key"),
        ("capo", "capo"),
        ("cejilla", "capo"),
        ("tempo", "tempo"),
        ("bpm", "tempo"),
        ("time", "time"),
        ("compas", "time"),
        ("compás", "time"),
    ];
    let trimmed = line.trim();
    let (label, value) = trimmed.split_once(':').or_else(|| trimmed.split_once('='))?;
    let name = label.trim().to_lowercase();
    let (_, directive) = FIELDS.iter().find(|(word, _)| *word == name)?;
    let value = value.trim();
    (!value.is_empty() && chord_row(trimmed).is_none()).then(|| format!("{{{directive}: {value}}}"))
}

/// The lines a chord sheet opens with that say something about the song, as the directives they stand for, and how many lines
/// they take, the blank ones before them included.
fn leading_fields(lines: &[String]) -> (Vec<String>, usize) {
    let mut found = Vec::new();
    let mut consumed = 0;
    for line in lines.iter().take(12) {
        if line.trim().is_empty() {
            if found.is_empty() {
                consumed += 1;
                continue;
            }
            break;
        }
        let Some(directive) = field_directive(line) else {
            break;
        };
        found.push(directive);
        consumed += 1;
    }
    (found, consumed)
}

/// The text of a song to open in the editor from what was pasted over the list of songs, whatever it is: ChordPro is kept as it is
/// (its directives and its chords are the song), a chord sheet is converted, and anything else is the words of a new song. A
/// song with no `{title}` gets an empty one to be filled in, which saving asks for.
pub fn prepare_pasted_song(text: &str) -> String {
    // A page copied from the web is full of no-break spaces, and a file of every kind of line break.
    let cleaned = text.replace('\u{a0}', " ").replace('\u{feff}', "");
    let normalized = directives::split_lines(&cleaned).join("\n");
    let has_title = directives::split_lines(&normalized).iter().any(|line| directives::match_directive(line.trim()).is_some_and(|directive| matches!(directive.name.as_str(), "title" | "t")));
    let with_header = |body: String, has_title: bool| if has_title { body } else { format!("{{title: }}\n{body}") };
    if looks_like_chordpro(&normalized) {
        return with_header(normalized, has_title);
    }
    convert_song(&normalized).unwrap_or_else(|| format!("{{title: }}\n{{artist: }}\n{{key: }}\n\n{normalized}"))
}

/// The ChordPro of a whole pasted chord sheet as a new song: the body as [`convert_fragment`] writes it under a header with a
/// title, which is the first line where that is a short line of words, else left for the player to write.
pub fn convert_song(text: &str) -> Option<String> {
    let mut lines = directives::split_lines(text);
    while lines.first().is_some_and(|line| line.trim().is_empty()) {
        lines.remove(0);
    }
    let mut header: Vec<String> = Vec::new();
    // The first line names the song where it is a short line of words, alone or over the fields: `Sola`, `Alguien - Sola`.
    if let Some(first) = lines.first() {
        let candidate = first.trim();
        let next_is_field_or_blank = lines.get(1).map_or(true, |next| next.trim().is_empty() || field_directive(next).is_some());
        let is_title = chord_row(candidate).is_none()
            && heading(candidate).is_none()
            && !candidate.contains([':', '[', '{'])
            && candidate.chars().count() <= 80
            && next_is_field_or_blank;
        if is_title {
            match candidate.split_once(" - ") {
                Some((artist, title)) => {
                    header.push(format!("{{title: {}}}", title.trim()));
                    header.push(format!("{{artist: {}}}", artist.trim()));
                }
                None => header.push(format!("{{title: {candidate}}}")),
            }
            lines.remove(0);
        }
    }
    let (fields, consumed) = leading_fields(&lines);
    header.extend(fields);
    let rest = lines[consumed..].join("\n");
    let body = convert_fragment(&rest)?;
    if header.is_empty() {
        return Some(format!("{{title: }}\n{{artist: }}\n{{key: }}\n\n{body}"));
    }
    if !header.iter().any(|line| line.starts_with("{title:")) {
        header.insert(0, "{title: }".to_string());
    }
    Some(format!("{}\n\n{}", header.join("\n"), body.trim_start_matches('\n')))
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
        assert!(song.starts_with("{title: Sound of Silence}\n\n"), "{song}");
        assert!(song.ends_with("[Am]Hello[G] darkness"), "{song}");
        let untitled = convert_song("Am   G\nHello darkness\n").unwrap();
        assert!(untitled.starts_with("{title: }\n"));
    }

    #[test]
    fn a_pasted_chordpro_song_keeps_its_directives_and_gets_no_second_header() {
        let pasted = "{title: Sola}\r\n{artist: Alguien}\r\n{key: Am}\r\n\r\n[Am]Hola [G]mundo\r\n";
        assert_eq!(prepare_pasted_song(pasted), "{title: Sola}\n{artist: Alguien}\n{key: Am}\n\n[Am]Hola [G]mundo");
        // Without a title it is given an empty one to fill in, and nothing else is added.
        assert_eq!(prepare_pasted_song("[Am]Hola [G]mundo\n[C]otra [F]linea\n"), "{title: }\n[Am]Hola [G]mundo\n[C]otra [F]linea");
        assert!(looks_like_chordpro("{c: Estribillo}\nla la"));
        assert!(!looks_like_chordpro("{Verse 2}\nla la"));
        assert!(!looks_like_chordpro("Am G\nHola mundo"));
    }

    #[test]
    fn a_sheet_with_fields_over_it_gets_them_as_directives() {
        let pasted = "Tono: Am\nCapo: 2\nArtista: Alguien\n\nAm        G\nHola mundo cruel\n";
        let song = prepare_pasted_song(pasted);
        assert!(song.starts_with("{title: }\n{key: Am}\n{capo: 2}\n{artist: Alguien}\n"), "{song}");
        assert!(song.ends_with("{artist: Alguien}\n\n[Am]Hola mundo[G] cruel"), "{song}");
        let titled = prepare_pasted_song("Alguien - Sola\nTono: G\n\nG   C\nla la la\n");
        assert!(titled.starts_with("{key: G}") || titled.starts_with("{title: Sola}"), "{titled}");
    }

    #[test]
    fn the_converted_text_reads_back_as_the_chords_that_were_set() {
        let converted = convert_fragment("Am        G\nHello darkness my old friend\n").unwrap();
        let song = crate::parser::parse(&converted, crate::notation::ChordNotation::Standard);
        assert!(song.has_chords());
    }
}

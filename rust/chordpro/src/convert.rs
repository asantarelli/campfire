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
use crate::tokens;

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
    matches!(word, "N.C." | "NC" | "|" | "/" | "-" | "%" | "\"" | "\u{201c}" | "\u{201d}" | "(" | ")") || (word.starts_with('x') && word[1..].chars().all(|c| c.is_ascii_digit()) && word.len() > 1)
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
        let mut name = word.trim_matches(|c| c == '(' || c == ')').to_string();
        // A full stop after a chord (`Am.`) is typing, not part of its name.
        if !chord_names::is_chord_name(&name) && name.ends_with('.') && chord_names::is_chord_name(name.trim_end_matches('.')) {
            name = name.trim_end_matches('.').to_string();
        }
        if chord_names::is_chord_name(&name) {
            real += 1;
            found.push((start, name));
        } else if !is_row_filler(&word) {
            return None;
        }
    }
    (real > 0).then_some(found)
}

/// The mark a line of lyrics opens its part with (`A.      Pensar que…`, `1.  Hola`): one letter or digit, a full stop and at least
/// two spaces. Its label and how many characters the mark takes, spaces included.
fn section_mark(line: &str) -> Option<(String, usize)> {
    let chars: Vec<char> = line.chars().collect();
    let first = *chars.first().filter(|c| c.is_alphanumeric())?;
    if chars.get(1) != Some(&'.') {
        return None;
    }
    let spaces = chars[2..].iter().take_while(|c| c.is_whitespace()).count();
    let prefix = 2 + spaces;
    (spaces >= 2 && prefix < chars.len() && chars[prefix..].iter().any(|c| !c.is_whitespace())).then(|| (first.to_uppercase().to_string(), prefix))
}

/// A heading written in brackets (`[Primera Parte]`, `[Riff 1]`, `[Intro] G C D`): the environment it opens, its label, and what
/// follows it on the line. A bracket holding a chord is no heading, it is a chord already written in ChordPro.
fn bracket_heading(line: &str) -> Option<(&'static str, String, String)> {
    let trimmed = line.trim_start();
    let inner_end = trimmed.strip_prefix('[')?.find(']')?;
    let inner = &trimmed[1..1 + inner_end];
    let label = inner.trim().trim_end_matches(':').trim();
    if label.is_empty() || label.chars().count() > 40 || inner.contains('[') || !label.chars().any(char::is_alphabetic) || chord_names::is_chord_name(label) {
        return None;
    }
    let lower = label.to_lowercase();
    let environment = if lower.contains("pre-estribillo") || lower.contains("pre estribillo") || lower.contains("pre-chorus") || lower.contains("prechorus") || lower.contains("pre-coro") {
        "pre-chorus"
    } else if lower.contains("intro") {
        "intro"
    } else if lower.contains("estribillo") || lower.contains("coro") || lower.contains("chorus") || lower.contains("refrain") {
        "chorus"
    } else if lower.contains("puente") || lower.contains("bridge") {
        "bridge"
    } else if lower.contains("solo") {
        "solo"
    } else if lower.contains("final") || lower.contains("outro") {
        "outro"
    } else if lower.contains("riff") {
        "riff"
    } else {
        "verse"
    };
    // What stands after the bracket keeps the columns it had, so that the chords after a heading stay over their words.
    let before: usize = line.chars().count() - trimmed.chars().count();
    let rest_start = before + trimmed[..inner_end + 2].chars().count();
    let rest: String = " ".repeat(rest_start) + line.chars().skip(rest_start).collect::<String>().as_str();
    Some((environment, label.to_string(), rest))
}

/// A line of tablature (`e|--3h5--|`), which a chord sheet writes under its parts.
fn is_tab_line(line: &str) -> bool {
    line.contains('|') && tokens::is_staff_line(line)
}

/// A note in parentheses on a line of its own (`(Riff 2 y 3)`), which is a comment on the song and not part of it.
fn parenthesized_note(line: &str) -> Option<String> {
    let trimmed = line.trim();
    let inner = trimmed.strip_prefix('(')?.strip_suffix(')')?.trim();
    (!inner.is_empty() && chord_row(trimmed).is_none() && !inner.contains(['(', ')', '[', '{'])).then(|| inner.to_string())
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

/// How far into a word a chord may stand and still be set at the start of that word.
const MAX_OFFSET_IN_WORD: usize = 2;

/// `lyrics` with the chords of `row` set into it at the columns they stood at, as a chord sheet in characters means them: a chord a
/// letter away from the start of a word is set at that start (a row typed by hand is rarely exact), the others where they are, and
/// the ones past the end of the words after them, apart.
fn set_chords(lyrics: &str, row: &[(usize, String)]) -> String {
    let chars: Vec<char> = lyrics.trim_end().chars().collect();
    let length = chars.len();
    let starts: Vec<usize> = (0..length).filter(|&index| !chars[index].is_whitespace() && (index == 0 || chars[index - 1].is_whitespace())).collect();
    let mut insertions: std::collections::BTreeMap<usize, Vec<String>> = std::collections::BTreeMap::new();
    let mut previous = 0;
    for (column, name) in row {
        let mut at = (*column).min(length);
        if *column < length {
            // A chord over the first letters of a word belongs to the word, and over a space to the word after it; deeper in, it
            // is over a syllable and stays there.
            let own_start = starts.iter().copied().filter(|start| *start <= *column).max();
            let next_start = starts.iter().copied().find(|start| *start > *column);
            if let Some(start) = own_start.filter(|start| column - start <= MAX_OFFSET_IN_WORD && !chars[*column].is_whitespace()) {
                at = start;
            } else if chars[*column].is_whitespace() || own_start.is_none() {
                if let Some(start) = next_start.filter(|start| start - column <= MAX_OFFSET_IN_WORD) {
                    at = start;
                }
            } else if let Some(start) = next_start.filter(|start| start - column <= 1) {
                at = start;
            }
        }
        // The chords go in the order they stand in, never back over one that was set already.
        at = at.max(previous);
        previous = at;
        insertions.entry(at).or_default().push(format!("[{name}]"));
    }
    let mut result = String::new();
    for index in 0..=length {
        if let Some(due) = insertions.get(&index) {
            if index == length {
                if !result.is_empty() && !result.ends_with(' ') {
                    result.push(' ');
                }
                result.push_str(&due.join(" "));
            } else {
                result.push_str(&due.concat());
            }
        }
        if let Some(character) = chars.get(index) {
            result.push(*character);
        }
    }
    result
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
    let mut lines = directives::split_lines(text);
    let mut output: Vec<String> = Vec::new();
    let mut open_environment: Option<&'static str> = None;
    let mut index = 0;
    fn close(output: &mut Vec<String>, open_environment: &mut Option<&'static str>) {
        if let Some(environment) = open_environment.take() {
            while output.last().is_some_and(|line| line.trim().is_empty()) {
                output.pop();
            }
            output.push(format!("{{end_of_{environment}}}"));
            output.push(String::new());
        }
    }
    fn open_part(output: &mut Vec<String>, open_environment: &mut Option<&'static str>, environment: &'static str, label: &str) {
        close(output, open_environment);
        output.push(format!("{{start_of_{environment}: {label}}}"));
        *open_environment = Some(environment);
    }
    while index < lines.len() {
        let line = lines[index].clone();
        let line = &line;
        if let Some((environment, label, rest)) = bracket_heading(line) {
            open_part(&mut output, &mut open_environment, environment, &label);
            if chord_row(&rest).is_some() {
                // The chords after the heading are a row of their own, read again as the row they are.
                lines[index] = rest;
            } else {
                index += 1;
            }
            continue;
        }
        if is_tab_line(line) {
            output.push("{start_of_tab}".to_string());
            while index < lines.len() && is_tab_line(&lines[index]) {
                output.push(lines[index].trim_end().to_string());
                index += 1;
            }
            output.push("{end_of_tab}".to_string());
            continue;
        }
        if let Some(note) = parenthesized_note(line) {
            output.push(format!("{{comment: {note}}}"));
            index += 1;
            continue;
        }
        if let Some((environment, label)) = heading(line) {
            open_part(&mut output, &mut open_environment, environment, &label);
            index += 1;
            continue;
        }
        if let Some(row) = chord_row(line) {
            let next = lines.get(index + 1);
            let lyrics = next.filter(|next| !next.trim().is_empty() && chord_row(next).is_none() && heading(next).is_none());
            match lyrics {
                Some(lyrics) => {
                    // The mark of a part stands in the columns the chords were measured in, so they move left with the words.
                    let (words, shift) = match section_mark(lyrics) {
                        Some((label, prefix)) => {
                            open_part(&mut output, &mut open_environment, "verse", &label);
                            (lyrics.chars().skip(prefix).collect::<String>(), prefix)
                        }
                        None => {
                            let indent = lyrics.chars().take_while(|c| c.is_whitespace()).count();
                            (lyrics.chars().skip(indent).collect::<String>(), indent)
                        }
                    };
                    let shifted: Vec<(usize, String)> = row.iter().map(|(column, name)| (column.saturating_sub(shift), name.clone())).collect();
                    output.push(set_chords(&words, &shifted));
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
        if let Some((label, prefix)) = section_mark(line) {
            open_part(&mut output, &mut open_environment, "verse", &label);
            output.push(line.chars().skip(prefix).collect::<String>().trim_end().to_string());
            index += 1;
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

/// What was pasted with the debris of a web page taken off: no-break spaces and byte order marks, one kind of line break, and the
/// quotation marks a page puts around a whole block of text (one opening a line, one closing the last), whose place is kept as a
/// space so that the columns the chords were measured in stay where they were.
pub fn clean_pasted(text: &str) -> String {
    let cleaned = text.replace('\u{a0}', " ").replace('\u{feff}', "");
    let mut lines = directives::split_lines(&cleaned);
    let is_quote = |c: char| matches!(c, '"' | '\u{201c}' | '\u{201d}');
    let closes = lines.iter().rposition(|line| !line.trim().is_empty()).filter(|&index| lines[index].trim_end().ends_with(is_quote));
    let opens = lines.iter().position(|line| line.starts_with(is_quote));
    if let (Some(closing), Some(opening)) = (closes, opens) {
        if opening < closing || (opening == closing && lines[opening].chars().filter(|c| is_quote(*c)).count() >= 2) {
            let mut first: Vec<char> = lines[opening].chars().collect();
            first[0] = ' ';
            lines[opening] = first.into_iter().collect();
            let mut last: String = lines[closing].trim_end().to_string();
            last.pop();
            lines[closing] = last;
        }
    }
    lines.join("\n")
}

/// What to write into the editor for a text pasted at the caret: a chord sheet converted to ChordPro (its chords set into the words,
/// its parts as sections), anything else as it is, both cleaned of what a web page leaves in a copy.
pub fn prepare_pasted_fragment(text: &str) -> String {
    let cleaned = clean_pasted(text);
    convert_fragment(&cleaned).unwrap_or(cleaned)
}

/// The text of a song to open in the editor from what was pasted over the list of songs, whatever it is: ChordPro is kept as it is
/// (its directives and its chords are the song), a chord sheet is converted, and anything else is the words of a new song. A
/// song with no `{title}` gets an empty one to be filled in, which saving asks for.
pub fn prepare_pasted_song(text: &str) -> String {
    // A page copied from the web is full of no-break spaces, and a file of every kind of line break.
    let normalized = clean_pasted(text);
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
        let is_field_or_blank = |line: Option<&String>| line.map_or(true, |line| line.trim().is_empty() || field_directive(line).is_some());
        let looks_like_words = |line: &str| {
            let line = line.trim();
            !line.is_empty() && chord_row(line).is_none() && heading(line).is_none() && !line.contains([':', '[', '{']) && line.chars().count() <= 60
        };
        // `Title`, then `Artist`, then a blank line or the fields, as a page of CifraClub copies.
        let artist_line = lines.get(1).filter(|line| looks_like_words(line) && is_field_or_blank(lines.get(2))).map(|line| line.trim().to_string());
        let next_is_field_or_blank = artist_line.is_some() || is_field_or_blank(lines.get(1));
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
                None => {
                    header.push(format!("{{title: {candidate}}}"));
                    if let Some(artist) = &artist_line {
                        header.push(format!("{{artist: {artist}}}"));
                        lines.remove(1);
                    }
                }
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
        assert_eq!(convert_fragment(sheet).as_deref(), Some("[C]Hi [G] [D]"));
    }

    #[test]
    fn a_row_with_no_words_is_a_line_of_chords() {
        let sheet = "C  G  Am  F\n\nC    G\nla la\n";
        let converted = convert_fragment(sheet).unwrap();
        assert!(converted.contains("[C] [G] [Am] [F]"), "{converted}");
        assert!(converted.contains("[C]la la [G]"), "{converted}");
    }

    #[test]
    fn headings_become_sections() {
        let sheet = "Verse 1\nC    G\nHola amigos\n\nChorus:\nAm   F\nLa la la la\n\nBridge\nC G Am F\n";
        let converted = convert_fragment(sheet).unwrap();
        assert_eq!(
            converted,
            "{start_of_verse: Verse 1}\n[C]Hola [G]amigos\n{end_of_verse}\n\n{start_of_chorus: Chorus}\n[Am]La la [F]la la\n{end_of_chorus}\n\n{start_of_bridge: Bridge}\n[C] [G] [Am] [F]\n{end_of_bridge}"
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
        assert!(song.ends_with("[Am]Hello [G]darkness"), "{song}");
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
        assert!(song.ends_with("{artist: Alguien}\n\n[Am]Hola mundo [G]cruel"), "{song}");
        let titled = prepare_pasted_song("Alguien - Sola\nTono: G\n\nG   C\nla la la\n");
        assert!(titled.starts_with("{key: G}") || titled.starts_with("{title: Sola}"), "{titled}");
    }

    const CIFRACLUB: &str = "Sasha, Sissi y el Círculo de Baba\nFito Páez\n\nTono: Am\n\n\"        Am                       Dm\nA.      Pensar que no dijo nada, la ma~ana que lo vi\nE7                            Am\nPensar que el amaba tanto a su divina Sissi\n\n        F               G            Am\nB.      Reptar, reptar en horas de la siesta\nF                G         Am\nCambiar, cambiar las leyes del amar\n\nAm.                   Dm\nAsi el meo tres veces mis zapatos\nAm\nY llego hasta el bar\n Dm                             Am\nPito ese cigarrillo hasta explotar\"";

    #[test]
    fn a_page_of_cifraclub_becomes_a_song() {
        let song = prepare_pasted_song(CIFRACLUB);
        assert!(song.starts_with("{title: Sasha, Sissi y el Círculo de Baba}\n{artist: Fito Páez}\n{key: Am}\n\n"), "{song}");
        assert!(!song.contains('"'), "{song}");
        // The parts marked A. and B. are sections, and the chords moved left with the words.
        assert!(song.contains("{start_of_verse: A}\n[Am]Pensar que no dijo nada, [Dm]la ma~ana"), "{song}");
        assert!(song.contains("[E7]Pensar que el amaba tanto a su divina Sissi") || song.contains("[E7]Pensar que el amaba"), "{song}");
        assert!(song.contains("{end_of_verse}\n\n{start_of_verse: B}\n"), "{song}");
        assert!(song.contains("[Am]Asi el meo tres veces [Dm]mis zapatos"), "{song}");
        assert!(song.contains("[Dm]Pito ese cigarrillo hasta explot[Am]ar"), "{song}");
        let parsed = crate::parser::parse(&song, crate::notation::ChordNotation::Standard);
        assert_eq!(parsed.metadata.title.as_deref(), Some("Sasha, Sissi y el Círculo de Baba"));
        assert_eq!(parsed.metadata.artist.as_deref(), Some("Fito Páez"));
        assert_eq!(parsed.metadata.key.as_deref(), Some("Am"));
        assert!(parsed.has_chords());
    }

    const CHARLY: &str = "De Mi\nCharly García\n\nTono: G\n\n[Intro] G  C/G  G  D\n        G  C/G  G  D\n\nE|-----------------|      \nB|--3h5p3-3s12-----|      \nG|-------------0-12| (4x) \n\n[Primera Parte]\n\nG                              C/G  G\nCuando estes mal cuando estes so---lo\n        Em7       D        F         C           G   D\nNo te olvides de mi porque se que te puedo estimular\n(Riff 2 y 3)\n\n[Estribillo]\n\nC                    Bb2\nNo pienses que estoy loco\n[Riff 4]\n   G7(4)\nEs solo una manera de actuar\n  A7                   D5+\nEstoy comunicado con todo lo demas\n\n( C  Bb2     G )\n( C  Bb2  A7  D7 )\n";

    #[test]
    fn a_page_with_headings_in_brackets_tablature_and_notes_becomes_a_song() {
        let song = prepare_pasted_song(CHARLY);
        assert!(song.starts_with("{title: De Mi}\n{artist: Charly García}\n{key: G}\n\n"), "{song}");
        // The heading takes the chords that follow it on its line, and the rows under it are lines of chords.
        assert!(song.contains("{start_of_intro: Intro}\n[G] [C/G] [G] [D]\n[G] [C/G] [G] [D]\n"), "{song}");
        assert!(song.contains("{start_of_tab}\nE|-----------------|\nB|--3h5p3-3s12-----|\nG|-------------0-12| (4x)\n{end_of_tab}"), "{song}");
        assert!(song.contains("{start_of_verse: Primera Parte}"), "{song}");
        assert!(song.contains("[G]Cuando estes mal cuando estes so---lo") || song.contains("[G]Cuando estes mal"), "{song}");
        assert!(song.contains("{comment: Riff 2 y 3}"), "{song}");
        assert!(song.contains("{start_of_chorus: Estribillo}"), "{song}");
        assert!(song.contains("{start_of_riff: Riff 4}"), "{song}");
        assert!(song.contains("[C] [Bb2] [G]\n[C] [Bb2] [A7] [D7]"), "{song}");
        let parsed = crate::parser::parse(&song, crate::notation::ChordNotation::Standard);
        assert_eq!(parsed.metadata.title.as_deref(), Some("De Mi"));
        assert_eq!(parsed.metadata.key.as_deref(), Some("G"));
        assert!(parsed.has_chords());
    }

    #[test]
    fn what_is_pasted_into_the_editor_is_cleaned_and_converted_too() {
        let fragment = prepare_pasted_fragment("\"   Am        G\nHola mundo cruel\"");
        assert_eq!(fragment, "Hola [Am]mundo cru[G]el");
        assert_eq!(prepare_pasted_fragment("solo palabras\ncon \"comillas\" dentro"), "solo palabras\ncon \"comillas\" dentro");
        assert_eq!(clean_pasted("a\u{a0}b\r\nc"), "a b\nc");
    }

    #[test]
    fn a_full_stop_after_a_chord_and_a_mark_of_a_part_are_read() {
        assert_eq!(chord_row("Am.        Dm").map(|row| row.len()), Some(2));
        assert_eq!(section_mark("A.      Pensar").map(|(label, prefix)| (label, prefix)), Some(("A".to_string(), 8)));
        assert_eq!(section_mark("A. una sola separación"), None);
        assert_eq!(section_mark("Etc. y más"), None);
    }

    #[test]
    fn the_converted_text_reads_back_as_the_chords_that_were_set() {
        let converted = convert_fragment("Am        G\nHello darkness my old friend\n").unwrap();
        let song = crate::parser::parse(&converted, crate::notation::ChordNotation::Standard);
        assert!(song.has_chords());
    }
}

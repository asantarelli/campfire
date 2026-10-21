// This file is part of Campfire.
// Copyright (c) Pandula Péter 2017-2026.
// https://github.com/pandulapeter/campfire
//
// This Source Code Form is subject to the terms of the Mozilla Public License, v. 2.0.
// If a copy of the MPL was not distributed with this file, You can obtain one at
// https://mozilla.org/MPL/2.0/.

//! Linear recognition and rewriting of the notes in a chord name.
//!
//! A port of the Kotlin `ChordProChordNames`: a name is walked once, each part of it handed to a [`ChordNameReader`] in
//! the order it is written, so that "is this a chord name" and "which notes does it stand for" are the same walk and can
//! never disagree. Text is handled as characters, never as bytes: a chord name holds `♯`, `♭`, `°`, `ø` and `Δ`.

/// The signs that may follow a note letter: the ASCII ones a file is written with and the typographic ones it may hold.
pub(crate) const ACCIDENTALS: &str = "#b♯♭";

const QUALITIES: [&str; 16] = ["maj", "Maj", "min", "mi", "dim", "aug", "sus", "add", "m", "M", "+", "-", "°", "ø", "Δ", "∆"];
const ALTERATIONS: [&str; 13] = ["maj", "Maj", "min", "mi", "dim", "aug", "sus", "add", "M", "#", "b", "♯", "♭"];
const OMISSIONS: [&str; 2] = ["no", "omit"];

/// Every spelling of a Latin note that is read, in the order they are tried, and the standard letter of each.
const LATIN_NOTES: [(&str, char); 11] = [
    ("Sol", 'G'),
    ("Do", 'C'),
    ("Dó", 'C'),
    ("Re", 'D'),
    ("Ré", 'D'),
    ("Mi", 'E'),
    ("Fa", 'F'),
    ("Fá", 'F'),
    ("La", 'A'),
    ("Lá", 'A'),
    ("Si", 'B'),
];
const LATIN_INITIALS: &str = "DRMFSL";
const LATIN_SECOND_LETTERS: &str = "oeiaOEIAóéáÓÉÁ";

/// What [`read`] hands over while it walks a name, each part once it has been recognized. The numbers are the digits as
/// written, never empty except for an alteration written without one (`+`, `sus`, `maj`).
#[allow(unused_variables)]
pub trait ChordNameReader {
    /// The root as written, its accidental included: `C`, `F#`, `Bb`, `H`.
    fn root(&mut self, note: &str) {}
    /// The word right after the root: `m`, `maj`, `dim`, `sus`, `add`, `+`, `°`, `ø`, `Δ` and the rest.
    fn quality(&mut self, quality: &str) {}
    /// The number after the quality: `7` in `Cm7`, `69` in `C69`, `2` in `Csus2`.
    fn number(&mut self, number: &str) {}
    /// The `alt` after the number of `C7alt`.
    fn altered(&mut self) {}
    /// A degree added on its own: a number in parentheses (`C7(9)`) or after a slash (`C6/9`).
    fn added(&mut self, degree: &str) {}
    /// A word or sign after the number, with its digits: `b` and `5` in `Cm7b5`, `add` and `9` in `Cm(add9)`.
    fn alteration(&mut self, word: &str, degree: &str) {}
    /// The degree of a `no3` or an `omit5`.
    fn omitted(&mut self, degree: &str) {}
    /// The note after the last slash, as written, which may be in lowercase.
    fn bass(&mut self, note: &str) {}
}

/// A reader nobody is listening through, which is how a name is only asked about.
struct Nobody;

impl ChordNameReader for Nobody {}

/// Whether [`word`] is a chord name as a file spells one.
pub fn is_chord_name(word: &str) -> bool {
    read(word, &mut Nobody)
}

/// Walks a chord name, handing each part of it to [`reader`] in the order it is written, and answers whether the whole of
/// it was a chord name. A name the walk rejects halfway has handed over parts that the caller then throws away.
pub fn read(word: &str, reader: &mut dyn ChordNameReader) -> bool {
    let name = chars(&unwrapped(word));
    if name.is_empty() {
        return false;
    }
    let Some(mut index) = note_end(&name, 0) else {
        return false;
    };
    reader.root(&slice(&name, 0, index));
    if let Some(quality) = QUALITIES.iter().find(|quality| starts_with_at(&name, quality, index)) {
        reader.quality(quality);
        index += quality.chars().count();
    }
    let number_start = index;
    index = digits_end(&name, index);
    if index > number_start {
        reader.number(&slice(&name, number_start, index));
    }
    if starts_with_at(&name, "alt", index) && index > number_start {
        reader.altered();
        index += 3;
    }
    while index < name.len() {
        match name[index] {
            '(' => match group_end(&name, index + 1, reader) {
                Some(end) => index = end,
                None => return false,
            },
            '/' => {
                let digits = digits_end(&name, index + 1);
                if digits > index + 1 {
                    reader.added(&slice(&name, index + 1, digits));
                    index = digits;
                } else {
                    if bass_note_end(&name, index + 1) != Some(name.len()) {
                        return false;
                    }
                    reader.bass(&slice(&name, index + 1, name.len()));
                    return true;
                }
            }
            _ => match alteration_end(&name, index, reader) {
                Some(end) => index = end,
                None => return false,
            },
        }
    }
    true
}

/// The notes a chord name is made of as written: the chord, and its bass after the last slash where there is one.
pub fn notes(name: &str) -> Vec<String> {
    let chord = chars(&unwrapped(name));
    match chord.iter().rposition(|&c| c == '/') {
        Some(separator) if bass_note_end(&chord, separator + 1).is_some() => {
            vec![slice(&chord, 0, separator), slice(&chord, separator + 1, chord.len())]
        }
        _ => vec![chord.iter().collect()],
    }
}

/// [`name`] with [`rewrite`] applied to every note of it, its parentheses kept.
pub fn rewrite_notes(name: &str, rewrite: &dyn Fn(&str) -> String) -> String {
    let notes = notes(name);
    let mut rewritten = vec![rewrite(&notes[0])];
    rewritten.extend(notes.iter().skip(1).map(|bass| rewrite_bass_note(bass, rewrite)));
    let joined = rewritten.join("/");
    if is_parenthesized(name) {
        format!("({joined})")
    } else {
        joined
    }
}

/// [`rewrite`] applied to the bass note of a chord name, in capitals however the file spells it, and folded back to the
/// case it was written in: a chart that writes `D/f#` has to get `E/g#` back rather than `E/G#`.
fn rewrite_bass_note(note: &str, rewrite: &dyn Fn(&str) -> String) -> String {
    let Some(letter) = note.chars().next() else {
        return rewrite(note);
    };
    if !letter.is_lowercase() {
        return rewrite(note);
    }
    let upper: String = letter.to_uppercase().chain(note.chars().skip(1)).collect();
    let rewritten = rewrite(&upper);
    let mut rewritten_chars = rewritten.chars();
    match rewritten_chars.next() {
        Some(first) => first.to_lowercase().chain(rewritten_chars).collect(),
        None => rewritten,
    }
}

/// [`word`] as the minor chord a lowercase root stands for in Central European charts (`a` is `Am`, `h7` is `Hm7`, `f#`
/// is `F#m`, `(e)` is `(Em)`), or `None` for anything else. A suffix that starts with an `m` is not taken, since `am` would
/// say minor twice and `amaj7` would be neither.
pub fn lowercase_minor_expanded(word: &str) -> Option<String> {
    let name = chars(&unwrapped(word));
    let root = *name.first().filter(|c| ('a'..='h').contains(c))?;
    let note_length = if name.get(1).is_some_and(|&c| ACCIDENTALS.contains(c)) { 2 } else { 1 };
    if starts_with_at(&name, "m", note_length) {
        return None;
    }
    let expanded = format!(
        "{}{}m{}",
        root.to_ascii_uppercase(),
        slice(&name, 1, note_length),
        slice(&name, note_length, name.len())
    );
    if !is_chord_name(&expanded) {
        return None;
    }
    Some(if is_parenthesized(word) { format!("({expanded})") } else { expanded })
}

/// The other way: [`name`], a minor chord, written with a lowercase root and no `m`.
pub fn lowercase_minor_folded(name: &str) -> String {
    let chord = chars(&unwrapped(name));
    let note_length = if chord.get(1).is_some_and(|&c| ACCIDENTALS.contains(c)) { 2 } else { 1 };
    let rest = slice(&chord, note_length, chord.len());
    let folded = format!(
        "{}{}{}",
        chord[0].to_lowercase(),
        slice(&chord, 1, note_length),
        rest.strip_prefix('m').unwrap_or(&rest)
    );
    if is_parenthesized(name) {
        format!("({folded})")
    } else {
        folded
    }
}

/// [`word`] as the chord a Latin name stands for (`Lam` is `Am`, `Sib7` is `Bb7`, `DO/MI` is `C/E`, `Re/fa#` is `D/f#`,
/// `(Sol)` is `(G)`), or `None` for anything else. The root is read in title case or in capitals, never in lowercase,
/// which is how a word is written; the bass note after the last `/` in any case.
pub fn latin_expanded(word: &str) -> Option<String> {
    let name = chars(&unwrapped(word));
    // A cheap refusal first, since this is asked of every name of every file that is read.
    if !name.first().is_some_and(|&c| LATIN_INITIALS.contains(c)) || !name.get(1).is_some_and(|&c| LATIN_SECOND_LETTERS.contains(c)) {
        return None;
    }
    let (letter, length) = latin_note_at(&name, 0, false)?;
    let mut rest: Vec<char> = name[length..].to_vec();
    if let Some(slash) = rest.iter().rposition(|&c| c == '/') {
        if let Some((bass_letter, bass_length)) = latin_note_at(&rest, slash + 1, true) {
            let note_end = slash + 1 + bass_length;
            let end = if rest.get(note_end).is_some_and(|&c| ACCIDENTALS.contains(c)) { note_end + 1 } else { note_end };
            if end == rest.len() {
                let bass = if rest[slash + 1].is_lowercase() { bass_letter.to_ascii_lowercase() } else { bass_letter };
                let mut rebuilt: Vec<char> = rest[..=slash].to_vec();
                rebuilt.push(bass);
                rebuilt.extend_from_slice(&rest[note_end..]);
                rest = rebuilt;
            }
        }
    }
    let expanded: String = std::iter::once(letter).chain(rest).collect();
    if !is_chord_name(&expanded) {
        return None;
    }
    Some(if is_parenthesized(word) { format!("({expanded})") } else { expanded })
}

/// The length of the Latin note [`key`] starts with, in title case or capitals and without its accidental, or `None`.
pub fn latin_note_length(key: &str) -> Option<usize> {
    latin_note_at(&chars(key), 0, false).map(|(_, length)| length)
}

/// The standard letter of the Latin note [`text`] spells at [`index`], and its length, or `None` where it spells none.
fn latin_note_at(text: &[char], index: usize, is_any_case: bool) -> Option<(char, usize)> {
    LATIN_NOTES.iter().find_map(|&(spelling, letter)| {
        let length = spelling.chars().count();
        let end = (index + length).min(text.len());
        let candidate: String = text.get(index..end)?.iter().collect();
        let upper: String = spelling.chars().flat_map(char::to_uppercase).collect();
        let matches = candidate == spelling
            || candidate == upper
            || (is_any_case && candidate.to_lowercase() == spelling.to_lowercase());
        matches.then_some((letter, length))
    })
}

fn is_parenthesized(name: &str) -> bool {
    let chars: Vec<char> = name.chars().collect();
    chars.len() > 2 && chars[0] == '(' && chars[chars.len() - 1] == ')'
}

fn unwrapped(name: &str) -> String {
    if is_parenthesized(name) {
        let chars: Vec<char> = name.chars().collect();
        chars[1..chars.len() - 1].iter().collect()
    } else {
        name.to_string()
    }
}

fn chars(text: &str) -> Vec<char> {
    text.chars().collect()
}

fn slice(text: &[char], from: usize, to: usize) -> String {
    text[from..to].iter().collect()
}

fn starts_with_at(name: &[char], pattern: &str, index: usize) -> bool {
    let mut position = index;
    for expected in pattern.chars() {
        if name.get(position) != Some(&expected) {
            return false;
        }
        position += 1;
    }
    true
}

/// The index after the note at [`index`], or `None` where there is none: a capital letter from `A` to `H`, and the
/// accidental after it where there is one.
fn note_end(name: &[char], index: usize) -> Option<usize> {
    let letter = *name.get(index)?;
    if !('A'..='H').contains(&letter) {
        return None;
    }
    Some(if name.get(index + 1).is_some_and(|&c| ACCIDENTALS.contains(c)) { index + 2 } else { index + 1 })
}

/// The same for the note after a `/`. A chart that writes its minor roots in lowercase writes the bass note that way too
/// (`D/f#`, `C/h`), and unlike a root there is nothing a lowercase letter there could be instead: the slash has already
/// said a note follows.
fn bass_note_end(name: &[char], index: usize) -> Option<usize> {
    let letter = *name.get(index)?;
    if !('A'..='H').contains(&letter.to_ascii_uppercase()) {
        return None;
    }
    Some(if name.get(index + 1).is_some_and(|&c| ACCIDENTALS.contains(c)) { index + 2 } else { index + 1 })
}

/// A chord's numbers are ASCII digits: `char::is_numeric` would take every script's.
fn digits_end(name: &[char], start: usize) -> usize {
    let mut index = start;
    while name.get(index).is_some_and(char::is_ascii_digit) {
        index += 1;
    }
    index
}

fn alteration_end(name: &[char], index: usize, reader: &mut dyn ChordNameReader) -> Option<usize> {
    let word: String = match ALTERATIONS.iter().find(|alteration| starts_with_at(name, alteration, index)) {
        Some(alteration) => (*alteration).to_string(),
        None => match name[index] {
            sign @ ('+' | '-') => sign.to_string(),
            _ => return None,
        },
    };
    let word_length = word.chars().count();
    let end = digits_end(name, index + word_length);
    if word == "-" && end == index + 1 {
        return None;
    }
    reader.alteration(&word, &slice(name, index + word_length, end));
    Some(end)
}

fn group_end(name: &[char], start: usize, reader: &mut dyn ChordNameReader) -> Option<usize> {
    let mut index = start;
    let mut requires_item = true;
    while index < name.len() && name[index] != ')' {
        if !requires_item && name[index] == ',' {
            index += 1;
            while name.get(index) == Some(&' ') {
                index += 1;
            }
            requires_item = true;
            continue;
        }
        let number = digits_end(name, index);
        index = if number > index {
            reader.added(&slice(name, index, number));
            number
        } else if let Some(omission) = OMISSIONS.iter().find(|omission| starts_with_at(name, omission, index)) {
            let omission_length = omission.chars().count();
            let end = digits_end(name, index + omission_length);
            if end <= index + omission_length {
                return None;
            }
            reader.omitted(&slice(name, index + omission_length, end));
            end
        } else {
            alteration_end(name, index, reader)?
        };
        requires_item = false;
    }
    if index > start && !requires_item && name.get(index) == Some(&')') {
        Some(index + 1)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_usual_spelling_is_a_chord_name() {
        for name in ["B7(b9)", "B6/9", "B6/9/F#", "Bm(maj7)", "Bmi7", "B-", "B7alt", "BΔ7", "C(add9)", "G7(#9, b13)", "(Bm7/F#)"] {
            assert!(is_chord_name(name), "{name}");
        }
    }

    #[test]
    fn prose_and_incomplete_names_are_not_chords() {
        for name in ["Bridge", "Halt", "Calt", "C(", "C()", "C/", "C/E/G", "E-----", "A-ha", "(x2)"] {
            assert!(!is_chord_name(name), "{name}");
        }
    }

    #[test]
    fn the_bass_follows_the_last_slash() {
        assert_eq!(notes("B6/9/F#"), vec!["B6/9", "F#"]);
        assert_eq!(rewrite_notes("(Bm7/F#)", &|note| note.to_lowercase()), "(bm7/f#)");
    }

    #[test]
    fn a_lowercase_root_is_a_minor_chord() {
        assert_eq!(lowercase_minor_expanded("a").as_deref(), Some("Am"));
        assert_eq!(lowercase_minor_expanded("h7").as_deref(), Some("Hm7"));
        assert_eq!(lowercase_minor_expanded("f#").as_deref(), Some("F#m"));
        assert_eq!(lowercase_minor_expanded("(e)").as_deref(), Some("(Em)"));
        assert_eq!(lowercase_minor_expanded("e/G").as_deref(), Some("Em/G"));
        for name in ["am", "amaj7", "add", "fine", "A", "N.C."] {
            assert_eq!(lowercase_minor_expanded(name), None, "{name}");
        }
        assert_eq!(lowercase_minor_folded("C#m7"), "c#7");
        assert_eq!(lowercase_minor_folded("(Bbm)"), "(bb)");
    }

    #[test]
    fn a_bass_note_may_be_written_in_lowercase() {
        for name in ["D/f#", "A/c#", "C/h", "Dm/f#"] {
            assert!(is_chord_name(name), "{name}");
        }
        assert_eq!(notes("D/f#"), vec!["D", "f#"]);
    }

    #[test]
    fn a_lowercase_root_is_still_not_a_chord_on_its_own() {
        for name in ["a", "h7", "f#", "break"] {
            assert!(!is_chord_name(name), "{name}");
        }
    }

    #[test]
    fn a_chord_number_is_an_ascii_digit() {
        assert!(!is_chord_name("C\u{0663}7"));
        assert!(is_chord_name("C7"));
    }

    #[test]
    fn a_lowercase_minor_with_a_lowercase_bass_expands() {
        assert_eq!(lowercase_minor_expanded("d/f#").as_deref(), Some("Dm/f#"));
    }

    #[test]
    fn latin_names_are_read_as_the_chords_they_name() {
        assert_eq!(latin_expanded("Lam").as_deref(), Some("Am"));
        assert_eq!(latin_expanded("Sib7").as_deref(), Some("Bb7"));
        assert_eq!(latin_expanded("DO/MI").as_deref(), Some("C/E"));
        assert_eq!(latin_expanded("Re/fa#").as_deref(), Some("D/f#"));
        assert_eq!(latin_expanded("(Sol)").as_deref(), Some("(G)"));
        assert_eq!(latin_expanded("Dom").as_deref(), Some("Cm"));
        for name in ["Am", "Do ut", "re", "N.C."] {
            assert_eq!(latin_expanded(name), None, "{name}");
        }
    }

    #[test]
    fn no_name_is_a_chord_in_both_readings() {
        // A standard chord that starts like a Latin note would need an `o`, an `e`, an `i` or an `a` right after its
        // letter, and no quality or alteration there is starts with one.
        for root in ["D", "R", "M", "F", "S", "L"] {
            for tail in ["m", "maj", "M", "min", "mi", "dim", "aug", "sus", "add", "+", "-", "°", "ø", "Δ", "∆", "7", "9", "alt", "b5", "#5", "(b9)"] {
                let name = format!("{root}{tail}");
                assert!(!(is_chord_name(&name) && latin_expanded(&name).is_some()), "{name}");
            }
        }
    }
}

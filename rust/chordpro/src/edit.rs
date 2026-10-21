// This file is part of Campfire.
// Copyright (c) Pandula Péter 2017-2026.
// https://github.com/pandulapeter/campfire
//
// This Source Code Form is subject to the terms of the Mozilla Public License, v. 2.0.
// If a copy of the MPL was not distributed with this file, You can obtain one at
// https://mozilla.org/MPL/2.0/.

//! What the editor's buttons write: a chord around the selection, a section around some lines, a directive of the header.
//!
//! Every function takes the text and the selection (as character offsets, which is what a text field counts in) and answers
//! the new text and the new selection, so that the interface only has to put them back.

use crate::directives;
use crate::parser;

/// A text after an edit, with the selection the caret should have in it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Edited {
    pub text: String,
    pub selection_start: usize,
    pub selection_end: usize,
}

fn chars(text: &str) -> Vec<char> {
    text.chars().collect()
}

fn collect(chars: &[char]) -> String {
    chars.iter().collect()
}

/// `prefix` and `suffix` around the selection, inside the line it is in (`[` and `]` around a word is a chord). With nothing
/// selected the caret ends up between the two, where the name is typed; with a selection, the selection is kept on what it
/// held.
pub fn wrap_selection(text: &str, start: usize, end: usize, prefix: &str, suffix: &str) -> Edited {
    let chars = chars(text);
    let (start, end) = ordered(start, end, chars.len());
    let mut result = collect(&chars[..start]);
    result.push_str(prefix);
    result.push_str(&collect(&chars[start..end]));
    result.push_str(suffix);
    result.push_str(&collect(&chars[end..]));
    let inner_start = start + prefix.chars().count();
    Edited { text: result, selection_start: inner_start, selection_end: inner_start + (end - start) }
}

/// `inserted` written at the caret as it is, with the caret after it. A selection is replaced.
pub fn insert_at_caret(text: &str, start: usize, end: usize, inserted: &str) -> Edited {
    let chars = chars(text);
    let (start, end) = ordered(start, end, chars.len());
    let mut result = collect(&chars[..start]);
    result.push_str(inserted);
    result.push_str(&collect(&chars[end..]));
    let caret = start + inserted.chars().count();
    Edited { text: result, selection_start: caret, selection_end: caret }
}

/// `{name: selection}` on a line of its own: the line the caret is in where that is empty, a new line after it where it is not.
pub fn insert_directive(text: &str, start: usize, end: usize, name: &str, has_value: bool) -> Edited {
    let chars = chars(text);
    let (start, end) = ordered(start, end, chars.len());
    let selected = collect(&chars[start..end]);
    let selected = selected.trim();
    let line = if has_value { format!("{{{name}: {selected}}}") } else { format!("{{{name}}}") };
    let value_start = if has_value { name.chars().count() + 3 } else { 0 };
    let value_length = if has_value { selected.chars().count() } else { 0 };
    place_on_own_line(&chars, start, &line, value_start, value_length)
}

/// `{start_of_name}`, the selected lines and `{end_of_name}`, on lines of their own: the lines the selection touches are
/// moved into the section. With nothing selected the caret is on the empty line between the two, where the section is typed.
pub fn insert_section(text: &str, start: usize, end: usize, name: &str) -> Edited {
    let chars = chars(text);
    let (start, end) = ordered(start, end, chars.len());
    let opening = format!("{{start_of_{name}}}");
    let closing = format!("{{end_of_{name}}}");
    let caret_offset = opening.chars().count() + 1;
    if start == end {
        return place_on_own_line(&chars, start, &format!("{opening}\n\n{closing}"), caret_offset, 0);
    }
    let first = chars[..start].iter().rposition(|&c| c == '\n').map_or(0, |index| index + 1);
    // A selection that ends right after a line break does not touch the line after it.
    let last = if chars[end - 1] == '\n' { end - 1 } else { end };
    let last = chars[last..].iter().position(|&c| c == '\n').map_or(chars.len(), |offset| last + offset);
    let selected = collect(&chars[first..last]);
    let block = format!("{opening}\n{selected}\n{closing}");
    let result = format!("{}{block}{}", collect(&chars[..first]), collect(&chars[last..]));
    let selection_start = first + caret_offset;
    Edited { text: result, selection_start, selection_end: selection_start + selected.chars().count() }
}

/// Puts `block` on lines of its own at the line the caret is in, and selects `selection_length` characters of it from
/// `selection_offset`: the line itself where it is empty, a new line after it where it is not.
fn place_on_own_line(chars: &[char], caret: usize, block: &str, selection_offset: usize, selection_length: usize) -> Edited {
    let line_start = chars[..caret].iter().rposition(|&c| c == '\n').map_or(0, |index| index + 1);
    let line_end = chars[caret..].iter().position(|&c| c == '\n').map_or(chars.len(), |offset| caret + offset);
    let line_is_empty = chars[line_start..line_end].iter().all(|c| c.is_whitespace());
    let (head, tail, block_start) = if line_is_empty {
        (collect(&chars[..line_start]), collect(&chars[line_end..]), line_start)
    } else {
        (collect(&chars[..line_end]) + "\n", collect(&chars[line_end..]), line_end + 1)
    };
    let selection_start = block_start + selection_offset;
    Edited { text: format!("{head}{block}{tail}"), selection_start, selection_end: selection_start + selection_length }
}

/// Where the body of the song `lines` make up begins, the way the parser reads it: at the first line that is not blank, a `#`
/// comment or a directive that only sets something — or past the end of a song that has none. What stands before it is the
/// header.
pub fn body_start_index(lines: &[String]) -> usize {
    lines
        .iter()
        .position(|line| {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with('#') {
                return false;
            }
            match directives::match_directive(trimmed) {
                None => true,
                Some(directive) => !directives::has_selector_suffix(&directive.name) && parser::starts_body(&directive),
            }
        })
        .unwrap_or(lines.len())
}

/// A metadata directive written into the header: `{name: }` after the last header line, with the caret inside it. A directive a
/// song can say only once (a title, a year) that the file has already moves the caret to its value instead of writing a
/// second; one that may be repeated (`tag`) is added after the last of its kind.
pub fn insert_metadata(text: &str, name: &str, is_repeatable: bool) -> Edited {
    insert_header_line(text, name, is_repeatable, &format!("{{{name}: }}"), name.chars().count() + 3, &|directive| directive.name == name)
}

/// The same for `{meta: key value}`.
pub fn insert_meta(text: &str, key: &str, is_repeatable: bool) -> Edited {
    let line = format!("{{meta: {key} }}");
    let key_length = key.chars().count();
    let finds = |directive: &directives::Directive| {
        directive.name == "meta" && directive.value.as_deref().is_some_and(|value| value.trim().split(' ').next().is_some_and(|first| first.eq_ignore_ascii_case(key)))
    };
    insert_header_line(text, key, is_repeatable, &line, "meta: ".chars().count() + key_length + 2, &finds)
}

fn insert_header_line(text: &str, _name: &str, is_repeatable: bool, line: &str, caret_in_line: usize, finds: &dyn Fn(&directives::Directive) -> bool) -> Edited {
    let lines = directives::split_lines(text);
    let body_start = body_start_index(&lines);
    let existing: Vec<usize> = (0..body_start)
        .filter(|&index| directives::match_directive(lines[index].trim()).is_some_and(|directive| finds(&directive)))
        .collect();
    let line_start_of = |index: usize| lines[..index].iter().map(|line| line.chars().count() + 1).sum::<usize>();
    if let (false, Some(&first)) = (is_repeatable, existing.first()) {
        // The caret goes to the end of the value of the line that is there.
        let value_end = line_start_of(first) + lines[first].trim_end().chars().count().saturating_sub(1);
        return Edited { text: text.to_string(), selection_start: value_end, selection_end: value_end };
    }
    // After the last line of its kind, else after the last header line that says something, else at the top.
    let after = existing.last().map(|&index| index + 1).unwrap_or_else(|| {
        (0..body_start).rev().find(|&index| !lines[index].trim().is_empty() && !lines[index].trim().starts_with('#')).map_or(0, |index| index + 1)
    });
    let mut new_lines = lines.clone();
    new_lines.insert(after, line.to_string());
    let separator = crate::text_rewriter::line_separator_of(text);
    let mut result = crate::text_rewriter::join_lines(&new_lines, text);
    if lines.len() == 1 && lines[0].is_empty() {
        // An empty text has no line break to keep.
        result = format!("{line}{separator}");
    }
    let selection = line_start_of_in(&new_lines, after) + caret_in_line;
    Edited { text: result, selection_start: selection, selection_end: selection }
}

fn line_start_of_in(lines: &[String], index: usize) -> usize {
    lines[..index].iter().map(|line| line.chars().count() + 1).sum()
}

fn ordered(start: usize, end: usize, length: usize) -> (usize, usize) {
    let (start, end) = (start.min(length), end.min(length));
    (start.min(end), start.max(end))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_chord_goes_around_the_selection_or_around_the_caret() {
        let edited = wrap_selection("hello world", 6, 11, "[", "]");
        assert_eq!(edited.text, "hello [world]");
        assert_eq!((edited.selection_start, edited.selection_end), (7, 12));
        let empty = wrap_selection("hello", 5, 5, "[", "]");
        assert_eq!(empty.text, "hello[]");
        assert_eq!((empty.selection_start, empty.selection_end), (6, 6));
        // A backwards selection is the same selection.
        assert_eq!(wrap_selection("abc", 3, 1, "[", "]").text, "a[bc]");
        // Counted in characters, not bytes.
        assert_eq!(wrap_selection("canción", 3, 4, "[", "]").text, "can[c]ión");
    }

    #[test]
    fn a_symbol_is_written_at_the_caret() {
        let edited = insert_at_caret("C", 1, 1, "♯");
        assert_eq!(edited.text, "C♯");
        assert_eq!(edited.selection_start, 2);
        assert_eq!(insert_at_caret("abc", 0, 3, "|").text, "|");
    }

    #[test]
    fn a_directive_takes_a_line_of_its_own() {
        let on_empty = insert_directive("one\n\ntwo", 4, 4, "comment", true);
        assert_eq!(on_empty.text, "one\n{comment: }\ntwo");
        assert_eq!(on_empty.selection_start, 4 + "{comment: ".chars().count());
        let after_text = insert_directive("one\ntwo", 1, 1, "comment", true);
        assert_eq!(after_text.text, "one\n{comment: }\ntwo");
        let wrapping = insert_directive("hello world", 6, 11, "comment", true);
        assert_eq!(wrapping.text, "hello world\n{comment: world}");
        assert_eq!(&wrapping.text.chars().collect::<Vec<_>>()[wrapping.selection_start..wrapping.selection_end], &['w', 'o', 'r', 'l', 'd']);
        assert_eq!(insert_directive("", 0, 0, "chorus", false).text, "{chorus}");
    }

    #[test]
    fn a_section_wraps_the_selected_lines() {
        let empty = insert_section("", 0, 0, "verse");
        assert_eq!(empty.text, "{start_of_verse}\n\n{end_of_verse}");
        assert_eq!(empty.selection_start, "{start_of_verse}\n".chars().count());
        let wrapped = insert_section("[Am]one\n[G]two\nthree", 0, 14, "chorus");
        assert_eq!(wrapped.text, "{start_of_chorus}\n[Am]one\n[G]two\n{end_of_chorus}\nthree");
    }

    #[test]
    fn the_header_ends_where_the_body_begins() {
        let lines: Vec<String> = ["{title: A}", "# note", "", "{key: G}", "[Am]body", "{artist: late}"].iter().map(|line| line.to_string()).collect();
        assert_eq!(body_start_index(&lines), 4);
        let only_header: Vec<String> = ["{title: A}", "{key: G}"].iter().map(|line| line.to_string()).collect();
        assert_eq!(body_start_index(&only_header), 2);
        let starts_with_section: Vec<String> = ["{title: A}", "{sov}", "x"].iter().map(|line| line.to_string()).collect();
        assert_eq!(body_start_index(&starts_with_section), 1);
    }

    #[test]
    fn a_new_directive_goes_into_the_header_after_the_last_line_of_it() {
        let edited = insert_metadata("{title: A}\n{key: G}\n[Am]body\n", "artist", false);
        assert_eq!(edited.text, "{title: A}\n{key: G}\n{artist: }\n[Am]body\n");
        assert_eq!(edited.selection_start, "{title: A}\n{key: G}\n".chars().count() + "{artist: ".chars().count());
        let in_empty = insert_metadata("", "title", false);
        assert_eq!(in_empty.text, "{title: }\n");
        let before_body = insert_metadata("[Am]body", "title", false);
        assert_eq!(before_body.text, "{title: }\n[Am]body");
    }

    #[test]
    fn a_directive_that_can_only_be_said_once_takes_the_caret_to_the_one_there() {
        let text = "{title: A}\n{key: G}\n[Am]body\n";
        let edited = insert_metadata(text, "key", false);
        assert_eq!(edited.text, text);
        // Right before the closing brace of the key line.
        assert_eq!(edited.selection_start, "{title: A}\n{key: G".chars().count());
    }

    #[test]
    fn a_repeatable_directive_is_added_after_the_last_of_its_kind() {
        let edited = insert_metadata("{title: A}\n{tag: one}\n{key: G}\n[Am]x", "tag", true);
        assert_eq!(edited.text, "{title: A}\n{tag: one}\n{tag: }\n{key: G}\n[Am]x");
    }

    #[test]
    fn a_meta_item_is_written_with_its_key() {
        let edited = insert_meta("{title: A}\n[Am]x", "language", true);
        assert_eq!(edited.text, "{title: A}\n{meta: language }\n[Am]x");
        assert_eq!(edited.selection_start, "{title: A}\n{meta: language ".chars().count());
        let again = insert_meta("{title: A}\n{meta: cover https://x.y/a.png}\n[Am]x", "cover", false);
        assert_eq!(again.text, "{title: A}\n{meta: cover https://x.y/a.png}\n[Am]x");
    }

    #[test]
    fn the_line_endings_of_the_file_are_kept() {
        let edited = insert_metadata("{title: A}\r\n[Am]x\r\n", "key", false);
        assert_eq!(edited.text, "{title: A}\r\n{key: }\r\n[Am]x\r\n");
    }
}

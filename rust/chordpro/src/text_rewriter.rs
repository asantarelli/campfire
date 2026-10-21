// This file is part of Campfire.
// Copyright (c) Pandula Péter 2017-2026.
// https://github.com/pandulapeter/campfire
//
// This Source Code Form is subject to the terms of the Mozilla Public License, v. 2.0.
// If a copy of the MPL was not distributed with this file, You can obtain one at
// https://mozilla.org/MPL/2.0/.

//! Rewrites the chord names of a raw ChordPro text in place, keeping every other character where it was: the text is written
//! straight back to the user's file, so a transposition must not touch a byte that is not a chord.
//!
//! A port of the part of the Kotlin `ChordProChordRewriter` that works on text. The chord rows above a tab and the frets of a
//! tab are left as written.

use crate::directives::{self, ScannedLine};
use crate::rewriter;
use crate::tokens;

/// The line separator `text` is written with: CRLF where any line of it ends that way, a bare CR where the file uses those and
/// no CRLF, LF otherwise.
pub fn line_separator_of(text: &str) -> &'static str {
    if text.contains("\r\n") {
        "\r\n"
    } else if text.contains('\r') {
        "\r"
    } else {
        "\n"
    }
}

/// The inverse of [`directives::split_lines`] for one particular `original`: the separator it is written with and the trailing
/// line break put back where it had one, so that an edit of one line leaves every other byte of the file as it was.
pub fn join_lines(lines: &[String], original: &str) -> String {
    let separator = line_separator_of(original);
    let joined = lines.join(separator);
    if (original.ends_with('\n') || original.ends_with('\r')) && !lines.is_empty() {
        joined + separator
    } else {
        joined
    }
}

/// `text` with `rename` applied to every chord name it writes: the chords in brackets over its lyrics and in its comments and
/// labels, the cells of its grids and the `{key}`. A tab, an environment handed to another program, a `#` comment and every
/// other directive are left as they are.
pub fn rewrite_chord_names_in_text(text: &str, rename: &dyn Fn(&str) -> String) -> String {
    let lines: Vec<String> = directives::scan(text).iter().map(|line| rewrite_line(line, rename)).collect();
    join_lines(&lines, text)
}

fn rewrite_line(line: &ScannedLine, rename: &dyn Fn(&str) -> String) -> String {
    if line.is_source_comment || line.trimmed.is_empty() {
        return line.raw.clone();
    }
    if let Some(directive) = &line.directive {
        return rewrite_directive(line, &directive.name, rename);
    }
    match line.environment.as_deref() {
        Some("tab") => line.raw.clone(),
        Some("grid") => rewrite_grid_line(&line.raw, &line.trimmed, rename),
        Some(environment) if directives::is_delegate_environment(environment) => line.raw.clone(),
        _ => rewriter::rewrite_brackets(&line.raw, rename),
    }
}

fn rewrite_directive(line: &ScannedLine, name: &str, rename: &dyn Fn(&str) -> String) -> String {
    let is_key = name == "key";
    if !is_key && !directives::has_chords_in_value(name) {
        return line.raw.clone();
    }
    let Some(value_start) = directives::value_start(&line.trimmed) else {
        return line.raw.clone();
    };
    let raw: Vec<char> = line.raw.chars().collect();
    let leading = raw.iter().take_while(|c| c.is_whitespace()).count();
    // The directive ends with its closing brace, which is the last character of the trimmed line.
    let trimmed_length = line.trimmed.chars().count();
    let value_from = leading + value_start;
    let value_to = leading + trimmed_length - 1;
    if value_from > value_to {
        return line.raw.clone();
    }
    let value: String = raw[value_from..value_to].iter().collect();
    let rewritten = if is_key {
        let leading_space: String = value.chars().take_while(|c| c.is_whitespace()).collect();
        let trailing_space: String = value.chars().rev().take_while(|c| c.is_whitespace()).collect::<Vec<_>>().into_iter().rev().collect();
        format!("{leading_space}{}{trailing_space}", rewriter::rename_key(value.trim(), rename))
    } else {
        rewriter::rewrite_brackets(&value, rename)
    };
    let mut result: String = raw[..value_from].iter().collect();
    result.push_str(&rewritten);
    result.extend(&raw[value_to..]);
    result
}

/// A grid line with `rename` applied to the words that are chords, in the columns they were.
fn rewrite_grid_line(raw: &str, trimmed: &str, rename: &dyn Fn(&str) -> String) -> String {
    let kinds = tokens::parse_grid_tokens(trimmed);
    let mut kinds = kinds.into_iter();
    let mut result = String::new();
    let mut word = String::new();
    let mut flush = |word: &mut String, result: &mut String| {
        if word.is_empty() {
            return;
        }
        match kinds.next() {
            Some(crate::model::GridToken::Chord(_)) => {
                let renamed: Vec<String> = tokens::cell_chords(word).into_iter().map(rename).collect();
                result.push_str(&renamed.join("~"));
            }
            _ => result.push_str(word),
        }
        word.clear();
    };
    for character in raw.chars() {
        if character.is_whitespace() {
            flush(&mut word, &mut result);
            result.push(character);
        } else {
            word.push(character);
        }
    }
    flush(&mut word, &mut result);
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    fn upper(name: &str) -> String {
        name.to_uppercase()
    }

    #[test]
    fn only_the_chords_of_a_text_are_rewritten() {
        let text = "{title: am i blue}\n{key: Am}\n# am\n[am]am [g]i\n{c: [am] x}\n{sot}\n[am]\ne|-0-|\n{eot}\n";
        let rewritten = rewrite_chord_names_in_text(text, &upper);
        assert_eq!(rewritten, "{title: am i blue}\n{key: AM}\n# am\n[AM]am [G]i\n{c: [AM] x}\n{sot}\n[am]\ne|-0-|\n{eot}\n");
    }

    #[test]
    fn a_grid_keeps_its_columns() {
        let text = "{sog}\nA  | am  .  | c~g % |  Coda\n{eog}\n";
        assert_eq!(rewrite_chord_names_in_text(text, &upper), "{sog}\nA  | AM  .  | C~G % |  Coda\n{eog}\n");
    }

    #[test]
    fn delegated_text_is_never_a_chord() {
        let text = "{start_of_abc}\n[CEG]\n{end_of_abc}\n";
        assert_eq!(rewrite_chord_names_in_text(text, &upper), text);
    }

    #[test]
    fn the_line_endings_and_the_last_break_are_kept() {
        assert_eq!(rewrite_chord_names_in_text("[am]x\r\n[g]y", &upper), "[AM]x\r\n[G]y");
        assert_eq!(rewrite_chord_names_in_text("[am]x\r\n[g]y\r\n", &upper), "[AM]x\r\n[G]y\r\n");
        assert_eq!(rewrite_chord_names_in_text("[am]x\r[g]y\r", &upper), "[AM]x\r[G]y\r");
        assert_eq!(rewrite_chord_names_in_text("", &upper), "");
    }

    #[test]
    fn a_directive_written_with_spaces_keeps_them() {
        assert_eq!(rewrite_chord_names_in_text("  { key :  Am  }  \n", &upper), "  { key :  AM  }  \n");
        assert_eq!(rewrite_chord_names_in_text("{key}\n", &upper), "{key}\n");
    }
}

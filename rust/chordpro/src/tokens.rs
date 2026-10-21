// This file is part of Campfire.
// Copyright (c) Pandula Péter 2017-2026.
// https://github.com/pandulapeter/campfire
//
// This Source Code Form is subject to the terms of the Mozilla Public License, v. 2.0.
// If a copy of the MPL was not distributed with this file, You can obtain one at
// https://mozilla.org/MPL/2.0/.

//! The words of a line, the cells of a grid line and the staff of a tab.
//!
//! Whitespace is [`char::is_whitespace`], never an ASCII-only test: pasted charts are full of non-breaking spaces.

use crate::model::GridToken;

const BAR_LINES: [&str; 6] = ["|", "||", "|.", "|:", ":|", ":|:"];
pub(crate) const GRID_CELL_CHORD_SEPARATOR: char = '~';
const MINIMUM_STAFF_DASH_COUNT: usize = 3;

/// Splits a grid line into tokens. Whatever comes before the first bar line is the left margin and whatever follows the
/// last one the right margin, so on a line that has a bar both are text: a margin label such as `A` or `Coda` names a part of
/// the song, and taking it for a chord would transpose it. A `/` marks where a chord is played and is not one either.
pub fn parse_grid_tokens(trimmed_line: &str) -> Vec<GridToken> {
    let words: Vec<&str> = trimmed_line.split(char::is_whitespace).filter(|word| !word.is_empty()).collect();
    let first_bar = words.iter().position(|word| is_bar(word));
    let last_bar = words.iter().rposition(|word| is_bar(word));
    words
        .iter()
        .enumerate()
        .map(|(index, &word)| {
            if let (Some(first), Some(last)) = (first_bar, last_bar) {
                if index < first || index > last {
                    return GridToken::Text(word.to_string());
                }
            }
            if is_bar(word) {
                GridToken::Bar(word.to_string())
            } else if word == "." {
                GridToken::Beat
            } else if word == "%" || word == "%%" {
                GridToken::Repeat(word.to_string())
            } else if word == "/" {
                GridToken::Text(word.to_string())
            } else {
                GridToken::Chord(word.to_string())
            }
        })
        .collect()
}

/// The bar lines of a grid, the repeats and the voltas (`|1`, `:|2`, `:|2>`) included.
pub fn is_bar(word: &str) -> bool {
    BAR_LINES.contains(&word) || is_volta(word)
}

/// `:?\|\d+>?`, matched by hand.
fn is_volta(word: &str) -> bool {
    let rest = word.strip_prefix(':').unwrap_or(word);
    let Some(rest) = rest.strip_prefix('|') else {
        return false;
    };
    let rest = rest.strip_suffix('>').unwrap_or(rest);
    !rest.is_empty() && rest.chars().all(|c| c.is_ascii_digit())
}

/// The chords of one grid cell: several chords are written into a cell by joining them with a `~`.
pub fn cell_chords(cell: &str) -> Vec<&str> {
    cell.split(GRID_CELL_CHORD_SEPARATOR).collect()
}

/// A tablature line: enough dashes to be a staff, and made mostly of the characters a staff is made of. The letters of a
/// technique (`h`, `p`, `x`, …) and the string name in front of the line are the minority that is allowed.
pub fn is_staff_line(line: &str) -> bool {
    let mut dashes = 0;
    let mut staff = 0;
    let mut other = 0;
    for character in line.chars() {
        if character == '-' {
            dashes += 1;
            staff += 1;
        } else if character == '|' || character.is_numeric() {
            staff += 1;
        } else if !character.is_whitespace() {
            other += 1;
        }
    }
    dashes >= MINIMUM_STAFF_DASH_COUNT && staff >= other
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_grid_line_has_margins_around_its_bars() {
        let tokens = parse_grid_tokens("A | Am . . | C % | Coda");
        assert_eq!(
            tokens,
            vec![
                GridToken::Text("A".into()),
                GridToken::Bar("|".into()),
                GridToken::Chord("Am".into()),
                GridToken::Beat,
                GridToken::Beat,
                GridToken::Bar("|".into()),
                GridToken::Chord("C".into()),
                GridToken::Repeat("%".into()),
                GridToken::Bar("|".into()),
                GridToken::Text("Coda".into()),
            ]
        );
    }

    #[test]
    fn voltas_are_bars() {
        for word in ["|1", ":|2", ":|2>", "|3>"] {
            assert!(is_bar(word), "{word}");
        }
        for word in ["|", "||", ":|:"] {
            assert!(is_bar(word), "{word}");
        }
        for word in ["|x", "1|", "Am"] {
            assert!(!is_bar(word), "{word}");
        }
    }

    #[test]
    fn a_staff_needs_dashes_and_mostly_staff_characters() {
        assert!(is_staff_line("e|---0---3---|"));
        assert!(is_staff_line("G|--2h4--|"));
        assert!(!is_staff_line("Hello - world"));
        assert!(!is_staff_line("Am   G   C"));
    }
}

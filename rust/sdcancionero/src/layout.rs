// This file is part of Campfire.
// Copyright (c) Pandula Péter 2017-2026.
// https://github.com/pandulapeter/campfire
//
// This Source Code Form is subject to the terms of the Mozilla Public License, v. 2.0.
// If a copy of the MPL was not distributed with this file, You can obtain one at
// https://mozilla.org/MPL/2.0/.

//! Where the words and the chords of a song go on the page, as plain numbers: no interface type in here, so that every rule is
//! tested with a measuring function instead of a window.

use chordpro::model::Lyrics;

/// A chord (or an annotation) anchored over a row of text.
#[derive(Debug, Clone, PartialEq)]
pub struct ChordAt {
    /// The character offset in the row's text the chord sits over.
    pub offset: usize,
    pub name: String,
    pub is_annotation: bool,
}

/// One row of a lyrics line that was wrapped to the width of its column.
#[derive(Debug, Clone, PartialEq)]
pub struct Row {
    pub text: String,
    pub chords: Vec<ChordAt>,
}

/// The tokens of `text`: every word with the spaces after it, so that a row ends on a word and starts on one.
fn tokens(text: &str) -> Vec<String> {
    let mut tokens = Vec::new();
    let mut current = String::new();
    let mut in_spaces = false;
    for character in text.chars() {
        if character.is_whitespace() {
            in_spaces = true;
        } else if in_spaces {
            tokens.push(std::mem::take(&mut current));
            in_spaces = false;
        }
        current.push(character);
    }
    if !current.is_empty() || tokens.is_empty() {
        tokens.push(current);
    }
    tokens
}

/// A row of lyrics laid out: its text in pieces and where each chord stands.
///
/// A chord stands exactly over the character it was written before, whatever the font. Where two chords would run into each other
/// it is the words that give way, never the chord: the text after the first chord starts further right, so the second chord keeps
/// its place over its own word.
#[derive(Debug, Clone, PartialEq)]
pub struct Placed {
    /// Pieces of the row's text, each with the `x` it starts at. The piece after a chord starts at that chord's `x`.
    pub segments: Vec<(f32, String)>,
    /// The `x` of each chord of the row, in the order they are given.
    pub chord_xs: Vec<f32>,
    /// How wide the row is, text and chords.
    pub width: f32,
}

/// Lays out `text` with `chords` over it. `widths` are the widths of the chords as they are shown, `gap` the least space between
/// two of them and `measure` the width of a text in the lyrics font.
pub fn place_row(text: &str, chords: &[ChordAt], widths: &[f32], gap: f32, measure: &dyn Fn(&str) -> f32) -> Placed {
    let characters: Vec<char> = text.chars().collect();
    let mut segments: Vec<(f32, String)> = Vec::new();
    let mut chord_xs = Vec::with_capacity(chords.len());
    let mut segment_x = 0.0f32;
    let mut cursor = 0;
    let mut free = f32::MIN;
    let mut widest = 0.0f32;
    for (chord, width) in chords.iter().zip(widths) {
        let offset = chord.offset.clamp(cursor, characters.len());
        let piece: String = characters[cursor..offset].iter().collect();
        let end = segment_x + measure(&piece);
        if !piece.is_empty() {
            segments.push((segment_x, piece));
        }
        // The chord stands where the text before it ends; only another chord can push it, and then the text moves with it.
        let chord_x = end.max(free);
        chord_xs.push(chord_x);
        free = chord_x + width + gap;
        widest = widest.max(chord_x + width);
        segment_x = chord_x;
        cursor = offset;
    }
    let rest: String = characters[cursor..].iter().collect();
    let end = segment_x + measure(rest.trim_end());
    if !rest.is_empty() {
        segments.push((segment_x, rest));
    }
    Placed { segments, chord_xs, width: end.max(widest) }
}

/// `lyrics` as rows no wider than `max_width`, cut between words, each with the chords that stand over it. The width of a row is
/// the one `place_row` gives it, so the room the chords take is counted. A word wider than the column goes on a row of its own
/// and overflows rather than being cut. `measure` is the width of a text in the lyrics font, `chord_width` that of a chord as it is
/// shown (given its name and whether it is an annotation).
pub fn wrap_lyrics(lyrics: &Lyrics, max_width: f32, measure: &dyn Fn(&str) -> f32, chord_width: &dyn Fn(&str, bool) -> f32, gap: f32) -> Vec<Row> {
    let mut row_texts: Vec<String> = Vec::new();
    let mut current = String::new();
    let mut consumed = 0;
    let fits = |text: &str, consumed: usize| {
        let length = text.chars().count();
        let chords: Vec<ChordAt> = lyrics
            .chords
            .iter()
            .filter(|chord| chord.position >= consumed && chord.position <= consumed + length)
            .map(|chord| ChordAt { offset: chord.position - consumed, name: chord.name.clone(), is_annotation: chord.is_annotation })
            .collect();
        let widths: Vec<f32> = chords.iter().map(|chord| chord_width(&chord.name, chord.is_annotation)).collect();
        // Spaces at the end of a row are not part of its width: they would push a word to the next row for nothing.
        place_row(text.trim_end(), &chords, &widths, gap, measure).width <= max_width
    };
    for token in tokens(&lyrics.text) {
        let candidate = format!("{current}{token}");
        if !current.is_empty() && !fits(&candidate, consumed) {
            consumed += current.chars().count();
            row_texts.push(std::mem::take(&mut current));
        }
        current.push_str(&token);
    }
    row_texts.push(current);
    let mut rows: Vec<Row> = row_texts.into_iter().map(|text| Row { text, chords: Vec::new() }).collect();
    // A chord belongs to the row its offset falls in; one after the last character belongs to the last row.
    let mut row_start = 0;
    let mut starts = Vec::with_capacity(rows.len());
    for row in &rows {
        starts.push(row_start);
        row_start += row.text.chars().count();
    }
    for chord in &lyrics.chords {
        let index = starts.iter().rposition(|&start| start <= chord.position).unwrap_or(0);
        rows[index].chords.push(ChordAt { offset: chord.position - starts[index], name: chord.name.clone(), is_annotation: chord.is_annotation });
    }
    rows
}

/// One piece of a song laid out in a column, with the height it takes.
#[derive(Debug, Clone, PartialEq)]
pub struct Item<T> {
    pub content: T,
    pub height: f32,
    /// Whether a column may end right before this item.
    pub can_break_before: bool,
    /// Whether a column has to end right before this item (`{column_break}`).
    pub must_break_before: bool,
}

/// `items` shared between `columns` columns, in reading order, each about as tall as the others: a column ends at the first
/// place a column may end once it holds its share of the height, and always where one must. Never fewer than one column and
/// never more than `columns`, an empty column left out.
pub fn into_columns<T>(items: Vec<Item<T>>, columns: usize) -> Vec<Vec<Item<T>>> {
    let columns = columns.max(1);
    let total: f32 = items.iter().map(|item| item.height).sum();
    let mut result: Vec<Vec<Item<T>>> = vec![Vec::new()];
    let mut column_height = 0.0;
    let mut placed_height = 0.0;
    for item in items {
        let remaining_columns = columns - result.len();
        let share = if remaining_columns == 0 { f32::MAX } else { (total - placed_height + column_height) / (remaining_columns + 1) as f32 };
        let is_full = column_height > 0.0 && column_height >= share;
        let breaks = !result.last().is_some_and(Vec::is_empty) && ((item.must_break_before && remaining_columns > 0) || (item.can_break_before && is_full && remaining_columns > 0));
        if breaks {
            result.push(Vec::new());
            column_height = 0.0;
        }
        column_height += item.height;
        placed_height += item.height;
        result.last_mut().expect("a column is open").push(item);
    }
    result.retain(|column| !column.is_empty());
    if result.is_empty() {
        result.push(Vec::new());
    }
    result
}

/// How many columns fit in `available_width` at the most, each at least `minimum_column_width` wide with `spacing` between
/// them, and at most `wanted`. Always at least one.
pub fn columns_that_fit(available_width: f32, wanted: usize, minimum_column_width: f32, spacing: f32) -> usize {
    (1..=wanted.max(1)).rev().find(|&count| count == 1 || (available_width - spacing * (count - 1) as f32) / count as f32 >= minimum_column_width).unwrap_or(1)
}

#[cfg(test)]
mod tests {
    use super::*;
    use chordpro::model::LyricChord;

    /// Every character is ten wide.
    fn ten(text: &str) -> f32 {
        text.chars().count() as f32 * 10.0
    }

    fn lyrics(text: &str, chords: &[(usize, &str)]) -> Lyrics {
        Lyrics {
            text: text.to_string(),
            chords: chords.iter().map(|(position, name)| LyricChord { position: *position, name: name.to_string(), is_annotation: false }).collect(),
        }
    }

    fn no_chord_room(_: &str, _: bool) -> f32 {
        0.0
    }

    fn chord_at(offset: usize, name: &str) -> ChordAt {
        ChordAt { offset, name: name.to_string(), is_annotation: false }
    }

    #[test]
    fn a_line_that_fits_is_one_row() {
        let rows = wrap_lyrics(&lyrics("Hello world", &[(0, "Am"), (6, "G")]), 200.0, &ten, &no_chord_room, 0.0);
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].text, "Hello world");
        assert_eq!(rows[0].chords.iter().map(|c| c.offset).collect::<Vec<_>>(), [0, 6]);
    }

    #[test]
    fn a_long_line_is_cut_between_words_and_the_chords_follow_their_words() {
        // 10 characters a row at most.
        let rows = wrap_lyrics(&lyrics("one two three four", &[(0, "A"), (4, "B"), (8, "C"), (14, "D")]), 100.0, &ten, &no_chord_room, 0.0);
        let texts: Vec<&str> = rows.iter().map(|row| row.text.as_str()).collect();
        assert_eq!(texts, ["one two ", "three four"]);
        assert_eq!(rows[0].chords.iter().map(|c| (c.offset, c.name.as_str())).collect::<Vec<_>>(), [(0, "A"), (4, "B")]);
        assert_eq!(rows[1].chords.iter().map(|c| (c.offset, c.name.as_str())).collect::<Vec<_>>(), [(0, "C"), (6, "D")]);
    }

    #[test]
    fn a_word_wider_than_the_column_keeps_a_row_of_its_own() {
        let rows = wrap_lyrics(&lyrics("a extraordinarily b", &[]), 50.0, &ten, &no_chord_room, 0.0);
        let texts: Vec<&str> = rows.iter().map(|row| row.text.as_str()).collect();
        assert_eq!(texts, ["a ", "extraordinarily ", "b"]);
    }

    #[test]
    fn a_chord_after_the_last_character_stays_on_the_last_row() {
        let rows = wrap_lyrics(&lyrics("hello world", &[(11, "C")]), 60.0, &ten, &no_chord_room, 0.0);
        assert_eq!(rows.last().unwrap().chords.len(), 1);
        assert_eq!(rows.last().unwrap().chords[0].offset, "world".len());
    }

    #[test]
    fn the_room_the_chords_take_is_counted_when_a_line_is_cut() {
        // Without chords "aa bb cc" is 80 wide and fits; a chord of 40 over each word needs more.
        let text = "aa bb cc";
        let chords = [(0, "Cmaj7"), (3, "Dm7"), (6, "G7")];
        assert_eq!(wrap_lyrics(&lyrics(text, &chords), 100.0, &ten, &no_chord_room, 0.0).len(), 1);
        let rows = wrap_lyrics(&lyrics(text, &chords), 100.0, &ten, &|_, _| 40.0, 5.0);
        assert!(rows.len() > 1, "{rows:?}");
    }

    #[test]
    fn a_line_of_chords_alone_is_one_row_without_text() {
        let rows = wrap_lyrics(&lyrics("", &[(0, "Am"), (0, "G"), (0, "C")]), 200.0, &ten, &no_chord_room, 0.0);
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].chords.len(), 3);
        let placed = place_row("", &rows[0].chords, &[20.0, 10.0, 10.0], 8.0, &ten);
        assert_eq!(placed.chord_xs, [0.0, 28.0, 46.0]);
        assert!(placed.segments.is_empty());
    }

    #[test]
    fn a_chord_stands_exactly_over_the_character_it_was_written_before() {
        let chords = [chord_at(0, "Am7"), chord_at(6, "G"), chord_at(16, "C")];
        let placed = place_row("hello wonderful world", &chords, &[30.0, 10.0, 10.0], 5.0, &ten);
        // Where each chord is, the text below it is the text that came after it in the file.
        assert_eq!(placed.chord_xs, [0.0, 60.0, 160.0]);
        assert_eq!(placed.segments, [(0.0, "hello ".to_string()), (60.0, "wonderful ".to_string()), (160.0, "world".to_string())]);
        assert_eq!(placed.width, 210.0);
    }

    #[test]
    fn a_chord_that_would_collide_moves_the_words_not_itself() {
        // The first chord is 30 wide, but the word under the second starts at 10: the words after the first chord move right.
        let chords = [chord_at(0, "Am7"), chord_at(1, "G")];
        let placed = place_row("ab", &chords, &[30.0, 10.0], 5.0, &ten);
        assert_eq!(placed.chord_xs, [0.0, 35.0]);
        // The "b" is under its chord, and the "a" is under the first.
        assert_eq!(placed.segments, [(0.0, "a".to_string()), (35.0, "b".to_string())]);
        assert_eq!(placed.width, 45.0);
    }

    #[test]
    fn every_chord_sits_on_the_start_of_the_text_it_was_written_before() {
        let text = "Hello darkness my old friend";
        let chords = [chord_at(0, "Am"), chord_at(2, "Fmaj7"), chord_at(6, "G"), chord_at(10, "C/E"), chord_at(20, "Dm7b5")];
        let widths: Vec<f32> = chords.iter().map(|chord| chord.name.chars().count() as f32 * 12.0).collect();
        let placed = place_row(text, &chords, &widths, 6.0, &ten);
        let characters: Vec<char> = text.chars().collect();
        for (chord, x) in chords.iter().zip(&placed.chord_xs) {
            let after: String = characters[chord.offset..].iter().collect();
            let starting_here = placed.segments.iter().find(|(start, _)| start == x).map(|(_, piece)| piece.clone()).unwrap_or_default();
            // The piece that starts at the chord is the beginning of what followed it in the file.
            assert!(after.starts_with(&starting_here) || starting_here.is_empty(), "{} at {x}: {starting_here:?}", chord.name);
            assert!(!starting_here.is_empty(), "{} has no text under it", chord.name);
        }
        // The pieces put together are the text, nothing lost and nothing doubled.
        let joined: String = placed.segments.iter().map(|(_, piece)| piece.as_str()).collect();
        assert_eq!(joined, text);
    }


    fn items(heights: &[f32]) -> Vec<Item<usize>> {
        heights.iter().enumerate().map(|(index, &height)| Item { content: index, height, can_break_before: true, must_break_before: false }).collect()
    }

    fn contents(columns: &[Vec<Item<usize>>]) -> Vec<Vec<usize>> {
        columns.iter().map(|column| column.iter().map(|item| item.content).collect()).collect()
    }

    #[test]
    fn items_are_shared_between_columns_in_order() {
        let columns = into_columns(items(&[10.0; 6]), 2);
        assert_eq!(contents(&columns), [vec![0, 1, 2], vec![3, 4, 5]]);
        let columns = into_columns(items(&[10.0; 6]), 3);
        assert_eq!(contents(&columns), [vec![0, 1], vec![2, 3], vec![4, 5]]);
    }

    #[test]
    fn one_column_holds_everything() {
        assert_eq!(contents(&into_columns(items(&[10.0; 4]), 1)), [vec![0, 1, 2, 3]]);
        assert_eq!(contents(&into_columns(Vec::<Item<usize>>::new(), 3)), [Vec::<usize>::new()]);
    }

    #[test]
    fn a_column_does_not_end_where_it_may_not() {
        // Item 3 may not start a column, so the break moves to the next place that may.
        let mut list = items(&[10.0; 6]);
        list[3].can_break_before = false;
        let columns = into_columns(list, 2);
        assert_eq!(contents(&columns), [vec![0, 1, 2, 3], vec![4, 5]]);
    }

    #[test]
    fn a_column_break_is_obeyed() {
        let mut list = items(&[10.0; 6]);
        list[1].must_break_before = true;
        let columns = into_columns(list, 3);
        assert_eq!(columns[0].len(), 1);
        assert_eq!(columns.len(), 3);
    }

    #[test]
    fn there_are_never_more_columns_than_asked_for() {
        let mut list = items(&[10.0; 6]);
        for item in &mut list {
            item.must_break_before = true;
        }
        assert_eq!(into_columns(list, 2).len(), 2);
    }

    #[test]
    fn columns_are_limited_by_the_width() {
        assert_eq!(columns_that_fit(1000.0, 3, 300.0, 20.0), 3);
        assert_eq!(columns_that_fit(700.0, 3, 300.0, 20.0), 2);
        assert_eq!(columns_that_fit(200.0, 3, 300.0, 20.0), 1);
        assert_eq!(columns_that_fit(1000.0, 1, 300.0, 20.0), 1);
        assert_eq!(columns_that_fit(1000.0, 0, 300.0, 20.0), 1);
    }
}

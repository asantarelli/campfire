// This file is part of Campfire.
// Copyright (c) Pandula Péter 2017-2026.
// https://github.com/pandulapeter/campfire
//
// This Source Code Form is subject to the terms of the Mozilla Public License, v. 2.0.
// If a copy of the MPL was not distributed with this file, You can obtain one at
// https://mozilla.org/MPL/2.0/.

//! A song on the screen: chords over the words, spread over as many columns as the window holds.
//!
//! The song is turned into pieces of known height first (`build_items`), shared out between the columns by
//! [`crate::layout`], and only then painted, so that no widget has to be measured to know where the next one goes.

use chordpro::model::{Block, CommentStyle, Line, Section, SectionType, Song};
use chordpro::notation::{self, ChordNotation};
use eframe::egui::{self, Align2, Color32, FontId, Pos2, Rect, Stroke, Ui, Vec2};

use crate::layout::{self, ChordAt, Item};
use crate::theme::{self, SongColors};

/// The spacing between two columns, and the narrowest a column is allowed to be in units of the text size.
const COLUMN_SPACING: f32 = 36.0;
const MINIMUM_COLUMN_WIDTH_IN_TEXT_SIZES: f32 = 16.0;

/// What a song is drawn with.
pub struct ViewOptions<'a> {
    pub columns: usize,
    pub text_size: f32,
    pub notation: ChordNotation,
    pub accidental_signs: bool,
    pub show_chords: bool,
    pub colors: &'a SongColors,
}

/// One piece of a song laid out in a column.
#[derive(Debug, Clone, PartialEq)]
enum Drawable {
    Heading(String),
    /// A row of lyrics with the chords over it, named the way the reader reads them.
    /// A row of lyrics laid out by `layout::place_row`: its pieces of text, and its chords with the name they are shown under, the
    /// `x` they stand at and how wide they are.
    Lyrics { placed: layout::Placed, chords: Vec<(ChordAt, String, f32, f32)> },
    Monospace(String),
    Comment { text: String, style: CommentStyle },
    Spacer,
}

/// The heights the fonts give, which every piece of a song is measured in.
#[derive(Clone, Copy)]
struct Metrics {
    lyrics: f32,
    chords: f32,
    heading: f32,
    monospace: f32,
}

/// The name a chord is shown under in `notation`; annotations are shown as they are written.
pub fn displayed_chord_name(name: &str, is_annotation: bool, notation: ChordNotation, accidental_signs: bool) -> String {
    if is_annotation {
        return name.to_string();
    }
    let shown = notation::shown_name(name, notation);
    if accidental_signs {
        notation::with_accidental_signs(&shown)
    } else {
        shown
    }
}

fn section_heading(section: &Section, counters: &mut [u32; 3]) -> Option<String> {
    if let Some(label) = &section.label {
        return Some(label.clone());
    }
    if section.is_continuation {
        return None;
    }
    let (index, name) = match &section.section_type {
        SectionType::Verse => (Some(0), "Verso".to_string()),
        SectionType::Chorus => (Some(1), "Estribillo".to_string()),
        SectionType::Bridge => (Some(2), "Puente".to_string()),
        SectionType::Custom(name) => (None, capitalized(name)),
        SectionType::Paragraph => return None,
    };
    match index {
        Some(index) => {
            counters[index] += 1;
            // A chorus that comes back is the same chorus, so only the first of its kind says its number when it is alone.
            Some(if counters[index] == 1 && index != 0 { name } else { format!("{name} {}", counters[index]) })
        }
        None => Some(name),
    }
}

fn capitalized(name: &str) -> String {
    let mut chars = name.chars();
    chars.next().map(|first| first.to_uppercase().chain(chars).collect()).unwrap_or_default()
}

/// The pieces of `song`, each with its height, in reading order.
fn build_items(song: &Song, options: &ViewOptions, metrics: Metrics, column_width: f32, measure: &dyn Fn(&str, bool) -> f32) -> Vec<Item<Drawable>> {
    let mut items = Vec::new();
    let mut counters = [0u32; 3];
    let mut break_next = false;
    push_blocks(&song.blocks, options, metrics, column_width, measure, &mut items, &mut counters, &mut break_next);
    items
}

#[allow(clippy::too_many_arguments)]
fn push_blocks(
    blocks: &[Block],
    options: &ViewOptions,
    metrics: Metrics,
    column_width: f32,
    measure: &dyn Fn(&str, bool) -> f32,
    items: &mut Vec<Item<Drawable>>,
    counters: &mut [u32; 3],
    break_next: &mut bool,
) {
    let push = |items: &mut Vec<Item<Drawable>>, content: Drawable, height: f32, can_break_before: bool, break_next: &mut bool| {
        let must_break_before = std::mem::take(break_next);
        items.push(Item { content, height, can_break_before: can_break_before || must_break_before, must_break_before });
    };
    for block in blocks {
        match block {
            Block::Section(section) => {
                let mut can_break = true;
                if let Some(heading) = section_heading(section, counters) {
                    // Space before a heading, except at the very top of the page.
                    let height = metrics.heading + if items.is_empty() { 2.0 } else { 12.0 };
                    push(items, Drawable::Heading(heading), height, true, break_next);
                    can_break = false;
                } else if !items.is_empty() && !section.is_continuation {
                    push(items, Drawable::Spacer, metrics.lyrics * 0.8, true, break_next);
                    can_break = false;
                }
                for line in &section.lines {
                    match line {
                        Line::Lyrics(lyrics) => {
                            // Chords that are not shown take no room.
                            let bare;
                            let lyrics = if options.show_chords {
                                lyrics
                            } else {
                                bare = chordpro::model::Lyrics { text: lyrics.text.clone(), chords: Vec::new() };
                                &bare
                            };
                            let shown = |name: &str, is_annotation: bool| displayed_chord_name(name, is_annotation, options.notation, options.accidental_signs);
                            let gap = options.text_size * 0.35;
                            let rows = layout::wrap_lyrics(lyrics, column_width, &|text| measure(text, false), &|name, is_annotation| measure(&shown(name, is_annotation), true), gap);
                            for (index, row) in rows.into_iter().enumerate() {
                                let names: Vec<String> = row.chords.iter().map(|chord| shown(&chord.name, chord.is_annotation)).collect();
                                let widths: Vec<f32> = names.iter().map(|name| measure(name, true)).collect();
                                let placed = layout::place_row(&row.text, &row.chords, &widths, gap, &|text| measure(text, false));
                                let chords: Vec<(ChordAt, String, f32, f32)> =
                                    row.chords.into_iter().zip(names).zip(placed.chord_xs.iter().zip(widths)).map(|((chord, name), (x, width))| (chord, name, *x, width)).collect();
                                let height = metrics.lyrics + if chords.is_empty() { 0.0 } else { metrics.chords } + 2.0;
                                push(items, Drawable::Lyrics { placed, chords }, height, can_break && index == 0, break_next);
                                can_break = true;
                            }
                        }
                        Line::Tab(tab) => {
                            push(items, Drawable::Monospace(tab.text.clone()), metrics.monospace + 1.0, can_break, break_next);
                            can_break = true;
                        }
                        Line::Grid(grid) => {
                            push(items, Drawable::Monospace(grid_text(grid)), metrics.monospace + 3.0, can_break, break_next);
                            can_break = true;
                        }
                        Line::Blank => {
                            push(items, Drawable::Spacer, metrics.lyrics * 0.6, false, break_next);
                        }
                    }
                }
            }
            Block::ChorusRecall { label, blocks } => {
                let heading = label.clone().unwrap_or_else(|| "Estribillo".to_string());
                push(items, Drawable::Heading(heading), metrics.heading + if items.is_empty() { 2.0 } else { 12.0 }, true, break_next);
                let mut inner_counters = *counters;
                push_blocks(blocks, options, metrics, column_width, measure, items, &mut inner_counters, break_next);
            }
            Block::Comment(comment) => {
                let height = metrics.lyrics + if comment.style == CommentStyle::Box { 10.0 } else { 4.0 };
                push(items, Drawable::Comment { text: comment.text.clone(), style: comment.style }, height, true, break_next);
            }
            Block::Break => *break_next = true,
            Block::Transpose { .. } => {}
        }
    }
}

fn grid_text(grid: &chordpro::model::Grid) -> String {
    use chordpro::model::GridToken;
    grid.tokens
        .iter()
        .map(|token| match token {
            GridToken::Bar(text) | GridToken::Repeat(text) | GridToken::Text(text) => text.clone(),
            GridToken::Chord(name) => name.clone(),
            GridToken::Beat => ".".to_string(),
        })
        .collect::<Vec<_>>()
        .join(" ")
}

/// Where each chord of the song is on the screen, for the places that answer a tap on one.
pub struct ChordHit {
    pub rect: Rect,
    /// The chord as the file writes it.
    pub name: String,
}

/// Paints `song` in the width of `ui` and returns the chords it drew, with where.
pub fn show(ui: &mut Ui, song: &Song, options: &ViewOptions) -> Vec<ChordHit> {
    let width = ui.available_width();
    let columns = layout::columns_that_fit(width, options.columns, options.text_size * MINIMUM_COLUMN_WIDTH_IN_TEXT_SIZES, COLUMN_SPACING);
    let column_width = ((width - COLUMN_SPACING * (columns - 1) as f32) / columns as f32).max(80.0);
    let lyric_font = FontId::new(options.text_size, theme::view_family());
    let chord_font = FontId::new(options.text_size * 0.92, theme::bold());
    let heading_font = FontId::new(options.text_size * 0.9, theme::bold());
    let mono_font = FontId::monospace(options.text_size * 0.82);
    let context = ui.ctx().clone();
    let (metrics, measure) = {
        let metrics = Metrics {
            lyrics: context.fonts_mut(|fonts| fonts.row_height(&lyric_font)),
            chords: context.fonts_mut(|fonts| fonts.row_height(&chord_font)),
            heading: context.fonts_mut(|fonts| fonts.row_height(&heading_font)),
            monospace: context.fonts_mut(|fonts| fonts.row_height(&mono_font)),
        };
        let measure = |text: &str, is_chord: bool| {
            let font = if is_chord { chord_font.clone() } else { lyric_font.clone() };
            context.fonts_mut(|fonts| fonts.layout(text.to_string(), font, Color32::WHITE, f32::INFINITY).size().x)
        };
        (metrics, measure)
    };
    let items = build_items(song, options, metrics, column_width, &measure);
    let columns_of_items = layout::into_columns(items, columns);
    let heights: Vec<f32> = columns_of_items.iter().map(|column| column.iter().map(|item| item.height).sum()).collect();
    let total_height = heights.iter().cloned().fold(0.0, f32::max);
    let (rect, _) = ui.allocate_exact_size(Vec2::new(width, total_height + 24.0), egui::Sense::hover());
    let painter = ui.painter().clone();
    let colors = options.colors;
    let mut hits = Vec::new();
    for (index, column) in columns_of_items.iter().enumerate() {
        let left = rect.left() + index as f32 * (column_width + COLUMN_SPACING);
        let mut y = rect.top();
        for item in column {
            match &item.content {
                Drawable::Heading(text) => {
                    let top = y + item.height - metrics.heading;
                    painter.text(Pos2::new(left, top), Align2::LEFT_TOP, text, heading_font.clone(), colors.heading);
                }
                Drawable::Lyrics { placed, chords } => {
                    let has_chords = !chords.is_empty();
                    let text_top = y + if has_chords { metrics.chords } else { 0.0 };
                    // Each chord is over the piece of text that starts at it, so it is exactly over the word it was written before.
                    for (chord, name, x, width) in chords {
                        let color = if chord.is_annotation { colors.comment } else { colors.chord };
                        let position = Pos2::new(left + x, y);
                        painter.text(position, Align2::LEFT_TOP, name, chord_font.clone(), color);
                        if !chord.is_annotation {
                            hits.push(ChordHit { rect: Rect::from_min_size(position, Vec2::new(*width, metrics.chords)), name: chord.name.clone() });
                        }
                    }
                    for (x, piece) in &placed.segments {
                        painter.text(Pos2::new(left + x, text_top), Align2::LEFT_TOP, piece, lyric_font.clone(), colors.text);
                    }
                }
                Drawable::Monospace(text) => {
                    painter.text(Pos2::new(left, y), Align2::LEFT_TOP, text, mono_font.clone(), colors.text);
                }
                Drawable::Comment { text, style } => {
                    let color = if *style == CommentStyle::Box { colors.text } else { colors.comment };
                    if *style == CommentStyle::Box {
                        let box_rect = Rect::from_min_size(Pos2::new(left, y), Vec2::new(column_width, item.height - 2.0));
                        painter.rect_filled(box_rect, 4.0, colors.comment_box);
                        painter.rect_stroke(box_rect, 4.0, Stroke::new(1.0, colors.muted), egui::StrokeKind::Inside);
                        painter.text(Pos2::new(left + 6.0, y + 4.0), Align2::LEFT_TOP, text, lyric_font.clone(), color);
                    } else {
                        let font = if *style == CommentStyle::Italic { FontId::new(options.text_size, theme::bold()) } else { lyric_font.clone() };
                        painter.text(Pos2::new(left, y + 2.0), Align2::LEFT_TOP, text, font, color);
                    }
                }
                Drawable::Spacer => {}
            }
            y += item.height;
        }
    }
    hits
}

#[cfg(test)]
mod tests {
    use super::*;
    use chordpro::parser;

    fn options(colors: &SongColors) -> ViewOptions<'_> {
        ViewOptions { columns: 2, text_size: 18.0, notation: ChordNotation::Standard, accidental_signs: false, show_chords: true, colors }
    }

    fn items_of(text: &str, show_chords: bool) -> Vec<Item<Drawable>> {
        let colors = SongColors::of(true);
        let mut options = options(&colors);
        options.show_chords = show_chords;
        let song = parser::parse(text, ChordNotation::Standard);
        let metrics = Metrics { lyrics: 20.0, chords: 18.0, heading: 18.0, monospace: 16.0 };
        build_items(&song, &options, metrics, 200.0, &|text, _| text.chars().count() as f32 * 8.0)
    }

    #[test]
    fn a_line_with_chords_is_taller_than_one_without() {
        let items = items_of("[Am]Hello\nplain\n", true);
        assert_eq!(items[0].height, 20.0 + 18.0 + 2.0);
        assert_eq!(items[1].height, 20.0 + 2.0);
        let hidden = items_of("[Am]Hello\n", false);
        assert_eq!(hidden[0].height, 20.0 + 2.0);
    }

    #[test]
    fn sections_are_headed_by_their_label_or_their_kind() {
        let items = items_of("{sov}\n[G]one\n{eov}\n{sov: Primera}\n[C]two\n{eov}\n{soc}\n[D]three\n{eoc}\n", true);
        let headings: Vec<&str> = items.iter().filter_map(|item| if let Drawable::Heading(text) = &item.content { Some(text.as_str()) } else { None }).collect();
        assert_eq!(headings, ["Verso 1", "Primera", "Estribillo"]);
    }

    #[test]
    fn a_column_break_forces_the_next_piece_into_a_new_column() {
        let items = items_of("one\n{column_break}\ntwo\n", true);
        assert!(items.iter().any(|item| item.must_break_before));
    }

    #[test]
    fn a_recalled_chorus_is_headed_and_repeated() {
        let items = items_of("{soc}\n[G]la la\n{eoc}\n{sov}\nverse\n{eov}\n{chorus}\n", true);
        let lyrics = items.iter().filter(|item| matches!(item.content, Drawable::Lyrics { .. })).count();
        assert_eq!(lyrics, 3);
    }

    #[test]
    fn chords_are_named_in_the_readers_notation() {
        assert_eq!(displayed_chord_name("Bb", false, ChordNotation::German, false), "B");
        assert_eq!(displayed_chord_name("B", false, ChordNotation::German, true), "H");
        assert_eq!(displayed_chord_name("Bbm7", false, ChordNotation::Standard, true), "B♭m7");
        assert_eq!(displayed_chord_name("Am", false, ChordNotation::Latin, false), "Lam");
        assert_eq!(displayed_chord_name("Bb", true, ChordNotation::Latin, true), "Bb");
    }
}

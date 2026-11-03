// This file is part of Campfire.
// Copyright (c) Pandula Péter 2017-2026.
// https://github.com/pandulapeter/campfire
//
// This Source Code Form is subject to the terms of the Mozilla Public License, v. 2.0.
// If a copy of the MPL was not distributed with this file, You can obtain one at
// https://mozilla.org/MPL/2.0/.

//! A chord diagram: the strings upright, the frets across, a dot where a finger goes.

use chordpro::spelling;
use chordpro::voicings::{Voicing, FRETS_SHOWN};
use eframe::egui::{self, Align2, Color32, CursorIcon, FontId, Pos2, Rect, Sense, Stroke, Ui, Vec2};

use crate::settings::DotLabels;
use crate::theme::{self, SongColors};

/// What a diagram needs besides the shape to say more of it: what its dots write, whether they are colored by the function of
/// their note, and the strings it is drawn on (as MIDI notes) and the chord's root where it has one, which the notes are read from.
pub struct Look<'a> {
    pub labels: DotLabels,
    pub is_colored: bool,
    pub strings: &'a [i32],
    pub root: Option<i32>,
}

impl Look<'_> {
    /// The note the string sounds at the fret, as a pitch class.
    fn pitch_class(&self, string: usize, fret: u8) -> Option<i32> {
        self.strings.get(string).map(|midi| (midi + fret as i32).rem_euclid(12))
    }

    /// What a dot or an open string says: its finger, its note or its place in the chord.
    fn label(&self, string: usize, fret: u8, finger: Option<u8>) -> Option<String> {
        match self.labels {
            DotLabels::None => None,
            DotLabels::Fingers => finger.filter(|&finger| finger > 0).map(|finger| finger.to_string()),
            DotLabels::Notes => self.pitch_class(string, fret).map(|pitch| spelling::note_label(pitch, self.root)),
            DotLabels::Degrees => self.pitch_class(string, fret).and_then(|pitch| spelling::degree_label(pitch, self.root)).map(str::to_string),
        }
    }

    /// The color of the note by its function in the chord, where the diagram colors them and the note has one.
    fn function_color(&self, string: usize, fret: u8) -> Option<Color32> {
        if !self.is_colored {
            return None;
        }
        let pitch = self.pitch_class(string, fret)?;
        match (pitch - self.root?).rem_euclid(12) {
            3 | 4 => Some(THIRD_COLOR),
            6..=8 => Some(FIFTH_COLOR),
            9..=11 => Some(SEVENTH_COLOR),
            _ => None,
        }
    }

    /// The frequency of the note the string sounds at the fret, for a tap on it to be heard.
    fn frequency(&self, string: usize, fret: u8) -> Option<f32> {
        self.strings.get(string).map(|midi| 440.0 * 2f32.powf(((midi + fret as i32) as f32 - 69.0) / 12.0))
    }
}

const THIRD_COLOR: Color32 = Color32::from_rgb(0x42, 0xA5, 0xF5);
const FIFTH_COLOR: Color32 = Color32::from_rgb(0x66, 0xBB, 0x6A);
const SEVENTH_COLOR: Color32 = Color32::from_rgb(0xAB, 0x47, 0xBC);

/// The size of one diagram at a text size of 18: wide enough for six strings, tall enough for five frets and its name.
pub fn diagram_size(scale: f32) -> Vec2 {
    Vec2::new(96.0 * scale, 132.0 * scale)
}

/// Draws `voicing` of the chord called `name` in a rectangle of `diagram_size(scale)`, and answers the frequency of the string the
/// player pressed on it, for it to be heard.
pub fn show(ui: &mut Ui, name: &str, voicing: &Voicing, colors: &SongColors, scale: f32, look: &Look) -> Option<f32> {
    let can_sound = !look.strings.is_empty();
    let (rect, response) = ui.allocate_exact_size(diagram_size(scale), if can_sound { Sense::click() } else { Sense::hover() });
    draw(ui, rect, name, voicing, colors, scale, look);
    if !can_sound {
        return None;
    }
    let response = response.on_hover_cursor(CursorIcon::PointingHand).on_hover_text("Tocá una cuerda para oír su nota");
    let position = response.interact_pointer_pos().filter(|_| response.clicked())?;
    let strings = voicing.frets.len();
    let (left, right) = (rect.left() + 16.0 * scale, rect.right() - 12.0 * scale);
    let gap = (right - left) / (strings.saturating_sub(1)).max(1) as f32;
    let string = (((position.x - left) / gap).round().max(0.0) as usize).min(strings.saturating_sub(1));
    let fret = voicing.frets.get(string).copied().flatten()?;
    look.frequency(string, fret)
}

pub fn draw(ui: &Ui, rect: Rect, name: &str, voicing: &Voicing, colors: &SongColors, scale: f32, look: &Look) {
    let painter = ui.painter();
    let strings = voicing.frets.len();
    let title_height = 24.0 * scale;
    let marker_height = 14.0 * scale;
    let left = rect.left() + 16.0 * scale;
    let right = rect.right() - 12.0 * scale;
    let top = rect.top() + title_height + marker_height;
    let bottom = rect.bottom() - 6.0 * scale;
    let string_gap = (right - left) / (strings.saturating_sub(1)).max(1) as f32;
    let fret_gap = (bottom - top) / FRETS_SHOWN as f32;
    painter.text(Pos2::new(rect.center().x, rect.top()), Align2::CENTER_TOP, name, FontId::new(16.0 * scale, theme::bold()), colors.chord);
    let base = voicing.base_fret();
    // A number inside the dot reads against it: dark on a light dot, light on a dark one.
    let finger_color = if colors.text.r() > 0x80 { Color32::from_gray(0x18) } else { Color32::from_gray(0xF2) };
    let line = Stroke::new(1.0, colors.muted);
    for fret in 0..=FRETS_SHOWN {
        let y = top + fret as f32 * fret_gap;
        // The nut is thick where the diagram starts at the top of the neck.
        let stroke = if fret == 0 && base == 1 { Stroke::new(3.0 * scale, colors.text) } else { line };
        painter.line_segment([Pos2::new(left, y), Pos2::new(right, y)], stroke);
    }
    for string in 0..strings {
        let x = left + string as f32 * string_gap;
        painter.line_segment([Pos2::new(x, top), Pos2::new(x, bottom)], line);
    }
    if base > 1 {
        painter.text(Pos2::new(rect.left() + 1.0, top + fret_gap * 0.5), Align2::LEFT_CENTER, format!("{base}"), FontId::proportional(11.0 * scale), colors.muted);
    }
    // A barre: the lowest fret held on two strings or more, drawn as one bar under its dots.
    let lowest = voicing.lowest_pressed();
    if let Some(lowest) = lowest {
        let held: Vec<usize> = voicing.frets.iter().enumerate().filter(|(_, fret)| **fret == Some(lowest)).map(|(string, _)| string).collect();
        if held.len() >= 2 && (held[0]..=held[held.len() - 1]).all(|string| voicing.frets[string].is_some_and(|fret| fret >= lowest)) {
            let y = top + ((lowest - base) as f32 + 0.5) * fret_gap;
            let a = Pos2::new(left + held[0] as f32 * string_gap, y);
            let b = Pos2::new(left + held[held.len() - 1] as f32 * string_gap, y);
            painter.line_segment([a, b], Stroke::new(fret_gap * 0.55, colors.text));
            if matches!(look.labels, DotLabels::Notes | DotLabels::Degrees) || look.is_colored {
                for string in held[0]..=held[held.len() - 1] {
                    if voicing.frets[string] != Some(lowest) {
                        continue;
                    }
                    let center = Pos2::new(left + string as f32 * string_gap, y);
                    if let Some(color) = look.function_color(string, lowest) {
                        painter.circle_filled(center, fret_gap * 0.3, color);
                    }
                    if let Some(label) = look.label(string, lowest, None).filter(|_| matches!(look.labels, DotLabels::Notes | DotLabels::Degrees)) {
                        painter.text(center, Align2::CENTER_CENTER, label, FontId::new(fret_gap * 0.5, theme::bold()), finger_color);
                    }
                }
            }
        }
    }
    for (string, fret) in voicing.frets.iter().enumerate() {
        let x = left + string as f32 * string_gap;
        match fret {
            None => {
                painter.text(Pos2::new(x, top - marker_height * 0.5), Align2::CENTER_CENTER, "×", FontId::proportional(13.0 * scale), colors.muted);
            }
            Some(0) => {
                let center = Pos2::new(x, top - marker_height * 0.5);
                match look.label(string, 0, None).filter(|_| matches!(look.labels, DotLabels::Notes | DotLabels::Degrees)) {
                    Some(label) => {
                        let color = look.function_color(string, 0).unwrap_or(colors.text);
                        painter.text(center, Align2::CENTER_CENTER, label, FontId::new(11.0 * scale, theme::bold()), color);
                    }
                    None => {
                        painter.circle_stroke(center, 4.0 * scale, Stroke::new(1.3, colors.text));
                    }
                }
            }
            Some(fret) => {
                let row = (*fret as i32 - base as i32).clamp(0, FRETS_SHOWN as i32 - 1) as f32;
                let y = top + (row + 0.5) * fret_gap;
                let finger = voicing.fingers.as_ref().and_then(|fingers| fingers.get(string)).copied();
                // The dot is as big as the room lets it where it carries a note.
                let says_note = matches!(look.labels, DotLabels::Notes | DotLabels::Degrees);
                let radius = if says_note { (fret_gap * 0.44).min(string_gap * 0.5) } else { (fret_gap * 0.34).min(string_gap * 0.42) };
                painter.circle_filled(Pos2::new(x, y), radius, look.function_color(string, *fret).unwrap_or(colors.text));
                // Without a label asked for, the fingers a song's own definition names are shown, as they always were.
                let label = if look.labels == DotLabels::None { finger.filter(|&finger| finger > 0).map(|finger| finger.to_string()) } else { look.label(string, *fret, finger) };
                if let Some(label) = label {
                    let size = if label.chars().count() > 1 { radius * 1.05 } else { radius * 1.35 };
                    painter.text(Pos2::new(x, y), Align2::CENTER_CENTER, label, FontId::new(size, theme::bold()), finger_color);
                }
            }
        }
    }
}

/// The colors a dot on the diagram stands out in, for a place that draws on a card.
pub fn card_color(is_dark: bool) -> Color32 {
    if is_dark {
        Color32::from_gray(0x24)
    } else {
        Color32::from_gray(0xF2)
    }
}

/// A card around a diagram, so that several of them read as a row of separate things.
pub fn card<R>(ui: &mut Ui, is_dark: bool, add_contents: impl FnOnce(&mut Ui) -> R) -> R {
    egui::Frame::new().fill(card_color(is_dark)).corner_radius(8.0).inner_margin(egui::Margin::same(6)).show(ui, add_contents).inner
}

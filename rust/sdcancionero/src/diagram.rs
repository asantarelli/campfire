// This file is part of Campfire.
// Copyright (c) Pandula Péter 2017-2026.
// https://github.com/pandulapeter/campfire
//
// This Source Code Form is subject to the terms of the Mozilla Public License, v. 2.0.
// If a copy of the MPL was not distributed with this file, You can obtain one at
// https://mozilla.org/MPL/2.0/.

//! A chord diagram: the strings upright, the frets across, a dot where a finger goes.

use chordpro::voicings::{Voicing, FRETS_SHOWN};
use eframe::egui::{self, Align2, Color32, FontId, Pos2, Rect, Sense, Stroke, Ui, Vec2};

use crate::theme::{self, SongColors};

/// The size of one diagram at a text size of 18: wide enough for six strings, tall enough for five frets and its name.
pub fn diagram_size(scale: f32) -> Vec2 {
    Vec2::new(96.0 * scale, 132.0 * scale)
}

/// Draws `voicing` of the chord called `name` in a rectangle of `diagram_size(scale)`.
pub fn show(ui: &mut Ui, name: &str, voicing: &Voicing, colors: &SongColors, scale: f32) {
    let (rect, _) = ui.allocate_exact_size(diagram_size(scale), Sense::hover());
    draw(ui, rect, name, voicing, colors, scale);
}

pub fn draw(ui: &Ui, rect: Rect, name: &str, voicing: &Voicing, colors: &SongColors, scale: f32) {
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
        }
    }
    // A number inside the dot reads against it: dark on a light dot, light on a dark one.
    let finger_color = if colors.text.r() > 0x80 { Color32::from_gray(0x18) } else { Color32::from_gray(0xF2) };
    for (string, fret) in voicing.frets.iter().enumerate() {
        let x = left + string as f32 * string_gap;
        match fret {
            None => {
                painter.text(Pos2::new(x, top - marker_height * 0.5), Align2::CENTER_CENTER, "×", FontId::proportional(13.0 * scale), colors.muted);
            }
            Some(0) => {
                painter.circle_stroke(Pos2::new(x, top - marker_height * 0.5), 4.0 * scale, Stroke::new(1.3, colors.text));
            }
            Some(fret) => {
                let row = (*fret as i32 - base as i32).clamp(0, FRETS_SHOWN as i32 - 1) as f32;
                let y = top + (row + 0.5) * fret_gap;
                let radius = (fret_gap * 0.34).min(string_gap * 0.42);
                painter.circle_filled(Pos2::new(x, y), radius, colors.text);
                if let Some(finger) = voicing.fingers.as_ref().and_then(|fingers| fingers.get(string)).filter(|&&finger| finger > 0) {
                    painter.text(Pos2::new(x, y), Align2::CENTER_CENTER, finger.to_string(), FontId::proportional(radius * 1.35), finger_color);
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
pub fn card(ui: &mut Ui, is_dark: bool, add_contents: impl FnOnce(&mut Ui)) {
    egui::Frame::new().fill(card_color(is_dark)).corner_radius(8.0).inner_margin(egui::Margin::same(6)).show(ui, add_contents);
}

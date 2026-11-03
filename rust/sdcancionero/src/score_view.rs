// This file is part of Campfire.
// Copyright (c) Pandula Péter 2017-2026.
// https://github.com/pandulapeter/campfire
//
// This Source Code Form is subject to the terms of the Mozilla Public License, v. 2.0.
// If a copy of the MPL was not distributed with this file, You can obtain one at
// https://mozilla.org/MPL/2.0/.

//! A Guitar Pro score on the screen: its title, the track to read, and the staff and the tablature of it, set to the width of the window.

use eframe::egui::{self, Align2, Color32, FontId, Pos2, RichText, ScrollArea, Shape, Stroke, Ui, Vec2};
use guitarpro::draw::{Anchor, Draw, TextStyle, Tone};
use guitarpro::layout::{self, Layout};
use guitarpro::model::Score;

use crate::theme::{self, SongColors};

/// What a score asks of the window around it.
#[derive(Debug, PartialEq, Eq)]
pub enum ScoreAction {
    None,
    Close,
}

/// How big a staff space is on the screen, in pixels.
const MIN_SPACE: f32 = 6.0;
const MAX_SPACE: f32 = 20.0;
const DEFAULT_SPACE: f32 = 10.0;

#[derive(Debug, Clone, Copy, PartialEq)]
struct Key {
    track: usize,
    width: u32,
    notation: bool,
    tab: bool,
    rhythm: bool,
}

pub struct ScoreView {
    score: Score,
    title: String,
    artist: String,
    track: usize,
    notation: bool,
    tab: bool,
    rhythm: bool,
    /// How many pixels a space between two lines of a staff takes.
    space: f32,
    laid_out: Option<(Key, Layout)>,
}

impl ScoreView {
    pub fn new(score: Score, title: String, artist: String) -> ScoreView {
        // The first track that has strings to play: a drum track is not read from a staff.
        let track = score.tracks.iter().position(|track| !track.is_percussion).unwrap_or(0);
        ScoreView { score, title, artist, track, notation: true, tab: true, rhythm: true, space: DEFAULT_SPACE, laid_out: None }
    }

    /// The toolbar and the score. Answers what the window should do next.
    pub fn show(&mut self, ui: &mut Ui, colors: &SongColors) -> ScoreAction {
        let mut action = ScoreAction::None;
        ui.horizontal_wrapped(|ui| {
            if ui.button("← Canciones").clicked() {
                action = ScoreAction::Close;
            }
            ui.separator();
            let names: Vec<String> = self.score.tracks.iter().enumerate().map(|(index, track)| format!("{}. {}", index + 1, if track.name.trim().is_empty() { "Pista" } else { track.name.trim() })).collect();
            let selected = names.get(self.track).cloned().unwrap_or_default();
            egui::ComboBox::from_id_salt("score_track").selected_text(selected).show_ui(ui, |ui| {
                for (index, name) in names.iter().enumerate() {
                    ui.selectable_value(&mut self.track, index, name);
                }
            });
            ui.separator();
            ui.checkbox(&mut self.notation, "Partitura");
            ui.checkbox(&mut self.tab, "Tablatura");
            ui.add_enabled(self.tab, egui::Checkbox::new(&mut self.rhythm, "Ritmo"));
            ui.separator();
            ui.label("Tamaño");
            ui.add(egui::Slider::new(&mut self.space, MIN_SPACE..=MAX_SPACE).show_value(false));
        });
        // One of them is always shown.
        if !self.notation && !self.tab {
            self.notation = true;
        }
        ui.separator();
        ui.label(RichText::new(self.title.clone()).size(22.0).strong());
        let details: Vec<String> = [self.artist.trim(), self.score.album.trim()].iter().filter(|text| !text.is_empty()).map(|text| text.to_string()).collect();
        if !details.is_empty() || self.score.tempo > 0 {
            let tempo = if self.score.tempo > 0 { format!("♩ = {}", self.score.tempo) } else { String::new() };
            let line = [details.join(" · "), tempo].into_iter().filter(|part| !part.is_empty()).collect::<Vec<_>>().join("   ");
            ui.label(RichText::new(line).color(colors.muted));
        }
        ui.add_space(6.0);
        let space = self.space;
        let width_units = ((ui.available_width() - 24.0) / space).max(30.0);
        let key = Key { track: self.track, width: width_units.round() as u32, notation: self.notation, tab: self.tab, rhythm: self.rhythm };
        if self.laid_out.as_ref().map(|(known, _)| *known) != Some(key) {
            let options = layout::Options { width: key.width as f32, notation: key.notation, tab: key.tab, rhythm: key.rhythm };
            self.laid_out = Some((key, layout::layout(&self.score, self.track, &options)));
        }
        let Some((_, page)) = &self.laid_out else {
            return action;
        };
        let panel = ui.visuals().panel_fill;
        ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| {
            let size = Vec2::new(page.width * space + 12.0, page.height * space + 24.0);
            let (rect, _) = ui.allocate_exact_size(size, egui::Sense::hover());
            let origin = Pos2::new(rect.left() + 6.0, rect.top() + 6.0);
            let clip = ui.clip_rect();
            let painter = ui.painter_at(rect);
            for system in &page.systems {
                let (top, bottom) = (origin.y + system.top * space, origin.y + (system.top + system.height) * space);
                if bottom < clip.top() - 20.0 || top > clip.bottom() + 20.0 {
                    continue;
                }
                for draw in &system.draws {
                    paint(&painter, draw, origin, space, colors, panel);
                }
            }
        });
        action
    }
}

fn color_of(tone: Tone, colors: &SongColors) -> Color32 {
    match tone {
        Tone::Ink => colors.text,
        Tone::Muted => colors.muted,
        Tone::Accent => colors.chord,
    }
}

fn at(origin: Pos2, space: f32, x: f32, y: f32) -> Pos2 {
    Pos2::new(origin.x + x * space, origin.y + y * space)
}

fn paint(painter: &egui::Painter, draw: &Draw, origin: Pos2, space: f32, colors: &SongColors, panel: Color32) {
    match draw {
        Draw::Line { x1, y1, x2, y2, width, tone } => {
            painter.line_segment([at(origin, space, *x1, *y1), at(origin, space, *x2, *y2)], Stroke::new((width * space).max(1.0), color_of(*tone, colors)));
        }
        Draw::Polyline { points, width, tone } => {
            let points: Vec<Pos2> = points.iter().map(|(x, y)| at(origin, space, *x, *y)).collect();
            painter.add(Shape::line(points, Stroke::new((width * space).max(1.0), color_of(*tone, colors))));
        }
        Draw::Polygon { points, tone } => {
            let points: Vec<Pos2> = points.iter().map(|(x, y)| at(origin, space, *x, *y)).collect();
            painter.add(Shape::convex_polygon(points, color_of(*tone, colors), Stroke::NONE));
        }
        Draw::Ellipse { x, y, rx, ry, angle, filled, width, tone } => {
            let (sin, cos) = angle.sin_cos();
            let points: Vec<Pos2> = (0..28)
                .map(|step| {
                    let t = step as f32 / 28.0 * std::f32::consts::TAU;
                    let (px, py) = (rx * t.cos(), ry * t.sin());
                    at(origin, space, x + px * cos - py * sin, y + px * sin + py * cos)
                })
                .collect();
            let color = color_of(*tone, colors);
            if *filled {
                painter.add(Shape::convex_polygon(points, color, Stroke::NONE));
            } else {
                painter.add(Shape::closed_line(points, Stroke::new((width * space).max(1.0), color)));
            }
        }
        Draw::Text { x, y, text, size, anchor, style, masks, tone } => {
            let font = FontId::new((size * space).max(7.0), if *style == TextStyle::Bold { theme::bold() } else { egui::FontFamily::Proportional });
            let align = match anchor {
                Anchor::Left => Align2::LEFT_CENTER,
                Anchor::Center => Align2::CENTER_CENTER,
                Anchor::Right => Align2::RIGHT_CENTER,
            };
            let position = at(origin, space, *x, *y);
            let color = color_of(*tone, colors);
            if *masks {
                let galley = painter.layout_no_wrap(text.clone(), font.clone(), color);
                let rect = align.anchor_size(position, galley.size());
                painter.rect_filled(rect.expand(1.5), 2.0, panel);
            }
            painter.text(position, align, text, font, color);
        }
    }
}

/// The score the file holds, or what is wrong with it said for the player.
pub fn open(bytes: &[u8]) -> Result<Score, String> {
    guitarpro::read(bytes).map_err(|error| error.to_string())
}


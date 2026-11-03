// This file is part of Campfire.
// Copyright (c) Pandula Péter 2017-2026.
// https://github.com/pandulapeter/campfire
//
// This Source Code Form is subject to the terms of the Mozilla Public License, v. 2.0.
// If a copy of the MPL was not distributed with this file, You can obtain one at
// https://mozilla.org/MPL/2.0/.

//! The signs of music written down, drawn with strokes and ellipses rather than set in a font, so that a score looks the same wherever
//! it is painted and nothing has to be bundled for it. Every shape is in units of the space between two lines of a staff, with the staff's
//! top line at y = 0 and its bottom one at y = 4.

use crate::draw::{bezier, Draw};

/// The thickness of the thin lines of a staff, a stem and a bar line.
pub const LINE: f32 = 0.1;
/// How thick a stem is.
pub const STEM: f32 = 0.12;
/// How thick a beam is.
pub const BEAM: f32 = 0.5;
/// Half the width of a notehead.
pub const HEAD_RX: f32 = 0.62;

/// A notehead at (`x`, `y`): filled for a quarter note and shorter, hollow for a half note and a whole one.
pub fn notehead(x: f32, y: f32, value: u8) -> Draw {
    match value {
        1 => Draw::Ellipse { x, y, rx: HEAD_RX + 0.1, ry: 0.5, angle: 0.0, filled: false, width: 0.2, tone: crate::draw::Tone::Ink },
        2 => Draw::Ellipse { x, y, rx: HEAD_RX - 0.05, ry: 0.42, angle: -0.45, filled: false, width: 0.14, tone: crate::draw::Tone::Ink },
        _ => Draw::Ellipse { x, y, rx: HEAD_RX, ry: 0.5, angle: -0.4, filled: true, width: 0.0, tone: crate::draw::Tone::Ink },
    }
}

/// The flags of a stem that ends at (`x`, `tip`): `count` of them, hanging from the stem's end towards its notes.
pub fn flags(x: f32, tip: f32, is_up: bool, count: u8) -> Vec<Draw> {
    let sign = if is_up { 1.0 } else { -1.0 };
    (0..count)
        .map(|index| {
            let start = tip + sign * index as f32 * 0.85;
            Draw::polyline(bezier((x, start), (x + 0.1, start + sign * 0.9), (x + 1.2, start + sign * 1.3), (x + 0.75, start + sign * 2.5)), 0.26)
        })
        .collect()
}

/// A rest of `value` (1 whole, 2 half, 4 quarter...) at `x`, centered on the staff.
pub fn rest(x: f32, value: u8) -> Vec<Draw> {
    match value {
        1 => vec![Draw::polygon(vec![(x - 0.6, 1.0), (x + 0.6, 1.0), (x + 0.6, 1.5), (x - 0.6, 1.5)])],
        2 => vec![Draw::polygon(vec![(x - 0.6, 1.5), (x + 0.6, 1.5), (x + 0.6, 2.0), (x - 0.6, 2.0)])],
        4 => vec![Draw::polyline(vec![(x - 0.25, 0.65), (x + 0.35, 1.2), (x - 0.35, 1.9), (x + 0.35, 2.6), (x - 0.1, 3.1), (x + 0.2, 3.5)], 0.3)],
        _ => {
            let count = match value {
                8 => 1,
                16 => 2,
                32 => 3,
                _ => 4,
            };
            let mut draws = Vec::new();
            // The stem slants from the top right to the bottom left, and each flag is a dot with a hook to the stem.
            let top = 1.0 - (count as f32 - 1.0) * 0.45;
            let bottom = top + 1.5 + (count as f32 - 1.0) * 0.9;
            draws.push(Draw::line(x + 0.45, top, x - 0.2, bottom, 0.14));
            for index in 0..count {
                let y = top + 0.3 + index as f32 * 0.9;
                let xs = x + 0.45 - (y - top) * 0.43 / 1.0 * 0.5;
                draws.push(Draw::dot(xs - 0.3, y + 0.25, 0.26));
                draws.push(Draw::polyline(bezier((xs - 0.3, y + 0.3), (xs, y + 0.25), (xs + 0.2, y + 0.05), (xs + 0.4, y - 0.15)), 0.14));
            }
            draws
        }
    }
}

/// A sharp centered at (`x`, `y`).
pub fn sharp(x: f32, y: f32) -> Vec<Draw> {
    vec![
        Draw::line(x - 0.2, y - 1.0, x - 0.2, y + 1.1, 0.1),
        Draw::line(x + 0.2, y - 1.1, x + 0.2, y + 1.0, 0.1),
        Draw::line(x - 0.5, y + 0.15, x + 0.5, y - 0.25, 0.26),
        Draw::line(x - 0.5, y + 0.7, x + 0.5, y + 0.3, 0.26),
    ]
}

/// A flat centered at (`x`, `y`), its belly on the line the note sits on.
pub fn flat(x: f32, y: f32) -> Vec<Draw> {
    vec![
        Draw::line(x - 0.25, y - 1.5, x - 0.25, y + 0.5, 0.1),
        Draw::polyline(bezier((x - 0.25, y + 0.5), (x + 0.2, y + 0.15), (x + 0.65, y - 0.15), (x + 0.5, y - 0.45)), 0.2)
            .clone_with_more(bezier((x + 0.5, y - 0.45), (x + 0.4, y - 0.8), (x + 0.1, y - 0.8), (x - 0.25, y - 0.45))),
    ]
}

/// A natural centered at (`x`, `y`).
pub fn natural(x: f32, y: f32) -> Vec<Draw> {
    vec![
        Draw::line(x - 0.25, y - 1.2, x - 0.25, y + 0.55, 0.1),
        Draw::line(x + 0.25, y - 0.55, x + 0.25, y + 1.2, 0.1),
        Draw::line(x - 0.25, y - 0.1, x + 0.25, y - 0.4, 0.24),
        Draw::line(x - 0.25, y + 0.45, x + 0.25, y + 0.15, 0.24),
    ]
}

/// The treble clef, its bowl around the second line from the bottom, which is where the G is: the clef a guitar is written in.
pub fn treble_clef(x: f32) -> Vec<Draw> {
    // One stroke: from the tail at the bottom up the spine, round the top loop, down across the spine and round the bowl.
    let path: [(f32, f32); 26] = [
        (0.6, 5.95), (0.95, 6.1), (1.25, 5.85), (1.27, 5.3), (1.22, 4.2), (1.25, 3.0), (1.3, 1.6), (1.36, 0.3), (1.46, -0.9), (1.56, -1.7), (1.36, -2.05),
        (0.95, -1.7), (0.6, -0.8), (0.55, 0.3), (0.75, 1.5), (1.2, 2.4), (1.75, 3.1), (2.05, 3.8), (1.85, 4.5), (1.3, 4.8), (0.75, 4.5), (0.6, 3.9), (0.85, 3.35),
        (1.35, 3.3), (1.55, 3.7), (1.3, 4.0),
    ];
    let points: Vec<(f32, f32)> = crate::draw::smooth(&path).into_iter().map(|(px, py)| (x + px, py)).collect();
    vec![Draw::polyline(points, 0.17), Draw::dot(x + 0.55, 5.85, 0.2)]
}

/// A repeat's two dots, on the spaces either side of the middle line, at `x`.
pub fn repeat_dots(x: f32) -> Vec<Draw> {
    vec![Draw::dot(x, 1.5, 0.2), Draw::dot(x, 2.5, 0.2)]
}

/// The sharps or the flats of a key signature, as many as `fifths` says (positive for sharps, negative for flats) from `x`, spaced `gap`
/// apart: each on the line or the space it always stands on.
pub fn key_signature(x: f32, fifths: i32, gap: f32) -> Vec<Draw> {
    // The staff positions in half spaces below the top line, in the order the signs are added.
    const SHARPS: [f32; 7] = [0.0, 1.5, -0.5, 1.0, 2.5, 0.5, 2.0];
    const FLATS: [f32; 7] = [2.0, 0.5, 2.5, 1.0, 3.0, 1.5, 3.5];
    let count = fifths.unsigned_abs().min(7) as usize;
    let mut draws = Vec::new();
    for index in 0..count {
        let sign_x = x + index as f32 * gap;
        if fifths > 0 {
            draws.extend(sharp(sign_x, SHARPS[index]));
        } else {
            draws.extend(flat(sign_x, FLATS[index]));
        }
    }
    draws
}

impl Draw {
    /// A polyline with the points of another polyline after its own, which is how a shape of one stroke made of two curves is built.
    fn clone_with_more(self, more: Vec<(f32, f32)>) -> Draw {
        match self {
            Draw::Polyline { mut points, width, tone } => {
                points.extend(more);
                Draw::Polyline { points, width, tone }
            }
            other => other,
        }
    }
}

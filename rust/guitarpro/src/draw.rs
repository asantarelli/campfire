// This file is part of Campfire.
// Copyright (c) Pandula Péter 2017-2026.
// https://github.com/pandulapeter/campfire
//
// This Source Code Form is subject to the terms of the Mozilla Public License, v. 2.0.
// If a copy of the MPL was not distributed with this file, You can obtain one at
// https://mozilla.org/MPL/2.0/.

//! What a laid out score is drawn with: lines, curves, ellipses and texts, in units of the space between two lines of a staff, so that
//! whoever paints it (a window, a page) only has to scale it. Nothing in it names a color or a font: a [`Tone`] says how much a mark
//! matters and the painter picks what that looks like.

/// How much a mark stands out.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tone {
    /// The score itself: notes, lines, numbers.
    Ink,
    /// What stands beside it: measure numbers, the names of the strings.
    Muted,
    /// What the reader looks for: markers, chords, tempo.
    Accent,
}

/// Where a text is anchored on the point it is given.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Anchor {
    Left,
    Center,
    Right,
}

/// How a text is set.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextStyle {
    Normal,
    Bold,
    Italic,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Draw {
    /// A straight line of a thickness.
    Line { x1: f32, y1: f32, x2: f32, y2: f32, width: f32, tone: Tone },
    /// A line through points, drawn smooth enough for a curve to look like one.
    Polyline { points: Vec<(f32, f32)>, width: f32, tone: Tone },
    /// A filled shape with no hollow and no dent.
    Polygon { points: Vec<(f32, f32)>, tone: Tone },
    /// An ellipse turned by `angle` (radians), filled or outlined.
    Ellipse { x: f32, y: f32, rx: f32, ry: f32, angle: f32, filled: bool, width: f32, tone: Tone },
    /// A text centered on `y`, `size` units tall, over a patch of the background where it is `masks` the lines behind it (the number on a
    /// string of tablature).
    Text { x: f32, y: f32, text: String, size: f32, anchor: Anchor, style: TextStyle, masks: bool, tone: Tone },
}

impl Draw {
    pub fn line(x1: f32, y1: f32, x2: f32, y2: f32, width: f32) -> Draw {
        Draw::Line { x1, y1, x2, y2, width, tone: Tone::Ink }
    }

    pub fn polyline(points: Vec<(f32, f32)>, width: f32) -> Draw {
        Draw::Polyline { points, width, tone: Tone::Ink }
    }

    pub fn polygon(points: Vec<(f32, f32)>) -> Draw {
        Draw::Polygon { points, tone: Tone::Ink }
    }

    pub fn dot(x: f32, y: f32, radius: f32) -> Draw {
        Draw::Ellipse { x, y, rx: radius, ry: radius, angle: 0.0, filled: true, width: 0.0, tone: Tone::Ink }
    }

    pub fn text(x: f32, y: f32, text: impl Into<String>, size: f32, anchor: Anchor, style: TextStyle, tone: Tone) -> Draw {
        Draw::Text { x, y, text: text.into(), size, anchor, style, masks: false, tone }
    }

    /// The same drawn in another tone.
    pub fn toned(self, tone: Tone) -> Draw {
        match self {
            Draw::Line { x1, y1, x2, y2, width, .. } => Draw::Line { x1, y1, x2, y2, width, tone },
            Draw::Polyline { points, width, .. } => Draw::Polyline { points, width, tone },
            Draw::Polygon { points, .. } => Draw::Polygon { points, tone },
            Draw::Ellipse { x, y, rx, ry, angle, filled, width, .. } => Draw::Ellipse { x, y, rx, ry, angle, filled, width, tone },
            Draw::Text { x, y, text, size, anchor, style, masks, .. } => Draw::Text { x, y, text, size, anchor, style, masks, tone },
        }
    }

    /// The same moved by (`dx`, `dy`).
    pub fn moved(self, dx: f32, dy: f32) -> Draw {
        match self {
            Draw::Line { x1, y1, x2, y2, width, tone } => Draw::Line { x1: x1 + dx, y1: y1 + dy, x2: x2 + dx, y2: y2 + dy, width, tone },
            Draw::Polyline { points, width, tone } => Draw::Polyline { points: points.into_iter().map(|(x, y)| (x + dx, y + dy)).collect(), width, tone },
            Draw::Polygon { points, tone } => Draw::Polygon { points: points.into_iter().map(|(x, y)| (x + dx, y + dy)).collect(), tone },
            Draw::Ellipse { x, y, rx, ry, angle, filled, width, tone } => Draw::Ellipse { x: x + dx, y: y + dy, rx, ry, angle, filled, width, tone },
            Draw::Text { x, y, text, size, anchor, style, masks, tone } => Draw::Text { x: x + dx, y: y + dy, text, size, anchor, style, masks, tone },
        }
    }
}

/// Points along a cubic curve, enough of them for it to look smooth.
pub fn bezier(p0: (f32, f32), p1: (f32, f32), p2: (f32, f32), p3: (f32, f32)) -> Vec<(f32, f32)> {
    const STEPS: usize = 14;
    (0..=STEPS)
        .map(|step| {
            let t = step as f32 / STEPS as f32;
            let u = 1.0 - t;
            let (a, b, c, d) = (u * u * u, 3.0 * u * u * t, 3.0 * u * t * t, t * t * t);
            (a * p0.0 + b * p1.0 + c * p2.0 + d * p3.0, a * p0.1 + b * p1.1 + c * p2.1 + d * p3.1)
        })
        .collect()
}

/// A smooth line through `points`, which is how a shape drawn by hand (a clef) is kept to a short list of them.
pub fn smooth(points: &[(f32, f32)]) -> Vec<(f32, f32)> {
    const STEPS: usize = 8;
    if points.len() < 3 {
        return points.to_vec();
    }
    let mut result = Vec::new();
    for index in 0..points.len() - 1 {
        let p0 = points[index.saturating_sub(1)];
        let p1 = points[index];
        let p2 = points[index + 1];
        let p3 = points[(index + 2).min(points.len() - 1)];
        for step in 0..STEPS {
            let t = step as f32 / STEPS as f32;
            let (t2, t3) = (t * t, t * t * t);
            let blend = |a: f32, b: f32, c: f32, d: f32| 0.5 * (2.0 * b + (-a + c) * t + (2.0 * a - 5.0 * b + 4.0 * c - d) * t2 + (-a + 3.0 * b - 3.0 * c + d) * t3);
            result.push((blend(p0.0, p1.0, p2.0, p3.0), blend(p0.1, p1.1, p2.1, p3.1)));
        }
    }
    result.push(points[points.len() - 1]);
    result
}

/// An arc over two points, `height` high at its middle (negative above the line between them, as the y axis points down): a tie or a slur.
pub fn arc(x1: f32, y1: f32, x2: f32, y2: f32, height: f32, width: f32) -> Draw {
    let (mx, my) = ((x1 + x2) / 2.0, (y1 + y2) / 2.0);
    let control = (mx, my + height * 1.33);
    // A quadratic curve written as a cubic one.
    let c1 = (x1 + (control.0 - x1) * 2.0 / 3.0, y1 + (control.1 - y1) * 2.0 / 3.0);
    let c2 = (x2 + (control.0 - x2) * 2.0 / 3.0, y2 + (control.1 - y2) * 2.0 / 3.0);
    Draw::polyline(bezier((x1, y1), c1, c2, (x2, y2)), width)
}

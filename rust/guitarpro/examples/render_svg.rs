// This file is part of Campfire.
// Copyright (c) Pandula Péter 2017-2026.
// https://github.com/pandulapeter/campfire
//
// This Source Code Form is subject to the terms of the Mozilla Public License, v. 2.0.
// If a copy of the MPL was not distributed with this file, You can obtain one at
// https://mozilla.org/MPL/2.0/.

//! Writes what the layout makes of a track of a Guitar Pro file as JSON, one object per mark, to be looked at without a window (see
//! `examples/preview.py`): `cargo run -p guitarpro --example render_svg -- song.gp5 out.json [track] [width]`.

use guitarpro::draw::{Anchor, Draw, TextStyle, Tone};
use guitarpro::layout::{layout, Options};
use serde_json::{json, Value};

fn tone(tone: Tone) -> &'static str {
    match tone {
        Tone::Ink => "ink",
        Tone::Muted => "muted",
        Tone::Accent => "accent",
    }
}

fn main() {
    let arguments: Vec<String> = std::env::args().collect();
    let bytes = std::fs::read(&arguments[1]).expect("the file");
    let track: usize = arguments.get(3).and_then(|value| value.parse().ok()).unwrap_or(0);
    let width: f32 = arguments.get(4).and_then(|value| value.parse().ok()).unwrap_or(78.0);
    let score = guitarpro::read(&bytes).expect("a Guitar Pro file");
    let page = layout(&score, track, &Options { width, ..Options::default() });
    let mut marks: Vec<Value> = Vec::new();
    for system in &page.systems {
        for draw in &system.draws {
            marks.push(match draw {
                Draw::Line { x1, y1, x2, y2, width, tone: t } => json!({"k": "line", "p": [x1, y1, x2, y2], "w": width, "t": tone(*t)}),
                Draw::Polyline { points, width, tone: t } => json!({"k": "poly", "p": points, "w": width, "t": tone(*t)}),
                Draw::Polygon { points, tone: t } => json!({"k": "fill", "p": points, "t": tone(*t)}),
                Draw::Ellipse { x, y, rx, ry, angle, filled, width, tone: t } => json!({"k": "ellipse", "p": [x, y, rx, ry, angle], "f": filled, "w": width, "t": tone(*t)}),
                Draw::Text { x, y, text, size, anchor, style, masks, tone: t } => json!({
                    "k": "text", "p": [x, y], "s": text, "z": size, "a": match anchor { Anchor::Left => "l", Anchor::Center => "c", Anchor::Right => "r" },
                    "y": match style { TextStyle::Normal => "n", TextStyle::Bold => "b", TextStyle::Italic => "i" }, "m": masks, "t": tone(*t)
                }),
            });
        }
    }
    let output = json!({"width": page.width, "height": page.height, "marks": marks});
    std::fs::write(&arguments[2], output.to_string()).expect("the output");
}

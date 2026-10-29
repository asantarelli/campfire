// This file is part of Campfire.
// Copyright (c) Pandula Péter 2017-2026.
// https://github.com/pandulapeter/campfire
//
// This Source Code Form is subject to the terms of the Mozilla Public License, v. 2.0.
// If a copy of the MPL was not distributed with this file, You can obtain one at
// https://mozilla.org/MPL/2.0/.

//! The core of SDCancionero: ChordPro text in, a song out, and the chords of it moved to another key.
//!
//! It depends on no interface and on no file system, so every part of it is tested with plain text.

pub mod chord;
pub mod chord_names;
pub mod convert;
pub mod definitions;
pub mod directives;
pub mod edit;
pub mod model;
pub mod notation;
pub mod parser;
pub mod scales;
pub mod tokens;
pub mod transposer;
pub mod values;
pub mod voicings;

mod chorus_recalls;
mod metadata_builder;
mod normalize;
mod rewriter;
mod section_builder;

pub use rewriter::{chords_in, diagram_chords_in};
pub mod text_rewriter;

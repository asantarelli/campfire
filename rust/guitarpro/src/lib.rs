// This file is part of Campfire.
// Copyright (c) Pandula Péter 2017-2026.
// https://github.com/pandulapeter/campfire
//
// This Source Code Form is subject to the terms of the Mozilla Public License, v. 2.0.
// If a copy of the MPL was not distributed with this file, You can obtain one at
// https://mozilla.org/MPL/2.0/.

//! Guitar Pro files, read for a score to be shown: the tracks, their measures and the notes on strings and frets.
//!
//! It depends on no interface and on no file system, so every part of it is tested with plain bytes.

pub mod draw;
mod glyphs;
pub mod layout;
pub mod model;
mod reader;
pub mod song;

pub use reader::{read, Error};

// This file is part of Campfire.
// Copyright (c) Pandula Péter 2017-2026.
// https://github.com/pandulapeter/campfire
//
// This Source Code Form is subject to the terms of the Mozilla Public License, v. 2.0.
// If a copy of the MPL was not distributed with this file, You can obtain one at
// https://mozilla.org/MPL/2.0/.

//! The library folder of SDCancionero: songs and setlists on disk, and the zip archives that carry them between devices.
//!
//! It reads and writes the layout the Kotlin app keeps (`songs/*.cho`, `setlists/*.setlist.json`, the zip an export
//! produces), so one folder, or one archive, can be used by both.

pub mod archive;
pub mod names;
pub mod setlist;
pub mod store;
pub mod text;

pub use setlist::{Setlist, SetlistSong};
pub use store::{default_root, Collision, ExportError, ImportPlan, ImportReport, Library, SongEntry};

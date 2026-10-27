// This file is part of Campfire.
// Copyright (c) Pandula Péter 2017-2026.
// https://github.com/pandulapeter/campfire
//
// This Source Code Form is subject to the terms of the Mozilla Public License, v. 2.0.
// If a copy of the MPL was not distributed with this file, You can obtain one at
// https://mozilla.org/MPL/2.0/.

//! The small pictures set before the words of a button: characters of Segoe MDL2 Assets, the icon font of Windows 10 and
//! later, which `theme::install_fonts` puts behind the interface's own faces. Where the computer does not have it the
//! buttons have only their words.

pub const SETTINGS: &str = "\u{E713}";
pub const TOOLS: &str = "\u{E90F}";
pub const EXPORT: &str = "\u{E898}";
pub const IMPORT: &str = "\u{E896}";
pub const TRASH: &str = "\u{E74D}";
pub const NEW: &str = "\u{E710}";
pub const SONGS: &str = "\u{E8D6}";
pub const SETLISTS: &str = "\u{E8FD}";
pub const EDIT: &str = "\u{E70F}";
pub const DUPLICATE: &str = "\u{E8C8}";
pub const OPEN: &str = "\u{E838}";
pub const LISTEN: &str = "\u{E7F6}";
pub const RESTART: &str = "\u{E892}";
pub const PLAY_PAUSE: &str = "\u{E768}\u{E769}";
pub const PLAY: &str = "\u{E768}";
pub const STOP: &str = "\u{E71A}";

/// The words of a button with its picture in front, or the words alone where there is no font for it.
pub fn label(icon: &str, text: &str) -> String {
    if crate::theme::has_icons() {
        format!("{icon}  {text}")
    } else {
        text.to_string()
    }
}

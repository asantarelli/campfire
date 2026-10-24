// This file is part of Campfire.
// Copyright (c) Pandula Péter 2017-2026.
// https://github.com/pandulapeter/campfire
//
// This Source Code Form is subject to the terms of the Mozilla Public License, v. 2.0.
// If a copy of the MPL was not distributed with this file, You can obtain one at
// https://mozilla.org/MPL/2.0/.

//! The play and pause of whatever is playing music on the computer: the player the musician plays along with, next to
//! the song being read, which the key of a keyboard's media row works and so does this.

/// Presses and lets go the play/pause key of the media row, which Windows hands to the player that is playing or was
/// playing last. It reads nothing: the app learns neither what is playing nor whether anything is.
#[cfg(windows)]
pub fn play_pause() {
    const VK_MEDIA_PLAY_PAUSE: u8 = 0xB3;
    const KEYEVENTF_KEYUP: u32 = 0x0002;
    #[link(name = "user32")]
    extern "system" {
        fn keybd_event(key: u8, scan: u8, flags: u32, extra: usize);
    }
    // SAFETY: a call of the system with plain numbers, which holds on to nothing.
    unsafe {
        keybd_event(VK_MEDIA_PLAY_PAUSE, 0, 0, 0);
        keybd_event(VK_MEDIA_PLAY_PAUSE, 0, KEYEVENTF_KEYUP, 0);
    }
}

/// Nothing outside of Windows, where the app is not run.
#[cfg(not(windows))]
pub fn play_pause() {}

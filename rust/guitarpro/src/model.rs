// This file is part of Campfire.
// Copyright (c) Pandula Péter 2017-2026.
// https://github.com/pandulapeter/campfire
//
// This Source Code Form is subject to the terms of the Mozilla Public License, v. 2.0.
// If a copy of the MPL was not distributed with this file, You can obtain one at
// https://mozilla.org/MPL/2.0/.

//! What a Guitar Pro file says, as much of it as a score is read from: the tracks, their measures and the beats of each, with
//! the notes on strings and frets and what is done to them.

/// How many ticks a quarter note lasts, the unit every duration is counted in.
pub const QUARTER_TICKS: u32 = 960;

#[derive(Debug, Clone, PartialEq, Default)]
pub struct Score {
    pub title: String,
    pub subtitle: String,
    pub artist: String,
    pub album: String,
    pub words: String,
    pub music: String,
    pub copyright: String,
    /// Who wrote the tablature.
    pub tabbed_by: String,
    pub notice: Vec<String>,
    /// Beats a minute.
    pub tempo: i32,
    /// The key as the number of sharps (positive) or flats (negative).
    pub key: i32,
    /// What each measure is, which every track shares.
    pub headers: Vec<MeasureHeader>,
    pub tracks: Vec<Track>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MeasureHeader {
    pub numerator: u8,
    pub denominator: u8,
    pub is_repeat_open: bool,
    /// The number of times the repeat that closes here is played, or -1 where no repeat closes.
    pub repeat_close: i32,
    /// The endings this measure is one of, as bits: 1 the first, 2 the second.
    pub repeat_alternative: u8,
    pub marker: Option<String>,
    /// The key this measure changes to, as sharps (positive) or flats (negative), and whether it is minor.
    pub key_change: Option<(i32, bool)>,
    pub has_double_bar: bool,
    /// Where the measure starts, in ticks from the start of the song.
    pub start: u32,
}

impl MeasureHeader {
    /// How long the measure lasts, in ticks.
    pub fn length(&self) -> u32 {
        self.numerator as u32 * (QUARTER_TICKS * 4 / (self.denominator.max(1) as u32))
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Track {
    pub name: String,
    /// The tuning, one MIDI note per string from the highest (the first string) to the lowest.
    pub strings: Vec<i32>,
    /// The fret a capo is set at, 0 for none.
    pub capo: i32,
    pub is_percussion: bool,
    /// One per measure of the score.
    pub measures: Vec<Measure>,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct Measure {
    /// The beats of each voice, the first being the one a score is read from; the second, where the file has one, holds what is
    /// played with it.
    pub voices: Vec<Vec<Beat>>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Beat {
    /// Where it starts, in ticks from the start of the song.
    pub start: u32,
    pub duration: Duration,
    pub status: BeatStatus,
    pub notes: Vec<Note>,
    pub text: Option<String>,
    /// The name of the chord written over it.
    pub chord: Option<String>,
    /// The tempo it changes to, beats a minute.
    pub tempo: Option<i32>,
    pub stroke: Option<Stroke>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BeatStatus {
    Empty,
    Normal,
    Rest,
}

/// The direction of a strum.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Stroke {
    Down,
    Up,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Duration {
    /// 1 a whole note, 2 a half, 4 a quarter, 8 an eighth...
    pub value: u8,
    pub is_dotted: bool,
    /// A tuplet is `enters` notes in the time of `times`: a triplet is 3 in the time of 2.
    pub enters: u8,
    pub times: u8,
}

impl Duration {
    pub fn new(value: u8) -> Duration {
        Duration { value, is_dotted: false, enters: 1, times: 1 }
    }

    /// How long it lasts, in ticks.
    pub fn ticks(&self) -> u32 {
        let mut ticks = QUARTER_TICKS * 4 / (self.value.max(1) as u32);
        if self.is_dotted {
            ticks += ticks / 2;
        }
        ticks * self.times.max(1) as u32 / self.enters.max(1) as u32
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NoteKind {
    Normal,
    /// Held from the note before it, on the same string.
    Tie,
    Dead,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Note {
    /// The string, 1 being the highest.
    pub string: u8,
    pub fret: i32,
    pub kind: NoteKind,
    pub effects: NoteEffects,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct NoteEffects {
    /// Hammer-on or pull-off to the next note.
    pub hammer: bool,
    pub slide: Option<Slide>,
    pub bend: Option<Bend>,
    pub harmonic: bool,
    pub vibrato: bool,
    pub palm_mute: bool,
    pub let_ring: bool,
    pub staccato: bool,
    pub ghost: bool,
    pub accent: bool,
    pub heavy_accent: bool,
    /// The fret of the grace note played just before it.
    pub grace: Option<i32>,
    pub trill: bool,
    pub tremolo_picking: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Slide {
    /// To the next note, which is played again.
    Shift,
    /// To the next note, which is not.
    Legato,
    OutDown,
    OutUp,
    InFromBelow,
    InFromAbove,
}

/// A bend: the points of the string's rise and fall, each a place along the note (0 to 12) and a height in quarters of a tone.
#[derive(Debug, Clone, PartialEq)]
pub struct Bend {
    pub points: Vec<(i32, i32)>,
}

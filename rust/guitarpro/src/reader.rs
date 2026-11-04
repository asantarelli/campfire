// This file is part of Campfire.
// Copyright (c) Pandula Péter 2017-2026.
// https://github.com/pandulapeter/campfire
//
// This Source Code Form is subject to the terms of the Mozilla Public License, v. 2.0.
// If a copy of the MPL was not distributed with this file, You can obtain one at
// https://mozilla.org/MPL/2.0/.

//! Reads the files Guitar Pro 3, 4 and 5 write (`.gp3`, `.gp4`, `.gp5`), which are one long run of little-endian numbers, flags and
//! strings with no index in front of anything: a reader has to know the layout of every piece to find where the next begins, so
//! the pieces this reader has no use for (the sound, the page, the mixer) are walked past rather than kept.

use std::fmt;

use crate::model::*;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    /// The file is not one of Guitar Pro's: it does not start with the name of a version.
    NotGuitarPro,
    /// A version of Guitar Pro this reader does not know how to read.
    Unsupported(String),
    /// The file ends before the score does.
    UnexpectedEnd,
    /// A number in the file that no score has.
    Corrupt(&'static str),
}

impl fmt::Display for Error {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::NotGuitarPro => write!(formatter, "No es un archivo de Guitar Pro."),
            Error::Unsupported(version) => write!(formatter, "No se puede leer esta versión de Guitar Pro ({version})."),
            Error::UnexpectedEnd => write!(formatter, "El archivo se corta antes de terminar la partitura."),
            Error::Corrupt(what) => write!(formatter, "El archivo está dañado ({what})."),
        }
    }
}

impl std::error::Error for Error {}

/// How many of anything a file may claim to hold, so that a damaged one is refused rather than allocated.
const MAX_COUNT: i32 = 100_000;

/// The version of the file, as `5.10` is (5, 10).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Version {
    major: u8,
    minor: u8,
}

impl Version {
    fn is_after_5_0(self) -> bool {
        self.major == 5 && self.minor > 0
    }
}

/// Reads a Guitar Pro 3, 4 or 5 file.
pub fn read(bytes: &[u8]) -> Result<Score, Error> {
    let mut reader = Reader { data: bytes, position: 0, version: Version { major: 0, minor: 0 } };
    reader.score()
}

struct Reader<'a> {
    data: &'a [u8],
    position: usize,
    version: Version,
}

impl Reader<'_> {
    // ---- the pieces of a file ------------------------------------------------------------------------------------------------------

    fn skip(&mut self, count: usize) -> Result<(), Error> {
        if self.position + count > self.data.len() {
            return Err(Error::UnexpectedEnd);
        }
        self.position += count;
        Ok(())
    }

    fn take(&mut self, count: usize) -> Result<&[u8], Error> {
        if self.position + count > self.data.len() {
            return Err(Error::UnexpectedEnd);
        }
        let slice = &self.data[self.position..self.position + count];
        self.position += count;
        Ok(slice)
    }

    fn u8(&mut self) -> Result<u8, Error> {
        Ok(self.take(1)?[0])
    }

    fn i8(&mut self) -> Result<i8, Error> {
        Ok(self.u8()? as i8)
    }

    fn boolean(&mut self) -> Result<bool, Error> {
        Ok(self.u8()? != 0)
    }

    fn i16(&mut self) -> Result<i16, Error> {
        let bytes = self.take(2)?;
        Ok(i16::from_le_bytes([bytes[0], bytes[1]]))
    }

    fn i32(&mut self) -> Result<i32, Error> {
        let bytes = self.take(4)?;
        Ok(i32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]))
    }

    /// A count the file claims, which is never negative and never more than a score could hold.
    fn count(&mut self, what: &'static str) -> Result<usize, Error> {
        let count = self.i32()?;
        if !(0..=MAX_COUNT).contains(&count) {
            return Err(Error::Corrupt(what));
        }
        Ok(count as usize)
    }

    /// Text as the file writes it: in UTF-8 where it reads as that, else in Windows-1252, which is what Guitar Pro itself writes.
    fn text(bytes: &[u8]) -> String {
        match std::str::from_utf8(bytes) {
            Ok(text) => text.to_string(),
            Err(_) => bytes.iter().map(|&byte| byte as char).collect(),
        }
    }

    /// A length of one byte and then `room` bytes, of which the length are the text.
    fn byte_size_string(&mut self, room: usize) -> Result<String, Error> {
        let length = self.u8()? as usize;
        let bytes = self.take(room)?;
        Ok(Self::text(&bytes[..length.min(room)]))
    }

    /// A length of four bytes, and then that many bytes of text.
    fn int_size_string(&mut self) -> Result<String, Error> {
        let length = self.count("text")?;
        Ok(Self::text(self.take(length)?))
    }

    /// A number of four bytes that is one more than the room of a `byte_size_string` that follows.
    fn int_byte_size_string(&mut self) -> Result<String, Error> {
        let length = self.i32()?;
        if length <= 0 {
            return Ok(String::new());
        }
        if length > MAX_COUNT {
            return Err(Error::Corrupt("text"));
        }
        self.byte_size_string(length as usize - 1)
    }

    // ---- the score --------------------------------------------------------------------------------------------------------------------

    fn score(&mut self) -> Result<Score, Error> {
        // A file too short to name a version is not one of Guitar Pro's.
        let version = self.byte_size_string(30).map_err(|_| Error::NotGuitarPro)?;
        self.version = parse_version(&version)?;
        let mut score = Score::default();
        let major = self.version.major;
        self.info(&mut score)?;
        if major <= 4 {
            self.boolean()?; // the triplet feel
        }
        if major >= 4 {
            self.lyrics()?;
        }
        if major == 5 {
            if self.version.is_after_5_0() {
                self.skip(4 + 4 + 11)?; // the master volume and its equalizer
            }
            self.skip(4 * 2 + 4 * 4 + 4 + 2)?; // the page: its size, margins, scale and what is shown on it
            for _ in 0..10 {
                self.int_byte_size_string()?;
            }
            self.int_byte_size_string()?; // the name of the tempo
        }
        score.tempo = self.i32()?;
        if major == 5 {
            if self.version.is_after_5_0() {
                self.boolean()?; // whether the tempo is hidden
            }
            score.key = self.i8()? as i32;
            self.skip(4)?; // the octave
        } else {
            score.key = self.i32()?;
            if major == 4 {
                self.skip(1)?; // the octave
            }
        }
        self.skip(64 * 12)?; // the 64 channels of the mixer
        if major == 5 {
            self.skip(19 * 2)?; // the signs that say where to jump
            self.skip(4)?; // the master reverb
        }
        let measure_count = self.count("measures")?;
        let track_count = self.count("tracks")?;
        self.headers(&mut score, measure_count)?;
        self.tracks(&mut score, track_count)?;
        self.measures(&mut score)?;
        Ok(score)
    }

    fn info(&mut self, score: &mut Score) -> Result<(), Error> {
        score.title = self.int_byte_size_string()?;
        score.subtitle = self.int_byte_size_string()?;
        score.artist = self.int_byte_size_string()?;
        score.album = self.int_byte_size_string()?;
        score.words = self.int_byte_size_string()?;
        score.music = if self.version.major == 5 { self.int_byte_size_string()? } else { score.words.clone() };
        score.copyright = self.int_byte_size_string()?;
        score.tabbed_by = self.int_byte_size_string()?;
        self.int_byte_size_string()?; // the instructions
        let notices = self.count("notice")?;
        for _ in 0..notices {
            score.notice.push(self.int_byte_size_string()?);
        }
        Ok(())
    }

    fn lyrics(&mut self) -> Result<(), Error> {
        self.skip(4)?; // the track they belong to
        for _ in 0..5 {
            self.skip(4)?;
            self.int_size_string()?;
        }
        Ok(())
    }

    fn headers(&mut self, score: &mut Score, count: usize) -> Result<(), Error> {
        let mut start = QUARTER_TICKS;
        for number in 0..count {
            let previous = score.headers.last().cloned();
            if self.version.major == 5 && number > 0 {
                self.skip(1)?;
            }
            let mut header = self.header(previous.as_ref())?;
            header.start = start;
            start += header.length();
            score.headers.push(header);
        }
        Ok(())
    }

    fn header(&mut self, previous: Option<&MeasureHeader>) -> Result<MeasureHeader, Error> {
        let flags = self.u8()?;
        let is_5 = self.version.major == 5;
        let numerator = if flags & 0x01 != 0 { self.i8()? as u8 } else { previous.map_or(4, |header| header.numerator) };
        let denominator = if flags & 0x02 != 0 { self.i8()? as u8 } else { previous.map_or(4, |header| header.denominator) };
        let mut header = MeasureHeader {
            numerator: numerator.max(1),
            denominator: denominator.max(1),
            is_repeat_open: flags & 0x04 != 0,
            repeat_close: -1,
            repeat_alternative: 0,
            marker: None,
            key_change: None,
            has_double_bar: flags & 0x80 != 0,
            start: 0,
        };
        if flags & 0x08 != 0 {
            header.repeat_close = self.i8()? as i32;
        }
        // Where the number of the ending is, and the marker and the key, is not the same in a Guitar Pro 5 file as in the others.
        if !is_5 && flags & 0x10 != 0 {
            header.repeat_alternative = self.u8()?;
        }
        if flags & 0x20 != 0 {
            let title = self.int_byte_size_string()?;
            self.skip(4)?; // its color
            header.marker = Some(title);
        }
        if flags & 0x40 != 0 {
            let root = self.i8()? as i32;
            let kind = self.i8()?;
            header.key_change = Some((root, kind != 0));
        }
        if is_5 {
            if flags & 0x10 != 0 {
                header.repeat_alternative = self.u8()?;
            }
            if header.repeat_close > -1 {
                header.repeat_close -= 1;
            }
            if flags & 0x03 != 0 {
                self.skip(4)?; // how the beats of the measure are beamed
            }
            if flags & 0x10 == 0 {
                self.skip(1)?;
            }
            self.skip(1)?; // the triplet feel
        }
        Ok(header)
    }

    fn tracks(&mut self, score: &mut Score, count: usize) -> Result<(), Error> {
        for number in 1..=count {
            let track = self.track(number)?;
            score.tracks.push(track);
        }
        if self.version.major == 5 {
            self.skip(if self.version.is_after_5_0() { 1 } else { 2 })?;
        }
        Ok(())
    }

    fn track(&mut self, number: usize) -> Result<Track, Error> {
        let is_5 = self.version.major == 5;
        if is_5 && (number == 1 || !self.version.is_after_5_0()) {
            self.skip(1)?;
        }
        let flags = self.u8()?;
        let name = self.byte_size_string(40)?;
        let string_count = self.count("strings")?.min(7);
        let mut strings = Vec::new();
        for index in 0..7 {
            let tuning = self.i32()?;
            if index < string_count {
                strings.push(tuning);
            }
        }
        self.skip(4)?; // the port
        let channel = self.i32()?;
        self.skip(4)?; // the channel of the effects
        self.skip(4)?; // how many frets the instrument has
        let capo = self.i32()?;
        self.skip(4)?; // its color
        if is_5 {
            self.skip(2 + 1 + 1 + 1 + 4 + 4 + 4 + 12)?;
            self.skip(4 + 4 + 4)?; // the instrument of the sound engine
            self.skip(if self.version.is_after_5_0() { 4 } else { 3 })?;
            if self.version.is_after_5_0() {
                self.skip(4)?; // its equalizer
                self.int_byte_size_string()?;
                self.int_byte_size_string()?;
            }
        }
        Ok(Track { name, strings, capo, is_percussion: flags & 0x01 != 0 || channel == 10, measures: Vec::new() })
    }

    fn measures(&mut self, score: &mut Score) -> Result<(), Error> {
        let voices = if self.version.major == 5 { 2 } else { 1 };
        let mut last_frets: Vec<Vec<i32>> = score.tracks.iter().map(|track| vec![-1; track.strings.len() + 1]).collect();
        for index in 0..score.headers.len() {
            let start = score.headers[index].start;
            for track_index in 0..score.tracks.len() {
                let strings = score.tracks[track_index].strings.len();
                let mut measure = Measure::default();
                for _ in 0..voices {
                    measure.voices.push(self.voice(start, strings, &mut last_frets[track_index])?);
                }
                if self.version.major == 5 {
                    // Where the line breaks. The last measure of a file may end without it, which Guitar Pro itself reads past.
                    self.position = (self.position + 1).min(self.data.len());
                }
                score.tracks[track_index].measures.push(measure);
            }
        }
        Ok(())
    }

    fn voice(&mut self, start: u32, strings: usize, last_frets: &mut [i32]) -> Result<Vec<Beat>, Error> {
        let count = self.count("beats")?;
        let mut beats = Vec::new();
        let mut at = start;
        for _ in 0..count {
            let beat = self.beat(at, strings, last_frets)?;
            if beat.status != BeatStatus::Empty {
                at += beat.duration.ticks();
            }
            beats.push(beat);
        }
        Ok(beats)
    }

    fn beat(&mut self, start: u32, strings: usize, last_frets: &mut [i32]) -> Result<Beat, Error> {
        let flags = self.u8()?;
        let status = if flags & 0x40 != 0 {
            match self.u8()? {
                0 => BeatStatus::Empty,
                2 => BeatStatus::Rest,
                _ => BeatStatus::Normal,
            }
        } else {
            BeatStatus::Normal
        };
        let exponent = (self.i8()? as i32 + 2).clamp(0, 6);
        let mut duration = Duration::new(1u8 << exponent);
        duration.is_dotted = flags & 0x01 != 0;
        if flags & 0x20 != 0 {
            (duration.enters, duration.times) = match self.i32()? {
                3 => (3, 2),
                5 => (5, 4),
                6 => (6, 4),
                7 => (7, 4),
                9 => (9, 8),
                10 => (10, 8),
                11 => (11, 8),
                12 => (12, 8),
                13 => (13, 8),
                _ => (1, 1),
            };
        }
        let mut beat = Beat { start, duration, status, notes: Vec::new(), text: None, chord: None, chord_shape: None, tempo: None, stroke: None };
        if flags & 0x02 != 0 {
            (beat.chord, beat.chord_shape) = self.chord(strings)?;
        }
        if flags & 0x04 != 0 {
            beat.text = Some(self.int_byte_size_string()?);
        }
        let mut whole_beat = NoteEffects::default();
        if flags & 0x08 != 0 {
            beat.stroke = self.beat_effects(&mut whole_beat)?;
        }
        if flags & 0x10 != 0 {
            beat.tempo = self.mix_table_change()?;
        }
        self.notes(&mut beat, strings, &whole_beat, last_frets)?;
        if self.version.major == 5 {
            let flags = self.i16()?;
            if flags & 0x0800 != 0 {
                self.skip(1)?;
            }
        }
        Ok(beat)
    }

    /// The chord diagram of a beat: its name, which may be empty, and its shape.
    fn chord(&mut self, strings: usize) -> Result<(Option<String>, Option<ChordShape>), Error> {
        let is_new = self.boolean()?;
        let name;
        let mut shape = None;
        if !is_new {
            name = self.int_byte_size_string()?;
            let first_fret = self.i32()?;
            if first_fret != 0 {
                shape = Some(self.chord_frets(first_fret, 6, strings)?);
            }
        } else if self.version.major == 3 {
            self.skip(1 + 3 + 4 * 5 + 1)?;
            name = self.byte_size_string(22)?;
            self.skip(3 * 4)?;
            let first_fret = self.i32()?;
            shape = Some(self.chord_frets(first_fret, 6, strings)?);
            self.skip(4 + 2 * 4 * 3 + 7 + 1)?;
        } else {
            self.skip(1 + 3 + 1 + 1 + 1 + 4 + 4 + 1)?;
            name = self.byte_size_string(22)?;
            self.skip(3)?;
            let first_fret = self.i32()?;
            shape = Some(self.chord_frets(first_fret, 7, strings)?);
            self.skip(1 + 5 * 3 + 7 + 1 + 7 + 1)?;
        }
        // A diagram with nothing played is not worth drawing.
        let shape = shape.filter(|shape| shape.frets.iter().any(|&fret| fret >= 0));
        Ok((Some(name).filter(|name| !name.is_empty()), shape))
    }

    /// The frets of a chord diagram, `stored` of them written, of which the track's `strings` are kept.
    fn chord_frets(&mut self, first_fret: i32, stored: usize, strings: usize) -> Result<ChordShape, Error> {
        let mut frets = Vec::new();
        for index in 0..stored {
            let fret = self.i32()?;
            if index < strings.min(7) {
                frets.push(fret);
            }
        }
        Ok(ChordShape { first_fret, frets })
    }

    /// The effects that belong to the beat as a whole, and the direction of its strum.
    fn beat_effects(&mut self, notes: &mut NoteEffects) -> Result<Option<Stroke>, Error> {
        let mut stroke = None;
        if self.version.major == 3 {
            let flags = self.u8()?;
            notes.vibrato |= flags & 0x01 != 0;
            if flags & 0x20 != 0 {
                if self.u8()? == 0 {
                    self.skip(4)?; // the tremolo bar
                } else {
                    self.skip(4)?; // a slap
                }
            }
            if flags & 0x40 != 0 {
                stroke = self.stroke()?;
            }
            notes.harmonic |= flags & 0x0C != 0;
        } else {
            let flags1 = self.u8()?;
            let flags2 = self.u8()?;
            if flags1 & 0x20 != 0 {
                self.skip(1)?; // the slap
            }
            if flags2 & 0x04 != 0 {
                self.bend()?;
            }
            if flags1 & 0x40 != 0 {
                stroke = self.stroke()?;
            }
            if flags2 & 0x02 != 0 {
                self.skip(1)?; // the pick stroke
            }
        }
        Ok(stroke)
    }

    /// The two speeds of a strum, down and up, of which the one that is not zero is its direction. Guitar Pro 5 writes them the other way round.
    fn stroke(&mut self) -> Result<Option<Stroke>, Error> {
        let first = self.i8()?;
        let second = self.i8()?;
        let (down, up) = if self.version.major == 5 { (second, first) } else { (first, second) };
        Ok(if up > 0 {
            Some(Stroke::Up)
        } else if down > 0 {
            Some(Stroke::Down)
        } else {
            None
        })
    }

    /// A change of the mixer on a beat, of which only the tempo is kept.
    fn mix_table_change(&mut self) -> Result<Option<i32>, Error> {
        let is_5 = self.version.major == 5;
        let (volume, balance, chorus, reverb, phaser, tremolo, tempo);
        if is_5 {
            let instrument = self.i8()?;
            let _ = instrument;
            self.skip(4 + 4 + 4)?; // the instrument of the sound engine
            self.skip(if self.version.is_after_5_0() { 4 } else { 3 })?;
            if !self.version.is_after_5_0() {
                self.skip(1)?;
            }
            volume = self.i8()?;
            balance = self.i8()?;
            chorus = self.i8()?;
            reverb = self.i8()?;
            phaser = self.i8()?;
            tremolo = self.i8()?;
            self.int_byte_size_string()?; // the name of the tempo
            tempo = self.i32()?;
        } else {
            self.skip(1)?; // the instrument
            volume = self.i8()?;
            balance = self.i8()?;
            chorus = self.i8()?;
            reverb = self.i8()?;
            phaser = self.i8()?;
            tremolo = self.i8()?;
            tempo = self.i32()?;
        }
        for changed in [volume, balance, chorus, reverb, phaser, tremolo] {
            if changed >= 0 {
                self.skip(1)?; // how long the change takes
            }
        }
        if tempo >= 0 {
            self.skip(1)?;
            if self.version.is_after_5_0() {
                self.skip(1)?; // whether the tempo is shown
            }
        }
        if self.version.major >= 4 {
            self.skip(1)?; // which of the changes are for every track
        }
        if is_5 {
            self.skip(1)?; // the wah
            if self.version.is_after_5_0() {
                self.int_byte_size_string()?;
                self.int_byte_size_string()?;
            }
        }
        Ok((tempo >= 0).then_some(tempo))
    }

    fn notes(&mut self, beat: &mut Beat, strings: usize, whole_beat: &NoteEffects, last_frets: &mut [i32]) -> Result<(), Error> {
        let flags = self.u8()?;
        for string in 1..=strings.min(7) {
            if flags & (1 << (7 - string)) != 0 {
                let note = self.note(string as u8, whole_beat, last_frets)?;
                beat.notes.push(note);
            }
        }
        Ok(())
    }

    fn note(&mut self, string: u8, whole_beat: &NoteEffects, last_frets: &mut [i32]) -> Result<Note, Error> {
        let is_5 = self.version.major == 5;
        let flags = self.u8()?;
        let mut effects = whole_beat.clone();
        effects.heavy_accent = flags & 0x02 != 0;
        effects.ghost = flags & 0x04 != 0;
        effects.accent = flags & 0x40 != 0;
        let mut kind = NoteKind::Normal;
        if flags & 0x20 != 0 {
            kind = match self.u8()? {
                2 => NoteKind::Tie,
                3 => NoteKind::Dead,
                _ => NoteKind::Normal,
            };
        }
        if !is_5 && flags & 0x01 != 0 {
            self.skip(2)?; // a duration of the note's own
        }
        if flags & 0x10 != 0 {
            self.skip(1)?; // how hard it is played
        }
        let mut fret = 0;
        if flags & 0x20 != 0 {
            let written = self.i8()? as i32;
            let value = if kind == NoteKind::Tie { last_frets.get(string as usize).copied().unwrap_or(-1) } else { written };
            fret = if (0..100).contains(&value) { value } else { 0 };
            if let Some(slot) = last_frets.get_mut(string as usize) {
                *slot = fret;
            }
        }
        if flags & 0x80 != 0 {
            self.skip(2)?; // the fingers
        }
        if is_5 {
            if flags & 0x01 != 0 {
                self.skip(8)?; // how much of its duration it sounds
            }
            self.skip(1)?;
        }
        if flags & 0x08 != 0 {
            self.note_effects(&mut effects)?;
        }
        Ok(Note { string, fret, kind, effects })
    }

    fn note_effects(&mut self, effects: &mut NoteEffects) -> Result<(), Error> {
        let major = self.version.major;
        let flags1 = self.u8()?;
        let flags2 = if major >= 4 { self.u8()? } else { 0 };
        effects.hammer = flags1 & 0x02 != 0;
        effects.let_ring = flags1 & 0x08 != 0;
        if major >= 4 {
            effects.staccato = flags2 & 0x01 != 0;
            effects.palm_mute = flags2 & 0x02 != 0;
            effects.vibrato |= flags2 & 0x40 != 0;
        }
        if flags1 & 0x01 != 0 {
            effects.bend = self.bend()?;
        }
        if flags1 & 0x10 != 0 {
            effects.grace = Some(self.grace()?);
        }
        if major >= 4 {
            if flags2 & 0x04 != 0 {
                self.skip(1)?; // the speed of the tremolo picking
                effects.tremolo_picking = true;
            }
            if flags2 & 0x08 != 0 {
                effects.slide = self.slide()?;
            }
            if flags2 & 0x10 != 0 {
                self.harmonic()?;
                effects.harmonic = true;
            }
            if flags2 & 0x20 != 0 {
                self.skip(2)?; // the fret and the period of the trill
                effects.trill = true;
            }
        } else if flags1 & 0x04 != 0 {
            effects.slide = Some(Slide::Shift);
        }
        Ok(())
    }

    fn bend(&mut self) -> Result<Option<Bend>, Error> {
        self.skip(1 + 4)?; // its kind and its height
        let count = self.count("bend points")?;
        let mut points = Vec::new();
        for _ in 0..count {
            let position = self.i32()?;
            let value = self.i32()?;
            self.skip(1)?; // whether it vibrates
            // A position is written in sixtieths of the note's length and a height in a hundredth of a semitone, a quarter of which is 25.
            points.push((((position as f32) * 12.0 / 60.0).round() as i32, ((value as f32) / 25.0).round() as i32));
        }
        Ok((!points.is_empty()).then_some(Bend { points }))
    }

    fn grace(&mut self) -> Result<i32, Error> {
        if self.version.major == 5 {
            let fret = self.u8()? as i32;
            self.skip(1 + 1 + 1 + 1)?;
            Ok(fret)
        } else {
            let fret = self.i8()? as i32;
            self.skip(1 + 1 + 1)?;
            Ok(fret)
        }
    }

    fn slide(&mut self) -> Result<Option<Slide>, Error> {
        let kind = self.i8()?;
        if self.version.major == 5 {
            // A set of bits; the first the note has is the one drawn.
            return Ok([(0x01, Slide::Shift), (0x02, Slide::Legato), (0x04, Slide::OutDown), (0x08, Slide::OutUp), (0x10, Slide::InFromBelow), (0x20, Slide::InFromAbove)]
                .into_iter()
                .find(|(bit, _)| kind & bit != 0)
                .map(|(_, slide)| slide));
        }
        Ok(match kind {
            1 => Some(Slide::Shift),
            2 => Some(Slide::Legato),
            3 => Some(Slide::OutDown),
            4 => Some(Slide::OutUp),
            -1 => Some(Slide::InFromBelow),
            -2 => Some(Slide::InFromAbove),
            _ => None,
        })
    }

    fn harmonic(&mut self) -> Result<(), Error> {
        let kind = self.i8()?;
        if self.version.major == 5 {
            match kind {
                2 => self.skip(3)?,
                3 => self.skip(1)?,
                _ => {}
            }
        }
        Ok(())
    }
}

/// The version a file's first string names: `FICHIER GUITAR PRO v5.10` is 5.10.
fn parse_version(text: &str) -> Result<Version, Error> {
    let name = text.trim();
    if !name.starts_with("FICHIER GUITAR PRO") {
        return Err(Error::NotGuitarPro);
    }
    let digits = name.rsplit_once('v').map(|(_, version)| version).ok_or(Error::NotGuitarPro)?;
    let (major, minor) = digits.split_once('.').ok_or(Error::NotGuitarPro)?;
    let major: u8 = major.trim().parse().map_err(|_| Error::NotGuitarPro)?;
    let minor: u8 = minor.trim().parse().map_err(|_| Error::NotGuitarPro)?;
    if !(3..=5).contains(&major) {
        return Err(Error::Unsupported(name.to_string()));
    }
    Ok(Version { major, minor })
}

// This file is part of Campfire.
// Copyright (c) Pandula Péter 2017-2026.
// https://github.com/pandulapeter/campfire
//
// This Source Code Form is subject to the terms of the Mozilla Public License, v. 2.0.
// If a copy of the MPL was not distributed with this file, You can obtain one at
// https://mozilla.org/MPL/2.0/.

//! Sets a track of a score on pages: measures shared out into systems as wide as the window, and in each the notes on a staff and on
//! tablature, with their stems, beams, ties and the marks of how they are played. The answer is a list of [`Draw`]s in staff spaces; what
//! they look like (colors, fonts, size) is for whoever paints them.

use std::collections::HashMap;

use crate::draw::{arc, bezier, Anchor, Draw, TextStyle, Tone};
use crate::glyphs::{self, BEAM, LINE, STEM};
use crate::model::*;

/// What to show and how wide to set it.
#[derive(Debug, Clone, PartialEq)]
pub struct Options {
    /// How wide a system may be, in staff spaces.
    pub width: f32,
    /// The notes on a staff.
    pub notation: bool,
    /// The notes on the strings.
    pub tab: bool,
    /// The stems and beams under the tablature, which say how long each note lasts.
    pub rhythm: bool,
}

impl Default for Options {
    fn default() -> Self {
        Options { width: 80.0, notation: true, tab: true, rhythm: true }
    }
}

/// The page: its systems from the top, and how big it is.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Layout {
    pub systems: Vec<System>,
    pub width: f32,
    pub height: f32,
}

/// One line of the score: some measures side by side.
#[derive(Debug, Clone, PartialEq)]
pub struct System {
    pub top: f32,
    pub height: f32,
    /// The index of the first measure it holds and of the last.
    pub first_measure: usize,
    pub last_measure: usize,
    pub draws: Vec<Draw>,
}

/// The space between two strings of the tablature.
const TAB_GAP: f32 = 1.2;
/// From the top of a system to the top of its first staff: room for the markers, the chords and the notes above the staff.
const ABOVE: f32 = 7.2;
/// From the notation staff to the tablature one: room for the notes and the stems under the staff.
const BETWEEN: f32 = 4.4;
/// Under the tablature, for the rhythm.
const RHYTHM: f32 = 4.2;
/// Between systems.
const SYSTEM_GAP: f32 = 2.0;
/// The room a clef takes at the start of a system.
const CLEF: f32 = 3.6;
/// A time signature's.
const TIME_SIGNATURE: f32 = 2.8;
/// The least a beat takes.
const MIN_BEAT: f32 = 2.4;
/// What a measure takes beside its beats: a little room after its bar line, and before the next.
const MEASURE_PADDING: f32 = 1.5;
/// The middle line of the staff as a position in steps (B4).
const MIDDLE_LINE: i32 = 34;

pub fn layout(score: &Score, track_index: usize, options: &Options) -> Layout {
    let Some(track) = score.tracks.get(track_index) else {
        return Layout::default();
    };
    let measure_count = track.measures.len().min(score.headers.len());
    if measure_count == 0 || (!options.notation && !options.tab) {
        return Layout::default();
    }
    let keys = keys_of(score, measure_count);
    let plans: Vec<MeasurePlan> = (0..measure_count).map(|index| plan_measure(track, index)).collect();
    let systems = break_into_systems(&plans, &keys, score, options);
    let mut painter = Painter { score, track, options, keys: &keys, plans: &plans, ties: HashMap::new(), pending: HashMap::new(), serial: 0 };
    let mut result = Layout { width: options.width, ..Layout::default() };
    let mut top = 0.0;
    for (number, system) in systems.iter().enumerate() {
        let built = painter.system(system, top, number, number + 1 == systems.len());
        top += built.height + SYSTEM_GAP;
        result.systems.push(built);
    }
    result.height = top;
    result
}

/// The key of every measure: the score's, and then what each measure that changes it says.
fn keys_of(score: &Score, count: usize) -> Vec<i32> {
    let mut key = score.key.clamp(-7, 7);
    (0..count)
        .map(|index| {
            if let Some((root, _)) = score.headers[index].key_change {
                key = root.clamp(-7, 7);
            }
            key
        })
        .collect()
}

// ---- planning the measures ------------------------------------------------------------------------------------------------------

struct BeatPlan<'a> {
    beat: &'a Beat,
    /// Where its room starts in the body of the measure, before the measure is stretched to the width of the system.
    x: f32,
}

struct MeasurePlan<'a> {
    beats: Vec<BeatPlan<'a>>,
    /// How wide the body is, before it is stretched.
    body: f32,
}

fn beat_width(beat: &Beat) -> f32 {
    let ticks = beat.duration.ticks() as f32;
    let mut width = (4.0 * (ticks / QUARTER_TICKS as f32).powf(0.55)).max(MIN_BEAT);
    if beat.notes.iter().any(|note| note.fret >= 10) {
        width = width.max(MIN_BEAT + 0.4);
    }
    if let Some(chord) = &beat.chord {
        width = width.max(chord.chars().count() as f32 * 0.8 + 0.6);
    }
    if let Some(text) = &beat.text {
        width = width.max(text.chars().count() as f32 * 0.55 + 0.6);
    }
    if beat.notes.iter().any(|note| note.effects.grace.is_some()) {
        width += 1.2;
    }
    // A note that is not on the lines of its key brings a sign before it, which takes room.
    if beat.notes.iter().any(|note| note.fret > 0) && beat.notes.len() > 2 {
        width += 0.3;
    }
    width
}

fn plan_measure(track: &Track, index: usize) -> MeasurePlan<'_> {
    let voice = track.measures.get(index).and_then(|measure| measure.voices.first());
    let mut beats = Vec::new();
    let mut x = 0.0;
    if let Some(voice) = voice {
        for beat in voice.iter().filter(|beat| beat.status != BeatStatus::Empty) {
            let width = beat_width(beat);
            beats.push(BeatPlan { beat, x });
            x += width;
        }
    }
    let body = if beats.is_empty() { 5.0 } else { x };
    MeasurePlan { beats, body }
}

// ---- the systems -------------------------------------------------------------------------------------------------------------------

struct SystemPlan {
    measures: std::ops::Range<usize>,
}

/// What stands at the start of a system before its first measure: the clef, the key and, on the first one, the time signature.
fn prefix_width(options: &Options, key: i32) -> f32 {
    let notation = if options.notation { CLEF + if key != 0 { key.unsigned_abs() as f32 + 0.8 } else { 0.0 } } else { 0.0 };
    let tab = if options.tab { 2.8 } else { 0.0 };
    0.6 + notation.max(tab)
}

/// What a measure takes besides its beats: a time signature where it changes, and the marks of a repeat.
fn measure_extra(score: &Score, index: usize) -> f32 {
    let header = &score.headers[index];
    let mut extra = MEASURE_PADDING;
    if index == 0 || changes_time(score, index) {
        extra += TIME_SIGNATURE;
    }
    if header.is_repeat_open {
        extra += 1.4;
    }
    if header.repeat_close > -1 {
        extra += 1.4;
    }
    extra
}

fn changes_time(score: &Score, index: usize) -> bool {
    let (header, previous) = (&score.headers[index], &score.headers[index - 1]);
    previous.numerator != header.numerator || previous.denominator != header.denominator
}

fn break_into_systems(plans: &[MeasurePlan], keys: &[i32], score: &Score, options: &Options) -> Vec<SystemPlan> {
    let mut systems = Vec::new();
    let mut start = 0;
    while start < plans.len() {
        let mut width = prefix_width(options, keys[start]);
        let mut end = start;
        while end < plans.len() {
            let needed = plans[end].body + measure_extra(score, end);
            if end > start && width + needed > options.width {
                break;
            }
            width += needed;
            end += 1;
        }
        systems.push(SystemPlan { measures: start..end });
        start = end;
    }
    systems
}

// ---- painting ----------------------------------------------------------------------------------------------------------------------

/// What a note leaves behind that a later one may join: a tie goes back to the note before, a slur on to the note after.
#[derive(Clone, Copy)]
struct Place {
    x: f32,
    tab_y: f32,
    system: usize,
    fret: i32,
}

#[derive(Clone, Copy, PartialEq)]
enum Join {
    Hammer,
    Slide,
}

struct Painter<'a> {
    score: &'a Score,
    track: &'a Track,
    options: &'a Options,
    keys: &'a [i32],
    plans: &'a [MeasurePlan<'a>],
    /// Where the last note of each string was, for a tie to go back to it.
    ties: HashMap<u8, Place>,
    /// The notes that join the next one on their string, and the number of the beat they were written on: only the beat right after
    /// is joined.
    pending: HashMap<u8, (Place, Join, usize)>,
    /// How many beats of the track have been drawn.
    serial: usize,
}

/// Where the staves of a system are.
#[derive(Clone, Copy)]
struct Staves {
    notation_top: f32,
    tab_top: f32,
    tab_bottom: f32,
    strings: usize,
    system: usize,
    left: f32,
}

impl Staves {
    fn string_y(&self, string: u8) -> f32 {
        self.tab_top + (string as f32 - 1.0) * TAB_GAP
    }

    /// The first line that anything is above.
    fn first_staff(&self, options: &Options) -> f32 {
        if options.notation {
            self.notation_top
        } else {
            self.tab_top
        }
    }
}

/// One beat as the staves see it.
struct Column<'a> {
    beat: &'a Beat,
    /// The middle of its notes.
    x: f32,
    /// Its place in the measure, in ticks.
    offset: u32,
    /// For each note, its position on the staff: the step it is on, and the accidental it needs where it needs one.
    heads: Vec<Head>,
}

struct Head {
    /// Steps up from the C of the octave below the staff's: a position the staff lines are counted in.
    step: i32,
    y: f32,
    accidental: Option<i8>,
}

impl Painter<'_> {
    fn system(&mut self, plan: &SystemPlan, top: f32, number: usize, is_last: bool) -> System {
        let (options, score, plans) = (self.options, self.score, self.plans);
        let strings = self.track.strings.len().max(1);
        let first = plan.measures.start;
        let key = self.keys[first];
        let prefix = prefix_width(options, key);
        let notation_top = top + ABOVE;
        let tab_top = if options.notation { notation_top + 4.0 + BETWEEN } else { top + ABOVE - 1.0 };
        let tab_bottom = tab_top + (strings as f32 - 1.0) * TAB_GAP;
        let bottom = if options.tab { tab_bottom + if options.rhythm { RHYTHM } else { 1.5 } } else { notation_top + 4.0 + 3.5 };
        let staves = Staves { notation_top, tab_top, tab_bottom, strings, system: number, left: 0.0 };
        let right = options.width;
        let mut draws: Vec<Draw> = Vec::new();

        // The measures are stretched alike so that the system is as wide as the window.
        let fixed: f32 = prefix + plan.measures.clone().map(|index| measure_extra(score, index)).sum::<f32>();
        let bodies: f32 = plan.measures.clone().map(|index| plans[index].body).sum();
        let stretch = if bodies <= 0.0 { 1.0 } else { (right - fixed) / bodies };
        let stretch = if is_last { stretch.clamp(1.0, 1.4) } else { stretch.max(1.0) };

        let mut x = prefix;
        for index in plan.measures.clone() {
            let header = &score.headers[index];
            let measure_left = x;
            let mut cursor = x;
            if header.is_repeat_open {
                cursor += 1.4;
            }
            if index == 0 || changes_time(score, index) {
                self.time_signature(&mut draws, cursor + 0.3, header, &staves);
                cursor += TIME_SIGNATURE;
            }
            let body_left = cursor + MEASURE_PADDING * 0.6;
            let body_width = plans[index].body * stretch;
            let bar = body_left + body_width + MEASURE_PADDING * 0.4 + if header.repeat_close > -1 { 1.4 } else { 0.0 };
            self.bar_line_start(&mut draws, measure_left, header, &staves);
            self.paint_measure(&mut draws, index, body_left, stretch, &staves);
            let is_final = index + 1 == self.track.measures.len().min(score.headers.len());
            self.bar_line_end(&mut draws, bar, header, is_final, &staves);
            if index == first {
                draws.push(Draw::text(measure_left + 0.2, staves.first_staff(options) - 1.1, (index + 1).to_string(), 0.8, Anchor::Left, TextStyle::Normal, Tone::Muted));
            }
            if let Some(marker) = &header.marker {
                draws.push(Draw::text(body_left - 0.3, top + 0.7, marker.clone(), 1.15, Anchor::Left, TextStyle::Bold, Tone::Accent));
            }
            x = bar;
        }
        // The staff lines run to the last bar, and the system's start is marked.
        let line_right = if is_last { x } else { right.max(x) };
        let mut under: Vec<Draw> = Vec::new();
        if options.notation {
            for line in 0..5 {
                let y = notation_top + line as f32;
                under.push(Draw::line(0.0, y, line_right, y, LINE));
            }
            draws.extend(glyphs::treble_clef(0.6).into_iter().map(|draw| draw.moved(0.0, notation_top)));
            if key != 0 {
                draws.extend(glyphs::key_signature(0.6 + CLEF, key, 1.0).into_iter().map(|draw| draw.moved(0.0, notation_top)));
            }
            draws.push(Draw::text(1.5, notation_top + 6.4, "8", 0.9, Anchor::Center, TextStyle::Normal, Tone::Ink));
        }
        if options.tab {
            for string in 1..=strings as u8 {
                let y = staves.string_y(string);
                under.push(Draw::line(0.0, y, line_right, y, LINE));
            }
            let spread = (strings as f32 - 1.0) * TAB_GAP;
            for (position, letter) in ["T", "A", "B"].iter().enumerate() {
                draws.push(Draw::text(1.5, tab_top + spread * (0.22 + position as f32 * 0.28), *letter, 1.6, Anchor::Center, TextStyle::Bold, Tone::Ink));
            }
        }
        let join_bottom = if options.tab { tab_bottom } else { notation_top + 4.0 };
        under.push(Draw::line(0.0, staves.first_staff(options), 0.0, join_bottom, LINE * 1.5));
        under.extend(draws);
        System { top, height: bottom - top, first_measure: first, last_measure: plan.measures.end - 1, draws: under }
    }

    fn time_signature(&self, draws: &mut Vec<Draw>, x: f32, header: &MeasureHeader, staves: &Staves) {
        let center = x + 0.9;
        if self.options.notation {
            draws.push(Draw::text(center, staves.notation_top + 1.0, header.numerator.to_string(), 2.1, Anchor::Center, TextStyle::Bold, Tone::Ink));
            draws.push(Draw::text(center, staves.notation_top + 3.0, header.denominator.to_string(), 2.1, Anchor::Center, TextStyle::Bold, Tone::Ink));
        }
        if self.options.tab {
            let middle = (staves.tab_top + staves.tab_bottom) / 2.0;
            let spread = (staves.strings as f32 - 1.0) * TAB_GAP / 4.0;
            draws.push(Draw::text(center, middle - spread - 0.15, header.numerator.to_string(), 1.7, Anchor::Center, TextStyle::Bold, Tone::Ink));
            draws.push(Draw::text(center, middle + spread + 0.15, header.denominator.to_string(), 1.7, Anchor::Center, TextStyle::Bold, Tone::Ink));
        }
    }

    /// A vertical line across the staves.
    fn bar(&self, draws: &mut Vec<Draw>, x: f32, width: f32, staves: &Staves) {
        let options = self.options;
        if options.notation {
            draws.push(Draw::line(x, staves.notation_top, x, staves.notation_top + 4.0, width));
        }
        if options.tab {
            draws.push(Draw::line(x, staves.tab_top, x, staves.tab_bottom, width));
        }
    }

    fn dots(&self, draws: &mut Vec<Draw>, x: f32, staves: &Staves) {
        if self.options.notation {
            draws.extend(glyphs::repeat_dots(x).into_iter().map(|draw| draw.moved(0.0, staves.notation_top)));
        }
        if self.options.tab {
            let middle = (staves.tab_top + staves.tab_bottom) / 2.0;
            draws.push(Draw::dot(x, middle - TAB_GAP * 0.5, 0.2));
            draws.push(Draw::dot(x, middle + TAB_GAP * 0.5, 0.2));
        }
    }

    fn bar_line_start(&self, draws: &mut Vec<Draw>, x: f32, header: &MeasureHeader, staves: &Staves) {
        if header.is_repeat_open {
            self.bar(draws, x + 0.05, 0.4, staves);
            self.bar(draws, x + 0.6, LINE, staves);
            self.dots(draws, x + 1.1, staves);
        } else if x > 1.0 {
            self.bar(draws, x, LINE, staves);
        }
    }

    fn bar_line_end(&self, draws: &mut Vec<Draw>, x: f32, header: &MeasureHeader, is_final: bool, staves: &Staves) {
        if header.repeat_close > -1 {
            self.dots(draws, x - 1.3, staves);
            self.bar(draws, x - 0.6, LINE, staves);
            self.bar(draws, x - 0.05, 0.4, staves);
            if header.repeat_close > 1 {
                let top = staves.first_staff(self.options) - 0.9;
                draws.push(Draw::text(x - 0.4, top, format!("x{}", header.repeat_close), 0.9, Anchor::Right, TextStyle::Italic, Tone::Muted));
            }
        } else if is_final {
            self.bar(draws, x - 0.5, LINE, staves);
            self.bar(draws, x - 0.05, 0.4, staves);
        } else if header.has_double_bar {
            self.bar(draws, x - 0.4, LINE, staves);
            self.bar(draws, x, LINE, staves);
        } else {
            self.bar(draws, x, LINE, staves);
        }
    }

    // ---- a measure ---------------------------------------------------------------------------------------------------------------

    fn paint_measure(&mut self, draws: &mut Vec<Draw>, index: usize, body_left: f32, stretch: f32, staves: &Staves) {
        let plans = self.plans;
        let plan = &plans[index];
        let header = &self.score.headers[index];
        let key = self.keys[index];
        let measure_start = header.start;
        // The columns: where each beat's notes are.
        let mut columns: Vec<Column> = Vec::new();
        let mut accidentals: HashMap<i32, i8> = HashMap::new();
        for beat_plan in &plan.beats {
            let grace = if beat_plan.beat.notes.iter().any(|note| note.effects.grace.is_some()) { 1.2 } else { 0.0 };
            let x = body_left + beat_plan.x * stretch + 0.9 + grace;
            let mut heads = Vec::new();
            if self.options.notation && beat_plan.beat.status != BeatStatus::Rest {
                for note in &beat_plan.beat.notes {
                    heads.push(self.head_of(note, key, &mut accidentals));
                }
            }
            columns.push(Column { beat: beat_plan.beat, x, offset: beat_plan.beat.start.saturating_sub(measure_start), heads });
        }
        let groups = beam_groups(&columns, header);
        // Which way each column's stem goes.
        let mut stems_up = vec![true; columns.len()];
        if self.options.notation {
            for group in &groups {
                let up = stem_is_up(group.iter().flat_map(|&column| columns[column].heads.iter().map(|head| head.step)));
                for &column in group {
                    stems_up[column] = up;
                }
            }
        }
        if self.options.notation {
            self.paint_notation(draws, &columns, &groups, &stems_up, staves);
        }
        if self.options.tab {
            self.paint_tab(draws, &columns, staves);
            if self.options.rhythm {
                self.paint_rhythm(draws, &columns, &groups, staves);
            }
        }
        self.paint_labels(draws, &columns, index, staves);
    }

    /// Where a note is on the staff, and whether it needs an accidental there.
    fn head_of(&self, note: &Note, key: i32, accidentals: &mut HashMap<i32, i8>) -> Head {
        let strings = &self.track.strings;
        let tuning = strings.get(note.string as usize - 1).copied().unwrap_or(40);
        // A guitar is written an octave above where it sounds.
        let (step, alteration) = pitch_position(tuning + note.fret + 12, key);
        let letter = step.rem_euclid(7) as usize;
        let current = accidentals.get(&step).copied().unwrap_or_else(|| key_alteration(letter, key));
        let accidental = if alteration != current {
            accidentals.insert(step, alteration);
            Some(alteration)
        } else {
            None
        };
        Head { step, y: 4.0 - (step - 30) as f32 * 0.5, accidental }
    }

    // ---- the notation ----------------------------------------------------------------------------------------------------------

    fn paint_notation(&mut self, draws: &mut Vec<Draw>, columns: &[Column], groups: &[Vec<usize>], stems_up: &[bool], staves: &Staves) {
        let top = staves.notation_top;
        let mut ends: Vec<Option<StemEnd>> = vec![None; columns.len()];
        for (position, column) in columns.iter().enumerate() {
            let beat = column.beat;
            let value = beat.duration.value;
            if beat.status == BeatStatus::Rest || column.heads.is_empty() {
                draws.extend(glyphs::rest(column.x, value).into_iter().map(|draw| draw.moved(0.0, top)));
                if beat.duration.is_dotted {
                    draws.push(Draw::dot(column.x + 0.9, top + 1.4, 0.17));
                }
                continue;
            }
            let up = stems_up[position];
            // Heads that are a step apart do not fit side by side on one stem: the second goes to the other side of it.
            let mut ordered: Vec<usize> = (0..column.heads.len()).collect();
            ordered.sort_by_key(|&head| -column.heads[head].step);
            let mut shifted = vec![false; column.heads.len()];
            for pair in 1..ordered.len() {
                let (a, b) = (ordered[pair - 1], ordered[pair]);
                if (column.heads[a].step - column.heads[b].step).abs() == 1 && !shifted[a] {
                    shifted[b] = true;
                }
            }
            let side = if up { 1.0 } else { -1.0 };
            let mut accidental_slot = 0.0;
            for (head_index, head) in column.heads.iter().enumerate() {
                let head_x = column.x + if shifted[head_index] { side * (glyphs::HEAD_RX * 2.0 - 0.1) } else { 0.0 };
                let y = top + head.y;
                // The ledger lines the note needs.
                if head.y < -0.5 {
                    let mut line = -1.0;
                    while line >= head.y - 0.01 {
                        draws.push(Draw::line(head_x - 1.0, top + line, head_x + 1.0, top + line, LINE * 1.4));
                        line -= 1.0;
                    }
                } else if head.y > 4.5 {
                    let mut line = 5.0;
                    while line <= head.y + 0.01 {
                        draws.push(Draw::line(head_x - 1.0, top + line, head_x + 1.0, top + line, LINE * 1.4));
                        line += 1.0;
                    }
                }
                draws.push(glyphs::notehead(head_x, y, value));
                if let Some(alteration) = head.accidental {
                    let sign_x = column.x - 1.5 - accidental_slot;
                    let sign = match alteration {
                        1 => glyphs::sharp(sign_x, y),
                        -1 => glyphs::flat(sign_x, y),
                        _ => glyphs::natural(sign_x, y),
                    };
                    draws.extend(sign);
                    accidental_slot += 0.9;
                }
                if beat.duration.is_dotted {
                    let dot_y = if (head.y.fract() - 0.0).abs() < 0.01 { y - 0.5 } else { y };
                    draws.push(Draw::dot(head_x + 1.0, dot_y, 0.17));
                }
            }
            if value >= 2 {
                let near = column.heads.iter().map(|head| head.y).fold(f32::INFINITY, f32::min);
                let far = column.heads.iter().map(|head| head.y).fold(f32::NEG_INFINITY, f32::max);
                let stem_x = column.x + if up { glyphs::HEAD_RX - 0.04 } else { -(glyphs::HEAD_RX - 0.04) };
                ends[position] = Some(if up { StemEnd { x: stem_x, from: far, tip: (near - 3.3).min(2.0), up } } else { StemEnd { x: stem_x, from: near, tip: (far + 3.3).max(2.0), up } });
            }
        }
        self.stems_and_beams(draws, columns, groups, &ends, top);
        self.tuplets(draws, columns, &ends, top);
    }

    /// The stems, the flags of the notes alone and the beams of the groups, from the ends worked out for the stems.
    fn stems_and_beams(&self, draws: &mut Vec<Draw>, columns: &[Column], groups: &[Vec<usize>], ends: &[Option<StemEnd>], top: f32) {
        // A group's stems all reach its beam.
        let mut tips: Vec<Option<f32>> = ends.iter().map(|end| end.map(|end| end.tip)).collect();
        for group in groups {
            let present: Vec<usize> = group.iter().copied().filter(|&column| ends[column].is_some()).collect();
            if present.len() < 2 {
                continue;
            }
            let up = ends[present[0]].map_or(true, |end| end.up);
            let tip = present.iter().filter_map(|&column| ends[column].map(|end| end.tip)).fold(if up { f32::INFINITY } else { f32::NEG_INFINITY }, |best, tip| if up { best.min(tip) } else { best.max(tip) });
            for &column in &present {
                tips[column] = Some(tip);
            }
        }
        let grouped: Vec<bool> = {
            let mut flags = vec![false; columns.len()];
            for group in groups {
                if group.iter().filter(|&&column| ends[column].is_some()).count() >= 2 {
                    for &column in group {
                        flags[column] = true;
                    }
                }
            }
            flags
        };
        for (position, end) in ends.iter().enumerate() {
            let (Some(end), Some(tip)) = (end, tips[position]) else {
                continue;
            };
            draws.push(Draw::line(end.x, top + end.from, end.x, top + tip, STEM));
            let value = columns[position].beat.duration.value;
            if !grouped[position] && value >= 8 {
                draws.extend(glyphs::flags(end.x, top + tip, end.up, flag_count(value)));
            }
        }
        for group in groups {
            let present: Vec<usize> = group.iter().copied().filter(|&column| ends[column].is_some()).collect();
            if present.len() < 2 {
                continue;
            }
            let up = ends[present[0]].map_or(true, |end| end.up);
            let tip = tips[present[0]].unwrap_or(0.0) + top;
            let xs: Vec<f32> = present.iter().filter_map(|&column| ends[column].map(|end| end.x)).collect();
            let values: Vec<u8> = present.iter().map(|&column| columns[column].beat.duration.value).collect();
            draw_beams(draws, &xs, &values, tip, up);
        }
    }

    fn tuplets(&self, draws: &mut Vec<Draw>, columns: &[Column], ends: &[Option<StemEnd>], top: f32) {
        let mut position = 0;
        while position < columns.len() {
            let duration = columns[position].beat.duration;
            if duration.enters > 1 {
                let mut end = position;
                while end + 1 < columns.len() && columns[end + 1].beat.duration.enters == duration.enters && columns[end + 1].beat.duration.times == duration.times && end + 1 - position < duration.enters as usize {
                    end += 1;
                }
                let x = (columns[position].x + columns[end].x) / 2.0;
                let up = ends[position].map_or(true, |stem| stem.up);
                let tip = (position..=end).filter_map(|column| ends[column].map(|stem| stem.tip)).fold(if up { f32::INFINITY } else { f32::NEG_INFINITY }, |best, tip| if up { best.min(tip) } else { best.max(tip) });
                let y = if tip.is_finite() { top + tip + if up { -0.9 } else { 0.9 } } else { top - 0.8 };
                draws.push(Draw::text(x, y, duration.enters.to_string(), 0.9, Anchor::Center, TextStyle::Italic, Tone::Ink));
                position = end + 1;
            } else {
                position += 1;
            }
        }
    }

    // ---- the tablature ---------------------------------------------------------------------------------------------------------

    fn paint_tab(&mut self, draws: &mut Vec<Draw>, columns: &[Column], staves: &Staves) {
        for column in columns {
            let beat = column.beat;
            self.serial += 1;
            let serial = self.serial;
            self.pending.retain(|_, pending| pending.2 + 1 >= serial);
            for note in &beat.notes {
                let string = note.string;
                if string as usize > staves.strings {
                    continue;
                }
                let y = staves.string_y(string);
                let effects = &note.effects;
                let mut text = match note.kind {
                    NoteKind::Dead => "x".to_string(),
                    _ => note.fret.to_string(),
                };
                if note.kind == NoteKind::Tie || effects.ghost {
                    text = format!("({text})");
                }
                if effects.harmonic {
                    text = format!("<{text}>");
                }
                draws.push(Draw::Text { x: column.x, y, text, size: 1.0, anchor: Anchor::Center, style: TextStyle::Normal, masks: true, tone: Tone::Ink });
                if let Some(fret) = effects.grace {
                    draws.push(Draw::Text { x: column.x - 1.5, y, text: fret.to_string(), size: 0.7, anchor: Anchor::Center, style: TextStyle::Normal, masks: true, tone: Tone::Ink });
                }
                let place = Place { x: column.x, tab_y: y, system: staves.system, fret: note.fret };
                self.join_tab(draws, note, place, staves, serial);
                self.marks_of(draws, note, column.x, y, staves);
            }
        }
    }

    /// Draws what the note joins to the one before it, and keeps what it joins to the one after.
    fn join_tab(&mut self, draws: &mut Vec<Draw>, note: &Note, place: Place, staves: &Staves, serial: usize) {
        let string = note.string;
        if note.kind == NoteKind::Tie {
            match self.ties.get(&string) {
                Some(before) if before.system == place.system => draws.push(arc(before.x + 0.5, place.tab_y - 0.55, place.x - 0.5, place.tab_y - 0.55, -0.55, 0.12)),
                _ => draws.push(arc(staves.left + 3.4, place.tab_y - 0.55, place.x - 0.5, place.tab_y - 0.55, -0.45, 0.12)),
            }
        }
        if let Some((before, join, _)) = self.pending.remove(&string) {
            if before.system == place.system {
                match join {
                    Join::Hammer => draws.push(arc(before.x + 0.2, before.tab_y - 0.65, place.x - 0.2, place.tab_y - 0.65, -0.7, 0.12)),
                    Join::Slide => {
                        let rise = if place.fret >= before.fret { -0.35 } else { 0.35 };
                        draws.push(Draw::line(before.x + 0.55, before.tab_y - rise, place.x - 0.55, place.tab_y + rise, 0.12));
                    }
                }
            }
        }
        if note.effects.hammer {
            self.pending.insert(string, (place, Join::Hammer, serial));
        } else if matches!(note.effects.slide, Some(Slide::Shift) | Some(Slide::Legato)) {
            self.pending.insert(string, (place, Join::Slide, serial));
        }
        self.ties.insert(string, place);
    }

    /// The marks over and beside a fret: a bend's arrow, a vibrato, how the note is held.
    fn marks_of(&self, draws: &mut Vec<Draw>, note: &Note, x: f32, y: f32, staves: &Staves) {
        let effects = &note.effects;
        if let Some(bend) = &effects.bend {
            let peak = bend.points.iter().map(|point| point.1).max().unwrap_or(0);
            let label = match peak {
                ..=0 => "",
                1 | 2 => "½",
                3 | 4 => "full",
                5 | 6 => "1½",
                _ => "2",
            };
            if !label.is_empty() {
                let top = y - 1.9;
                draws.push(Draw::polyline(bezier((x + 0.45, y - 0.3), (x + 1.2, y - 0.4), (x + 1.4, y - 1.0), (x + 1.4, top + 0.4)), 0.12));
                draws.push(Draw::polygon(vec![(x + 1.4, top), (x + 1.15, top + 0.5), (x + 1.65, top + 0.5)]));
                draws.push(Draw::text(x + 1.4, top - 0.45, label, 0.75, Anchor::Center, TextStyle::Normal, Tone::Ink));
            }
        }
        if effects.vibrato {
            let base = staves.tab_top - 1.0;
            let points: Vec<(f32, f32)> = (0..=16).map(|step| (x - 0.8 + step as f32 * 0.16, base + (step as f32 * 1.4).sin() * 0.14)).collect();
            draws.push(Draw::polyline(points, 0.1));
        }
        match effects.slide {
            Some(Slide::OutDown) => draws.push(Draw::line(x + 0.5, y - 0.15, x + 1.4, y + 0.45, 0.12)),
            Some(Slide::OutUp) => draws.push(Draw::line(x + 0.5, y + 0.15, x + 1.4, y - 0.45, 0.12)),
            Some(Slide::InFromBelow) => draws.push(Draw::line(x - 1.4, y + 0.45, x - 0.5, y - 0.15, 0.12)),
            Some(Slide::InFromAbove) => draws.push(Draw::line(x - 1.4, y - 0.45, x - 0.5, y + 0.15, 0.12)),
            _ => {}
        }
        let label_y = staves.tab_top - 1.9;
        if effects.palm_mute {
            draws.push(Draw::text(x - 0.6, label_y, "P.M.", 0.7, Anchor::Left, TextStyle::Italic, Tone::Muted));
        } else if effects.let_ring {
            draws.push(Draw::text(x - 0.6, label_y, "let ring", 0.7, Anchor::Left, TextStyle::Italic, Tone::Muted));
        }
        if effects.accent || effects.heavy_accent {
            draws.push(Draw::text(x, staves.tab_top - 0.9, if effects.heavy_accent { "^" } else { ">" }, 0.9, Anchor::Center, TextStyle::Bold, Tone::Ink));
        }
        if effects.trill {
            draws.push(Draw::text(x, staves.tab_top - 0.9, "tr", 0.8, Anchor::Center, TextStyle::Italic, Tone::Ink));
        }
    }

    /// The stems and the beams that say how long the notes are, under the tablature.
    fn paint_rhythm(&self, draws: &mut Vec<Draw>, columns: &[Column], groups: &[Vec<usize>], staves: &Staves) {
        let base = staves.tab_bottom + 0.9;
        let tip = base + 2.4;
        let mut grouped = vec![false; columns.len()];
        for group in groups {
            if group.iter().filter(|&&column| columns[column].beat.status != BeatStatus::Rest).count() >= 2 {
                for &column in group {
                    grouped[column] = true;
                }
            }
        }
        for (position, column) in columns.iter().enumerate() {
            let beat = column.beat;
            if beat.status == BeatStatus::Rest {
                continue;
            }
            let value = beat.duration.value;
            if value >= 2 {
                draws.push(Draw::line(column.x, base, column.x, tip, STEM));
            }
            if value >= 8 && !grouped[position] {
                // Flags hang from the stem's end down the page: a stem pointing up has them below it.
                draws.extend(glyphs::flags(column.x, tip, false, flag_count(value)).into_iter().map(|draw| mirrored_down(draw, tip)));
            }
            if beat.duration.is_dotted {
                draws.push(Draw::dot(column.x + 0.6, tip + 0.3, 0.17));
            }
            if value == 1 {
                draws.push(Draw::line(column.x - 0.35, base, column.x + 0.35, base, 0.15));
            }
        }
        for group in groups {
            let present: Vec<usize> = group.iter().copied().filter(|&column| columns[column].beat.status != BeatStatus::Rest).collect();
            if present.len() < 2 {
                continue;
            }
            let xs: Vec<f32> = present.iter().map(|&column| columns[column].x).collect();
            let values: Vec<u8> = present.iter().map(|&column| columns[column].beat.duration.value).collect();
            // Stems point down from the tablature here, so the beams hang from their ends upwards, drawn as stems that point down.
            draw_beams(draws, &xs, &values, tip, false);
        }
    }

    // ---- what is written over the staves ---------------------------------------------------------------------------------------

    fn paint_labels(&self, draws: &mut Vec<Draw>, columns: &[Column], index: usize, staves: &Staves) {
        let first = staves.first_staff(self.options);
        let chord_y = first - 3.9;
        let text_y = first - 2.5;
        for (position, column) in columns.iter().enumerate() {
            if let Some(chord) = &column.beat.chord {
                draws.push(Draw::text(column.x - 0.9, chord_y, chord.clone(), 1.15, Anchor::Left, TextStyle::Bold, Tone::Accent));
            }
            if let Some(text) = &column.beat.text {
                draws.push(Draw::text(column.x - 0.9, text_y, text.clone(), 0.95, Anchor::Left, TextStyle::Italic, Tone::Ink));
            }
            let tempo = column.beat.tempo.or_else(|| if index == 0 && position == 0 { Some(self.score.tempo) } else { None });
            if let Some(tempo) = tempo.filter(|&tempo| tempo > 0) {
                let (x, y) = (column.x - 0.9, first - 5.4);
                draws.push(Draw::Ellipse { x: x + 0.5, y: y + 0.55, rx: 0.4, ry: 0.3, angle: -0.4, filled: true, width: 0.0, tone: Tone::Ink });
                draws.push(Draw::line(x + 0.82, y + 0.55, x + 0.82, y - 0.7, 0.1));
                draws.push(Draw::text(x + 1.5, y, format!("= {tempo}"), 0.9, Anchor::Left, TextStyle::Normal, Tone::Ink));
            }
        }
    }
}

// ---- stems and beams ---------------------------------------------------------------------------------------------------------------

#[derive(Clone, Copy)]
struct StemEnd {
    x: f32,
    /// The head the stem starts at: the one farthest from where it ends.
    from: f32,
    tip: f32,
    up: bool,
}

fn flag_count(value: u8) -> u8 {
    match value {
        8 => 1,
        16 => 2,
        32 => 3,
        _ => 4,
    }
}

/// Draws a flag drawn for a stem that points down as if for one that points up from `tip` instead: reflected through `tip`.
fn mirrored_down(draw: Draw, tip: f32) -> Draw {
    match draw {
        Draw::Polyline { points, width, tone } => Draw::Polyline { points: points.into_iter().map(|(x, y)| (x, 2.0 * tip - y)).collect(), width, tone },
        other => other,
    }
}

/// The beams of a group whose stems are at `xs` and last `values`: the first across all of them, and one more for each extra flag, for as
/// many of them in a row as have it, or a short stub where only one does.
fn draw_beams(draws: &mut Vec<Draw>, xs: &[f32], values: &[u8], tip: f32, up: bool) {
    let sign = if up { 1.0 } else { -1.0 };
    let levels = values.iter().map(|&value| flag_count(value)).max().unwrap_or(1);
    for level in 1..=levels {
        let offset = (level as f32 - 1.0) * 0.85 * sign;
        let mut index = 0;
        while index < xs.len() {
            if flag_count(values[index]) < level {
                index += 1;
                continue;
            }
            let mut end = index;
            while end + 1 < xs.len() && flag_count(values[end + 1]) >= level {
                end += 1;
            }
            let (from, to) = if end > index {
                (xs[index], xs[end] + STEM / 2.0)
            } else if index + 1 < xs.len() {
                (xs[index], xs[index] + 1.2)
            } else {
                (xs[index] - 1.2, xs[index])
            };
            let y1 = tip + offset;
            let y2 = y1 + BEAM * sign;
            draws.push(Draw::polygon(vec![(from, y1), (to, y1), (to, y2), (from, y2)]));
            index = end + 1;
        }
    }
}

/// Whether the stems of notes at `steps` go up: they do unless the note farthest from the middle line is above it.
fn stem_is_up(steps: impl Iterator<Item = i32>) -> bool {
    let steps: Vec<i32> = steps.collect();
    let (Some(&highest), Some(&lowest)) = (steps.iter().max(), steps.iter().min()) else {
        return true;
    };
    (highest - MIDDLE_LINE) <= (MIDDLE_LINE - lowest)
}

/// The beams a measure groups its beats in: the beats of the same part of the measure that are short enough, and have notes, joined.
fn beam_groups(columns: &[Column], header: &MeasureHeader) -> Vec<Vec<usize>> {
    let window = if header.denominator == 8 && header.numerator % 3 == 0 && header.numerator >= 6 { QUARTER_TICKS * 3 / 2 } else { QUARTER_TICKS };
    let mut groups: Vec<Vec<usize>> = Vec::new();
    let mut last_window = None;
    for (position, column) in columns.iter().enumerate() {
        let beat = column.beat;
        let beamable = beat.status != BeatStatus::Rest && beat.duration.value >= 8 && !beat.notes.is_empty();
        if !beamable {
            groups.push(vec![position]);
            last_window = None;
            continue;
        }
        let current = column.offset / window;
        if last_window == Some(current) {
            if let Some(group) = groups.last_mut() {
                group.push(position);
            }
        } else {
            groups.push(vec![position]);
        }
        last_window = Some(current);
    }
    groups
}

// ---- pitch --------------------------------------------------------------------------------------------------------------------------

/// The step on the staff a MIDI note is written on, counted in notes of the scale from a C (the C of the octave below the staff's is the
/// first), and how the note is altered there: 1 sharp, -1 flat, 0 as it is. The spelling follows the key: sharps in a sharp key and in C
/// major, flats in a flat one.
pub(crate) fn pitch_position(midi: i32, key: i32) -> (i32, i8) {
    const SHARPS: [(i32, i8); 12] = [(0, 0), (0, 1), (1, 0), (1, 1), (2, 0), (3, 0), (3, 1), (4, 0), (4, 1), (5, 0), (5, 1), (6, 0)];
    const FLATS: [(i32, i8); 12] = [(0, 0), (1, -1), (1, 0), (2, -1), (2, 0), (3, 0), (4, -1), (4, 0), (5, -1), (5, 0), (6, -1), (6, 0)];
    let octave = midi.div_euclid(12) - 1;
    let (letter, alteration) = if key < 0 { FLATS } else { SHARPS }[midi.rem_euclid(12) as usize];
    (octave * 7 + letter, alteration)
}

/// How the key alters a note of this letter (0 for C, 1 for D... 6 for B).
pub(crate) fn key_alteration(letter: usize, key: i32) -> i8 {
    const SHARP_ORDER: [usize; 7] = [3, 0, 4, 1, 5, 2, 6];
    const FLAT_ORDER: [usize; 7] = [6, 2, 5, 1, 4, 0, 3];
    let count = key.unsigned_abs().min(7) as usize;
    if key > 0 && SHARP_ORDER[..count].contains(&letter) {
        1
    } else if key < 0 && FLAT_ORDER[..count].contains(&letter) {
        -1
    } else {
        0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_note_is_written_on_the_step_of_its_letter() {
        // E4, the bottom line of the treble staff, and the G above it.
        assert_eq!(pitch_position(64, 0), (30, 0));
        assert_eq!(pitch_position(67, 0), (32, 0));
        // A sharp key spells its black keys with sharps, a flat one with flats.
        assert_eq!(pitch_position(66, 2), (31, 1));
        assert_eq!(pitch_position(70, -1), (34, -1));
    }

    #[test]
    fn a_key_alters_the_letters_it_names() {
        assert_eq!((0..7).map(|letter| key_alteration(letter, 0)).collect::<Vec<_>>(), [0; 7]);
        // G major has an F sharp, D major an F and a C sharp, F major a B flat.
        assert_eq!(key_alteration(3, 1), 1);
        assert_eq!(key_alteration(0, 1), 0);
        assert_eq!(key_alteration(0, 2), 1);
        assert_eq!(key_alteration(6, -1), -1);
        assert_eq!(key_alteration(2, -1), 0);
    }

    #[test]
    fn stems_go_away_from_the_notes_farthest_from_the_middle_line() {
        assert!(stem_is_up([30, 32].into_iter()));
        assert!(!stem_is_up([36, 38].into_iter()));
        assert!(stem_is_up([30, 36].into_iter()));
        assert!(stem_is_up([28, 38].into_iter()));
        assert!(!stem_is_up([30, 40].into_iter()));
    }
}

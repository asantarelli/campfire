// This file is part of Campfire.
// Copyright (c) Pandula Péter 2017-2026.
// https://github.com/pandulapeter/campfire
//
// This Source Code Form is subject to the terms of the Mozilla Public License, v. 2.0.
// If a copy of the MPL was not distributed with this file, You can obtain one at
// https://mozilla.org/MPL/2.0/.

//! The click of the metronome, as samples.
//!
//! Timing is by sample count, never by a timer: the sound card's own clock is the one the player hears, so the clicks stay
//! where the tempo puts them for as long as it runs, whatever else the computer is doing.

use std::f64::consts::TAU;

/// The slowest and the fastest tempo the metronome plays, in beats per minute.
pub const TEMPO_RANGE: std::ops::RangeInclusive<u32> = 30..=300;
pub const DEFAULT_TEMPO: u32 = 100;

/// How long a click sounds.
const CLICK_SECONDS: f64 = 0.03;
const ACCENT_FREQUENCY: f64 = 1600.0;
const BEAT_FREQUENCY: f64 = 1000.0;
const SUBDIVISION_FREQUENCY: f64 = 700.0;
const AMPLITUDE: f32 = 0.7;

/// Which beats of a bar are heard louder.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Accent {
    /// Only the first beat of the bar.
    FirstBeat,
    /// Every beat the same.
    None,
}

pub struct Metronome {
    sample_rate: f64,
    tempo: u32,
    beats_per_bar: u32,
    /// The note each beat is: 4 for a quarter note, 8 for an eighth.
    beat_unit: u32,
    accent: Accent,
    /// A softer click between two beats.
    has_subdivision: bool,
    /// How many samples have gone by since the last beat started, with the fraction a tempo that does not divide the sample rate leaves.
    position: f64,
    /// Which beat of the bar the next click is.
    beat_in_bar: u32,
    /// The click that is sounding: how many samples of it have been played and at what frequency.
    click_samples: usize,
    click_frequency: f64,
    click_is_audible: bool,
    /// How many beats have started, which the screen reads to flash.
    beats_started: u64,
    is_first_sample: bool,
    subdivision_done: bool,
}

impl Metronome {
    pub fn new(sample_rate: f32) -> Metronome {
        Metronome {
            sample_rate: sample_rate as f64,
            tempo: DEFAULT_TEMPO,
            beats_per_bar: 4,
            beat_unit: 4,
            accent: Accent::FirstBeat,
            has_subdivision: false,
            position: 0.0,
            beat_in_bar: 0,
            click_samples: 0,
            click_frequency: ACCENT_FREQUENCY,
            click_is_audible: false,
            beats_started: 0,
            is_first_sample: true,
            subdivision_done: false,
        }
    }

    pub fn set_tempo(&mut self, tempo: u32) {
        self.tempo = tempo.clamp(*TEMPO_RANGE.start(), *TEMPO_RANGE.end());
    }

    pub fn tempo(&self) -> u32 {
        self.tempo
    }

    /// The bar the metronome counts: `beats` beats of a `unit` note (3 and 4 for 3/4).
    pub fn set_time_signature(&mut self, beats: u32, unit: u32) {
        self.beats_per_bar = beats.clamp(1, 16);
        self.beat_unit = if [1, 2, 4, 8, 16].contains(&unit) { unit } else { 4 };
        self.beat_in_bar = self.beat_in_bar.min(self.beats_per_bar - 1);
    }

    pub fn set_accent(&mut self, accent: Accent) {
        self.accent = accent;
    }

    pub fn set_subdivision(&mut self, has_subdivision: bool) {
        self.has_subdivision = has_subdivision;
    }

    /// How many beats have started since the metronome was made.
    pub fn beats_started(&self) -> u64 {
        self.beats_started
    }

    /// The beat of the bar that sounded last, counted from 0, `None` before the first.
    pub fn current_beat(&self) -> Option<u32> {
        (self.beats_started > 0).then(|| (self.beat_in_bar + self.beats_per_bar - 1) % self.beats_per_bar)
    }

    /// How many samples a beat lasts at the tempo and the note the beat is.
    pub fn samples_per_beat(&self) -> f64 {
        self.sample_rate * 60.0 / self.tempo as f64 * 4.0 / self.beat_unit as f64
    }

    /// Starts again from the first beat of a bar, which sounds at once.
    pub fn restart(&mut self) {
        self.position = 0.0;
        self.beat_in_bar = 0;
        self.is_first_sample = true;
        self.subdivision_done = false;
        self.click_is_audible = false;
    }

    fn start_click(&mut self, frequency: f64) {
        self.click_samples = 0;
        self.click_frequency = frequency;
        self.click_is_audible = true;
    }

    /// Fills `buffer` with the next samples, mono: the caller copies them to every channel it plays.
    pub fn fill(&mut self, buffer: &mut [f32]) {
        let samples_per_beat = self.samples_per_beat();
        let click_length = (CLICK_SECONDS * self.sample_rate) as usize;
        for sample in buffer.iter_mut() {
            if self.is_first_sample || self.position >= samples_per_beat {
                if !self.is_first_sample {
                    self.position -= samples_per_beat;
                }
                self.is_first_sample = false;
                self.subdivision_done = false;
                let is_accented = self.accent == Accent::FirstBeat && self.beat_in_bar == 0;
                self.start_click(if is_accented { ACCENT_FREQUENCY } else { BEAT_FREQUENCY });
                self.beats_started += 1;
                self.beat_in_bar = (self.beat_in_bar + 1) % self.beats_per_bar;
            } else if self.has_subdivision && !self.subdivision_done && self.position >= samples_per_beat / 2.0 {
                self.subdivision_done = true;
                self.start_click(SUBDIVISION_FREQUENCY);
            }
            *sample = if self.click_is_audible && self.click_samples < click_length {
                let seconds = self.click_samples as f64 / self.sample_rate;
                let envelope = (-seconds * 180.0).exp();
                self.click_samples += 1;
                (AMPLITUDE as f64 * envelope * (TAU * self.click_frequency * seconds).sin()) as f32
            } else {
                self.click_is_audible = false;
                0.0
            };
            self.position += 1.0;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The sample every click starts at: where the output goes from silence to sound.
    fn onsets(metronome: &mut Metronome, total: usize) -> Vec<usize> {
        let mut buffer = vec![0.0; total];
        // Filled in the odd-sized pieces a sound card asks for.
        let mut filled = 0;
        let mut size = 509;
        while filled < total {
            let end = (filled + size).min(total);
            metronome.fill(&mut buffer[filled..end]);
            filled = end;
            size = if size == 509 { 1021 } else { 509 };
        }
        let mut found = Vec::new();
        let mut was_silent = true;
        for (index, sample) in buffer.iter().enumerate() {
            let is_silent = sample.abs() < 1e-6;
            if !is_silent && was_silent {
                found.push(index);
            }
            // Only a longer silence than a click's own zero crossings ends a click.
            was_silent = is_silent && buffer[index.saturating_sub(40)..=index].iter().all(|s| s.abs() < 1e-6);
        }
        found
    }

    #[test]
    fn the_clicks_are_where_the_tempo_puts_them_for_as_long_as_it_runs() {
        let mut metronome = Metronome::new(48_000.0);
        metronome.set_tempo(120);
        // Ten minutes: 1200 beats of 24000 samples.
        let found = onsets(&mut metronome, 48_000 * 600);
        assert_eq!(found.len(), 1200);
        for (beat, onset) in found.iter().enumerate() {
            assert!(onset.abs_diff(beat * 24_000) <= 2, "beat {beat} started at {onset}");
        }
    }

    #[test]
    fn a_tempo_that_does_not_divide_the_sample_rate_does_not_drift() {
        let mut metronome = Metronome::new(44_100.0);
        metronome.set_tempo(97);
        let found = onsets(&mut metronome, 44_100 * 300);
        let samples_per_beat: f64 = 44_100.0 * 60.0 / 97.0;
        assert_eq!(found.len(), (44_100.0 * 300.0 / samples_per_beat).ceil() as usize);
        for (beat, onset) in found.iter().enumerate() {
            let expected = beat as f64 * samples_per_beat;
            assert!((*onset as f64 - expected).abs() <= 2.0, "beat {beat}: {onset} instead of {expected}");
        }
    }

    #[test]
    fn the_first_beat_of_the_bar_is_higher_than_the_others() {
        let mut metronome = Metronome::new(48_000.0);
        metronome.set_tempo(60);
        metronome.set_time_signature(3, 4);
        let mut buffer = vec![0.0; 48_000 * 6];
        metronome.fill(&mut buffer);
        // The frequency of a click from the zero crossings in its first millisecond or two.
        let crossings = |start: usize| buffer[start..start + 240].windows(2).filter(|pair| pair[0] <= 0.0 && pair[1] > 0.0).count();
        let (first, second, third, fourth) = (crossings(0), crossings(48_000), crossings(96_000), crossings(144_000));
        assert!(first > second, "{first} {second}");
        assert_eq!(second, third);
        assert_eq!(first, fourth);
    }

    #[test]
    fn a_beat_that_is_an_eighth_note_is_twice_as_fast() {
        let mut metronome = Metronome::new(48_000.0);
        metronome.set_tempo(60);
        assert_eq!(metronome.samples_per_beat(), 48_000.0);
        metronome.set_time_signature(6, 8);
        assert_eq!(metronome.samples_per_beat(), 24_000.0);
    }

    #[test]
    fn a_subdivision_adds_a_softer_click_between_the_beats() {
        let mut metronome = Metronome::new(48_000.0);
        metronome.set_tempo(60);
        metronome.set_subdivision(true);
        let found = onsets(&mut metronome, 48_000 * 4);
        assert_eq!(found.len(), 8);
        assert!(found[1].abs_diff(24_000) <= 2);
    }

    #[test]
    fn the_tempo_is_held_within_what_a_metronome_plays() {
        let mut metronome = Metronome::new(48_000.0);
        metronome.set_tempo(5);
        assert_eq!(metronome.tempo(), 30);
        metronome.set_tempo(900);
        assert_eq!(metronome.tempo(), 300);
    }

    #[test]
    fn the_beat_that_sounded_last_is_known_for_the_screen() {
        let mut metronome = Metronome::new(48_000.0);
        metronome.set_tempo(60);
        metronome.set_time_signature(4, 4);
        assert_eq!(metronome.current_beat(), None);
        metronome.fill(&mut vec![0.0; 100]);
        assert_eq!(metronome.current_beat(), Some(0));
        metronome.fill(&mut vec![0.0; 48_000]);
        assert_eq!(metronome.current_beat(), Some(1));
        assert_eq!(metronome.beats_started(), 2);
        metronome.restart();
        metronome.fill(&mut vec![0.0; 10]);
        assert_eq!(metronome.current_beat(), Some(0));
    }

    #[test]
    fn the_output_never_clips() {
        let mut metronome = Metronome::new(48_000.0);
        metronome.set_tempo(300);
        let mut buffer = vec![0.0; 48_000];
        metronome.fill(&mut buffer);
        assert!(buffer.iter().all(|sample| sample.abs() <= 1.0));
        assert!(buffer.iter().any(|sample| sample.abs() > 0.3));
    }
}

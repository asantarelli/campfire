// This file is part of Campfire.
// Copyright (c) Pandula Péter 2017-2026.
// https://github.com/pandulapeter/campfire
//
// This Source Code Form is subject to the terms of the Mozilla Public License, v. 2.0.
// If a copy of the MPL was not distributed with this file, You can obtain one at
// https://mozilla.org/MPL/2.0/.

//! The pitch of a sound, from its samples, and the note it is closest to.
//!
//! The pitch is found with the McLeod method: the normalized square difference of the signal with itself, whose first strong peak
//! is the period. It is not fooled by a string whose second harmonic is louder than its first, which an autocorrelation alone is.

use std::f64::consts::TAU;

/// The lowest and the highest sound the tuner looks for, in hertz: a bass's low E and a ukulele's high notes, with room.
pub const LOWEST_FREQUENCY: f32 = 30.0;
pub const HIGHEST_FREQUENCY: f32 = 1500.0;
pub const DEFAULT_REFERENCE_PITCH: f32 = 440.0;
pub const REFERENCE_PITCH_RANGE: std::ops::RangeInclusive<f32> = 400.0..=480.0;

/// How many samples the detector wants to look at: at least two periods of the lowest sound.
pub fn window_length(sample_rate: f32) -> usize {
    ((sample_rate / LOWEST_FREQUENCY) * 2.0).ceil() as usize
}

/// A sound's pitch.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Pitch {
    pub frequency: f32,
    /// How sure the detector is, from 0 to 1: a clear note is close to 1, noise well under 0.8.
    pub clarity: f32,
}

/// The pitch of `samples`, `None` where they are silence, noise, or too few to hold two periods of anything the tuner looks for.
pub fn detect_pitch(samples: &[f32], sample_rate: f32) -> Option<Pitch> {
    let longest_period = (sample_rate / LOWEST_FREQUENCY) as usize;
    let shortest_period = ((sample_rate / HIGHEST_FREQUENCY) as usize).max(2);
    // Half of the window is what the signal is compared with its shifted self over, so it has to hold the longest period twice.
    if samples.len() < longest_period * 2 {
        return None;
    }
    let window = &samples[samples.len() - longest_period * 2..];
    let energy: f32 = window.iter().map(|sample| sample * sample).sum::<f32>() / window.len() as f32;
    if energy.sqrt() < 0.01 {
        return None;
    }
    let length = window.len() / 2;
    let mut nsdf = vec![0.0f32; longest_period + 1];
    for (lag, value) in nsdf.iter_mut().enumerate().skip(1) {
        let (mut correlation, mut norm) = (0.0f32, 0.0f32);
        for index in 0..length {
            correlation += window[index] * window[index + lag];
            norm += window[index] * window[index] + window[index + lag] * window[index + lag];
        }
        *value = if norm > 0.0 { 2.0 * correlation / norm } else { 0.0 };
    }
    // The peaks of the function: the highest point between each rise above zero and the next fall below it.
    let mut peaks: Vec<usize> = Vec::new();
    let mut index = 1;
    while index < nsdf.len() - 1 && nsdf[index] > 0.0 {
        index += 1;
    }
    while index < nsdf.len() - 1 {
        while index < nsdf.len() - 1 && nsdf[index] <= 0.0 {
            index += 1;
        }
        let mut best: Option<usize> = None;
        while index < nsdf.len() - 1 && nsdf[index] > 0.0 {
            if nsdf[index] > nsdf[index - 1] && nsdf[index] >= nsdf[index + 1] && best.is_none_or(|b| nsdf[index] > nsdf[b]) {
                best = Some(index);
            }
            index += 1;
        }
        if let Some(best) = best.filter(|&lag| lag >= shortest_period) {
            peaks.push(best);
        }
    }
    let highest = peaks.iter().map(|&lag| nsdf[lag]).fold(0.0f32, f32::max);
    if highest < 0.5 {
        return None;
    }
    // The first peak that is nearly as strong as the strongest: a harmonic that is louder than the note does not make an octave.
    let lag = *peaks.iter().find(|&&lag| nsdf[lag] >= 0.9 * highest)?;
    // The top of the parabola through the peak and its two neighbors, which is where the period really is between two samples.
    let (before, at, after) = (nsdf[lag - 1], nsdf[lag], nsdf[lag + 1]);
    let denominator = before - 2.0 * at + after;
    let shift = if denominator.abs() > f32::EPSILON { 0.5 * (before - after) / denominator } else { 0.0 };
    let period = lag as f32 + shift.clamp(-1.0, 1.0);
    Some(Pitch { frequency: sample_rate / period, clarity: at.clamp(0.0, 1.0) })
}

/// The names of the twelve notes, with sharps.
pub const NOTE_NAMES: [&str; 12] = ["C", "C#", "D", "D#", "E", "F", "F#", "G", "G#", "A", "A#", "B"];

/// The note a frequency is closest to.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Note {
    /// The MIDI number of the note: 69 is the A of `reference_pitch`.
    pub midi: i32,
    /// How far above (positive) or below the note the sound is, in cents: a hundredth of a semitone.
    pub cents: f32,
}

impl Note {
    pub fn of(frequency: f32, reference_pitch: f32) -> Note {
        let semitones = 12.0 * (frequency / reference_pitch).log2() + 69.0;
        let midi = semitones.round();
        Note { midi: midi as i32, cents: (semitones - midi) * 100.0 }
    }

    /// `A`, `C#`: the note without its octave.
    pub fn name(&self) -> &'static str {
        NOTE_NAMES[self.midi.rem_euclid(12) as usize]
    }

    /// The octave, as the scientific notation counts it: 4 for the one that has the middle C and the A of 440 hertz.
    pub fn octave(&self) -> i32 {
        self.midi.div_euclid(12) - 1
    }

    pub fn frequency(&self, reference_pitch: f32) -> f32 {
        frequency_of(self.midi, reference_pitch)
    }
}

/// The frequency of a MIDI note, with the A above the middle C at `reference_pitch`.
pub fn frequency_of(midi: i32, reference_pitch: f32) -> f32 {
    reference_pitch * 2f32.powf((midi - 69) as f32 / 12.0)
}

/// The strings of an instrument, as the notes the tuner offers to play and to listen for.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InstrumentTuning {
    pub name: &'static str,
    /// MIDI numbers, from the lowest string.
    pub strings: Vec<i32>,
}

pub fn instrument_tunings() -> Vec<InstrumentTuning> {
    vec![
        InstrumentTuning { name: "Guitarra", strings: vec![40, 45, 50, 55, 59, 64] },
        InstrumentTuning { name: "Guitarra, re bajo (Drop D)", strings: vec![38, 45, 50, 55, 59, 64] },
        InstrumentTuning { name: "Ukelele", strings: vec![67, 60, 64, 69] },
        InstrumentTuning { name: "Bajo", strings: vec![28, 33, 38, 43] },
        InstrumentTuning { name: "Mandolina / Violín", strings: vec![55, 62, 69, 76] },
    ]
}

/// Which string of `tuning` a note is closest to, and how far from it the note is in cents.
pub fn closest_string(tuning: &InstrumentTuning, frequency: f32, reference_pitch: f32) -> Option<(usize, f32)> {
    tuning
        .strings
        .iter()
        .enumerate()
        .map(|(index, &midi)| (index, 1200.0 * (frequency / frequency_of(midi, reference_pitch)).log2()))
        .min_by(|a, b| a.1.abs().total_cmp(&b.1.abs()))
}

/// A sine for the tuner to play as the note to tune to, which fades in and out so that starting and stopping it does not click.
pub struct ToneGenerator {
    sample_rate: f64,
    frequency: f64,
    phase: f64,
    gain: f32,
    is_playing: bool,
}

impl ToneGenerator {
    pub fn new(sample_rate: f32) -> ToneGenerator {
        ToneGenerator { sample_rate: sample_rate as f64, frequency: 440.0, phase: 0.0, gain: 0.0, is_playing: false }
    }

    pub fn play(&mut self, frequency: f32) {
        self.frequency = frequency as f64;
        self.is_playing = true;
    }

    pub fn stop(&mut self) {
        self.is_playing = false;
    }

    pub fn is_playing(&self) -> bool {
        self.is_playing
    }

    /// Fills `buffer` with the next samples, mono.
    pub fn fill(&mut self, buffer: &mut [f32]) {
        let step = 1.0 / (0.02 * self.sample_rate) as f32;
        for sample in buffer.iter_mut() {
            self.gain = if self.is_playing { (self.gain + step).min(1.0) } else { (self.gain - step).max(0.0) };
            *sample = if self.gain > 0.0 { (0.5 * self.gain as f64 * (TAU * self.phase).sin()) as f32 } else { 0.0 };
            self.phase = (self.phase + self.frequency / self.sample_rate).fract();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE_RATE: f32 = 48_000.0;

    /// `harmonics` are the amplitudes of the partials, from the fundamental.
    fn tone(frequency: f32, harmonics: &[f32], length: usize) -> Vec<f32> {
        (0..length)
            .map(|index| {
                let time = index as f64 / SAMPLE_RATE as f64;
                harmonics.iter().enumerate().map(|(partial, amplitude)| *amplitude as f64 * (TAU * frequency as f64 * (partial + 1) as f64 * time).sin()).sum::<f64>() as f32
            })
            .collect()
    }

    fn cents_between(a: f32, b: f32) -> f32 {
        1200.0 * (a / b).log2()
    }

    #[test]
    fn a_pure_tone_is_found_within_a_few_cents() {
        for frequency in [82.41, 110.0, 196.0, 329.63, 440.0, 880.0, 1318.5] {
            let samples = tone(frequency, &[0.5], window_length(SAMPLE_RATE) + 100);
            let pitch = detect_pitch(&samples, SAMPLE_RATE).unwrap_or_else(|| panic!("{frequency} Hz"));
            assert!(cents_between(pitch.frequency, frequency).abs() < 3.0, "{frequency} Hz was heard as {}", pitch.frequency);
            assert!(pitch.clarity > 0.9);
        }
    }

    #[test]
    fn a_string_with_a_louder_second_harmonic_is_not_heard_an_octave_up() {
        // A low E whose partials are stronger than its fundamental, as a guitar's is through a small microphone.
        let samples = tone(82.41, &[0.3, 0.6, 0.5, 0.2], window_length(SAMPLE_RATE) + 100);
        let pitch = detect_pitch(&samples, SAMPLE_RATE).expect("a note");
        assert!(cents_between(pitch.frequency, 82.41).abs() < 5.0, "heard as {}", pitch.frequency);
    }

    #[test]
    fn silence_and_noise_have_no_pitch() {
        assert_eq!(detect_pitch(&vec![0.0; 6000], SAMPLE_RATE), None);
        assert_eq!(detect_pitch(&vec![0.001; 6000], SAMPLE_RATE), None);
        // A deterministic noise.
        let mut state = 12345u32;
        let noise: Vec<f32> = (0..6000)
            .map(|_| {
                state = state.wrapping_mul(1664525).wrapping_add(1013904223);
                (state >> 8) as f32 / (1 << 24) as f32 - 0.5
            })
            .collect();
        assert_eq!(detect_pitch(&noise, SAMPLE_RATE), None);
    }

    #[test]
    fn too_few_samples_are_no_pitch() {
        assert_eq!(detect_pitch(&tone(440.0, &[0.5], 500), SAMPLE_RATE), None);
    }

    #[test]
    fn a_frequency_is_the_closest_note_with_its_cents() {
        let a = Note::of(440.0, 440.0);
        assert_eq!((a.name(), a.octave(), a.midi), ("A", 4, 69));
        assert!(a.cents.abs() < 0.01);
        let sharp = Note::of(446.0, 440.0);
        assert_eq!(sharp.name(), "A");
        assert!((sharp.cents - 23.5).abs() < 0.5, "{}", sharp.cents);
        let flat = Note::of(82.0, 440.0);
        assert_eq!((flat.name(), flat.octave()), ("E", 2));
        assert!(flat.cents < 0.0);
        let c = Note::of(261.63, 440.0);
        assert_eq!((c.name(), c.octave()), ("C", 4));
        // A different reference pitch moves every note.
        assert_eq!(Note::of(432.0, 432.0).name(), "A");
        assert!(Note::of(432.0, 440.0).cents < -25.0);
    }

    #[test]
    fn a_note_has_the_frequency_it_is_named_by() {
        assert!((frequency_of(69, 440.0) - 440.0).abs() < 1e-3);
        assert!((frequency_of(40, 440.0) - 82.41).abs() < 0.01);
        assert!((frequency_of(60, 440.0) - 261.63).abs() < 0.01);
    }

    #[test]
    fn the_closest_string_is_the_one_being_tuned() {
        let guitar = &instrument_tunings()[0];
        let (string, cents) = closest_string(guitar, 112.0, 440.0).unwrap();
        assert_eq!(string, 1);
        assert!(cents > 0.0 && cents < 40.0, "{cents}");
        let (string, _) = closest_string(guitar, 330.0, 440.0).unwrap();
        assert_eq!(string, 5);
    }

    #[test]
    fn the_reference_tone_is_a_clean_sine_that_fades_in_and_out() {
        let mut generator = ToneGenerator::new(SAMPLE_RATE);
        let mut silence = vec![1.0; 100];
        generator.fill(&mut silence);
        assert!(silence.iter().all(|sample| *sample == 0.0));
        generator.play(220.0);
        let mut buffer = vec![0.0; 12_000];
        generator.fill(&mut buffer);
        assert!(buffer[0].abs() < 0.01, "starts silent");
        let pitch = detect_pitch(&buffer, SAMPLE_RATE).expect("a note");
        assert!(cents_between(pitch.frequency, 220.0).abs() < 3.0);
        generator.stop();
        let mut tail = vec![0.0; 3_000];
        generator.fill(&mut tail);
        assert!(tail.last().unwrap().abs() < 1e-6);
        assert!(!generator.is_playing());
    }
}

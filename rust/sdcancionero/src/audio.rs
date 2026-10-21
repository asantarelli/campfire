// This file is part of Campfire.
// Copyright (c) Pandula Péter 2017-2026.
// https://github.com/pandulapeter/campfire
//
// This Source Code Form is subject to the terms of the Mozilla Public License, v. 2.0.
// If a copy of the MPL was not distributed with this file, You can obtain one at
// https://mozilla.org/MPL/2.0/.

//! The sound card: what the metronome and the reference tone play through, and what the tuner listens to.
//!
//! The samples themselves are made and read by the `instruments` crate; this only moves them. An output or an input exists only
//! while the screen that uses it does: the metronome never outlives its screen, and the tuner listens only while it is shown and
//! keeps nothing past the few thousand samples it needs to find a pitch.

use std::collections::VecDeque;
use std::sync::{Arc, Mutex};

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{FromSample, Sample, SampleFormat, SizedSample, Stream, StreamConfig};
use instruments::metronome::{Accent, Metronome};
use instruments::tuner::ToneGenerator;

/// What the output callback plays, shared with the window that controls it.
struct Playing {
    metronome: Metronome,
    is_metronome_running: bool,
    tone: ToneGenerator,
}

/// The output stream: a metronome and a reference tone.
pub struct AudioOutput {
    _stream: Stream,
    playing: Arc<Mutex<Playing>>,
}

fn describe(error: impl std::fmt::Display) -> String {
    error.to_string()
}

impl AudioOutput {
    pub fn new() -> Result<AudioOutput, String> {
        let host = cpal::default_host();
        let device = host.default_output_device().ok_or("No hay una salida de audio disponible.")?;
        let supported = device.default_output_config().map_err(describe)?;
        let sample_format = supported.sample_format();
        let config: StreamConfig = supported.into();
        let sample_rate = config.sample_rate as f32;
        let playing = Arc::new(Mutex::new(Playing { metronome: Metronome::new(sample_rate), is_metronome_running: false, tone: ToneGenerator::new(sample_rate) }));
        let stream = match sample_format {
            SampleFormat::F32 => build_output::<f32>(&device, config, playing.clone())?,
            SampleFormat::I16 => build_output::<i16>(&device, config, playing.clone())?,
            SampleFormat::U16 => build_output::<u16>(&device, config, playing.clone())?,
            SampleFormat::I32 => build_output::<i32>(&device, config, playing.clone())?,
            other => return Err(format!("La salida de audio usa un formato que no se maneja ({other}).")),
        };
        stream.play().map_err(describe)?;
        Ok(AudioOutput { _stream: stream, playing })
    }

    fn with_playing<R>(&self, action: impl FnOnce(&mut Playing) -> R) -> R {
        let mut playing = self.playing.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
        action(&mut playing)
    }

    /// Sets the bar and the tempo the metronome counts; it keeps its place unless it was stopped.
    pub fn configure_metronome(&self, tempo: u32, beats: u32, unit: u32, accent: Accent, has_subdivision: bool) {
        self.with_playing(|playing| {
            playing.metronome.set_tempo(tempo);
            playing.metronome.set_time_signature(beats, unit);
            playing.metronome.set_accent(accent);
            playing.metronome.set_subdivision(has_subdivision);
        });
    }

    pub fn start_metronome(&self) {
        self.with_playing(|playing| {
            playing.metronome.restart();
            playing.is_metronome_running = true;
        });
    }

    pub fn stop_metronome(&self) {
        self.with_playing(|playing| playing.is_metronome_running = false);
    }

    pub fn is_metronome_running(&self) -> bool {
        self.with_playing(|playing| playing.is_metronome_running)
    }

    /// The beat of the bar that sounded last, `None` while stopped or before the first.
    pub fn current_beat(&self) -> Option<u32> {
        self.with_playing(|playing| if playing.is_metronome_running { playing.metronome.current_beat() } else { None })
    }

    pub fn play_tone(&self, frequency: f32) {
        self.with_playing(|playing| playing.tone.play(frequency));
    }

    pub fn stop_tone(&self) {
        self.with_playing(|playing| playing.tone.stop());
    }

    pub fn is_tone_playing(&self) -> bool {
        self.with_playing(|playing| playing.tone.is_playing())
    }
}

fn build_output<T>(device: &cpal::Device, config: StreamConfig, playing: Arc<Mutex<Playing>>) -> Result<Stream, String>
where
    T: SizedSample + FromSample<f32>,
{
    let channels = config.channels as usize;
    let mut mono: Vec<f32> = Vec::new();
    let mut tone: Vec<f32> = Vec::new();
    device
        .build_output_stream(
            config,
            move |data: &mut [T], _: &cpal::OutputCallbackInfo| {
                let frames = data.len() / channels;
                mono.resize(frames, 0.0);
                tone.resize(frames, 0.0);
                {
                    let mut playing = playing.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
                    if playing.is_metronome_running {
                        playing.metronome.fill(&mut mono);
                    } else {
                        mono.iter_mut().for_each(|sample| *sample = 0.0);
                    }
                    playing.tone.fill(&mut tone);
                }
                for (frame, (click, note)) in data.chunks_mut(channels).zip(mono.iter().zip(tone.iter())) {
                    let value: T = T::from_sample((click + note).clamp(-1.0, 1.0));
                    for sample in frame.iter_mut() {
                        *sample = value;
                    }
                }
            },
            |error| eprintln!("Error de audio: {error}"),
            None,
        )
        .map_err(describe)
}

/// How many samples the tuner keeps: enough for the longest period it looks for, twice, with room.
const KEPT_SAMPLES: usize = 16_384;

/// The microphone, for as long as the tuner is on the screen.
pub struct AudioInput {
    _stream: Stream,
    samples: Arc<Mutex<VecDeque<f32>>>,
    sample_rate: f32,
}

impl AudioInput {
    pub fn new() -> Result<AudioInput, String> {
        let host = cpal::default_host();
        let device = host.default_input_device().ok_or("No hay un micrófono disponible.")?;
        let supported = device.default_input_config().map_err(describe)?;
        let sample_format = supported.sample_format();
        let config: StreamConfig = supported.into();
        let sample_rate = config.sample_rate as f32;
        let samples = Arc::new(Mutex::new(VecDeque::with_capacity(KEPT_SAMPLES)));
        let stream = match sample_format {
            SampleFormat::F32 => build_input::<f32>(&device, config, samples.clone())?,
            SampleFormat::I16 => build_input::<i16>(&device, config, samples.clone())?,
            SampleFormat::U16 => build_input::<u16>(&device, config, samples.clone())?,
            SampleFormat::I32 => build_input::<i32>(&device, config, samples.clone())?,
            other => return Err(format!("El micrófono usa un formato que no se maneja ({other}).")),
        };
        stream.play().map_err(describe)?;
        Ok(AudioInput { _stream: stream, samples, sample_rate })
    }

    pub fn sample_rate(&self) -> f32 {
        self.sample_rate
    }

    /// The last `length` samples heard, fewer where fewer have arrived.
    pub fn latest(&self, length: usize) -> Vec<f32> {
        let samples = self.samples.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
        let skip = samples.len().saturating_sub(length);
        samples.iter().skip(skip).copied().collect()
    }
}

fn build_input<T>(device: &cpal::Device, config: StreamConfig, samples: Arc<Mutex<VecDeque<f32>>>) -> Result<Stream, String>
where
    T: SizedSample,
    f32: FromSample<T>,
{
    let channels = config.channels as usize;
    device
        .build_input_stream(
            config,
            move |data: &[T], _: &cpal::InputCallbackInfo| {
                let mut samples = samples.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
                for frame in data.chunks(channels) {
                    // The channels are one sound: their mean.
                    let sum: f32 = frame.iter().map(|sample| f32::from_sample(*sample)).sum();
                    if samples.len() == KEPT_SAMPLES {
                        samples.pop_front();
                    }
                    samples.push_back(sum / frame.len() as f32);
                }
            },
            |error| eprintln!("Error de audio: {error}"),
            None,
        )
        .map_err(describe)
}

/// The sample type must be usable as a sample at all, which `Sample` says.
#[allow(dead_code)]
fn assert_sample<T: Sample>() {}

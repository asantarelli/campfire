// This file is part of Campfire.
// Copyright (c) Pandula Péter 2017-2026.
// https://github.com/pandulapeter/campfire
//
// This Source Code Form is subject to the terms of the Mozilla Public License, v. 2.0.
// If a copy of the MPL was not distributed with this file, You can obtain one at
// https://mozilla.org/MPL/2.0/.

//! The tools: a metronome, a tuner and the chord diagrams.
//!
//! The sound card is opened only while the tool that needs it is on the screen: the metronome never outlives its screen, and the
//! tuner listens only while it is shown.

use std::time::{Duration, Instant};

use chordpro::chord;
use chordpro::definitions;
use chordpro::model::ChordDefinition;
use chordpro::notation::{self, ChordNotation};
use chordpro::scales;
use chordpro::voicings::{self, Tuning, Voicing};
use eframe::egui::{self, Align2, Color32, FontId, Pos2, RichText, Sense, Stroke, Ui, Vec2};
use instruments::metronome::{Accent, DEFAULT_TEMPO, TEMPO_RANGE};
use instruments::tuner::{self, InstrumentTuning, Note, Pitch};

use crate::audio::{AudioInput, AudioOutput};
use crate::diagram;
use crate::settings::{Instrument, Settings};
use crate::theme::{self, SongColors};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToolTab {
    Metronome,
    Tuner,
    Chords,
    Scales,
}

const TIME_SIGNATURES: [(u32, u32); 8] = [(2, 4), (3, 4), (4, 4), (5, 4), (6, 8), (7, 8), (9, 8), (12, 8)];
/// The suffixes the chord picker offers, as they are written after a root.
const SUFFIXES: [&str; 16] = ["", "m", "7", "maj7", "m7", "sus2", "sus4", "dim", "aug", "6", "m6", "9", "add9", "m7b5", "7sus4", "5"];
const ROOTS: [&str; 12] = ["C", "C#", "D", "Eb", "E", "F", "F#", "G", "G#", "A", "Bb", "B"];
/// How long the tuner keeps showing a note after the sound that made it has stopped.
const HOLD_NOTE: Duration = Duration::from_millis(700);
const DETECTION_INTERVAL: Duration = Duration::from_millis(50);
/// Taps further apart than this start a new count.
const TAP_RESET: Duration = Duration::from_secs(2);

pub struct Tools {
    pub tab: ToolTab,
    output: Option<AudioOutput>,
    output_error: Option<String>,
    input: Option<AudioInput>,
    input_error: Option<String>,
    tempo: u32,
    beats: u32,
    unit: u32,
    accent: Accent,
    has_subdivision: bool,
    taps: Vec<Instant>,
    reference_pitch: f32,
    tuning_index: usize,
    tunings: Vec<InstrumentTuning>,
    heard: Option<(Pitch, Instant)>,
    last_detection: Option<Instant>,
    playing_string: Option<usize>,
    root: usize,
    suffix: usize,
    query: String,
    scale_root: usize,
    scale_index: usize,
    /// Whether the dots of the neck say the degree of their note rather than its name.
    scale_degrees: bool,
}

/// The metronome beside a song: its tempo and a button, so that it can be played along with the song on the screen.
pub struct SongMetronome {
    output: Option<AudioOutput>,
    error: Option<String>,
    tempo: u32,
    beats: u32,
    unit: u32,
}

impl SongMetronome {
    /// A metronome at the tempo and in the time the song asks for, else at the usual ones.
    pub fn for_song(timing: &SongTiming) -> SongMetronome {
        let (beats, unit) = timing.time.unwrap_or((4, 4));
        SongMetronome { output: None, error: None, tempo: timing.tempo.unwrap_or(DEFAULT_TEMPO).clamp(*TEMPO_RANGE.start(), *TEMPO_RANGE.end()), beats, unit }
    }

    fn is_running(&self) -> bool {
        self.output.as_ref().is_some_and(AudioOutput::is_metronome_running)
    }

    fn apply(&self) {
        if let Some(output) = &self.output {
            output.configure_metronome(self.tempo, self.beats, self.unit, Accent::FirstBeat, false);
        }
    }

    fn toggle(&mut self) {
        if self.is_running() {
            if let Some(output) = &self.output {
                output.stop_metronome();
            }
            return;
        }
        if self.output.is_none() {
            match AudioOutput::new() {
                Ok(output) => self.output = Some(output),
                Err(error) => {
                    self.error = Some(error);
                    return;
                }
            }
        }
        self.apply();
        if let Some(output) = &self.output {
            output.start_metronome();
        }
    }

    pub fn show(&mut self, ui: &mut Ui, colors: &SongColors) {
        let mut changed = false;
        if ui.small_button("−").on_hover_text("Bajar el tempo").clicked() {
            self.tempo = self.tempo.saturating_sub(1).max(*TEMPO_RANGE.start());
            changed = true;
        }
        let mut tempo = self.tempo;
        if ui.add(egui::DragValue::new(&mut tempo).range(TEMPO_RANGE).suffix(" bpm")).changed() {
            self.tempo = tempo;
            changed = true;
        }
        if ui.small_button("+").on_hover_text("Subir el tempo").clicked() {
            self.tempo = (self.tempo + 1).min(*TEMPO_RANGE.end());
            changed = true;
        }
        let label = if self.is_running() { "Detener metrónomo" } else { "Metrónomo" };
        if ui.selectable_label(self.is_running(), label).on_hover_text("Marca el pulso mientras tocás").clicked() {
            self.toggle();
        }
        if self.is_running() {
            let current = self.output.as_ref().and_then(AudioOutput::current_beat);
            for beat in 0..self.beats {
                let (rect, _) = ui.allocate_exact_size(Vec2::splat(12.0), Sense::hover());
                let color = if current == Some(beat) { if beat == 0 { theme::ORANGE } else { theme::PURPLE } } else { colors.muted.gamma_multiply(0.45) };
                ui.painter().circle_filled(rect.center(), 5.0, color);
            }
            ui.ctx().request_repaint();
        }
        if let Some(error) = &self.error {
            ui.colored_label(Color32::from_rgb(0xE5, 0x57, 0x57), error);
        }
        if changed {
            self.apply();
        }
    }
}

/// What the song on the screen knows about its timing, so that the metronome can follow it.
pub struct SongTiming {
    pub tempo: Option<u32>,
    pub time: Option<(u32, u32)>,
}

impl Tools {
    pub fn new(tab: ToolTab) -> Tools {
        Tools {
            tab,
            output: None,
            output_error: None,
            input: None,
            input_error: None,
            tempo: DEFAULT_TEMPO,
            beats: 4,
            unit: 4,
            accent: Accent::FirstBeat,
            has_subdivision: false,
            taps: Vec::new(),
            reference_pitch: tuner::DEFAULT_REFERENCE_PITCH,
            tuning_index: 0,
            tunings: tuner::instrument_tunings(),
            heard: None,
            last_detection: None,
            playing_string: None,
            root: 0,
            suffix: 0,
            query: String::new(),
            scale_root: 0,
            scale_index: 0,
            scale_degrees: false,
        }
    }

    /// Takes the tempo and the bar a song asks for.
    pub fn follow(&mut self, timing: &SongTiming) {
        if let Some(tempo) = timing.tempo {
            self.tempo = tempo.clamp(*TEMPO_RANGE.start(), *TEMPO_RANGE.end());
        }
        if let Some((beats, unit)) = timing.time {
            self.beats = beats;
            self.unit = unit;
        }
        self.apply_metronome();
    }

    fn ensure_output(&mut self) -> bool {
        if self.output.is_none() && self.output_error.is_none() {
            match AudioOutput::new() {
                Ok(output) => self.output = Some(output),
                Err(error) => self.output_error = Some(error),
            }
        }
        self.output.is_some()
    }

    fn apply_metronome(&mut self) {
        if let Some(output) = &self.output {
            output.configure_metronome(self.tempo, self.beats, self.unit, self.accent, self.has_subdivision);
        }
    }

    fn set_tab(&mut self, tab: ToolTab) {
        if tab == self.tab {
            return;
        }
        // Each tool has the sound card to itself: the click stops, the tone stops, the microphone closes.
        if let Some(output) = &self.output {
            output.stop_metronome();
            output.stop_tone();
        }
        self.playing_string = None;
        if tab != ToolTab::Tuner {
            self.input = None;
            self.heard = None;
        }
        self.tab = tab;
    }

    pub fn show(&mut self, ui: &mut Ui, settings: &Settings, colors: &SongColors) {
        ui.horizontal(|ui| {
            for (tab, label) in [(ToolTab::Metronome, "Metrónomo"), (ToolTab::Tuner, "Afinador"), (ToolTab::Chords, "Acordes"), (ToolTab::Scales, "Escalas")] {
                if ui.selectable_label(self.tab == tab, RichText::new(label).size(16.0)).clicked() {
                    self.set_tab(tab);
                }
            }
        });
        ui.separator();
        match self.tab {
            ToolTab::Metronome => self.metronome(ui, colors),
            ToolTab::Tuner => self.tuner(ui, settings, colors),
            ToolTab::Chords => self.chords(ui, settings, colors),
            ToolTab::Scales => self.scales(ui, settings, colors),
        }
    }

    fn metronome(&mut self, ui: &mut Ui, colors: &SongColors) {
        let mut changed = false;
        let is_running = self.output.as_ref().is_some_and(AudioOutput::is_metronome_running);
        ui.vertical_centered(|ui| {
            ui.add_space(8.0);
            ui.label(RichText::new(format!("{}", self.tempo)).size(72.0).strong());
            ui.label(RichText::new("pulsos por minuto").color(colors.muted));
            ui.add_space(8.0);
            ui.horizontal(|ui| {
                ui.add_space((ui.available_width() - 360.0).max(0.0) / 2.0);
                for (label, change) in [("−5", -5i32), ("−1", -1), ("+1", 1), ("+5", 5)] {
                    if ui.button(RichText::new(label).size(16.0)).clicked() {
                        self.tempo = (self.tempo as i32 + change).clamp(*TEMPO_RANGE.start() as i32, *TEMPO_RANGE.end() as i32) as u32;
                        changed = true;
                    }
                }
                if ui.button(RichText::new("Marcar el pulso").size(16.0)).on_hover_text("Tocá varias veces al ritmo").clicked() {
                    self.tap();
                    changed = true;
                }
            });
            let mut tempo = self.tempo;
            ui.spacing_mut().slider_width = (ui.available_width() - 120.0).clamp(160.0, 520.0);
            if ui.add(egui::Slider::new(&mut tempo, TEMPO_RANGE).show_value(false)).changed() {
                self.tempo = tempo;
                changed = true;
            }
            ui.add_space(8.0);
            ui.horizontal(|ui| {
                ui.add_space((ui.available_width() - 420.0).max(0.0) / 2.0);
                ui.label("Compás");
                egui::ComboBox::from_id_salt("time_signature").selected_text(format!("{}/{}", self.beats, self.unit)).show_ui(ui, |ui| {
                    for (beats, unit) in TIME_SIGNATURES {
                        if ui.selectable_label(self.beats == beats && self.unit == unit, format!("{beats}/{unit}")).clicked() {
                            self.beats = beats;
                            self.unit = unit;
                            changed = true;
                        }
                    }
                });
                let mut accented = self.accent == Accent::FirstBeat;
                if ui.checkbox(&mut accented, "Acentuar el primer pulso").changed() {
                    self.accent = if accented { Accent::FirstBeat } else { Accent::None };
                    changed = true;
                }
                if ui.checkbox(&mut self.has_subdivision, "Subdivisión").changed() {
                    changed = true;
                }
            });
            ui.add_space(14.0);
            self.beat_lights(ui, colors);
            ui.add_space(14.0);
            let label = if is_running { "Detener" } else { "Iniciar" };
            if ui.add_sized([180.0, 44.0], egui::Button::new(RichText::new(label).size(20.0))).on_hover_text("Espacio").clicked() || ui.input(|input| input.key_pressed(egui::Key::Space)) {
                if is_running {
                    if let Some(output) = &self.output {
                        output.stop_metronome();
                    }
                } else if self.ensure_output() {
                    self.apply_metronome();
                    if let Some(output) = &self.output {
                        output.start_metronome();
                    }
                }
            }
            if let Some(error) = &self.output_error {
                ui.add_space(8.0);
                ui.colored_label(Color32::from_rgb(0xE5, 0x57, 0x57), error);
            }
        });
        if changed {
            self.apply_metronome();
        }
        if is_running {
            ui.ctx().request_repaint();
        }
    }

    /// One light a beat of the bar, the one that sounded last lit and the first of the bar in the brand's orange.
    fn beat_lights(&mut self, ui: &mut Ui, colors: &SongColors) {
        let size = 26.0;
        let gap = 12.0;
        let width = self.beats as f32 * size + (self.beats - 1) as f32 * gap;
        let (rect, _) = ui.allocate_exact_size(Vec2::new(width, size), Sense::hover());
        let current = self.output.as_ref().and_then(AudioOutput::current_beat);
        for beat in 0..self.beats {
            let center = Pos2::new(rect.left() + size / 2.0 + beat as f32 * (size + gap), rect.center().y);
            let is_lit = current == Some(beat);
            let color = if is_lit { if beat == 0 { theme::ORANGE } else { theme::PURPLE } } else { colors.muted.gamma_multiply(0.45) };
            ui.painter().circle_filled(center, size / 2.0, color);
        }
    }

    fn tap(&mut self) {
        let now = Instant::now();
        if self.taps.last().is_some_and(|last| now.duration_since(*last) > TAP_RESET) {
            self.taps.clear();
        }
        self.taps.push(now);
        if self.taps.len() > 8 {
            self.taps.remove(0);
        }
        if self.taps.len() >= 2 {
            let span = self.taps[self.taps.len() - 1].duration_since(self.taps[0]).as_secs_f64();
            let beats = (self.taps.len() - 1) as f64;
            let tempo = (60.0 * beats / span).round() as i64;
            self.tempo = tempo.clamp(*TEMPO_RANGE.start() as i64, *TEMPO_RANGE.end() as i64) as u32;
        }
    }

    fn tuner(&mut self, ui: &mut Ui, settings: &Settings, colors: &SongColors) {
        if self.input.is_none() && self.input_error.is_none() {
            match AudioInput::new() {
                Ok(input) => self.input = Some(input),
                Err(error) => self.input_error = Some(error),
            }
        }
        self.listen();
        let notation = settings.notation;
        ui.vertical_centered(|ui| {
            ui.add_space(6.0);
            match (&self.heard, &self.input_error) {
                (Some((pitch, _)), _) => {
                    let note = Note::of(pitch.frequency, self.reference_pitch);
                    let name = shown_note(note.name(), notation, settings.accidental_signs);
                    ui.label(RichText::new(format!("{name}{}", note.octave())).size(80.0).strong());
                    ui.label(RichText::new(format!("{:.1} Hz  ·  {:+.0} centésimas", pitch.frequency, note.cents)).color(colors.muted));
                    ui.add_space(8.0);
                    cents_meter(ui, note.cents, colors);
                }
                (None, Some(error)) => {
                    ui.add_space(24.0);
                    ui.colored_label(Color32::from_rgb(0xE5, 0x57, 0x57), error);
                    ui.label("Revisá que el micrófono esté conectado y que Windows permita a las aplicaciones usarlo.");
                }
                (None, None) => {
                    ui.label(RichText::new("—").size(80.0).color(colors.muted));
                    ui.label(RichText::new("Tocá una cuerda: el afinador escucha por el micrófono.").color(colors.muted));
                    ui.add_space(8.0);
                    cents_meter(ui, 0.0, colors);
                }
            }
            ui.add_space(14.0);
            ui.horizontal(|ui| {
                ui.add_space((ui.available_width() - 420.0).max(0.0) / 2.0);
                ui.label("Instrumento");
                let names: Vec<&'static str> = self.tunings.iter().map(|tuning| tuning.name).collect();
                let mut chosen: Option<usize> = None;
                egui::ComboBox::from_id_salt("tuning").selected_text(names[self.tuning_index]).show_ui(ui, |ui| {
                    for (index, name) in names.iter().enumerate() {
                        if ui.selectable_label(self.tuning_index == index, *name).clicked() {
                            chosen = Some(index);
                        }
                    }
                });
                if let Some(index) = chosen {
                    self.tuning_index = index;
                    self.stop_tone();
                }
                ui.label("La =");
                let mut pitch = self.reference_pitch;
                if ui.add(egui::DragValue::new(&mut pitch).range(tuner::REFERENCE_PITCH_RANGE).speed(0.2).suffix(" Hz")).changed() {
                    self.reference_pitch = pitch;
                    self.stop_tone();
                }
                if ui.small_button("440").on_hover_text("Restablecer la altura de referencia").clicked() {
                    self.reference_pitch = tuner::DEFAULT_REFERENCE_PITCH;
                }
            });
            ui.add_space(10.0);
            ui.label(RichText::new("Tono de referencia: tocá una cuerda para escuchar su nota").color(colors.muted));
            ui.add_space(4.0);
            let closest = self.heard.as_ref().and_then(|(pitch, _)| tuner::closest_string(&self.tunings[self.tuning_index], pitch.frequency, self.reference_pitch)).map(|(string, _)| string);
            let strings = self.tunings[self.tuning_index].strings.clone();
            ui.horizontal(|ui| {
                ui.add_space((ui.available_width() - strings.len() as f32 * 70.0).max(0.0) / 2.0);
                for (index, midi) in strings.iter().enumerate() {
                    let note = Note { midi: *midi, cents: 0.0 };
                    let label = format!("{}{}", shown_note(note.name(), notation, settings.accidental_signs), note.octave());
                    let is_playing = self.playing_string == Some(index);
                    let text = RichText::new(label).size(17.0);
                    let button = egui::Button::new(if closest == Some(index) { text.color(theme::ORANGE) } else { text }).selected(is_playing).min_size(Vec2::new(60.0, 36.0));
                    if ui.add(button).clicked() {
                        if is_playing {
                            self.stop_tone();
                        } else if self.ensure_output() {
                            if let Some(output) = &self.output {
                                output.play_tone(note.frequency(self.reference_pitch));
                            }
                            self.playing_string = Some(index);
                        }
                    }
                }
            });
            if let Some(error) = &self.output_error {
                ui.colored_label(Color32::from_rgb(0xE5, 0x57, 0x57), error);
            }
        });
        ui.ctx().request_repaint_after(DETECTION_INTERVAL);
    }

    fn stop_tone(&mut self) {
        if let Some(output) = &self.output {
            output.stop_tone();
        }
        self.playing_string = None;
    }

    /// Looks at what the microphone heard, at most every few hundredths of a second, and holds a note a moment past its end.
    fn listen(&mut self) {
        let now = Instant::now();
        // The tone the tuner plays itself is not a sound to tune: while it plays, nothing is listened to.
        let is_tone_playing = self.output.as_ref().is_some_and(AudioOutput::is_tone_playing);
        if let Some((_, since)) = &self.heard {
            if now.duration_since(*since) > HOLD_NOTE {
                self.heard = None;
            }
        }
        if is_tone_playing || self.last_detection.is_some_and(|last| now.duration_since(last) < DETECTION_INTERVAL) {
            return;
        }
        self.last_detection = Some(now);
        let Some(input) = &self.input else {
            return;
        };
        let samples = input.latest(tuner::window_length(input.sample_rate()));
        if let Some(pitch) = tuner::detect_pitch(&samples, input.sample_rate()) {
            if pitch.clarity > 0.85 {
                self.heard = Some((pitch, now));
            }
        }
    }

    fn scales(&mut self, ui: &mut Ui, settings: &Settings, colors: &SongColors) {
        ui.horizontal_wrapped(|ui| {
            ui.label("Nota");
            for (index, root) in ROOTS.iter().enumerate() {
                if ui.selectable_label(self.scale_root == index, shown_note(root, settings.notation, settings.accidental_signs)).clicked() {
                    self.scale_root = index;
                }
            }
        });
        ui.horizontal_wrapped(|ui| {
            ui.label("Escala");
            for (index, scale) in scales::SCALES.iter().enumerate() {
                if ui.selectable_label(self.scale_index == index, scale.name).clicked() {
                    self.scale_index = index;
                }
            }
        });
        ui.horizontal(|ui| {
            ui.label("En los círculos");
            ui.selectable_value(&mut self.scale_degrees, false, "Notas");
            ui.selectable_value(&mut self.scale_degrees, true, "Grados");
        });
        ui.separator();
        let root = ROOTS[self.scale_root];
        let scale = &scales::SCALES[self.scale_index];
        if let Some(notes) = scales::note_names(root, scale) {
            let notes: Vec<String> = notes.iter().map(|note| shown_note(note, settings.notation, settings.accidental_signs)).collect();
            ui.label(RichText::new(format!("{} {}", shown_note(root, settings.notation, settings.accidental_signs), scale.name)).size(24.0).strong());
            ui.label(RichText::new(notes.join("  ")).size(18.0).color(colors.chord));
        }
        ui.add_space(10.0);
        let tuning = settings.instrument.tuning();
        egui::ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| {
            if let Some(neck) = scales::on_fretboard(root, scale, &tuning, FRETBOARD_FRETS) {
                // What each of the scale's notes says inside its dot.
                let labels: Vec<String> = if self.scale_degrees {
                    scale.intervals.iter().map(|interval| scales::degree_label(*interval).replace('b', "♭")).collect()
                } else {
                    scales::note_names(root, scale).unwrap_or_default().iter().map(|note| shown_note(note, settings.notation, settings.accidental_signs)).collect()
                };
                egui::ScrollArea::horizontal().id_salt("neck").show(ui, |ui| fretboard(ui, &neck, &labels, colors));
            }
            for (title, with_seventh) in [("Acordes de esta escala", false), ("Acordes de séptima", true)] {
                let Some(chords) = scales::scale_chords(root, scale, with_seventh) else {
                    continue;
                };
                ui.add_space(12.0);
                ui.label(RichText::new(title).strong().size(16.0));
                ui.horizontal_wrapped(|ui| {
                    for chord in &chords {
                        ui.vertical(|ui| {
                            ui.label(RichText::new(&chord.degree).color(colors.muted));
                            let _ = pinned_chord(ui, &chord.name, settings, colors, &[]);
                        });
                    }
                });
            }
        });
    }

    fn chords(&mut self, ui: &mut Ui, settings: &Settings, colors: &SongColors) {
        ui.horizontal_wrapped(|ui| {
            ui.label("Nota");
            for (index, root) in ROOTS.iter().enumerate() {
                if ui.selectable_label(self.root == index && self.query.is_empty(), shown_note(root, settings.notation, settings.accidental_signs)).clicked() {
                    self.root = index;
                    self.query.clear();
                }
            }
        });
        ui.horizontal_wrapped(|ui| {
            ui.label("Tipo");
            for (index, suffix) in SUFFIXES.iter().enumerate() {
                let label = if suffix.is_empty() { "mayor" } else { suffix };
                if ui.selectable_label(self.suffix == index && self.query.is_empty(), label).clicked() {
                    self.suffix = index;
                    self.query.clear();
                }
            }
        });
        ui.horizontal(|ui| {
            ui.label("O escribilo");
            ui.add(egui::TextEdit::singleline(&mut self.query).hint_text("Am7, F#m, D/F#, Cmaj9…").desired_width(220.0));
        });
        ui.separator();
        let name = if self.query.trim().is_empty() { format!("{}{}", ROOTS[self.root], SUFFIXES[self.suffix]) } else { self.query.trim().to_string() };
        egui::ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| {
            let _ = chord_sheet(ui, &name, settings, colors, 8, 1.15, &[]);
        });
    }
}

fn shown_note(note: &str, notation: ChordNotation, accidental_signs: bool) -> String {
    let shown = notation::shown_name(note, notation);
    if accidental_signs {
        notation::with_accidental_signs(&shown)
    } else {
        shown
    }
}

/// How many frets of the neck the scales are drawn over.
const FRETBOARD_FRETS: u8 = 12;

/// A neck seen from the front, the high string on top: a dot where a note of the scale is, the root in orange.
fn fretboard(ui: &mut Ui, neck: &[Vec<Option<scales::FretNote>>], labels: &[String], colors: &SongColors) {
    let strings = neck.len();
    let (fret_width, string_gap, margin) = (54.0, 32.0, 30.0);
    let width = margin + (FRETBOARD_FRETS as f32 + 1.0) * fret_width;
    let height = (strings as f32 + 0.5) * string_gap + 24.0;
    let (rect, _) = ui.allocate_exact_size(Vec2::new(width, height), Sense::hover());
    let painter = ui.painter();
    let top = rect.top() + string_gap * 0.5;
    for fret in 0..=FRETBOARD_FRETS as usize {
        let x = rect.left() + margin + fret as f32 * fret_width;
        let stroke = if fret == 0 { Stroke::new(4.0, colors.text) } else { Stroke::new(1.0, colors.muted) };
        painter.line_segment([Pos2::new(x, top), Pos2::new(x, top + (strings - 1) as f32 * string_gap)], stroke);
        if fret > 0 {
            painter.text(Pos2::new(x - fret_width / 2.0, rect.bottom() - 2.0), Align2::CENTER_BOTTOM, format!("{fret}"), FontId::proportional(12.0), colors.muted);
        }
    }
    for (row, string) in neck.iter().rev().enumerate() {
        let y = top + row as f32 * string_gap;
        painter.line_segment([Pos2::new(rect.left() + margin, y), Pos2::new(rect.left() + margin + FRETBOARD_FRETS as f32 * fret_width, y)], Stroke::new(1.0, colors.muted));
        for (fret, note) in string.iter().enumerate() {
            let Some(note) = note else {
                continue;
            };
            let x = if fret == 0 { rect.left() + margin - 14.0 } else { rect.left() + margin + (fret as f32 - 0.5) * fret_width };
            let color = if note.is_root { theme::ORANGE } else { theme::PURPLE };
            painter.circle_filled(Pos2::new(x, y), 12.0, color);
            if let Some(label) = labels.get(note.degree) {
                let size = if label.chars().count() > 2 { 9.5 } else { 12.0 };
                painter.text(Pos2::new(x, y), Align2::CENTER_CENTER, label, FontId::new(size, theme::bold()), Color32::WHITE);
            }
        }
    }
}

/// A needle on a scale of fifty cents either way of the note, green where it is in tune.
fn cents_meter(ui: &mut Ui, cents: f32, colors: &SongColors) {
    let (rect, _) = ui.allocate_exact_size(Vec2::new(ui.available_width().min(480.0), 46.0), Sense::hover());
    let painter = ui.painter();
    let y = rect.center().y;
    painter.line_segment([Pos2::new(rect.left(), y), Pos2::new(rect.right(), y)], Stroke::new(2.0, colors.muted));
    for step in -5..=5 {
        let x = rect.center().x + step as f32 * rect.width() / 10.0;
        let height = if step == 0 { 18.0 } else { 9.0 };
        painter.line_segment([Pos2::new(x, y - height), Pos2::new(x, y + height)], Stroke::new(if step == 0 { 3.0 } else { 1.5 }, colors.muted));
    }
    let clamped = cents.clamp(-50.0, 50.0);
    let x = rect.center().x + clamped / 50.0 * rect.width() / 2.0;
    let color = if cents.abs() <= 5.0 { Color32::from_rgb(0x4C, 0xB8, 0x5C) } else { theme::ORANGE };
    painter.circle_filled(Pos2::new(x, y), 11.0, color);
    painter.text(Pos2::new(rect.left(), rect.bottom()), Align2::LEFT_BOTTOM, "♭", FontId::proportional(16.0), colors.muted);
    painter.text(Pos2::new(rect.right(), rect.bottom()), Align2::RIGHT_BOTTOM, "♯", FontId::proportional(16.0), colors.muted);
}

/// The notes of the chord called `name` and its shapes on the player's instrument, at most `limit` of them.
pub fn chord_sheet(ui: &mut Ui, name: &str, settings: &Settings, colors: &SongColors, limit: usize, scale: f32, definitions: &[ChordDefinition]) -> Option<f32> {
    let tuning: Tuning = settings.instrument.tuning();
    if let Some(voicing) = defined_voicing(name, settings, definitions) {
        ui.label(RichText::new(name).size(26.0).strong());
        ui.label(RichText::new("La forma que define la canción.").color(colors.muted));
        ui.add_space(6.0);
        let look = look_of(settings, &tuning, None);
        return diagram::card(ui, settings.is_dark, |ui| diagram::show(ui, name, &voicing, colors, scale, &look));
    }
    let Some(chord) = chord::parse(name, ChordNotation::Standard) else {
        ui.label(RichText::new(format!("«{name}» no es un acorde.")).color(colors.muted));
        return None;
    };
    let shown = shown_chord(name, settings);
    ui.label(RichText::new(shown).size(26.0).strong());
    if let Some(notes) = chord::spelled_note_names(name, settings.notation) {
        let notes: Vec<String> = notes.iter().map(|note| if settings.accidental_signs { notation::with_accidental_signs(note) } else { note.clone() }).collect();
        ui.label(RichText::new(format!("Notas: {}", notes.join("  "))).color(colors.muted));
    }
    let found = voicings::voicings(&chord, &tuning, limit);
    if found.is_empty() {
        ui.add_space(6.0);
        ui.label(RichText::new(format!("No hay una forma cómoda de este acorde en {}.", settings.instrument.name().to_lowercase())).color(colors.muted));
        return None;
    }
    ui.add_space(6.0);
    let look = look_of(settings, &tuning, Some(chord.root));
    let mut tapped = None;
    ui.horizontal_wrapped(|ui| {
        for voicing in &found {
            if let Some(frequency) = diagram::card(ui, settings.is_dark, |ui| diagram::show(ui, &shown_chord(name, settings), voicing, colors, scale, &look)) {
                tapped = Some(frequency);
            }
        }
    });
    tapped
}

/// What the diagrams of the player's instrument say beyond their shape, from the settings: the notes are read against the strings
/// of the instrument and the root of the chord.
fn look_of<'a>(settings: &Settings, tuning: &'a Tuning, root: Option<i32>) -> diagram::Look<'a> {
    diagram::Look { labels: settings.dot_labels, is_colored: settings.dots_colored_by_function, strings: &tuning.strings, root }
}

/// The shape the song itself defines for the chord `name` on the player's instrument (as many strings as it has), the last of them.
pub fn defined_voicing(name: &str, settings: &Settings, definitions: &[ChordDefinition]) -> Option<Voicing> {
    let strings = settings.instrument.tuning().strings.len();
    definitions.iter().rev().find(|definition| definition.name == name && definition.frets.len() == strings).map(definitions::voicing_of)
}

/// A chord of the strip pinned beside a song: its name and its shape, the song's own where it defines one and else the most
/// comfortable.
pub fn pinned_chord(ui: &mut Ui, name: &str, settings: &Settings, colors: &SongColors, definitions: &[ChordDefinition]) -> Option<f32> {
    let tuning = settings.instrument.tuning();
    let (voicing, shown) = match defined_voicing(name, settings, definitions) {
        // A name no table knows is written as the song writes it.
        Some(voicing) => (voicing, if chord::parse(name, ChordNotation::Standard).is_some() { shown_chord(name, settings) } else { name.to_string() }),
        None => {
            let Some(chord) = chord::parse(name, ChordNotation::Standard) else {
                return None;
            };
            let Some(voicing) = voicings::voicings(&chord, &tuning, 1).into_iter().next() else {
                return None;
            };
            (voicing, shown_chord(name, settings))
        }
    };
    let root = chord::parse(name, ChordNotation::Standard).map(|chord| chord.root);
    let look = look_of(settings, &tuning, root);
    diagram::card(ui, settings.is_dark, |ui| diagram::show(ui, &shown, &voicing, colors, 0.85, &look))
}

/// The name of a chord as the player reads it: in their notation, with the signs they chose.
pub fn shown_chord(name: &str, settings: &Settings) -> String {
    crate::viewer::displayed_chord_name(name, false, settings.notation, settings.accidental_signs)
}

impl Instrument {
    pub fn tuning(self) -> Tuning {
        match self {
            Instrument::Guitar => Tuning::guitar(),
            Instrument::Ukulele => Tuning::ukulele(),
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Instrument::Guitar => "Guitarra",
            Instrument::Ukulele => "Ukelele",
        }
    }
}

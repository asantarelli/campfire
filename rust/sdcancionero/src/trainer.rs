// This file is part of Campfire.
// Copyright (c) Pandula Péter 2017-2026.
// https://github.com/pandulapeter/campfire
//
// This Source Code Form is subject to the terms of the Mozilla Public License, v. 2.0.
// If a copy of the MPL was not distributed with this file, You can obtain one at
// https://mozilla.org/MPL/2.0/.

//! The practice tab of the tools: the neck trainer and the study mode. What is asked and how the help goes is `chordpro::training`; this
//! draws the questions and takes the answers, counting them for the session (the totals over every session are the settings').

use chordpro::chord;
use chordpro::notation::ChordNotation;
use chordpro::scales::FretNote;
use chordpro::training::{ChordStudy, FretPosition, FretboardTrainer, Rng, StudyHelp};
use chordpro::voicings::{self, Voicing};
use eframe::egui::{self, Color32, RichText, Ui};

use crate::diagram::{self, Look};
use crate::settings::{DotLabels, Settings};
use crate::theme::SongColors;
use crate::tools::{fretboard, shown_note, ROOTS};

/// An answer, which the screen adds to the totals the settings keep.
pub struct Answer {
    pub is_study: bool,
    pub is_hit: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Mode {
    Neck,
    Study,
}

/// The chords of a key the study asks about: the ones worth knowing by heart.
const STUDIED_SUFFIXES: [&str; 7] = ["", "m", "7", "maj7", "m7", "sus2", "sus4"];

/// How far up the neck the practice goes.
const TRAINED_FRETS: u8 = 12;

const SHARP_NAMES: [&str; 12] = ["C", "C#", "D", "D#", "E", "F", "F#", "G", "G#", "A", "A#", "B"];
const FLAT_NAMES: [&str; 12] = ["C", "Db", "D", "Eb", "E", "F", "Gb", "G", "Ab", "A", "Bb", "B"];

struct NeckSession {
    trainer: FretboardTrainer,
    /// The strings it was made for, so that a change of instrument starts a new one.
    strings: Vec<i32>,
    position: FretPosition,
    answered: Option<i32>,
}

struct StudyChord {
    name: String,
    voicing: Voicing,
    /// The pressed strings, which the questions are about.
    targets: Vec<usize>,
}

struct StudySession {
    study: ChordStudy,
    chords: Vec<StudyChord>,
    /// What it was made for, so that a change of key or of instrument starts a new one.
    made_for: (usize, Vec<i32>),
    index: usize,
    target: usize,
    answered: Option<i32>,
}

pub struct Trainer {
    mode: Mode,
    key: usize,
    neck: Option<NeckSession>,
    study: Option<StudySession>,
}

impl Default for Trainer {
    fn default() -> Self {
        Trainer { mode: Mode::Neck, key: 0, neck: None, study: None }
    }
}

impl Trainer {
    pub fn show(&mut self, ui: &mut Ui, settings: &Settings, colors: &SongColors) -> Option<Answer> {
        ui.horizontal(|ui| {
            ui.selectable_value(&mut self.mode, Mode::Neck, "Mástil");
            ui.selectable_value(&mut self.mode, Mode::Study, "Acordes");
        });
        ui.add_space(6.0);
        egui::ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| match self.mode {
            Mode::Neck => self.neck(ui, settings, colors),
            Mode::Study => self.study(ui, settings, colors),
        })
        .inner
    }

    fn neck(&mut self, ui: &mut Ui, settings: &Settings, colors: &SongColors) -> Option<Answer> {
        let tuning = settings.instrument.tuning();
        // The tuning lists the strings from the lowest, and the practice counts them from the highest, as a neck is drawn.
        let open_strings: Vec<i32> = tuning.strings.iter().rev().copied().collect();
        if self.neck.as_ref().is_none_or(|session| session.strings != open_strings) {
            let mut trainer = FretboardTrainer::new(open_strings.clone(), TRAINED_FRETS, Rng::from_time());
            let position = trainer.next();
            self.neck = Some(NeckSession { trainer, strings: open_strings, position, answered: None });
        }
        let session = self.neck.as_mut()?;
        ui.label(RichText::new("Aprendé dónde están las notas: un punto en el mástil y la nota que es.").color(colors.muted));
        ui.add_space(8.0);
        // The neck is drawn from the lowest string, with the highest on top.
        let rows = session.strings.len();
        let mut neck: Vec<Vec<Option<FretNote>>> = vec![vec![None; TRAINED_FRETS as usize + 1]; rows];
        neck[rows - 1 - session.position.row][session.position.fret as usize] = Some(FretNote { degree: 0, is_root: true });
        let correct = session.trainer.pitch_class_at(session.position);
        let label = if session.answered.is_some() { note_label(correct, settings, false) } else { "?".to_string() };
        egui::ScrollArea::horizontal().id_salt("trainer_neck").show(ui, |ui| fretboard(ui, &neck, &[label], colors));
        ui.add_space(8.0);
        let mut answer = None;
        if let Some(given) = session.answered {
            if given == correct {
                ui.label(RichText::new("¡Correcto!").size(18.0).strong());
            } else {
                ui.label(RichText::new(format!("Era {}.", note_label(correct, settings, true))).size(18.0).strong().color(Color32::from_rgb(0xE5, 0x57, 0x57)));
            }
        } else {
            ui.label(RichText::new("¿Qué nota es esta?").size(18.0).strong());
        }
        if let Some(chosen) = note_buttons(ui, settings, session.answered, correct) {
            let is_hit = session.trainer.answer(session.position, chosen);
            session.answered = Some(chosen);
            answer = Some(Answer { is_study: false, is_hit });
        }
        ui.add_space(6.0);
        if ui.add_enabled(session.answered.is_some(), egui::Button::new("Siguiente")).clicked() {
            session.position = session.trainer.next();
            session.answered = None;
        }
        ui.add_space(8.0);
        score(ui, colors, session.trainer.hits, session.trainer.attempts, session.trainer.streak, session.trainer.best_streak, settings.neck_hits, settings.neck_attempts);
        answer
    }

    fn study(&mut self, ui: &mut Ui, settings: &Settings, colors: &SongColors) -> Option<Answer> {
        let tuning = settings.instrument.tuning();
        ui.horizontal_wrapped(|ui| {
            ui.label("Tonalidad");
            for (index, root) in ROOTS.iter().enumerate() {
                if ui.selectable_label(self.key == index, shown_note(root, settings.notation, settings.accidental_signs)).clicked() {
                    self.key = index;
                }
            }
        });
        let made_for = (self.key, tuning.strings.clone());
        if self.study.as_ref().is_none_or(|session| session.made_for != made_for) {
            let chords: Vec<StudyChord> = STUDIED_SUFFIXES
                .iter()
                .filter_map(|suffix| {
                    let name = format!("{}{suffix}", ROOTS[self.key]);
                    let parsed = chord::parse(&name, ChordNotation::Standard)?;
                    let voicing = voicings::voicings(&parsed, &tuning, 1).into_iter().next()?;
                    let targets: Vec<usize> = voicing.frets.iter().enumerate().filter(|(_, fret)| fret.is_some_and(|fret| fret > 0)).map(|(string, _)| string).collect();
                    (!targets.is_empty()).then_some(StudyChord { name, voicing, targets })
                })
                .collect();
            self.study = if chords.is_empty() {
                None
            } else {
                let mut study = ChordStudy::new(chords.len(), Rng::from_time());
                let index = study.next_chord();
                let target = chords[index].targets[study.pick(chords[index].targets.len())];
                Some(StudySession { study, chords, made_for, index, target, answered: None })
            };
        }
        let Some(session) = self.study.as_mut() else {
            ui.label("No hay posiciones de acordes para estudiar con este instrumento.");
            return None;
        };
        ui.label(RichText::new("Aprendé qué nota sostiene cada punto: el acorde con un punto marcado. La ayuda se va retirando a medida que acertás un acorde.").color(colors.muted));
        ui.add_space(6.0);
        let chord = &session.chords[session.index];
        let help = session.study.help_for(session.index);
        let correct = (tuning.strings[session.target] + chord.voicing.frets[session.target].unwrap_or(0) as i32).rem_euclid(12);
        // What the diagram says is the study's: no notes, no colors, and no sound under a press, which would give the answer away.
        let look = Look { labels: DotLabels::None, is_colored: false, strings: &[], root: None, study: Some(session.target) };
        ui.vertical_centered(|ui| {
            diagram::card(ui, settings.is_dark, |ui| diagram::show(ui, &chord.name, &chord.voicing, colors, 1.4, &look));
        });
        ui.add_space(6.0);
        ui.label(RichText::new("¿Qué nota sostiene el punto marcado?").size(18.0).strong());
        match help {
            StudyHelp::Notes => {
                let mut notes: Vec<i32> = Vec::new();
                for (string, fret) in chord.voicing.frets.iter().enumerate() {
                    if let Some(fret) = fret {
                        let pitch = (tuning.strings[string] + *fret as i32).rem_euclid(12);
                        if !notes.contains(&pitch) {
                            notes.push(pitch);
                        }
                    }
                }
                let names: Vec<String> = notes.iter().map(|&pitch| note_label(pitch, settings, false)).collect();
                ui.label(RichText::new(format!("Las notas de este acorde: {}", names.join(" "))).color(colors.muted));
            }
            StudyHelp::String => {
                let open = tuning.strings[session.target].rem_euclid(12);
                ui.label(RichText::new(format!("Ayuda: la cuerda es {} al aire", note_label(open, settings, false))).color(colors.muted));
            }
            StudyHelp::Blank => {
                ui.label(RichText::new("Sin ayuda").color(colors.muted));
            }
        }
        let mut answer = None;
        if let Some(given) = session.answered {
            if given == correct {
                ui.label(RichText::new("¡Correcto!").size(18.0).strong());
            } else {
                ui.label(RichText::new(format!("Era {}.", note_label(correct, settings, true))).size(18.0).strong().color(Color32::from_rgb(0xE5, 0x57, 0x57)));
            }
        }
        if let Some(chosen) = note_buttons(ui, settings, session.answered, correct) {
            let is_hit = chosen == correct;
            session.study.record(session.index, is_hit);
            session.answered = Some(chosen);
            answer = Some(Answer { is_study: true, is_hit });
        }
        ui.add_space(6.0);
        if ui.add_enabled(session.answered.is_some(), egui::Button::new("Siguiente")).clicked() {
            session.index = session.study.next_chord();
            let targets = session.chords[session.index].targets.clone();
            session.target = targets[session.study.pick(targets.len())];
            session.answered = None;
        }
        ui.add_space(8.0);
        score(ui, colors, session.study.hits, session.study.attempts, session.study.streak, session.study.best_streak, settings.study_hits, settings.study_attempts);
        answer
    }
}

/// How the player reads the note at the pitch class: in their notation, with a black key named both ways where `both` says so.
fn note_label(pitch_class: i32, settings: &Settings, both: bool) -> String {
    let index = pitch_class.rem_euclid(12) as usize;
    let sharp = shown_note(SHARP_NAMES[index], settings.notation, settings.accidental_signs);
    let flat = shown_note(FLAT_NAMES[index], settings.notation, settings.accidental_signs);
    if sharp == flat {
        sharp
    } else if both {
        format!("{sharp}/{flat}")
    } else {
        sharp
    }
}

/// The twelve notes to answer with: once `answered` the right one is green and a wrong one the player gave is red. Answers the note pressed.
fn note_buttons(ui: &mut Ui, settings: &Settings, answered: Option<i32>, correct: i32) -> Option<i32> {
    let mut chosen = None;
    ui.horizontal_wrapped(|ui| {
        for pitch_class in 0..12 {
            let mut button = egui::Button::new(RichText::new(note_label(pitch_class, settings, true)).size(16.0)).min_size(egui::vec2(64.0, 32.0));
            if let Some(given) = answered {
                if pitch_class == correct {
                    button = button.fill(Color32::from_rgb(0x3E, 0x8E, 0x41));
                } else if pitch_class == given {
                    button = button.fill(Color32::from_rgb(0xB3, 0x3B, 0x3B));
                }
            }
            if ui.add_enabled(answered.is_none(), button).clicked() {
                chosen = Some(pitch_class);
            }
        }
    });
    chosen
}

/// How the answers have gone: this session's, the run of right ones, and every session's together.
#[allow(clippy::too_many_arguments)]
fn score(ui: &mut Ui, colors: &SongColors, hits: u32, attempts: u32, streak: u32, best: u32, total_hits: u32, total_attempts: u32) {
    ui.label(RichText::new(format!("Esta sesión: {hits} de {attempts}")).color(colors.muted));
    ui.label(RichText::new(format!("Seguidas: {streak} (mejor {best})")).color(colors.muted));
    ui.label(RichText::new(format!("Todas las sesiones: {total_hits} de {total_attempts}")).color(colors.muted));
}

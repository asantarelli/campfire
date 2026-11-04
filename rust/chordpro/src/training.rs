// This file is part of Campfire.
// Copyright (c) Pandula Péter 2017-2026.
// https://github.com/pandulapeter/campfire
//
// This Source Code Form is subject to the terms of the Mozilla Public License, v. 2.0.
// If a copy of the MPL was not distributed with this file, You can obtain one at
// https://mozilla.org/MPL/2.0/.

//! The practice: the neck trainer, which asks which note is at a place on the neck, and the study mode, which asks which note a dot of a
//! chord diagram holds with less and less help. Both ask more of what is missed. The Kotlin app has the same rules (`chordpro/training`).

/// A small random source, which is all the practice needs and which a test can seed.
#[derive(Debug, Clone)]
pub struct Rng(u64);

impl Rng {
    pub fn new(seed: u64) -> Rng {
        Rng(seed.max(1))
    }

    /// A source seeded from the clock, for the practice itself.
    pub fn from_time() -> Rng {
        let nanos = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|time| time.as_nanos() as u64).unwrap_or(1);
        Rng::new(nanos)
    }

    /// A number from 0 up to, not including, 1.
    pub fn next_f64(&mut self) -> f64 {
        // xorshift64*
        self.0 ^= self.0 >> 12;
        self.0 ^= self.0 << 25;
        self.0 ^= self.0 >> 27;
        (self.0.wrapping_mul(0x2545_F491_4F6C_DD1D) >> 11) as f64 / (1u64 << 53) as f64
    }

    /// A number from 0 up to, not including, `limit`.
    pub fn below(&mut self, limit: usize) -> usize {
        ((self.next_f64() * limit as f64) as usize).min(limit.saturating_sub(1))
    }
}

const INITIAL_WEIGHT: f64 = 1.0;
const HIT_FACTOR: f64 = 0.6;
const MISS_FACTOR: f64 = 2.0;
const MISS_BONUS: f64 = 1.0;
const MIN_WEIGHT: f64 = 0.3;
const MAX_WEIGHT: f64 = 16.0;

/// The index of one of `weights`, chosen in proportion to it, never `excluded` unless it is the only one there is.
fn choose_weighted(weights: &[f64], excluded: Option<usize>, rng: &mut Rng) -> usize {
    let mut candidates: Vec<usize> = (0..weights.len()).filter(|index| Some(*index) != excluded).collect();
    if candidates.is_empty() {
        candidates = (0..weights.len()).collect();
    }
    let mut point = rng.next_f64() * candidates.iter().map(|&index| weights[index]).sum::<f64>();
    for &candidate in &candidates {
        point -= weights[candidate];
        if point < 0.0 {
            return candidate;
        }
    }
    *candidates.last().unwrap_or(&0)
}

fn adjusted(weight: f64, is_hit: bool) -> f64 {
    if is_hit {
        (weight * HIT_FACTOR).max(MIN_WEIGHT)
    } else {
        (weight * MISS_FACTOR + MISS_BONUS).min(MAX_WEIGHT)
    }
}

/// A place on a neck: the `row` of a string counted from the highest one, and the `fret`, 0 being open.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FretPosition {
    pub row: usize,
    pub fret: u8,
}

/// The neck trainer. Where the player is wrong the place comes back sooner, and a place that is known is asked about less.
pub struct FretboardTrainer {
    open_strings: Vec<i32>,
    positions: Vec<FretPosition>,
    weights: Vec<f64>,
    last: Option<usize>,
    rng: Rng,
    pub attempts: u32,
    pub hits: u32,
    pub streak: u32,
    pub best_streak: u32,
}

impl FretboardTrainer {
    /// `open_strings` are the open notes as MIDI notes from the highest string, and `max_fret` the highest fret asked about.
    pub fn new(open_strings: Vec<i32>, max_fret: u8, rng: Rng) -> FretboardTrainer {
        let positions: Vec<FretPosition> = (0..open_strings.len()).flat_map(|row| (0..=max_fret).map(move |fret| FretPosition { row, fret })).collect();
        FretboardTrainer { weights: vec![INITIAL_WEIGHT; positions.len()], open_strings, positions, last: None, rng, attempts: 0, hits: 0, streak: 0, best_streak: 0 }
    }

    /// The note at the place, as a pitch class from 0 (C) to 11.
    pub fn pitch_class_at(&self, position: FretPosition) -> i32 {
        (self.open_strings[position.row] + position.fret as i32).rem_euclid(12)
    }

    /// The next place to ask about.
    pub fn next(&mut self) -> FretPosition {
        let index = choose_weighted(&self.weights, self.last, &mut self.rng);
        self.last = Some(index);
        self.positions[index]
    }

    /// Records the player's answer for the place and says whether it was the note there.
    pub fn answer(&mut self, position: FretPosition, pitch_class: i32) -> bool {
        let is_hit = pitch_class.rem_euclid(12) == self.pitch_class_at(position);
        if let Some(index) = self.positions.iter().position(|known| *known == position) {
            self.weights[index] = adjusted(self.weights[index], is_hit);
        }
        self.attempts += 1;
        if is_hit {
            self.hits += 1;
            self.streak += 1;
            self.best_streak = self.best_streak.max(self.streak);
        } else {
            self.streak = 0;
        }
        is_hit
    }
}

/// How much a chord's diagram helps while the player is asked the note of one of its dots, from the most to the least.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StudyHelp {
    /// The notes of the chord are listed.
    Notes,
    /// The string the dot is on is named.
    String,
    /// Nothing but the dot.
    Blank,
}

const HELPS: [StudyHelp; 3] = [StudyHelp::Notes, StudyHelp::String, StudyHelp::Blank];
const HITS_TO_ADVANCE: u32 = 2;

/// The study mode: it takes the help away from a chord after two right answers on it and gives it back after a wrong one, and comes back
/// to a chord that is missed sooner.
pub struct ChordStudy {
    levels: Vec<usize>,
    streaks: Vec<u32>,
    weights: Vec<f64>,
    last: Option<usize>,
    rng: Rng,
    pub attempts: u32,
    pub hits: u32,
    pub streak: u32,
    pub best_streak: u32,
}

impl ChordStudy {
    pub fn new(chord_count: usize, rng: Rng) -> ChordStudy {
        ChordStudy { levels: vec![0; chord_count], streaks: vec![0; chord_count], weights: vec![INITIAL_WEIGHT; chord_count], last: None, rng, attempts: 0, hits: 0, streak: 0, best_streak: 0 }
    }

    pub fn help_for(&self, chord: usize) -> StudyHelp {
        HELPS[self.levels[chord]]
    }

    /// The next chord to ask about.
    pub fn next_chord(&mut self) -> usize {
        let index = choose_weighted(&self.weights, self.last, &mut self.rng);
        self.last = Some(index);
        index
    }

    /// Picks one of `count` dots to ask about.
    pub fn pick(&mut self, count: usize) -> usize {
        self.rng.below(count)
    }

    /// Records whether the player was right about the chord.
    pub fn record(&mut self, chord: usize, is_hit: bool) {
        self.attempts += 1;
        self.weights[chord] = adjusted(self.weights[chord], is_hit);
        if is_hit {
            self.hits += 1;
            self.streak += 1;
            self.best_streak = self.best_streak.max(self.streak);
            self.streaks[chord] += 1;
            if self.streaks[chord] >= HITS_TO_ADVANCE && self.levels[chord] + 1 < HELPS.len() {
                self.levels[chord] += 1;
                self.streaks[chord] = 0;
            }
        } else {
            self.streak = 0;
            self.streaks[chord] = 0;
            self.levels[chord] = self.levels[chord].saturating_sub(1);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const GUITAR: [i32; 6] = [64, 59, 55, 50, 45, 40];

    #[test]
    fn the_note_at_a_place_is_the_open_note_of_its_string_and_the_frets_above_it() {
        let trainer = FretboardTrainer::new(GUITAR.to_vec(), 12, Rng::new(1));
        assert_eq!(trainer.pitch_class_at(FretPosition { row: 0, fret: 0 }), 4);
        assert_eq!(trainer.pitch_class_at(FretPosition { row: 0, fret: 8 }), 0);
        assert_eq!(trainer.pitch_class_at(FretPosition { row: 4, fret: 3 }), 0);
    }

    #[test]
    fn the_score_counts_the_right_answers_and_the_best_run_of_them() {
        let mut trainer = FretboardTrainer::new(GUITAR.to_vec(), 12, Rng::new(2));
        let place = FretPosition { row: 5, fret: 3 };
        assert!(trainer.answer(place, 7));
        assert!(trainer.answer(place, 7));
        assert!(!trainer.answer(place, 8));
        assert!(trainer.answer(place, 7));
        assert_eq!((trainer.attempts, trainer.hits, trainer.best_streak, trainer.streak), (4, 3, 2, 1));
    }

    #[test]
    fn a_place_is_never_asked_twice_in_a_row_and_stays_within_the_frets_asked() {
        let mut trainer = FretboardTrainer::new(GUITAR.to_vec(), 5, Rng::new(3));
        let mut before = None;
        for _ in 0..300 {
            let place = trainer.next();
            assert_ne!(Some(place), before);
            assert!(place.row < 6 && place.fret <= 5);
            before = Some(place);
        }
    }

    #[test]
    fn a_place_that_is_missed_comes_back_more_often_than_one_that_is_known() {
        let mut trainer = FretboardTrainer::new(vec![40, 45], 3, Rng::new(4));
        let (known, missed) = (FretPosition { row: 0, fret: 0 }, FretPosition { row: 0, fret: 1 });
        for _ in 0..6 {
            trainer.answer(known, 4);
        }
        for _ in 0..4 {
            trainer.answer(missed, 0);
        }
        let asked: Vec<FretPosition> = (0..2000).map(|_| trainer.next()).collect();
        let count = |place: FretPosition| asked.iter().filter(|&&asked| asked == place).count();
        assert!(count(missed) > count(known) * 2);
    }

    #[test]
    fn the_help_goes_after_two_right_answers_on_a_chord_and_comes_back_with_a_wrong_one() {
        let mut study = ChordStudy::new(2, Rng::new(5));
        assert_eq!(study.help_for(0), StudyHelp::Notes);
        study.record(0, true);
        assert_eq!(study.help_for(0), StudyHelp::Notes);
        study.record(0, true);
        assert_eq!(study.help_for(0), StudyHelp::String);
        for _ in 0..6 {
            study.record(0, true);
        }
        assert_eq!(study.help_for(0), StudyHelp::Blank);
        study.record(0, false);
        assert_eq!(study.help_for(0), StudyHelp::String);
        assert_eq!(study.help_for(1), StudyHelp::Notes);
    }

    #[test]
    fn the_chords_that_are_missed_are_asked_about_more_often() {
        let mut study = ChordStudy::new(3, Rng::new(6));
        for _ in 0..5 {
            study.record(0, true);
        }
        for _ in 0..3 {
            study.record(1, false);
        }
        let asked: Vec<usize> = (0..2000).map(|_| study.next_chord()).collect();
        assert!(asked.iter().filter(|&&chord| chord == 1).count() > asked.iter().filter(|&&chord| chord == 0).count());
    }
}

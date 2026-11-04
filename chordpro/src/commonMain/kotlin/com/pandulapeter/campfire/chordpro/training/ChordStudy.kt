/*
 * This file is part of Campfire.
 * Copyright (c) Pandula Péter 2017-2026.
 * https://github.com/pandulapeter/campfire
 *
 * This Source Code Form is subject to the terms of the Mozilla Public License, v. 2.0.
 * If a copy of the MPL was not distributed with this file, You can obtain one at
 * https://mozilla.org/MPL/2.0/.
 */
package com.pandulapeter.campfire.chordpro.training

import kotlin.random.Random

/** How much a chord's diagram helps while the player is asked the note of one of its dots, from the most to the least. */
public enum class StudyHelp {
    /** The dots carry their fingers and the notes of the chord are listed. */
    NOTES,

    /** The dots carry their fingers only. */
    FINGERS,

    /** The dots carry nothing: the one asked about is marked and the rest are covered. */
    BLANK,
}

/**
 * The study mode: for each of [chordCount] chords it asks which note a finger holds, taking the help away as the player gets it right and
 * giving it back, and coming back to the chord sooner, where the player gets it wrong.
 *
 * Two right answers in a row on a chord take one help away from it; a wrong one gives one back.
 */
public class ChordStudy(
    chordCount: Int,
    private val random: Random = Random.Default,
) {
    private val levels = MutableList(chordCount) { 0 }
    private val streaks = MutableList(chordCount) { 0 }
    private val weights = MutableList(chordCount) { INITIAL_WEIGHT }
    private var last: Int? = null

    public var attempts: Int = 0
        private set
    public var hits: Int = 0
        private set

    /** How many answers in a row were right, up to now, whichever chords they were about. */
    public var streak: Int = 0
        private set
    public var bestStreak: Int = 0
        private set

    /** The help the chord at [chord] has at the moment. */
    public fun helpFor(chord: Int): StudyHelp = StudyHelp.entries[levels[chord]]

    /** The next chord to ask about: the ones missed more often come more often. */
    public fun nextChord(): Int = chooseWeighted(weights, excluded = last, random = random).also { last = it }

    /** Records whether the player was right about [chord]. */
    public fun record(chord: Int, isHit: Boolean) {
        attempts++
        weights[chord] = adjustedWeight(weights[chord], isHit)
        if (isHit) {
            hits++
            streak++
            bestStreak = maxOf(bestStreak, streak)
            streaks[chord]++
            if (streaks[chord] >= HITS_TO_ADVANCE && levels[chord] < StudyHelp.entries.lastIndex) {
                levels[chord]++
                streaks[chord] = 0
            }
        } else {
            streak = 0
            streaks[chord] = 0
            if (levels[chord] > 0) levels[chord]--
        }
    }

    private companion object {
        const val HITS_TO_ADVANCE = 2
    }
}

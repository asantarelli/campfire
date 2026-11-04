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

import kotlin.math.max
import kotlin.random.Random

/** A place on a neck: the [row] of a string counted from the top of the diagram, the highest string, and the [fret], 0 being open. */
public data class FretPosition(val row: Int, val fret: Int)

/**
 * The neck trainer: it asks which note is at a place on the neck and keeps the score. Where the player is wrong the place comes back
 * sooner, and a place that is known is asked about less, so the practice goes to what is not learnt yet.
 *
 * @param openStrings The open note of each string as a MIDI note, from the highest string, which is the row 0 a diagram draws on top.
 * @param maxFret The highest fret asked about.
 */
public class FretboardTrainer(
    private val openStrings: List<Int>,
    private val maxFret: Int = 12,
    private val random: Random = Random.Default,
) {
    private val positions = openStrings.indices.flatMap { row -> (0..maxFret).map { FretPosition(row, it) } }
    private val weights = MutableList(positions.size) { INITIAL_WEIGHT }
    private var last: Int? = null

    public var attempts: Int = 0
        private set
    public var hits: Int = 0
        private set

    /** How many answers in a row were right, up to now. */
    public var streak: Int = 0
        private set
    public var bestStreak: Int = 0
        private set

    /** The note at [position], as a pitch class from 0 (C) to 11. */
    public fun pitchClassAt(position: FretPosition): Int = (openStrings[position.row] + position.fret).mod(OCTAVE)

    /** The next place to ask about. */
    public fun next(): FretPosition {
        val index = chooseWeighted(weights, excluded = last, random = random)
        last = index
        return positions[index]
    }

    /** Records the player's [pitchClass] for [position] and says whether it was the note there. */
    public fun answer(position: FretPosition, pitchClass: Int): Boolean {
        val isHit = pitchClass.mod(OCTAVE) == pitchClassAt(position)
        val index = positions.indexOf(position)
        if (index >= 0) weights[index] = adjustedWeight(weights[index], isHit)
        attempts++
        if (isHit) {
            hits++
            streak++
            bestStreak = max(bestStreak, streak)
        } else {
            streak = 0
        }
        return isHit
    }

    /** How the answers of this session were, from 0 to 1, or null before the first. */
    public val accuracy: Float? get() = if (attempts == 0) null else hits.toFloat() / attempts

    private companion object {
        const val OCTAVE = 12
    }
}

/*
 * This file is part of Campfire.
 * Copyright (c) Pandula Péter 2017-2026.
 * https://github.com/pandulapeter/campfire
 *
 * This Source Code Form is subject to the terms of the Mozilla Public License, v. 2.0.
 * If a copy of the MPL was not distributed with this file, You can obtain one at
 * https://mozilla.org/MPL/2.0/.
 */
package com.pandulapeter.campfire.guitarpro.layout

import kotlin.math.absoluteValue
import kotlin.math.floor

/** Where a note is written: [step] counted in notes of the scale, [alteration] 1 for a sharp, -1 for a flat and 0 for neither. */
internal data class PitchPosition(val step: Int, val alteration: Int)

private val SHARP_SPELLING = listOf(0 to 0, 0 to 1, 1 to 0, 1 to 1, 2 to 0, 3 to 0, 3 to 1, 4 to 0, 4 to 1, 5 to 0, 5 to 1, 6 to 0)
private val FLAT_SPELLING = listOf(0 to 0, 1 to -1, 1 to 0, 2 to -1, 2 to 0, 3 to 0, 4 to -1, 4 to 0, 5 to -1, 5 to 0, 6 to -1, 6 to 0)
private val SHARP_ORDER = listOf(3, 0, 4, 1, 5, 2, 6)
private val FLAT_ORDER = listOf(6, 2, 5, 1, 4, 0, 3)

/**
 * The step on the staff a MIDI note is written on, counted in notes of the scale from a C (the C of the octave below the staff's is the
 * first), and how the note is altered there. The spelling follows the key: sharps in a sharp key and in C major, flats in a flat one.
 */
internal fun pitchPosition(midi: Int, key: Int): PitchPosition {
    val octave = floor(midi / 12.0).toInt() - 1
    val (letter, alteration) = (if (key < 0) FLAT_SPELLING else SHARP_SPELLING)[((midi % 12) + 12) % 12]
    return PitchPosition(octave * 7 + letter, alteration)
}

/** How the key alters a note of this letter (0 for C, 1 for D... 6 for B). */
internal fun keyAlteration(letter: Int, key: Int): Int {
    val count = key.absoluteValue.coerceAtMost(7)
    return when {
        key > 0 && letter in SHARP_ORDER.take(count) -> 1
        key < 0 && letter in FLAT_ORDER.take(count) -> -1
        else -> 0
    }
}

/*
 * This file is part of Campfire.
 * Copyright (c) Pandula Péter 2017-2026.
 * https://github.com/pandulapeter/campfire
 *
 * This Source Code Form is subject to the terms of the Mozilla Public License, v. 2.0.
 * If a copy of the MPL was not distributed with this file, You can obtain one at
 * https://mozilla.org/MPL/2.0/.
 */
package com.pandulapeter.campfire.guitarpro.model

/** How many ticks a quarter note lasts, the unit every duration is counted in. */
public const val QUARTER_TICKS: Int = 960

public data class Beat(
    /** Where it starts, in ticks from the start of the song. */
    val start: Int,
    val duration: Duration,
    val status: BeatStatus,
    val notes: List<Note>,
    val text: String? = null,
    /** The name of the chord written over it. */
    val chord: String? = null,
    /** The diagram of the chord written over it, which a file may hold without a name. */
    val chordShape: ChordShape? = null,
    /** The tempo it changes to, beats a minute. */
    val tempo: Int? = null,
)

/**
 * A chord diagram: the fret it starts at and the fret of each string from the highest, -1 for a string that is not played and 0 for one that is
 * played open.
 */
public data class ChordShape(val firstFret: Int, val frets: List<Int>)

public enum class BeatStatus {
    EMPTY,
    NORMAL,
    REST,
}

public data class Duration(
    /** 1 a whole note, 2 a half, 4 a quarter, 8 an eighth... */
    val value: Int,
    val isDotted: Boolean = false,
    /** A tuplet is `enters` notes in the time of `times`: a triplet is 3 in the time of 2. */
    val enters: Int = 1,
    val times: Int = 1,
) {

    /** How long it lasts, in ticks. */
    val ticks: Int
        get() {
            var ticks = QUARTER_TICKS * 4 / value.coerceAtLeast(1)
            if (isDotted) {
                ticks += ticks / 2
            }
            return ticks * times.coerceAtLeast(1) / enters.coerceAtLeast(1)
        }
}

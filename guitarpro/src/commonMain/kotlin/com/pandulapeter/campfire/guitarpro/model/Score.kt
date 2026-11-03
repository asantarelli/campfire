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

/**
 * What a Guitar Pro file says, as much of it as a score is read from: the tracks, their measures and the beats of each, with the notes
 * on strings and frets and what is done to them.
 */
public data class Score(
    val title: String = "",
    val subtitle: String = "",
    val artist: String = "",
    val album: String = "",
    val words: String = "",
    val music: String = "",
    val copyright: String = "",
    /** Who wrote the tablature. */
    val tabbedBy: String = "",
    /** Beats a minute. */
    val tempo: Int = 0,
    /** The key as the number of sharps (positive) or flats (negative). */
    val key: Int = 0,
    /** What each measure is, which every track shares. */
    val headers: List<MeasureHeader> = emptyList(),
    val tracks: List<Track> = emptyList(),
)

public data class MeasureHeader(
    val numerator: Int,
    val denominator: Int,
    val isRepeatOpen: Boolean = false,
    /** The number of times the repeat that closes here is played, or -1 where no repeat closes. */
    val repeatClose: Int = -1,
    /** The endings this measure is one of, as bits: 1 the first, 2 the second. */
    val repeatAlternative: Int = 0,
    val marker: String? = null,
    /** The key this measure changes to, as sharps (positive) or flats (negative). */
    val keyChange: Int? = null,
    val hasDoubleBar: Boolean = false,
    /** Where the measure starts, in ticks from the start of the song. */
    val start: Int = 0,
) {

    /** How long the measure lasts, in ticks. */
    val length: Int get() = numerator * (QUARTER_TICKS * 4 / denominator.coerceAtLeast(1))
}

public data class Track(
    val name: String,
    /** The tuning, one MIDI note per string from the highest (the first string) to the lowest. */
    val strings: List<Int>,
    /** The fret a capo is set at, 0 for none. */
    val capo: Int,
    val isPercussion: Boolean,
    /** One per measure of the score. */
    val measures: List<Measure>,
)

public data class Measure(
    /**
     * The beats of each voice, the first being the one a score is read from; the second, where the file has one, holds what is played
     * with it.
     */
    val voices: List<List<Beat>>,
)

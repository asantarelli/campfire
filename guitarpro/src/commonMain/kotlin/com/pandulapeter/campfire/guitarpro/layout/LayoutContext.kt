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

import com.pandulapeter.campfire.guitarpro.model.Beat
import com.pandulapeter.campfire.guitarpro.model.Score
import com.pandulapeter.campfire.guitarpro.model.Track

/** What every painter of a layout reads: the score and its track, what is shown, and what planning the measures decided. */
internal class LayoutContext(
    val score: Score,
    val track: Track,
    val options: LayoutOptions,
    val keys: List<Int>,
    val plans: List<MeasurePlan>,
)

/** Where the staves of a system are. */
internal class Staves(
    val notationTop: Float,
    val tabTop: Float,
    val tabBottom: Float,
    val strings: Int,
    val system: Int,
) {

    fun stringY(string: Int): Float = tabTop + (string - 1f) * TAB_GAP

    /** The first line that anything is above. */
    fun firstStaff(options: LayoutOptions): Float = if (options.showNotation) notationTop else tabTop
}

/** One beat as the staves see it. */
internal class Column(
    val beat: Beat,
    /** The middle of its notes. */
    val x: Float,
    /** Its place in the measure, in ticks. */
    val offset: Int,
    /** For each note, its position on the staff. */
    val heads: List<Head>,
)

internal class Head(
    /** Steps up from the C of the octave below the staff's: a position the staff lines are counted in. */
    val step: Int,
    val y: Float,
    /** The sign the note needs before it where it needs one: 1 a sharp, -1 a flat, 0 a natural. */
    val accidental: Int?,
)

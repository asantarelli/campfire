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

import com.pandulapeter.campfire.guitarpro.draw.Draw
import com.pandulapeter.campfire.guitarpro.draw.Glyphs
import com.pandulapeter.campfire.guitarpro.draw.Point
import com.pandulapeter.campfire.guitarpro.draw.polygonOf
import com.pandulapeter.campfire.guitarpro.model.BeatStatus
import com.pandulapeter.campfire.guitarpro.model.MeasureHeader
import com.pandulapeter.campfire.guitarpro.model.QUARTER_TICKS

/** Where a stem is, the head it starts at (the one farthest from where it ends), where it ends and which way it points. */
internal data class StemEnd(val x: Float, val from: Float, val tip: Float, val isUp: Boolean)

/** The beams of a group whose stems are at [xs] and last [values]: the first across all of them, and one more for each extra flag. */
internal fun drawBeams(draws: MutableList<Draw>, xs: List<Float>, values: List<Int>, tip: Float, isUp: Boolean) {
    val sign = if (isUp) 1f else -1f
    val levels = values.maxOfOrNull { Glyphs.flagCountOf(it) } ?: 1
    for (level in 1..levels) {
        val offset = (level - 1f) * 0.85f * sign
        var index = 0
        while (index < xs.size) {
            if (Glyphs.flagCountOf(values[index]) < level) {
                index++
                continue
            }
            var end = index
            while (end + 1 < xs.size && Glyphs.flagCountOf(values[end + 1]) >= level) {
                end++
            }
            // Where only one note has the level, a short stub points to its neighbour.
            val (from, to) = when {
                end > index -> xs[index] to xs[end] + Glyphs.STEM / 2f
                index + 1 < xs.size -> xs[index] to xs[index] + 1.2f
                else -> xs[index] - 1.2f to xs[index]
            }
            val y1 = tip + offset
            val y2 = y1 + Glyphs.BEAM * sign
            draws += polygonOf(from to y1, to to y1, to to y2, from to y2)
            index = end + 1
        }
    }
}

/** A flag drawn for a stem that points up from [tip], reflected through it for a stem that points down. */
internal fun mirroredDown(draw: Draw, tip: Float): Draw =
    if (draw is Draw.Polyline) draw.copy(points = draw.points.map { Point(it.x, 2f * tip - it.y) }) else draw

/** Whether the stems of notes at [steps] go up: they do unless the note farthest from the middle line is above it. */
internal fun stemIsUp(steps: List<Int>): Boolean {
    val highest = steps.maxOrNull() ?: return true
    val lowest = steps.minOrNull() ?: return true
    return highest - MIDDLE_LINE <= MIDDLE_LINE - lowest
}

/** The beams a measure groups its beats in: the beats of the same part of the measure that are short enough, and have notes, joined. */
internal fun beamGroups(columns: List<Column>, header: MeasureHeader): List<List<Int>> {
    val window = if (header.denominator == 8 && header.numerator % 3 == 0 && header.numerator >= 6) QUARTER_TICKS * 3 / 2 else QUARTER_TICKS
    val groups = mutableListOf<MutableList<Int>>()
    var lastWindow: Int? = null
    for ((position, column) in columns.withIndex()) {
        val beat = column.beat
        val isBeamable = beat.status != BeatStatus.REST && beat.duration.value >= 8 && beat.notes.isNotEmpty()
        if (!isBeamable) {
            groups += mutableListOf(position)
            lastWindow = null
            continue
        }
        val current = column.offset / window
        if (lastWindow == current) {
            groups.last() += position
        } else {
            groups += mutableListOf(position)
        }
        lastWindow = current
    }
    return groups
}

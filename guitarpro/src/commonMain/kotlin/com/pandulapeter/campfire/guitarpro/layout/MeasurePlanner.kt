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
import com.pandulapeter.campfire.guitarpro.model.BeatStatus
import com.pandulapeter.campfire.guitarpro.model.QUARTER_TICKS
import com.pandulapeter.campfire.guitarpro.model.Score
import com.pandulapeter.campfire.guitarpro.model.Track
import kotlin.math.max
import kotlin.math.pow

/** The space between two strings of the tablature. */
internal const val TAB_GAP = 1.2f

/** From the top of a system to the top of its first staff: room for the markers, the chords and the notes above the staff. */
internal const val ABOVE = 7.2f

/** What a system takes more above its staff where a chord diagram is drawn there. */
internal const val DIAGRAM_BAND = 6f

/** From the notation staff to the tablature one: room for the notes and the stems under the staff. */
internal const val BETWEEN = 4.4f

/** Under the tablature, for the rhythm. */
internal const val RHYTHM = 4.2f

/** Between systems. */
internal const val SYSTEM_GAP = 2f

/** The room a clef takes at the start of a system. */
internal const val CLEF = 3.6f

/** A time signature's. */
internal const val TIME_SIGNATURE = 2.8f

/** What a measure takes beside its beats: a little room after its bar line, and before the next. */
internal const val MEASURE_PADDING = 1.5f

/** The middle line of the staff as a position in steps (B4). */
internal const val MIDDLE_LINE = 34

/** The least a beat takes. */
private const val MIN_BEAT = 2.4f

/** One beat in the room of its measure: where its room starts in the body, before the measure is stretched, and how much it has. */
internal class BeatPlan(val beat: Beat, val x: Float, val width: Float)

/** A measure's beats, and how wide its body is before it is stretched to the width of the system. */
internal class MeasurePlan(val beats: List<BeatPlan>, val body: Float)

/** The measures a system holds. */
internal class SystemPlan(val measures: IntRange)

private fun beatWidth(beat: Beat): Float {
    var width = max(4f * (beat.duration.ticks.toFloat() / QUARTER_TICKS).pow(0.55f), MIN_BEAT)
    if (beat.notes.any { it.fret >= 10 }) {
        width = max(width, MIN_BEAT + 0.4f)
    }
    beat.chord?.let { width = max(width, it.length * 0.8f + 0.6f) }
    beat.text?.let { width = max(width, it.length * 0.55f + 0.6f) }
    if (beat.notes.any { it.effects.grace != null }) {
        width += 1.2f
    }
    // A note that is not on the lines of its key brings a sign before it, which takes room.
    if (beat.notes.any { it.fret > 0 } && beat.notes.size > 2) {
        width += 0.3f
    }
    return width
}

internal fun planMeasure(track: Track, index: Int): MeasurePlan {
    val voice = track.measures.getOrNull(index)?.voices?.firstOrNull().orEmpty()
    val beats = mutableListOf<BeatPlan>()
    var x = 0f
    for (beat in voice.filter { it.status != BeatStatus.EMPTY }) {
        val width = beatWidth(beat)
        beats += BeatPlan(beat, x, width)
        x += width
    }
    return MeasurePlan(beats, if (beats.isEmpty()) 5f else x)
}

/** What stands at the start of a system before its first measure: the clef, the key and, on the first one, the time signature. */
internal fun prefixWidth(options: LayoutOptions, key: Int): Float {
    val notation = if (options.showNotation) CLEF + keyWidth(key) else 0f
    val tab = if (options.showTab) 2.8f else 0f
    return 0.6f + max(notation, tab)
}

/** What a measure takes besides its beats: a time signature where it changes, and the marks of a repeat. */
internal fun measureExtra(score: Score, index: Int): Float {
    val header = score.headers[index]
    var extra = MEASURE_PADDING
    if (index == 0 || changesTime(score, index)) extra += TIME_SIGNATURE
    if (header.isRepeatOpen) extra += 1.4f
    if (header.repeatClose > -1) extra += 1.4f
    return extra
}

internal fun changesTime(score: Score, index: Int): Boolean {
    val (header, previous) = score.headers[index] to score.headers[index - 1]
    return previous.numerator != header.numerator || previous.denominator != header.denominator
}

internal fun breakIntoSystems(plans: List<MeasurePlan>, keys: List<Int>, score: Score, options: LayoutOptions): List<SystemPlan> {
    val systems = mutableListOf<SystemPlan>()
    var start = 0
    while (start < plans.size) {
        var width = prefixWidth(options, keys[start])
        var end = start
        while (end < plans.size) {
            val needed = plans[end].body + measureExtra(score, end)
            if (end > start && width + needed > options.width) break
            width += needed
            end++
        }
        systems += SystemPlan(start until end)
        start = end
    }
    return systems
}

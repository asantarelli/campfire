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
import com.pandulapeter.campfire.guitarpro.model.Score
import kotlin.math.absoluteValue

/** What to show and how wide to set it. */
public data class LayoutOptions(
    /** How wide a system may be, in staff spaces. */
    val width: Float = 80f,
    /** The notes on a staff. */
    val showNotation: Boolean = true,
    /** The notes on the strings. */
    val showTab: Boolean = true,
    /** The stems and beams under the tablature, which say how long each note lasts. */
    val showRhythm: Boolean = true,
)

/** The page: its systems from the top, and how big it is, in staff spaces. */
public data class ScoreLayout(
    val systems: List<ScoreSystem> = emptyList(),
    val width: Float = 0f,
    val height: Float = 0f,
)

/** One line of the score: some measures side by side. */
public data class ScoreSystem(
    val top: Float,
    val height: Float,
    val draws: List<Draw>,
)

/**
 * Sets a track of a score on a page: measures shared out into systems as wide as the window, and in each the notes on a staff and on
 * tablature, with their stems, beams, ties and the marks of how they are played. The answer is a list of [Draw]s in staff spaces; what
 * they look like (colors, fonts, size) is for whoever paints them.
 */
public object ScoreLayouter {

    public fun layout(score: Score, trackIndex: Int, options: LayoutOptions): ScoreLayout {
        val track = score.tracks.getOrNull(trackIndex) ?: return ScoreLayout()
        val measureCount = minOf(track.measures.size, score.headers.size)
        if (measureCount == 0 || (!options.showNotation && !options.showTab)) return ScoreLayout()
        val keys = keysOf(score, measureCount)
        val plans = (0 until measureCount).map { planMeasure(track, it) }
        val systems = breakIntoSystems(plans, keys, score, options)
        val painter = SystemPainter(LayoutContext(score, track, options, keys, plans))
        var top = 0f
        val built = systems.mapIndexed { number, system ->
            painter.system(system, top, number, isLast = number + 1 == systems.size).also { top += it.height + SYSTEM_GAP }
        }
        return ScoreLayout(systems = built, width = options.width, height = top)
    }

    /** The key of every measure: the score's, and then what each measure that changes it says. */
    private fun keysOf(score: Score, count: Int): List<Int> {
        var key = score.key.coerceIn(-7, 7)
        return (0 until count).map { index ->
            score.headers[index].keyChange?.let { key = it.coerceIn(-7, 7) }
            key
        }
    }
}

/** How many sharps or flats a key signature takes room for, which is also how many there are. */
internal fun keyWidth(key: Int): Float = if (key != 0) key.absoluteValue + 0.8f else 0f

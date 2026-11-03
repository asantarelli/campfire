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

import com.pandulapeter.campfire.guitarpro.draw.Anchor
import com.pandulapeter.campfire.guitarpro.draw.Draw
import com.pandulapeter.campfire.guitarpro.draw.Glyphs
import com.pandulapeter.campfire.guitarpro.draw.TextStyle
import com.pandulapeter.campfire.guitarpro.draw.Tone
import com.pandulapeter.campfire.guitarpro.draw.dot
import com.pandulapeter.campfire.guitarpro.draw.moved
import com.pandulapeter.campfire.guitarpro.draw.textOf
import com.pandulapeter.campfire.guitarpro.model.MeasureHeader
import kotlin.math.max

/** One system of a track: the staves with their lines, clef and bar lines, and the measures between the bar lines. */
internal class SystemPainter(private val context: LayoutContext) {

    private val options = context.options
    private val score = context.score
    private val measurePainter = MeasurePainter(context)

    fun system(plan: SystemPlan, top: Float, number: Int, isLast: Boolean): ScoreSystem {
        val strings = max(context.track.strings.size, 1)
        val first = plan.measures.first
        val key = context.keys[first]
        val prefix = prefixWidth(options, key)
        val notationTop = top + ABOVE
        val tabTop = if (options.showNotation) notationTop + 4f + BETWEEN else top + ABOVE - 1f
        val tabBottom = tabTop + (strings - 1f) * TAB_GAP
        val bottom = when {
            options.showTab -> tabBottom + if (options.showRhythm) RHYTHM else 1.5f
            else -> notationTop + 4f + 3.5f
        }
        val staves = Staves(notationTop, tabTop, tabBottom, strings, number)
        val right = options.width
        val draws = mutableListOf<Draw>()

        // The measures are stretched alike so that the system is as wide as the window.
        val fixed = prefix + plan.measures.sumOf { measureExtra(score, it).toDouble() }.toFloat()
        val bodies = plan.measures.sumOf { context.plans[it].body.toDouble() }.toFloat()
        val rawStretch = if (bodies <= 0f) 1f else (right - fixed) / bodies
        val stretch = if (isLast) rawStretch.coerceIn(1f, 1.4f) else max(rawStretch, 1f)

        var x = prefix
        for (index in plan.measures) {
            val header = score.headers[index]
            val measureLeft = x
            var cursor = x
            if (header.isRepeatOpen) cursor += 1.4f
            if (index == 0 || changesTime(score, index)) {
                timeSignature(draws, cursor + 0.3f, header, staves)
                cursor += TIME_SIGNATURE
            }
            val bodyLeft = cursor + MEASURE_PADDING * 0.6f
            val bodyWidth = context.plans[index].body * stretch
            val bar = bodyLeft + bodyWidth + MEASURE_PADDING * 0.4f + if (header.repeatClose > -1) 1.4f else 0f
            barLineStart(draws, measureLeft, header, staves)
            measurePainter.paint(draws, index, bodyLeft, stretch, staves)
            val isFinal = index + 1 == minOf(context.track.measures.size, score.headers.size)
            barLineEnd(draws, bar, header, isFinal, staves)
            if (index == first) {
                draws += textOf(measureLeft + 0.2f, staves.firstStaff(options) - 1.1f, (index + 1).toString(), 0.8f, Anchor.START, TextStyle.NORMAL, Tone.MUTED)
            }
            header.marker?.let {
                draws += textOf(bodyLeft - 0.3f, top + 0.7f, it, 1.15f, Anchor.START, TextStyle.BOLD, Tone.ACCENT)
            }
            x = bar
        }
        // The staff lines run to the last bar, and the system's start is marked.
        val lineRight = if (isLast) x else max(right, x)
        val under = mutableListOf<Draw>()
        if (options.showNotation) {
            for (line in 0 until 5) {
                val y = notationTop + line
                under += Draw.Line(0f, y, lineRight, y, Glyphs.LINE)
            }
            draws += Glyphs.trebleClef(0.6f).map { it.moved(0f, notationTop) }
            if (key != 0) {
                draws += Glyphs.keySignature(0.6f + CLEF, key, 1f).map { it.moved(0f, notationTop) }
            }
            // A guitar sounds an octave below where it is written, which the clef says with an 8 under it.
            draws += textOf(1.5f, notationTop + 6.4f, "8", 0.9f, Anchor.CENTER, TextStyle.NORMAL, Tone.INK)
        }
        if (options.showTab) {
            for (string in 1..strings) {
                val y = staves.stringY(string)
                under += Draw.Line(0f, y, lineRight, y, Glyphs.LINE)
            }
            val spread = (strings - 1f) * TAB_GAP
            for ((position, letter) in listOf("T", "A", "B").withIndex()) {
                draws += textOf(1.5f, tabTop + spread * (0.22f + position * 0.28f), letter, 1.6f, Anchor.CENTER, TextStyle.BOLD, Tone.INK)
            }
        }
        val joinBottom = if (options.showTab) tabBottom else notationTop + 4f
        under += Draw.Line(0f, staves.firstStaff(options), 0f, joinBottom, Glyphs.LINE * 1.5f)
        return ScoreSystem(top = top, height = bottom - top, draws = under + draws)
    }

    private fun timeSignature(draws: MutableList<Draw>, x: Float, header: MeasureHeader, staves: Staves) {
        val center = x + 0.9f
        if (options.showNotation) {
            draws += textOf(center, staves.notationTop + 1f, header.numerator.toString(), 2.1f, Anchor.CENTER, TextStyle.BOLD, Tone.INK)
            draws += textOf(center, staves.notationTop + 3f, header.denominator.toString(), 2.1f, Anchor.CENTER, TextStyle.BOLD, Tone.INK)
        }
        if (options.showTab) {
            val middle = (staves.tabTop + staves.tabBottom) / 2f
            val spread = (staves.strings - 1f) * TAB_GAP / 4f
            draws += textOf(center, middle - spread - 0.15f, header.numerator.toString(), 1.7f, Anchor.CENTER, TextStyle.BOLD, Tone.INK)
            draws += textOf(center, middle + spread + 0.15f, header.denominator.toString(), 1.7f, Anchor.CENTER, TextStyle.BOLD, Tone.INK)
        }
    }

    /** A vertical line across the staves. */
    private fun bar(draws: MutableList<Draw>, x: Float, width: Float, staves: Staves) {
        if (options.showNotation) {
            draws += Draw.Line(x, staves.notationTop, x, staves.notationTop + 4f, width)
        }
        if (options.showTab) {
            draws += Draw.Line(x, staves.tabTop, x, staves.tabBottom, width)
        }
    }

    private fun dots(draws: MutableList<Draw>, x: Float, staves: Staves) {
        if (options.showNotation) {
            draws += Glyphs.repeatDots(x).map { it.moved(0f, staves.notationTop) }
        }
        if (options.showTab) {
            val middle = (staves.tabTop + staves.tabBottom) / 2f
            draws += dot(x, middle - TAB_GAP * 0.5f, 0.2f)
            draws += dot(x, middle + TAB_GAP * 0.5f, 0.2f)
        }
    }

    private fun barLineStart(draws: MutableList<Draw>, x: Float, header: MeasureHeader, staves: Staves) {
        if (header.isRepeatOpen) {
            bar(draws, x + 0.05f, 0.4f, staves)
            bar(draws, x + 0.6f, Glyphs.LINE, staves)
            dots(draws, x + 1.1f, staves)
        } else if (x > 1f) {
            bar(draws, x, Glyphs.LINE, staves)
        }
    }

    private fun barLineEnd(draws: MutableList<Draw>, x: Float, header: MeasureHeader, isFinal: Boolean, staves: Staves) {
        when {
            header.repeatClose > -1 -> {
                dots(draws, x - 1.3f, staves)
                bar(draws, x - 0.6f, Glyphs.LINE, staves)
                bar(draws, x - 0.05f, 0.4f, staves)
                if (header.repeatClose > 1) {
                    val top = staves.firstStaff(options) - 0.9f
                    draws += textOf(x - 0.4f, top, "x${header.repeatClose}", 0.9f, Anchor.END, TextStyle.ITALIC, Tone.MUTED)
                }
            }
            isFinal -> {
                bar(draws, x - 0.5f, Glyphs.LINE, staves)
                bar(draws, x - 0.05f, 0.4f, staves)
            }
            header.hasDoubleBar -> {
                bar(draws, x - 0.4f, Glyphs.LINE, staves)
                bar(draws, x, Glyphs.LINE, staves)
            }
            else -> bar(draws, x, Glyphs.LINE, staves)
        }
    }
}

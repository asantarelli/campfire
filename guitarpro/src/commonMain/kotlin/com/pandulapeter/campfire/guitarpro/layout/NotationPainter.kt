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
import com.pandulapeter.campfire.guitarpro.model.BeatStatus
import kotlin.math.abs
import kotlin.math.max
import kotlin.math.min

/** The notes of a measure on the staff: heads, accidentals, ledger lines, rests, stems, flags, beams and tuplets. */
internal class NotationPainter(private val options: LayoutOptions) {

    fun paint(draws: MutableList<Draw>, columns: List<Column>, groups: List<List<Int>>, stemsUp: BooleanArray, staves: Staves) {
        val top = staves.notationTop
        val ends = arrayOfNulls<StemEnd>(columns.size)
        for ((position, column) in columns.withIndex()) {
            val beat = column.beat
            val value = beat.duration.value
            if (beat.status == BeatStatus.REST || column.heads.isEmpty()) {
                draws += Glyphs.rest(column.x, value).map { it.moved(0f, top) }
                if (beat.duration.isDotted) {
                    draws += dot(column.x + 0.9f, top + 1.4f, 0.17f)
                }
                continue
            }
            val isUp = stemsUp[position]
            val shifted = shiftedHeads(column)
            val side = if (isUp) 1f else -1f
            var accidentalSlot = 0f
            for ((headIndex, head) in column.heads.withIndex()) {
                val headX = column.x + if (shifted[headIndex]) side * (Glyphs.HEAD_RX * 2f - 0.1f) else 0f
                val y = top + head.y
                ledgerLines(draws, headX, head.y, top)
                draws += Glyphs.notehead(headX, y, value)
                head.accidental?.let { alteration ->
                    val signX = column.x - 1.5f - accidentalSlot
                    draws += when (alteration) {
                        1 -> Glyphs.sharp(signX, y)
                        -1 -> Glyphs.flat(signX, y)
                        else -> Glyphs.natural(signX, y)
                    }
                    accidentalSlot += 0.9f
                }
                if (beat.duration.isDotted) {
                    val dotY = if (abs(head.y - head.y.toInt()) < 0.01f) y - 0.5f else y
                    draws += dot(headX + 1f, dotY, 0.17f)
                }
            }
            if (value >= 2) {
                val near = column.heads.minOf { it.y }
                val far = column.heads.maxOf { it.y }
                val stemX = column.x + if (isUp) Glyphs.HEAD_RX - 0.04f else -(Glyphs.HEAD_RX - 0.04f)
                ends[position] = if (isUp) StemEnd(stemX, far, min(near - 3.3f, 2f), true) else StemEnd(stemX, near, max(far + 3.3f, 2f), false)
            }
        }
        stemsAndBeams(draws, columns, groups, ends, top)
        tuplets(draws, columns, ends, top)
    }

    /** Heads that are a step apart do not fit side by side on one stem: the second goes to the other side of it. */
    private fun shiftedHeads(column: Column): BooleanArray {
        val ordered = column.heads.indices.sortedByDescending { column.heads[it].step }
        val shifted = BooleanArray(column.heads.size)
        for (pair in 1 until ordered.size) {
            val (a, b) = ordered[pair - 1] to ordered[pair]
            if (abs(column.heads[a].step - column.heads[b].step) == 1 && !shifted[a]) {
                shifted[b] = true
            }
        }
        return shifted
    }

    /** The ledger lines a note at [headY] (in spaces from the staff's top line) needs. */
    private fun ledgerLines(draws: MutableList<Draw>, headX: Float, headY: Float, top: Float) {
        if (headY < -0.5f) {
            var line = -1f
            while (line >= headY - 0.01f) {
                draws += Draw.Line(headX - 1f, top + line, headX + 1f, top + line, Glyphs.LINE * 1.4f)
                line -= 1f
            }
        } else if (headY > 4.5f) {
            var line = 5f
            while (line <= headY + 0.01f) {
                draws += Draw.Line(headX - 1f, top + line, headX + 1f, top + line, Glyphs.LINE * 1.4f)
                line += 1f
            }
        }
    }

    /** The stems, the flags of the notes alone and the beams of the groups, from the ends worked out for the stems. */
    private fun stemsAndBeams(draws: MutableList<Draw>, columns: List<Column>, groups: List<List<Int>>, ends: Array<StemEnd?>, top: Float) {
        // A group's stems all reach its beam.
        val tips = ends.map { it?.tip }.toMutableList()
        val grouped = BooleanArray(columns.size)
        for (group in groups) {
            val present = group.filter { ends[it] != null }
            if (present.size < 2) continue
            val isUp = ends[present.first()]!!.isUp
            val tips2 = present.map { ends[it]!!.tip }
            val tip = if (isUp) tips2.min() else tips2.max()
            present.forEach { tips[it] = tip }
            group.forEach { grouped[it] = true }
        }
        for ((position, end) in ends.withIndex()) {
            val tip = tips[position] ?: continue
            end ?: continue
            draws += Draw.Line(end.x, top + end.from, end.x, top + tip, Glyphs.STEM)
            val value = columns[position].beat.duration.value
            if (!grouped[position] && value >= 8) {
                draws += Glyphs.flags(end.x, top + tip, end.isUp, Glyphs.flagCountOf(value))
            }
        }
        for (group in groups) {
            val present = group.filter { ends[it] != null }
            if (present.size < 2) continue
            val tip = (tips[present.first()] ?: 0f) + top
            drawBeams(draws, present.map { ends[it]!!.x }, present.map { columns[it].beat.duration.value }, tip, ends[present.first()]!!.isUp)
        }
    }

    private fun tuplets(draws: MutableList<Draw>, columns: List<Column>, ends: Array<StemEnd?>, top: Float) {
        var position = 0
        while (position < columns.size) {
            val duration = columns[position].beat.duration
            if (duration.enters <= 1) {
                position++
                continue
            }
            var end = position
            while (end + 1 < columns.size && columns[end + 1].beat.duration.enters == duration.enters &&
                columns[end + 1].beat.duration.times == duration.times && end + 1 - position < duration.enters
            ) {
                end++
            }
            val x = (columns[position].x + columns[end].x) / 2f
            val isUp = ends[position]?.isUp ?: true
            val tips = (position..end).mapNotNull { ends[it]?.tip }
            val y = when {
                tips.isEmpty() -> top - 0.8f
                isUp -> top + tips.min() - 0.9f
                else -> top + tips.max() + 0.9f
            }
            draws += textOf(x, y, duration.enters.toString(), 0.9f, Anchor.CENTER, TextStyle.ITALIC, Tone.INK)
            position = end + 1
        }
    }
}

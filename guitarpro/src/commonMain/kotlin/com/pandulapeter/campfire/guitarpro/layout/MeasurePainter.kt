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
import com.pandulapeter.campfire.guitarpro.draw.textOf
import com.pandulapeter.campfire.guitarpro.model.BeatStatus
import com.pandulapeter.campfire.guitarpro.model.Note

/** One measure of a track: it works out where each beat's notes are and hands the columns to the painters of the staves. */
internal class MeasurePainter(private val context: LayoutContext) {

    private val options = context.options
    private val notationPainter = NotationPainter(options)

    /** Kept for as long as the track is painted, since a tie, a slur and a slide join notes that are in different measures. */
    private val tabPainter = TabPainter(options)

    fun paint(draws: MutableList<Draw>, index: Int, bodyLeft: Float, stretch: Float, staves: Staves) {
        val plan = context.plans[index]
        val header = context.score.headers[index]
        val key = context.keys[index]
        val accidentals = mutableMapOf<Int, Int>()
        val columns = plan.beats.map { beatPlan ->
            val beat = beatPlan.beat
            val grace = if (beat.notes.any { it.effects.grace != null }) 1.2f else 0f
            val heads = if (options.showNotation && beat.status != BeatStatus.REST) beat.notes.map { head(it, key, accidentals) } else emptyList()
            Column(
                beat = beat,
                x = bodyLeft + beatPlan.x * stretch + 0.9f + grace,
                offset = (beat.start - header.start).coerceAtLeast(0),
                heads = heads,
            )
        }
        val groups = beamGroups(columns, header)
        if (options.showNotation) {
            // Which way each column's stem goes.
            val stemsUp = BooleanArray(columns.size) { true }
            for (group in groups) {
                val isUp = stemIsUp(group.flatMap { column -> columns[column].heads.map { it.step } })
                group.forEach { stemsUp[it] = isUp }
            }
            notationPainter.paint(draws, columns, groups, stemsUp, staves)
        }
        if (options.showTab) {
            tabPainter.paint(draws, columns, staves)
            if (options.showRhythm) {
                tabPainter.paintRhythm(draws, columns, groups, staves)
            }
        }
        paintLabels(draws, columns, index, staves)
    }

    /** Where a note is on the staff, and whether it needs an accidental there. */
    private fun head(note: Note, key: Int, accidentals: MutableMap<Int, Int>): Head {
        val tuning = context.track.strings.getOrElse(note.string - 1) { DEFAULT_TUNING }
        // A guitar is written an octave above where it sounds.
        val (step, alteration) = pitchPosition(tuning + note.fret + 12, key)
        val current = accidentals[step] ?: keyAlteration(step.mod(7), key)
        val accidental = if (alteration != current) alteration.also { accidentals[step] = it } else null
        return Head(step = step, y = 4f - (step - 30) * 0.5f, accidental = accidental)
    }

    /** What is written over the staves: the chords, the texts and the tempo. */
    private fun paintLabels(draws: MutableList<Draw>, columns: List<Column>, index: Int, staves: Staves) {
        val first = staves.firstStaff(options)
        for ((position, column) in columns.withIndex()) {
            column.beat.chord?.let {
                draws += textOf(column.x - 0.9f, first - 3.9f, it, 1.15f, Anchor.START, TextStyle.BOLD, Tone.ACCENT)
            }
            column.beat.chordShape?.let {
                draws += Glyphs.chordDiagram(column.x, first - 11f, it.firstFret, it.frets)
            }
            column.beat.text?.let {
                draws += textOf(column.x - 0.9f, first - 2.5f, it, 0.95f, Anchor.START, TextStyle.ITALIC, Tone.INK)
            }
            val tempo = column.beat.tempo ?: if (index == 0 && position == 0) context.score.tempo else null
            if (tempo != null && tempo > 0) {
                val (x, y) = (column.x - 0.9f) to (first - 5.4f)
                draws += Draw.Ellipse(x + 0.5f, y + 0.55f, 0.4f, 0.3f, -0.4f, isFilled = true, width = 0f)
                draws += Draw.Line(x + 0.82f, y + 0.55f, x + 0.82f, y - 0.7f, 0.1f)
                draws += textOf(x + 1.5f, y, "= $tempo", 0.9f, Anchor.START, TextStyle.NORMAL, Tone.INK)
            }
        }
    }

    private companion object {
        /** The tuning of a string the track does not have, so that a note on it still lands somewhere. */
        const val DEFAULT_TUNING = 40
    }
}

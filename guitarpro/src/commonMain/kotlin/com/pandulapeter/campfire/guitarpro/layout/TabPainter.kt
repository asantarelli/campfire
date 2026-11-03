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
import com.pandulapeter.campfire.guitarpro.draw.Point
import com.pandulapeter.campfire.guitarpro.draw.TextStyle
import com.pandulapeter.campfire.guitarpro.draw.Tone
import com.pandulapeter.campfire.guitarpro.draw.arc
import com.pandulapeter.campfire.guitarpro.draw.bezier
import com.pandulapeter.campfire.guitarpro.draw.dot
import com.pandulapeter.campfire.guitarpro.draw.polygonOf
import com.pandulapeter.campfire.guitarpro.draw.textOf
import com.pandulapeter.campfire.guitarpro.model.BeatStatus
import com.pandulapeter.campfire.guitarpro.model.Note
import com.pandulapeter.campfire.guitarpro.model.NoteKind
import com.pandulapeter.campfire.guitarpro.model.Slide
import kotlin.math.sin

/** What a note leaves behind that a later one may join: a tie goes back to the note before, a slur on to the note after. */
private class Place(val x: Float, val tabY: Float, val system: Int, val fret: Int)

private enum class Join { HAMMER, SLIDE }

/** The notes of a measure on the strings, with what joins them and the marks of how they are played, and the rhythm under them. */
internal class TabPainter(private val options: LayoutOptions) {

    /** Where the last note of each string was, for a tie to go back to it. */
    private val ties = mutableMapOf<Int, Place>()

    /** The notes that join the next one on their string, and the number of the beat they were written on: only the beat right after is joined. */
    private val pending = mutableMapOf<Int, Triple<Place, Join, Int>>()

    /** How many beats of the track have been drawn. */
    private var serial = 0

    fun paint(draws: MutableList<Draw>, columns: List<Column>, staves: Staves) {
        for (column in columns) {
            serial++
            pending.entries.removeAll { it.value.third + 1 < serial }
            for (note in column.beat.notes) {
                if (note.string > staves.strings) continue
                val y = staves.stringY(note.string)
                val effects = note.effects
                var text = if (note.kind == NoteKind.DEAD) "x" else note.fret.toString()
                if (note.kind == NoteKind.TIE || effects.ghost) text = "($text)"
                if (effects.harmonic) text = "<$text>"
                draws += Draw.Text(column.x, y, text, 1f, Anchor.CENTER, TextStyle.NORMAL, masks = true)
                effects.grace?.let {
                    draws += Draw.Text(column.x - 1.5f, y, it.toString(), 0.7f, Anchor.CENTER, TextStyle.NORMAL, masks = true)
                }
                join(draws, note, Place(column.x, y, staves.system, note.fret))
                marks(draws, note, column.x, y, staves)
            }
        }
    }

    /** Draws what the note joins to the one before it, and keeps what it joins to the one after. */
    private fun join(draws: MutableList<Draw>, note: Note, place: Place) {
        val string = note.string
        if (note.kind == NoteKind.TIE) {
            val before = ties[string]
            draws += if (before != null && before.system == place.system) {
                arc(before.x + 0.5f, place.tabY - 0.55f, place.x - 0.5f, place.tabY - 0.55f, -0.55f, 0.12f)
            } else {
                arc(3.4f, place.tabY - 0.55f, place.x - 0.5f, place.tabY - 0.55f, -0.45f, 0.12f)
            }
        }
        pending.remove(string)?.let { (before, join, _) ->
            if (before.system == place.system) {
                if (join == Join.HAMMER) {
                    draws += arc(before.x + 0.2f, before.tabY - 0.65f, place.x - 0.2f, place.tabY - 0.65f, -0.7f, 0.12f)
                } else {
                    val rise = if (place.fret >= before.fret) -0.35f else 0.35f
                    draws += Draw.Line(before.x + 0.55f, before.tabY - rise, place.x - 0.55f, place.tabY + rise, 0.12f)
                }
            }
        }
        if (note.effects.hammer) {
            pending[string] = Triple(place, Join.HAMMER, serial)
        } else if (note.effects.slide == Slide.SHIFT || note.effects.slide == Slide.LEGATO) {
            pending[string] = Triple(place, Join.SLIDE, serial)
        }
        ties[string] = place
    }

    /** The marks over and beside a fret: a bend's arrow, a vibrato, how the note is held. */
    private fun marks(draws: MutableList<Draw>, note: Note, x: Float, y: Float, staves: Staves) {
        val effects = note.effects
        effects.bend?.let { bend ->
            val label = when (bend.points.maxOfOrNull { it.height } ?: 0) {
                in Int.MIN_VALUE..0 -> ""
                1, 2 -> "½"
                3, 4 -> "full"
                5, 6 -> "1½"
                else -> "2"
            }
            if (label.isNotEmpty()) {
                val top = y - 1.9f
                draws += Draw.Polyline(bezier(Point(x + 0.45f, y - 0.3f), Point(x + 1.2f, y - 0.4f), Point(x + 1.4f, y - 1f), Point(x + 1.4f, top + 0.4f)), 0.12f)
                draws += polygonOf((x + 1.4f) to top, (x + 1.15f) to (top + 0.5f), (x + 1.65f) to (top + 0.5f))
                draws += textOf(x + 1.4f, top - 0.45f, label, 0.75f, Anchor.CENTER, TextStyle.NORMAL, Tone.INK)
            }
        }
        if (effects.vibrato) {
            val base = staves.tabTop - 1f
            draws += Draw.Polyline((0..16).map { Point(x - 0.8f + it * 0.16f, base + sin(it * 1.4f) * 0.14f) }, 0.1f)
        }
        when (effects.slide) {
            Slide.OUT_DOWN -> draws += Draw.Line(x + 0.5f, y - 0.15f, x + 1.4f, y + 0.45f, 0.12f)
            Slide.OUT_UP -> draws += Draw.Line(x + 0.5f, y + 0.15f, x + 1.4f, y - 0.45f, 0.12f)
            Slide.IN_FROM_BELOW -> draws += Draw.Line(x - 1.4f, y + 0.45f, x - 0.5f, y - 0.15f, 0.12f)
            Slide.IN_FROM_ABOVE -> draws += Draw.Line(x - 1.4f, y - 0.45f, x - 0.5f, y + 0.15f, 0.12f)
            else -> Unit
        }
        val labelY = staves.tabTop - 1.9f
        if (effects.palmMute) {
            draws += textOf(x - 0.6f, labelY, "P.M.", 0.7f, Anchor.START, TextStyle.ITALIC, Tone.MUTED)
        } else if (effects.letRing) {
            draws += textOf(x - 0.6f, labelY, "let ring", 0.7f, Anchor.START, TextStyle.ITALIC, Tone.MUTED)
        }
        if (effects.accent || effects.heavyAccent) {
            draws += textOf(x, staves.tabTop - 0.9f, if (effects.heavyAccent) "^" else ">", 0.9f, Anchor.CENTER, TextStyle.BOLD, Tone.INK)
        }
        if (effects.trill) {
            draws += textOf(x, staves.tabTop - 0.9f, "tr", 0.8f, Anchor.CENTER, TextStyle.ITALIC, Tone.INK)
        }
    }

    /** The stems and the beams that say how long the notes are, under the tablature. */
    fun paintRhythm(draws: MutableList<Draw>, columns: List<Column>, groups: List<List<Int>>, staves: Staves) {
        val base = staves.tabBottom + 0.9f
        val tip = base + 2.4f
        val isSounding = { column: Int -> columns[column].beat.status != BeatStatus.REST }
        val grouped = BooleanArray(columns.size)
        for (group in groups) {
            if (group.count(isSounding) >= 2) group.forEach { grouped[it] = true }
        }
        for ((position, column) in columns.withIndex()) {
            val beat = column.beat
            if (beat.status == BeatStatus.REST) continue
            val value = beat.duration.value
            if (value >= 2) {
                draws += Draw.Line(column.x, base, column.x, tip, Glyphs.STEM)
            }
            if (value >= 8 && !grouped[position]) {
                // Flags hang from the stem's end down the page: a stem pointing up has them below it.
                draws += Glyphs.flags(column.x, tip, false, Glyphs.flagCountOf(value)).map { mirroredDown(it, tip) }
            }
            if (beat.duration.isDotted) {
                draws += dot(column.x + 0.6f, tip + 0.3f, 0.17f)
            }
            if (value == 1) {
                draws += Draw.Line(column.x - 0.35f, base, column.x + 0.35f, base, 0.15f)
            }
        }
        for (group in groups) {
            val present = group.filter(isSounding)
            if (present.size < 2) continue
            // Stems point down from the tablature here, so the beams hang from their ends upwards, drawn as stems that point down.
            drawBeams(draws, present.map { columns[it].x }, present.map { columns[it].beat.duration.value }, tip, isUp = false)
        }
    }
}

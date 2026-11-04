/*
 * This file is part of Campfire.
 * Copyright (c) Pandula Péter 2017-2026.
 * https://github.com/pandulapeter/campfire
 *
 * This Source Code Form is subject to the terms of the Mozilla Public License, v. 2.0.
 * If a copy of the MPL was not distributed with this file, You can obtain one at
 * https://mozilla.org/MPL/2.0/.
 */
package com.pandulapeter.campfire.guitarpro.reader

import com.pandulapeter.campfire.guitarpro.model.Beat
import com.pandulapeter.campfire.guitarpro.model.BeatStatus
import com.pandulapeter.campfire.guitarpro.model.ChordShape
import com.pandulapeter.campfire.guitarpro.model.Duration
import com.pandulapeter.campfire.guitarpro.model.Note
import com.pandulapeter.campfire.guitarpro.model.NoteEffects

/** One beat of a voice: its duration, what is written over it, and the notes on its strings. */
internal class BeatReader(
    private val cursor: ByteCursor,
    private val version: GuitarProVersion,
) {
    private val major get() = version.major
    private val noteReader = NoteReader(cursor, version)

    /** @param lastFrets the fret last played on each string, which a tied note repeats. */
    fun beat(start: Int, strings: Int, lastFrets: IntArray): Beat {
        val flags = cursor.u8()
        val status = if (flags and 0x40 != 0) {
            when (cursor.u8()) {
                0 -> BeatStatus.EMPTY
                2 -> BeatStatus.REST
                else -> BeatStatus.NORMAL
            }
        } else {
            BeatStatus.NORMAL
        }
        val exponent = (cursor.i8() + 2).coerceIn(0, 6)
        var duration = Duration(value = 1 shl exponent, isDotted = flags and 0x01 != 0)
        if (flags and 0x20 != 0) {
            val enters = cursor.i32()
            duration = duration.copy(enters = tupletEnters(enters), times = tupletTimes(enters))
        }
        val (chord, chordShape) = if (flags and 0x02 != 0) chord(strings) else null to null
        val text = if (flags and 0x04 != 0) cursor.intByteSizeString() else null
        var wholeBeat = NoteEffects()
        if (flags and 0x08 != 0) {
            wholeBeat = beatEffects()
        }
        val tempo = if (flags and 0x10 != 0) mixTableChange() else null
        val notes = notes(strings, wholeBeat, lastFrets)
        if (major == 5) {
            val flags2 = cursor.i16()
            if (flags2 and 0x0800 != 0) {
                cursor.skip(1)
            }
        }
        return Beat(start = start, duration = duration, status = status, notes = notes, text = text, chord = chord, chordShape = chordShape, tempo = tempo)
    }

    private fun tupletEnters(enters: Int) = if (enters in TUPLETS) enters else 1

    private fun tupletTimes(enters: Int) = when (enters) {
        3 -> 2
        5, 6, 7 -> 4
        in 9..13 -> 8
        else -> 1
    }

    /** The chord diagram of a beat: its name, which may be empty, and its shape. */
    private fun chord(strings: Int): Pair<String?, ChordShape?> {
        val isNew = cursor.boolean()
        val name: String
        var shape: ChordShape? = null
        if (!isNew) {
            name = cursor.intByteSizeString()
            val firstFret = cursor.i32()
            if (firstFret != 0) {
                shape = chordFrets(firstFret, stored = 6, strings = strings)
            }
        } else if (major == 3) {
            cursor.skip(1 + 3 + 4 * 5 + 1)
            name = cursor.byteSizeString(22)
            cursor.skip(3 * 4)
            val firstFret = cursor.i32()
            shape = chordFrets(firstFret, stored = 6, strings = strings)
            cursor.skip(4 + 2 * 4 * 3 + 7 + 1)
        } else {
            cursor.skip(1 + 3 + 1 + 1 + 1 + 4 + 4 + 1)
            name = cursor.byteSizeString(22)
            cursor.skip(3)
            val firstFret = cursor.i32()
            shape = chordFrets(firstFret, stored = 7, strings = strings)
            cursor.skip(1 + 5 * 3 + 7 + 1 + 7 + 1)
        }
        // A diagram with nothing played is not worth drawing.
        return name.ifEmpty { null } to shape?.takeIf { diagram -> diagram.frets.any { it >= 0 } }
    }

    /** The frets of a chord diagram, [stored] of them written, of which the track's [strings] are kept. */
    private fun chordFrets(firstFret: Int, stored: Int, strings: Int): ChordShape {
        val frets = mutableListOf<Int>()
        for (index in 0 until stored) {
            val fret = cursor.i32()
            if (index < strings.coerceAtMost(MAX_STRINGS)) frets += fret
        }
        return ChordShape(firstFret, frets)
    }

    /** The effects that belong to the beat as a whole, which each of its notes has too. */
    private fun beatEffects(): NoteEffects {
        if (major == 3) {
            val flags = cursor.u8()
            if (flags and 0x20 != 0) {
                cursor.u8() // Whether it is a slap or a tremolo bar.
                cursor.skip(4)
            }
            if (flags and 0x40 != 0) {
                cursor.skip(2) // The speeds of the stroke.
            }
            return NoteEffects(vibrato = flags and 0x01 != 0, harmonic = flags and 0x0C != 0)
        }
        val flags1 = cursor.u8()
        val flags2 = cursor.u8()
        if (flags1 and 0x20 != 0) {
            cursor.skip(1) // The slap.
        }
        if (flags2 and 0x04 != 0) {
            noteReader.bend()
        }
        if (flags1 and 0x40 != 0) {
            cursor.skip(2) // The speeds of the stroke.
        }
        if (flags2 and 0x02 != 0) {
            cursor.skip(1) // The pick stroke.
        }
        return NoteEffects()
    }

    /** A change of the mixer on a beat, of which only the tempo is kept. */
    private fun mixTableChange(): Int? {
        val isVersion5 = major == 5
        val changes = IntArray(6)
        val tempo: Int
        if (isVersion5) {
            cursor.skip(1 + 4 + 4 + 4) // The instrument, and the one of the sound engine.
            cursor.skip(if (version.isAfter5Dot0) 4 else 3)
            if (!version.isAfter5Dot0) {
                cursor.skip(1)
            }
            for (index in changes.indices) changes[index] = cursor.i8()
            cursor.intByteSizeString() // The name of the tempo.
            tempo = cursor.i32()
        } else {
            cursor.skip(1) // The instrument.
            for (index in changes.indices) changes[index] = cursor.i8()
            tempo = cursor.i32()
        }
        // Each change that is written is followed by how long it takes.
        changes.filter { it >= 0 }.forEach { _ -> cursor.skip(1) }
        if (tempo >= 0) {
            cursor.skip(1)
            if (version.isAfter5Dot0) {
                cursor.skip(1) // Whether the tempo is shown.
            }
        }
        if (major >= 4) {
            cursor.skip(1) // Which of the changes are for every track.
        }
        if (isVersion5) {
            cursor.skip(1) // The wah.
            if (version.isAfter5Dot0) {
                cursor.intByteSizeString()
                cursor.intByteSizeString()
            }
        }
        return tempo.takeIf { it >= 0 }
    }

    private fun notes(strings: Int, wholeBeat: NoteEffects, lastFrets: IntArray): List<Note> {
        val flags = cursor.u8()
        val notes = mutableListOf<Note>()
        for (string in 1..strings.coerceAtMost(MAX_STRINGS)) {
            if (flags and (1 shl (7 - string)) != 0) {
                notes += noteReader.note(string, wholeBeat, lastFrets)
            }
        }
        return notes
    }

    private companion object {
        const val MAX_STRINGS = 7
        val TUPLETS = setOf(3, 5, 6, 7, 9, 10, 11, 12, 13)
    }
}

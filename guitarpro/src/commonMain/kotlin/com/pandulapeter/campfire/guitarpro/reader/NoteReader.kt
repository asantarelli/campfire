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

import com.pandulapeter.campfire.guitarpro.model.Bend
import com.pandulapeter.campfire.guitarpro.model.BendPoint
import com.pandulapeter.campfire.guitarpro.model.Note
import com.pandulapeter.campfire.guitarpro.model.NoteEffects
import com.pandulapeter.campfire.guitarpro.model.NoteKind
import com.pandulapeter.campfire.guitarpro.model.Slide
import kotlin.math.roundToInt

/** One note on a string, and the effects it is played with. */
internal class NoteReader(
    private val cursor: ByteCursor,
    private val version: GuitarProVersion,
) {
    private val major get() = version.major

    fun note(string: Int, wholeBeat: NoteEffects, lastFrets: IntArray): Note {
        val isVersion5 = major == 5
        val flags = cursor.u8()
        var effects = wholeBeat.copy(heavyAccent = flags and 0x02 != 0, ghost = flags and 0x04 != 0, accent = flags and 0x40 != 0)
        var kind = NoteKind.NORMAL
        if (flags and 0x20 != 0) {
            kind = when (cursor.u8()) {
                2 -> NoteKind.TIE
                3 -> NoteKind.DEAD
                else -> NoteKind.NORMAL
            }
        }
        if (!isVersion5 && flags and 0x01 != 0) {
            cursor.skip(2) // A duration of the note's own.
        }
        if (flags and 0x10 != 0) {
            cursor.skip(1) // How hard it is played.
        }
        var fret = 0
        if (flags and 0x20 != 0) {
            val written = cursor.i8()
            val value = if (kind == NoteKind.TIE) lastFrets.getOrElse(string) { -1 } else written
            fret = if (value in 0 until MAX_FRET) value else 0
            if (string in lastFrets.indices) {
                lastFrets[string] = fret
            }
        }
        if (flags and 0x80 != 0) {
            cursor.skip(2) // The fingers.
        }
        if (isVersion5) {
            if (flags and 0x01 != 0) {
                cursor.skip(8) // How much of its duration it sounds.
            }
            cursor.skip(1)
        }
        if (flags and 0x08 != 0) {
            effects = effects(effects)
        }
        return Note(string = string, fret = fret, kind = kind, effects = effects)
    }

    private fun effects(start: NoteEffects): NoteEffects {
        var effects = start
        val flags1 = cursor.u8()
        val flags2 = if (major >= 4) cursor.u8() else 0
        effects = effects.copy(hammer = flags1 and 0x02 != 0, letRing = flags1 and 0x08 != 0)
        if (major >= 4) {
            effects = effects.copy(palmMute = flags2 and 0x02 != 0, vibrato = effects.vibrato || flags2 and 0x40 != 0)
        }
        if (flags1 and 0x01 != 0) {
            effects = effects.copy(bend = bend())
        }
        if (flags1 and 0x10 != 0) {
            effects = effects.copy(grace = grace())
        }
        if (major >= 4) {
            if (flags2 and 0x04 != 0) {
                cursor.skip(1) // The speed of the tremolo picking.
            }
            if (flags2 and 0x08 != 0) {
                effects = effects.copy(slide = slide())
            }
            if (flags2 and 0x10 != 0) {
                harmonic()
                effects = effects.copy(harmonic = true)
            }
            if (flags2 and 0x20 != 0) {
                cursor.skip(2) // The fret and the period of the trill.
                effects = effects.copy(trill = true)
            }
        } else if (flags1 and 0x04 != 0) {
            effects = effects.copy(slide = Slide.SHIFT)
        }
        return effects
    }

    fun bend(): Bend? {
        cursor.skip(1 + 4) // Its kind and its height.
        val points = List(cursor.count()) {
            val position = cursor.i32()
            val value = cursor.i32()
            cursor.skip(1) // Whether it vibrates.
            // A position is written in sixtieths of the note's length and a height in a hundredth of a semitone, a quarter of which is 25.
            BendPoint(position = (position * 12f / 60f).roundToInt(), height = (value / 25f).roundToInt())
        }
        return if (points.isEmpty()) null else Bend(points)
    }

    private fun grace(): Int {
        if (major == 5) {
            val fret = cursor.u8()
            cursor.skip(4)
            return fret
        }
        val fret = cursor.i8()
        cursor.skip(3)
        return fret
    }

    private fun slide(): Slide? {
        val kind = cursor.i8()
        if (major == 5) {
            // A set of bits; the first the note has is the one drawn.
            return BIT_SLIDES.firstOrNull { (bit, _) -> kind and bit != 0 }?.second
        }
        return when (kind) {
            1 -> Slide.SHIFT
            2 -> Slide.LEGATO
            3 -> Slide.OUT_DOWN
            4 -> Slide.OUT_UP
            -1 -> Slide.IN_FROM_BELOW
            -2 -> Slide.IN_FROM_ABOVE
            else -> null
        }
    }

    private fun harmonic() {
        val kind = cursor.i8()
        if (major == 5) {
            when (kind) {
                2 -> cursor.skip(3)
                3 -> cursor.skip(1)
            }
        }
    }

    private companion object {
        const val MAX_FRET = 100
        val BIT_SLIDES = listOf(
            0x01 to Slide.SHIFT,
            0x02 to Slide.LEGATO,
            0x04 to Slide.OUT_DOWN,
            0x08 to Slide.OUT_UP,
            0x10 to Slide.IN_FROM_BELOW,
            0x20 to Slide.IN_FROM_ABOVE,
        )
    }
}

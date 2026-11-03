/*
 * This file is part of Campfire.
 * Copyright (c) Pandula Péter 2017-2026.
 * https://github.com/pandulapeter/campfire
 *
 * This Source Code Form is subject to the terms of the Mozilla Public License, v. 2.0.
 * If a copy of the MPL was not distributed with this file, You can obtain one at
 * https://mozilla.org/MPL/2.0/.
 */
package com.pandulapeter.campfire.presentation.ui.chords

import com.pandulapeter.campfire.chordpro.model.ChordTuning
import com.pandulapeter.campfire.chordpro.model.ChordVoicing
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertNull

class ChordDiagramStyleTest {

    private fun notes(root: Int, vararg pitchClasses: Int) = pitchClasses.map { noteLabelOf(it, root) }

    @Test
    fun `the notes of a chord are spelled by their place above its root`() {
        assertEquals(listOf("C", "E", "G"), notes(0, 0, 4, 7))
        assertEquals(listOf("G", "B♭", "D"), notes(7, 7, 10, 2))
        assertEquals(listOf("D", "F♯", "A"), notes(2, 2, 6, 9))
        assertEquals(listOf("B♭", "D", "F"), notes(10, 10, 2, 5))
        assertEquals(listOf("F♯", "A♯", "C♯"), notes(6, 6, 10, 1))
        assertEquals(listOf("A♭", "C", "E♭"), notes(8, 8, 0, 3))
    }

    @Test
    fun `a shape with no root spells its notes with sharps`() {
        assertEquals(listOf("C♯", "A♯"), listOf(1, 10).map { noteLabelOf(it, -1) })
        assertNull(degreeLabelOf(4, -1))
    }

    @Test
    fun `the places in a chord are named as a player counts them`() {
        assertEquals(listOf("R", "♭3", "5", "♭7", "3", "7", "6"), listOf(0, 3, 7, 10, 4, 11, 9).map { degreeLabelOf(it, 0) })
        assertEquals("R", degreeLabelOf(9, 9))
        assertEquals("5", degreeLabelOf(4, 9))
    }

    @Test
    fun `the thirds, the fifths and the sevenths have a color each and the root and the other notes none`() {
        assertEquals(listOf(null, ChordFunction.THIRD, ChordFunction.THIRD, ChordFunction.FIFTH, ChordFunction.SEVENTH, null), listOf(0, 3, 4, 7, 10, 2).map(::chordFunctionOf))
    }

    @Test
    fun `a diagram knows the note each of its strings sounds`() {
        val geometry = chordDiagramGeometryOf(ChordVoicing.Fretted(listOf(null, 3, 2, 0, 1, 0)), ChordTuning.GUITAR, root = 0) as ChordDiagramGeometry.Fretted
        assertEquals(listOf(null, 0, 4, 7, 0, 4), geometry.pitchClasses)
        assertEquals(0, geometry.root)
    }
}

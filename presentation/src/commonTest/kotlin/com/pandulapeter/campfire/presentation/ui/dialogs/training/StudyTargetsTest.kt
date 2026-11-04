/*
 * This file is part of Campfire.
 * Copyright (c) Pandula Péter 2017-2026.
 * https://github.com/pandulapeter/campfire
 *
 * This Source Code Form is subject to the terms of the Mozilla Public License, v. 2.0.
 * If a copy of the MPL was not distributed with this file, You can obtain one at
 * https://mozilla.org/MPL/2.0/.
 */
package com.pandulapeter.campfire.presentation.ui.dialogs.training

import com.pandulapeter.campfire.chordpro.chords.ChordNotation
import com.pandulapeter.campfire.presentation.ui.chords.ChordDiagramGeometry
import kotlin.test.Test
import kotlin.test.assertEquals

class StudyTargetsTest {

    /** An F major shape: a barre across the first fret and two dots, the notes of each string below. */
    private val fMajor = ChordDiagramGeometry.Fretted(
        strings = 6,
        baseFret = 1,
        fretCount = 4,
        markers = List(6) { null },
        dots = listOf(
            ChordDiagramGeometry.Fretted.Dot(string = 2, row = 1, finger = 2, isRoot = false),
            ChordDiagramGeometry.Fretted.Dot(string = 3, row = 2, finger = 4, isRoot = true),
        ),
        barres = listOf(ChordDiagramGeometry.Fretted.Barre(row = 0, fromString = 0, toString = 5, finger = 1)),
        pitchClasses = listOf(5, 0, 9, 5, 0, 5),
        root = 5,
    )

    @Test
    fun `every dot is a question and a barre is one, at its first string`() {
        assertEquals(
            listOf(
                StudyTarget(string = 2, row = 1, finger = 2, pitchClass = 9),
                StudyTarget(string = 3, row = 2, finger = 4, pitchClass = 5),
                StudyTarget(string = 0, row = 0, finger = 1, pitchClass = 5),
            ),
            studyTargetsOf(fMajor),
        )
    }

    @Test
    fun `a dot whose string sounds no note is no question`() {
        assertEquals(emptyList(), studyTargetsOf(fMajor.copy(pitchClasses = emptyList())))
    }

    @Test
    fun `the notes of a shape are listed once each, in the order of its strings`() {
        assertEquals(listOf(5, 0, 9), notesOf(fMajor))
    }

    @Test
    fun `a black key is answered by both of its names`() {
        val choices = noteChoices(ChordNotation.STANDARD)
        assertEquals(12, choices.size)
        assertEquals("C", choices[0].label)
        assertEquals("C♯/D♭", choices[1].label)
        assertEquals("B", choices[11].label)
        assertEquals((0..11).toList(), choices.map { it.pitchClass })
    }
}

/*
 * This file is part of Campfire.
 * Copyright (c) Pandula Péter 2017-2026.
 * https://github.com/pandulapeter/campfire
 *
 * This Source Code Form is subject to the terms of the Mozilla Public License, v. 2.0.
 * If a copy of the MPL was not distributed with this file, You can obtain one at
 * https://mozilla.org/MPL/2.0/.
 */
package com.pandulapeter.campfire.chordpro.training

import kotlin.random.Random
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertFalse
import kotlin.test.assertNotEquals
import kotlin.test.assertTrue

class TrainingTest {

    /** A guitar from the highest string, as a diagram draws it. */
    private val guitar = listOf(64, 59, 55, 50, 45, 40)

    @Test
    fun `the note at a place is the open note of its string and the frets above it`() {
        val trainer = FretboardTrainer(guitar, random = Random(1))
        assertEquals(4, trainer.pitchClassAt(FretPosition(row = 0, fret = 0)))
        assertEquals(0, trainer.pitchClassAt(FretPosition(row = 0, fret = 8)))
        assertEquals(9, trainer.pitchClassAt(FretPosition(row = 4, fret = 0)))
        assertEquals(0, trainer.pitchClassAt(FretPosition(row = 4, fret = 3)))
    }

    @Test
    fun `the score counts the right answers and the best run of them`() {
        val trainer = FretboardTrainer(guitar, random = Random(2))
        val place = FretPosition(row = 5, fret = 3)
        assertTrue(trainer.answer(place, 7))
        assertTrue(trainer.answer(place, 7))
        assertFalse(trainer.answer(place, 8))
        assertTrue(trainer.answer(place, 7))
        assertEquals(4, trainer.attempts)
        assertEquals(3, trainer.hits)
        assertEquals(2, trainer.bestStreak)
        assertEquals(1, trainer.streak)
        assertEquals(0.75f, trainer.accuracy)
    }

    @Test
    fun `a place is never asked twice in a row, and every place is within the frets asked`() {
        val trainer = FretboardTrainer(guitar, maxFret = 5, random = Random(3))
        var before: FretPosition? = null
        repeat(300) {
            val place = trainer.next()
            assertNotEquals(before, place)
            assertTrue(place.row in guitar.indices && place.fret in 0..5)
            before = place
        }
    }

    @Test
    fun `a place that is missed comes back more often than one that is known`() {
        val trainer = FretboardTrainer(listOf(40, 45), maxFret = 3, random = Random(4))
        val known = FretPosition(0, 0)
        val missed = FretPosition(0, 1)
        repeat(6) { trainer.answer(known, 4) }
        repeat(4) { trainer.answer(missed, 0) }
        val asked = List(2000) { trainer.next() }
        assertTrue(asked.count { it == missed } > asked.count { it == known } * 2)
    }

    @Test
    fun `the help is taken away after two right answers on a chord and given back by a wrong one`() {
        val study = ChordStudy(chordCount = 2, random = Random(5))
        assertEquals(StudyHelp.NOTES, study.helpFor(0))
        study.record(0, isHit = true)
        assertEquals(StudyHelp.NOTES, study.helpFor(0))
        study.record(0, isHit = true)
        assertEquals(StudyHelp.FINGERS, study.helpFor(0))
        repeat(2) { study.record(0, isHit = true) }
        assertEquals(StudyHelp.BLANK, study.helpFor(0))
        repeat(4) { study.record(0, isHit = true) }
        assertEquals(StudyHelp.BLANK, study.helpFor(0))
        study.record(0, isHit = false)
        assertEquals(StudyHelp.FINGERS, study.helpFor(0))
        assertEquals(StudyHelp.NOTES, study.helpFor(1))
    }

    @Test
    fun `the chords that are missed are asked about more often`() {
        val study = ChordStudy(chordCount = 3, random = Random(6))
        repeat(5) { study.record(0, isHit = true) }
        repeat(3) { study.record(1, isHit = false) }
        val asked = List(2000) { study.nextChord() }
        assertTrue(asked.count { it == 1 } > asked.count { it == 0 })
        assertEquals(5, study.hits)
        assertEquals(8, study.attempts)
    }
}

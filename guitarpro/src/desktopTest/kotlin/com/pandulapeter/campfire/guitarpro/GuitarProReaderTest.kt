/*
 * This file is part of Campfire.
 * Copyright (c) Pandula Péter 2017-2026.
 * https://github.com/pandulapeter/campfire
 *
 * This Source Code Form is subject to the terms of the Mozilla Public License, v. 2.0.
 * If a copy of the MPL was not distributed with this file, You can obtain one at
 * https://mozilla.org/MPL/2.0/.
 */
package com.pandulapeter.campfire.guitarpro

import com.pandulapeter.campfire.guitarpro.GuitarProFixtures.NAMES
import com.pandulapeter.campfire.guitarpro.GuitarProFixtures.bytesOf
import com.pandulapeter.campfire.guitarpro.model.Beat
import com.pandulapeter.campfire.guitarpro.model.ChordShape
import com.pandulapeter.campfire.guitarpro.model.Score
import com.pandulapeter.campfire.guitarpro.reader.GuitarProException
import com.pandulapeter.campfire.guitarpro.reader.GuitarProException.Reason
import com.pandulapeter.campfire.guitarpro.reader.GuitarProReader
import kotlin.test.Test
import kotlin.test.assertContentEquals
import kotlin.test.assertEquals
import kotlin.test.assertFailsWith
import kotlin.test.assertNotNull
import kotlin.test.assertTrue

class GuitarProReaderTest {

    private fun scoreOf(name: String): Score = GuitarProReader.read(bytesOf(name))

    /** What a beat is, without what only some versions of the format keep. */
    private fun Beat.summary() = listOf(start, duration, status, notes.map { Triple(it.string, it.fret, it.kind) }, chord, text)

    @Test
    fun `the same song written in every version is read as the same notes`() {
        val scores = NAMES.map(::scoreOf)
        val expected = scores.first()
        for ((name, score) in NAMES.zip(scores)) {
            assertEquals(expected.title, score.title, name)
            assertEquals(expected.headers.size, score.headers.size, name)
            assertEquals(expected.headers.map { it.start to it.length }, score.headers.map { it.start to it.length }, name)
            assertEquals(expected.tracks.map { it.strings }, score.tracks.map { it.strings }, name)
            val beats = score.tracks.map { track -> track.measures.map { measure -> measure.voices.first().map { it.summary() } } }
            assertEquals(expected.tracks.map { track -> track.measures.map { measure -> measure.voices.first().map { it.summary() } } }, beats, name)
        }
    }

    @Test
    fun `the text of the file keeps its accents`() {
        val score = scoreOf("song.gp5")
        assertEquals("Práctica 002", score.title)
        assertEquals(100, score.tempo)
    }

    @Test
    fun `the effects of the notes and the text of the beats are kept`() {
        for (name in NAMES) {
            val score = scoreOf(name)
            val guitar = score.tracks.first()
            assertEquals("Verso", guitar.measures[0].voices[0][0].text, name)
            assertTrue(guitar.measures[0].voices[0][2].notes[0].effects.hammer, "$name hammer")
            assertTrue(guitar.measures[1].voices[0][4].notes.any { it.effects.slide != null }, "$name slide")
            assertNotNull(guitar.measures[3].voices[0][0].notes[0].effects.bend, "$name bend")
            assertTrue(guitar.measures[3].voices[0][1].notes[0].effects.harmonic, "$name harmonic")
            assertEquals("Intro", score.headers[0].marker, name)
            assertEquals(2, guitar.capo, name)
        }
    }

    @Test
    fun `a chord diagram is read with its frets, named or not, in every version`() {
        for (name in NAMES) {
            val guitar = scoreOf(name).tracks.first()
            val named = guitar.measures[0].voices[0][0]
            assertEquals("Am", named.chord, name)
            assertEquals(ChordShape(firstFret = 1, frets = listOf(0, 1, 2, 2, 0, -1)), named.chordShape, name)
            val unnamed = guitar.measures[1].voices[0][0]
            assertEquals(null, unnamed.chord, name)
            assertEquals(ChordShape(firstFret = 7, frets = listOf(0, 8, 9, 9, 7, -1)), unnamed.chordShape, name)
            assertEquals(null, guitar.measures[0].voices[0][1].chordShape, name)
        }
    }

    @Test
    fun `a repeat that closes says how many times it is played`() {
        val headers = scoreOf("song.gp4").headers
        assertTrue(headers[1].isRepeatOpen)
        assertEquals(2, headers[3].repeatClose)
        assertEquals(-1, headers[0].repeatClose)
    }

    @Test
    fun `what is not a guitar pro file is refused`() {
        assertEquals(Reason.NOT_GUITAR_PRO, assertFailsWith<GuitarProException> { GuitarProReader.read("not a score at all, but a text".encodeToByteArray()) }.reason)
        assertFailsWith<GuitarProException> { GuitarProReader.read(ByteArray(0)) }
    }

    @Test
    fun `a file that ends before its score does is refused`() {
        val bytes = bytesOf("song.gp5")
        assertEquals(Reason.UNEXPECTED_END, assertFailsWith<GuitarProException> { GuitarProReader.read(bytes.copyOf(bytes.size / 2)) }.reason)
    }

    @Test
    fun `a file whose last measure lacks the closing byte is read`() {
        // Guitar Pro 5 files from other editors end without the byte that says where the line breaks, and Guitar Pro reads past it.
        val bytes = bytesOf("song.gp5")
        val complete = GuitarProReader.read(bytes)
        val truncated = GuitarProReader.read(bytes.copyOf(bytes.size - 1))
        assertContentEquals(complete.headers.map { it.start }, truncated.headers.map { it.start })
    }
}

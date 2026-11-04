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

import com.pandulapeter.campfire.guitarpro.draw.Draw
import com.pandulapeter.campfire.guitarpro.GuitarProFixtures.bytesOf
import com.pandulapeter.campfire.guitarpro.layout.LayoutOptions
import com.pandulapeter.campfire.guitarpro.layout.ScoreLayouter
import com.pandulapeter.campfire.guitarpro.reader.GuitarProReader
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertTrue

class ScoreLayouterTest {

    private val score = GuitarProReader.read(bytesOf("song.gp5"))

    private fun texts(options: LayoutOptions) =
        ScoreLayouter.layout(score, 0, options).systems.flatMap { it.draws }.filterIsInstance<Draw.Text>().map { it.text }

    @Test
    fun `a narrower page has more systems and none is wider than the page`() {
        val wide = ScoreLayouter.layout(score, 0, LayoutOptions(width = 120f))
        val narrow = ScoreLayouter.layout(score, 0, LayoutOptions(width = 40f))
        assertTrue(narrow.systems.size > wide.systems.size)
        assertEquals(40f, narrow.width)
        assertTrue(narrow.height > wide.height)
    }

    @Test
    fun `systems follow one another down the page without overlapping`() {
        val systems = ScoreLayouter.layout(score, 0, LayoutOptions(width = 40f)).systems
        for ((above, below) in systems.zipWithNext()) {
            assertTrue(above.top + above.height <= below.top)
        }
    }

    @Test
    fun `the tablature shows the frets and the staff does not`() {
        val tab = texts(LayoutOptions(showNotation = false))
        val staff = texts(LayoutOptions(showTab = false))
        assertTrue("T" in tab && "A" in tab && "B" in tab)
        assertTrue("T" !in staff)
        assertTrue(staff.size < tab.size)
    }

    @Test
    fun `a score with nothing to show draws nothing`() {
        assertTrue(ScoreLayouter.layout(score, 0, LayoutOptions(showNotation = false, showTab = false)).systems.isEmpty())
        assertTrue(ScoreLayouter.layout(score, 99, LayoutOptions()).systems.isEmpty())
    }

    @Test
    fun `a chord diagram is drawn over its beat, with its first fret where it starts up the neck`() {
        val all = texts(LayoutOptions())
        assertTrue("Am" in all)
        assertTrue("7" in all)
    }

    @Test
    fun `the chord and the marker of a measure are written over it`() {
        val all = texts(LayoutOptions())
        assertTrue("Intro" in all)
        assertTrue("Verso" in all)
    }
}

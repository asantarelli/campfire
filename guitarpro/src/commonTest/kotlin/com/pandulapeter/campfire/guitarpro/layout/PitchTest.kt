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

import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertFalse
import kotlin.test.assertTrue

class PitchTest {

    @Test
    fun `a note is written on the step of its letter`() {
        // E4, the bottom line of the treble staff, and the G above it.
        assertEquals(PitchPosition(30, 0), pitchPosition(64, 0))
        assertEquals(PitchPosition(32, 0), pitchPosition(67, 0))
    }

    @Test
    fun `a sharp key spells its black keys with sharps and a flat one with flats`() {
        assertEquals(PitchPosition(31, 1), pitchPosition(66, 2))
        assertEquals(PitchPosition(34, -1), pitchPosition(70, -1))
    }

    @Test
    fun `a key alters the letters it names`() {
        assertEquals(List(7) { 0 }, (0 until 7).map { keyAlteration(it, 0) })
        // G major has an F sharp, D major an F and a C sharp, F major a B flat.
        assertEquals(1, keyAlteration(3, 1))
        assertEquals(0, keyAlteration(0, 1))
        assertEquals(1, keyAlteration(0, 2))
        assertEquals(-1, keyAlteration(6, -1))
        assertEquals(0, keyAlteration(2, -1))
    }

    @Test
    fun `stems go away from the notes farthest from the middle line`() {
        assertTrue(stemIsUp(listOf(30, 32)))
        assertFalse(stemIsUp(listOf(36, 38)))
        assertTrue(stemIsUp(listOf(30, 36)))
        assertTrue(stemIsUp(listOf(28, 38)))
        assertFalse(stemIsUp(listOf(30, 40)))
    }
}

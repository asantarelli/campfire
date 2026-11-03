/*
 * This file is part of Campfire.
 * Copyright (c) Pandula Péter 2017-2026.
 * https://github.com/pandulapeter/campfire
 *
 * This Source Code Form is subject to the terms of the Mozilla Public License, v. 2.0.
 * If a copy of the MPL was not distributed with this file, You can obtain one at
 * https://mozilla.org/MPL/2.0/.
 */
package com.pandulapeter.campfire.presentation.ui.dialogs.score

import com.pandulapeter.campfire.data.model.importing.ImportedFile
import com.pandulapeter.campfire.guitarpro.model.Score
import com.pandulapeter.campfire.guitarpro.model.Track
import com.pandulapeter.campfire.guitarpro.reader.GuitarProException.Reason
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertFalse
import kotlin.test.assertTrue

class ScoreReadingTest {

    @Test
    fun `a file is a score by its extension, whatever its case`() {
        assertTrue(ImportedFile(name = "exercises.GP5", bytes = ByteArray(0)).isGuitarProScore)
        assertTrue(ImportedFile(name = "a.gp3", bytes = ByteArray(0)).isGuitarProScore)
        assertFalse(ImportedFile(name = "song.cho", bytes = ByteArray(0)).isGuitarProScore)
        assertFalse(ImportedFile(name = "gp5", bytes = ByteArray(0)).isGuitarProScore)
    }

    @Test
    fun `bytes that are no score are a reason rather than a crash`() {
        assertEquals(ScoreReading.Failed(Reason.NOT_GUITAR_PRO), readScore("just some text, not a score".encodeToByteArray()))
        assertEquals(ScoreReading.Failed(Reason.NOT_GUITAR_PRO), readScore(ByteArray(0)))
    }

    @Test
    fun `a score opens on its first track that is not a drum kit`() {
        fun track(isPercussion: Boolean) = Track(name = "", strings = listOf(40), capo = 0, isPercussion = isPercussion, measures = emptyList())
        assertEquals(1, Score(tracks = listOf(track(true), track(false))).firstReadableTrack())
        assertEquals(0, Score(tracks = listOf(track(true))).firstReadableTrack())
        assertEquals(0, Score().firstReadableTrack())
    }
}

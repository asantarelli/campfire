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

import com.pandulapeter.campfire.chordpro.ChordProParser
import com.pandulapeter.campfire.chordpro.edit.ChordProPrettifier
import com.pandulapeter.campfire.guitarpro.song.ScoreSongText
import kotlin.test.Test
import kotlin.test.assertContentEquals
import kotlin.test.assertEquals
import kotlin.test.assertFalse
import kotlin.test.assertNull
import kotlin.test.assertTrue

/** What the library does to a song file leaves a score inside it as it was, which is the whole of how a score is kept. */
class ScoreSongTextTest {

    private val bytes = ByteArray(5000) { (it * 31 + it / 7).toByte() }

    @Test
    fun `the file comes back out of the song the metadata was read from`() {
        val text = ScoreSongText.encode(title = "Exercises", artist = "Somebody", bytes = bytes)
        val metadata = ChordProParser.parseMetadata(text)
        assertEquals("Exercises", metadata.title)
        assertEquals("Somebody", metadata.artist)
        assertTrue(ScoreSongText.isScore(metadata.custom))
        assertContentEquals(bytes, ScoreSongText.bytesOf(metadata.custom))
    }

    @Test
    fun `prettifying and editing the header keep the score`() {
        val text = ScoreSongText.encode(title = "Exercises", artist = "Somebody", bytes = bytes)
        val prettified = ChordProPrettifier.prettify(text)
        assertContentEquals(bytes, ScoreSongText.bytesOf(ChordProParser.parseMetadata(prettified).custom))
    }

    @Test
    fun `a title or an artist that would close the directive is cleaned`() {
        val metadata = ChordProParser.parseMetadata(ScoreSongText.encode(title = "A {sharp} } day", artist = "B\nC", bytes = bytes))
        assertEquals("A  sharp    day", metadata.title)
        assertEquals("B C", metadata.artist)
        assertTrue(ScoreSongText.isScore(metadata.custom))
    }

    @Test
    fun `a song that holds no score says so`() {
        val metadata = ChordProParser.parseMetadata("{title: Amazing Grace}\n[G]Amazing grace\n")
        assertFalse(ScoreSongText.isScore(metadata.custom))
        assertNull(ScoreSongText.bytesOf(metadata.custom))
    }
}

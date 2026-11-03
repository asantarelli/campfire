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

import androidx.compose.ui.test.ExperimentalTestApi
import androidx.compose.ui.test.hasAnyAncestor
import androidx.compose.ui.test.hasText
import androidx.compose.ui.test.isPopup
import androidx.compose.ui.test.onAllNodesWithContentDescription
import androidx.compose.ui.test.onAllNodesWithText
import androidx.compose.ui.test.onFirst
import androidx.compose.ui.test.onNodeWithContentDescription
import androidx.compose.ui.test.onNodeWithText
import androidx.compose.ui.test.performClick
import com.pandulapeter.campfire.data.model.importing.ImportedFile
import com.pandulapeter.campfire.presentation.resources.Res
import com.pandulapeter.campfire.presentation.resources.import_files
import com.pandulapeter.campfire.presentation.resources.score_staff
import com.pandulapeter.campfire.presentation.resources.score_tablature
import com.pandulapeter.campfire.presentation.resources.import_review_confirm
import com.pandulapeter.campfire.presentation.resources.songs_kind_chordpro
import com.pandulapeter.campfire.presentation.resources.songs_kind_guitar_pro
import com.pandulapeter.campfire.presentation.resources.songs_new_song
import com.pandulapeter.campfire.screenshots.AppTestScope
import com.pandulapeter.campfire.screenshots.runAppTest
import kotlin.test.Test

@OptIn(ExperimentalTestApi::class)
internal class ScoreSheetTest {

    @Test
    fun `a guitar pro file picked for import becomes a song marked with a G that opens its score`() = runAppTest(pickedFiles = listOf(score)) {
        importScore()
        onNodeWithText(SCORE_TITLE).assertExists()
        onNodeWithContentDescription(string(Res.string.songs_kind_guitar_pro)).assertExists()
        onNodeWithText(SCORE_TITLE).performClick()
        onNodeWithText(string(Res.string.score_staff)).assertExists()
        onNodeWithText(string(Res.string.score_tablature)).assertExists()
    }

    @Test
    fun `the tablature can be hidden but the staff cannot be hidden with it`() = runAppTest(pickedFiles = listOf(score)) {
        importScore()
        onNodeWithText(SCORE_TITLE).performClick()
        onNodeWithText(string(Res.string.score_tablature)).performClick()
        onNodeWithText(string(Res.string.score_staff)).performClick()
        // The page keeps one of the two: the staff, which the second tap would have taken away, is still shown.
        onNodeWithText(string(Res.string.score_staff)).assertExists()
    }

    @Test
    fun `the songs of the demo library are marked with a C`() = runAppTest {
        onAllNodesWithContentDescription(string(Res.string.songs_kind_chordpro)).onFirst().assertExists()
    }

    private fun AppTestScope.importScore() {
        onNodeWithContentDescription(string(Res.string.songs_new_song)).performClick()
        // In the menu's popup: the demo library is two songs, so the list under it ends in a hint with the same button.
        onNode(hasText(string(Res.string.import_files)) and hasAnyAncestor(isPopup())).performClick()
        waitUntil(timeoutMillis = IMPORT_TIMEOUT_MILLIS) {
            onAllNodesWithContentDescription(pluralString(Res.plurals.import_review_confirm, 1)).fetchSemanticsNodes().isNotEmpty()
        }
        onNodeWithContentDescription(pluralString(Res.plurals.import_review_confirm, 1)).performClick()
        waitUntil(timeoutMillis = IMPORT_TIMEOUT_MILLIS) { onAllNodesWithText(SCORE_TITLE).fetchSemanticsNodes().isNotEmpty() }
    }

    private val score
        get() = ImportedFile(name = "song.gp5", bytes = checkNotNull(AppTestScope::class.java.getResourceAsStream("/scores/song.gp5")).use { it.readBytes() })

    private companion object {
        const val SCORE_TITLE = "Práctica 002"
        const val IMPORT_TIMEOUT_MILLIS = 10_000L
    }
}

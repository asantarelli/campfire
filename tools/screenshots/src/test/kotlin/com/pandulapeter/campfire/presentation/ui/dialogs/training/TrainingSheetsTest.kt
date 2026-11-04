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

import androidx.compose.ui.test.ExperimentalTestApi
import androidx.compose.ui.test.onAllNodesWithText
import androidx.compose.ui.test.onNodeWithContentDescription
import androidx.compose.ui.test.onNodeWithText
import androidx.compose.ui.test.performClick
import com.pandulapeter.campfire.presentation.resources.Res
import com.pandulapeter.campfire.presentation.resources.training_neck_title
import com.pandulapeter.campfire.presentation.resources.training_next
import com.pandulapeter.campfire.presentation.resources.training_open_neck
import com.pandulapeter.campfire.presentation.resources.training_open_study
import com.pandulapeter.campfire.presentation.resources.training_study_title
import com.pandulapeter.campfire.screenshots.AppTestScope
import com.pandulapeter.campfire.screenshots.openChords
import com.pandulapeter.campfire.screenshots.openScales
import com.pandulapeter.campfire.screenshots.runAppTest
import kotlin.test.Test
import kotlin.test.assertTrue

@OptIn(ExperimentalTestApi::class)
internal class TrainingSheetsTest {

    @Test
    fun `the neck trainer opens from the Scales tab, takes an answer and goes on to the next`() = runAppTest {
        openScales()
        onNodeWithContentDescription(string(Res.string.training_open_neck)).performClick()
        onNodeWithText(string(Res.string.training_neck_title)).assertExists()
        answerAndGoOn()
    }

    @Test
    fun `the study mode opens from the Chords tab, takes an answer and goes on to the next`() = runAppTest {
        openChords()
        onNodeWithContentDescription(string(Res.string.training_open_study)).performClick()
        onNodeWithText(string(Res.string.training_study_title)).assertExists()
        answerAndGoOn()
    }

    /** Any of the twelve notes is an answer; once it is given the next question can be asked. */
    private fun AppTestScope.answerAndGoOn() {
        // The note buttons stand under the question: "E" is one of them whatever the question is.
        val buttons = onAllNodesWithText("E")
        assertTrue(buttons.fetchSemanticsNodes().isNotEmpty())
        buttons[buttons.fetchSemanticsNodes().lastIndex].performClick()
        onNodeWithText(string(Res.string.training_next)).assertExists().performClick()
    }
}

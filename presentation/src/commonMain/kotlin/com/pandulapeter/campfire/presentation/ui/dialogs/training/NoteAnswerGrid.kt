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

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.ExperimentalLayoutApi
import androidx.compose.foundation.layout.FlowRow
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.widthIn
import androidx.compose.material3.Button
import androidx.compose.material3.ButtonDefaults
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.dp

/**
 * The twelve notes to answer with. Once [answered] the buttons stop taking presses and show how it went: the right note in the accent
 * and the player's wrong one in the error color.
 *
 * @param correct The pitch class of the right answer, known to the grid only once the question is answered.
 */
@OptIn(ExperimentalLayoutApi::class)
@Composable
internal fun NoteAnswerGrid(
    modifier: Modifier = Modifier,
    choices: List<NoteChoice>,
    answered: Int?,
    correct: Int,
    onAnswered: (Int) -> Unit,
) = FlowRow(
    modifier = modifier.fillMaxWidth(),
    horizontalArrangement = Arrangement.spacedBy(8.dp, Alignment.CenterHorizontally),
    verticalArrangement = Arrangement.spacedBy(8.dp),
) {
    choices.forEach { choice ->
        val colors = when {
            answered == null -> ButtonDefaults.filledTonalButtonColors()
            choice.pitchClass == correct -> ButtonDefaults.buttonColors()
            choice.pitchClass == answered -> ButtonDefaults.buttonColors(
                containerColor = MaterialTheme.colorScheme.error,
                contentColor = MaterialTheme.colorScheme.onError,
            )
            else -> ButtonDefaults.filledTonalButtonColors()
        }
        Button(
            modifier = Modifier.widthIn(min = 72.dp),
            enabled = answered == null || choice.pitchClass == correct || choice.pitchClass == answered,
            colors = colors,
            onClick = { if (answered == null) onAnswered(choice.pitchClass) },
        ) {
            Text(text = choice.label, style = MaterialTheme.typography.titleMedium)
        }
    }
}

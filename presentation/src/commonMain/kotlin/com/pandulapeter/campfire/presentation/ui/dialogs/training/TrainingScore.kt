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

import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.text.style.TextAlign
import com.pandulapeter.campfire.presentation.localization.stringResource
import com.pandulapeter.campfire.presentation.resources.Res
import com.pandulapeter.campfire.presentation.resources.training_streak
import com.pandulapeter.campfire.presentation.resources.training_session
import com.pandulapeter.campfire.presentation.resources.training_total

/** How the answers have gone: this session's, the run of right ones in a row (with the best one), and every session's together. */
@Composable
internal fun TrainingScore(
    modifier: Modifier = Modifier,
    sessionHits: Int,
    sessionAttempts: Int,
    streak: Int,
    bestStreak: Int,
    totalHits: Int,
    totalAttempts: Int,
) = Column(modifier = modifier.fillMaxWidth()) {
    val style = MaterialTheme.typography.bodyMedium
    val color = MaterialTheme.colorScheme.onSurfaceVariant
    Text(
        modifier = Modifier.fillMaxWidth(),
        text = stringResource(Res.string.training_session, sessionHits, sessionAttempts),
        style = style,
        color = color,
        textAlign = TextAlign.Center,
    )
    Text(
        modifier = Modifier.fillMaxWidth(),
        text = stringResource(Res.string.training_streak, streak, bestStreak),
        style = style,
        color = color,
        textAlign = TextAlign.Center,
    )
    Text(
        modifier = Modifier.fillMaxWidth(),
        text = stringResource(Res.string.training_total, totalHits, totalAttempts),
        style = style,
        color = color,
        textAlign = TextAlign.Center,
    )
}

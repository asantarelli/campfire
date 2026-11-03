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

import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableIntStateOf
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.dp
import com.pandulapeter.campfire.guitarpro.model.Score
import com.pandulapeter.campfire.guitarpro.reader.GuitarProException.Reason
import com.pandulapeter.campfire.presentation.localization.stringResource
import com.pandulapeter.campfire.presentation.resources.Res
import com.pandulapeter.campfire.presentation.resources.score_error_corrupt
import com.pandulapeter.campfire.presentation.resources.score_error_not_guitar_pro
import com.pandulapeter.campfire.presentation.resources.score_error_unexpected_end
import com.pandulapeter.campfire.presentation.resources.score_error_unsupported
import com.pandulapeter.campfire.presentation.ui.CampfireViewModel
import com.pandulapeter.campfire.presentation.ui.dialogs.DialogType
import com.pandulapeter.campfire.presentation.ui.dialogs.sheet.CampfireBottomSheet

/**
 * The staff and the tablature of the Guitar Pro score a song holds, read as the sheet opens: the sheet is a viewer, and what is edited
 * in a score editor stays there. It is as wide as the window lets it be (a score is read in lines, and a longer line is a smaller
 * page of them), and a file that cannot be read says why in place of the score.
 */
@Composable
internal fun ScoreSheet(
    viewModel: CampfireViewModel,
    dialog: DialogType.GuitarProScore,
) {
    val reading = remember(dialog.bytes) { readScore(dialog.bytes) }
    CampfireBottomSheet(
        title = dialog.song.title,
        subtitle = dialog.song.artist,
        sheetMaxWidth = SCORE_SHEET_MAX_WIDTH,
        onDismiss = { viewModel.dialogHost.dismissSheet(dialog) },
    ) { contentPadding ->
        when (reading) {
            is ScoreReading.Read -> ScoreContent(score = reading.score, contentPadding = contentPadding)
            is ScoreReading.Failed -> Text(
                modifier = Modifier.fillMaxWidth().padding(16.dp),
                text = stringResource(reading.reason.message()),
                style = MaterialTheme.typography.bodyMedium,
                color = MaterialTheme.colorScheme.onSurfaceVariant,
            )
        }
    }
}

@Composable
private fun androidx.compose.foundation.layout.ColumnScope.ScoreContent(
    score: Score,
    contentPadding: PaddingValues,
) {
    var track by remember(score) { mutableIntStateOf(score.firstReadableTrack()) }
    var showNotation by remember { mutableStateOf(true) }
    var showTab by remember { mutableStateOf(true) }
    var showRhythm by remember { mutableStateOf(true) }
    var space by remember { mutableIntStateOf(ScoreSizes.DEFAULT) }
    ScoreControls(
        score = score,
        track = track,
        showNotation = showNotation,
        showTab = showTab,
        showRhythm = showRhythm,
        space = space,
        onTrackSelected = { track = it },
        // One of the two staves is always shown: a page with neither has nothing on it.
        onNotationToggled = { if (!showNotation || showTab) showNotation = !showNotation },
        onTabToggled = { if (!showTab || showNotation) showTab = !showTab },
        onRhythmToggled = { showRhythm = !showRhythm },
        onSpaceChanged = { space = it },
    )
    ScorePage(
        score = score,
        view = ScoreView(track = track, showNotation = showNotation, showTab = showTab, showRhythm = showRhythm, space = space),
        contentPadding = contentPadding,
    )
}

private fun Reason.message() = when (this) {
    Reason.NOT_GUITAR_PRO -> Res.string.score_error_not_guitar_pro
    Reason.UNSUPPORTED_VERSION -> Res.string.score_error_unsupported
    Reason.UNEXPECTED_END -> Res.string.score_error_unexpected_end
    Reason.CORRUPT -> Res.string.score_error_corrupt
}

private val SCORE_SHEET_MAX_WIDTH = 1200.dp

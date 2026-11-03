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

import androidx.compose.foundation.horizontalScroll
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.rememberScrollState
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Slider
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.semantics.Role
import androidx.compose.ui.semantics.contentDescription
import androidx.compose.ui.semantics.semantics
import androidx.compose.ui.unit.dp
import com.pandulapeter.campfire.guitarpro.model.Score
import com.pandulapeter.campfire.presentation.localization.stringResource
import com.pandulapeter.campfire.presentation.resources.Res
import com.pandulapeter.campfire.presentation.resources.score_rhythm
import com.pandulapeter.campfire.presentation.resources.score_size
import com.pandulapeter.campfire.presentation.resources.score_staff
import com.pandulapeter.campfire.presentation.resources.score_tablature
import com.pandulapeter.campfire.presentation.resources.score_track_unnamed
import com.pandulapeter.campfire.presentation.ui.components.controls.SelectableChip
import kotlin.math.roundToInt

/** The tracks of a score, what is shown of the chosen one, and how big it is set. */
@Composable
internal fun ScoreControls(
    score: Score,
    track: Int,
    showNotation: Boolean,
    showTab: Boolean,
    showRhythm: Boolean,
    space: Int,
    onTrackSelected: (Int) -> Unit,
    onNotationToggled: () -> Unit,
    onTabToggled: () -> Unit,
    onRhythmToggled: () -> Unit,
    onSpaceChanged: (Int) -> Unit,
) {
    if (score.tracks.size > 1) {
        Row(
            modifier = Modifier.fillMaxWidth().horizontalScroll(rememberScrollState()).padding(horizontal = 16.dp),
            horizontalArrangement = Arrangement.spacedBy(CHIP_GAP),
        ) {
            score.tracks.forEachIndexed { index, item ->
                val name = item.name.trim().ifEmpty { stringResource(Res.string.score_track_unnamed, index + 1) }
                ScoreChip(label = name, isSelected = index == track, role = Role.RadioButton, onClick = { onTrackSelected(index) })
            }
        }
    }
    Row(
        modifier = Modifier.fillMaxWidth().horizontalScroll(rememberScrollState()).padding(horizontal = 16.dp, vertical = 8.dp),
        horizontalArrangement = Arrangement.spacedBy(CHIP_GAP),
        verticalAlignment = Alignment.CenterVertically,
    ) {
        ScoreChip(label = stringResource(Res.string.score_staff), isSelected = showNotation, role = Role.Checkbox, onClick = onNotationToggled)
        ScoreChip(label = stringResource(Res.string.score_tablature), isSelected = showTab, role = Role.Checkbox, onClick = onTabToggled)
        ScoreChip(
            label = stringResource(Res.string.score_rhythm),
            isSelected = showRhythm && showTab,
            isEnabled = showTab,
            role = Role.Checkbox,
            onClick = onRhythmToggled,
        )
        val sizeDescription = stringResource(Res.string.score_size)
        Text(text = sizeDescription, style = MaterialTheme.typography.labelLarge, color = MaterialTheme.colorScheme.onSurfaceVariant)
        Slider(
            modifier = Modifier.width(SIZE_SLIDER_WIDTH).semantics { contentDescription = sizeDescription },
            value = space.toFloat(),
            onValueChange = { value -> value.roundToInt().let { if (it != space) onSpaceChanged(it) } },
            valueRange = ScoreSizes.MIN.toFloat()..ScoreSizes.MAX.toFloat(),
            steps = ScoreSizes.MAX - ScoreSizes.MIN - 1,
        )
    }
}

@Composable
private fun ScoreChip(
    label: String,
    isSelected: Boolean,
    role: Role,
    isEnabled: Boolean = true,
    onClick: () -> Unit,
) = SelectableChip(isSelected = isSelected, isEnabled = isEnabled, role = role, onClick = onClick) { contentColor ->
    Text(text = label, style = MaterialTheme.typography.labelLarge, color = contentColor)
}

private val CHIP_GAP = 8.dp
private val SIZE_SLIDER_WIDTH = 160.dp

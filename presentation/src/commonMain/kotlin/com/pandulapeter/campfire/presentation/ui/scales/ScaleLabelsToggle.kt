/*
 * This file is part of Campfire.
 * Copyright (c) Pandula Péter 2017-2026.
 * https://github.com/pandulapeter/campfire
 *
 * This Source Code Form is subject to the terms of the Mozilla Public License, v. 2.0.
 * If a copy of the MPL was not distributed with this file, You can obtain one at
 * https://mozilla.org/MPL/2.0/.
 */
package com.pandulapeter.campfire.presentation.ui.scales

import androidx.compose.animation.Crossfade
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.semantics.semantics
import androidx.compose.ui.semantics.stateDescription
import com.pandulapeter.campfire.presentation.localization.stringResource
import com.pandulapeter.campfire.presentation.resources.Res
import com.pandulapeter.campfire.presentation.resources.ic_scale_degrees
import com.pandulapeter.campfire.presentation.resources.ic_scale_notes
import com.pandulapeter.campfire.presentation.resources.scales_labels_degrees
import com.pandulapeter.campfire.presentation.resources.scales_labels_notes
import com.pandulapeter.campfire.presentation.resources.scales_labels_show_degrees
import com.pandulapeter.campfire.presentation.resources.scales_labels_show_notes
import org.jetbrains.compose.resources.painterResource

/**
 * The toggle between a neck's degrees and its notes, in the scale sheet's header and beside the Scales tab's search, one icon button rather than a segmented row, which took
 * most of a phone's header from the title. It shows the mode the neck is in now, as the app's other toggles show their
 * state, and tells a screen reader both: the tap's result as its description and the mode shown now as its state.
 */
@Composable
internal fun ScaleLabelsToggle(
    labels: ScaleLabels,
    onLabelsSelected: (ScaleLabels) -> Unit,
) {
    val shownLabel = stringResource(if (labels == ScaleLabels.DEGREES) Res.string.scales_labels_degrees else Res.string.scales_labels_notes)
    IconButton(
        modifier = Modifier.semantics { stateDescription = shownLabel },
        onClick = { onLabelsSelected(if (labels == ScaleLabels.DEGREES) ScaleLabels.NOTES else ScaleLabels.DEGREES) },
    ) {
        Crossfade(
            targetState = labels,
            label = "scaleLabelsToggle",
        ) { shownLabels ->
            if (shownLabels == ScaleLabels.DEGREES) {
                Icon(
                    painter = painterResource(Res.drawable.ic_scale_degrees),
                    contentDescription = stringResource(Res.string.scales_labels_show_notes),
                )
            } else {
                Icon(
                    painter = painterResource(Res.drawable.ic_scale_notes),
                    contentDescription = stringResource(Res.string.scales_labels_show_degrees),
                )
            }
        }
    }
}

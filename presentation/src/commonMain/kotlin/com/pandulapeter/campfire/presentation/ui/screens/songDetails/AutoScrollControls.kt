/*
 * This file is part of Campfire.
 * Copyright (c) Pandula Péter 2017-2026.
 * https://github.com/pandulapeter/campfire
 *
 * This Source Code Form is subject to the terms of the Mozilla Public License, v. 2.0.
 * If a copy of the MPL was not distributed with this file, You can obtain one at
 * https://mozilla.org/MPL/2.0/.
 */
package com.pandulapeter.campfire.presentation.ui.screens.songDetails

import androidx.compose.animation.AnimatedVisibility
import androidx.compose.animation.expandVertically
import androidx.compose.animation.fadeIn
import androidx.compose.animation.fadeOut
import androidx.compose.animation.scaleIn
import androidx.compose.animation.scaleOut
import androidx.compose.animation.shrinkVertically
import androidx.compose.foundation.ScrollState
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.BoxScope
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.SmallFloatingActionButton
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.focus.focusProperties
import androidx.compose.ui.graphics.painter.Painter
import androidx.compose.ui.semantics.contentDescription
import androidx.compose.ui.semantics.semantics
import androidx.compose.ui.unit.Dp
import androidx.compose.ui.unit.dp
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import com.pandulapeter.campfire.data.model.preferences.UserPreferences
import com.pandulapeter.campfire.presentation.localization.stringResource
import com.pandulapeter.campfire.presentation.resources.Res
import com.pandulapeter.campfire.presentation.resources.ic_media_play_pause
import com.pandulapeter.campfire.presentation.resources.ic_pause
import com.pandulapeter.campfire.presentation.resources.ic_play
import com.pandulapeter.campfire.presentation.resources.ic_subtract
import com.pandulapeter.campfire.presentation.resources.ic_add
import com.pandulapeter.campfire.presentation.resources.song_details_auto_scroll_faster
import com.pandulapeter.campfire.presentation.resources.song_details_auto_scroll_slower
import com.pandulapeter.campfire.presentation.resources.song_details_auto_scroll_speed
import com.pandulapeter.campfire.presentation.resources.song_details_auto_scroll_start
import com.pandulapeter.campfire.presentation.resources.song_details_auto_scroll_stop
import com.pandulapeter.campfire.presentation.resources.song_details_external_player
import com.pandulapeter.campfire.presentation.ui.CampfireViewModel
import com.pandulapeter.campfire.presentation.ui.platform.rememberExternalPlayerControl
import org.jetbrains.compose.resources.painterResource

/**
 * The buttons that make the song scroll itself, in the strip at the end of the screen the song leaves to its step
 * buttons: one that starts and stops it, and, while it runs, the two that set its speed with the speed between them,
 * which is the reader's ([UserPreferences.autoScrollSpeed]) and is kept for the next song. They are there only for a
 * song that has something to scroll to, and stop by themselves when anything else moves the song, when it reaches its
 * end and when the page changes (see [AutoScrollEffect]).
 *
 * [bottom] is where the column of buttons ends, above the step button that is the last of that strip.
 */
@Composable
internal fun BoxScope.AutoScrollControls(
    viewModel: CampfireViewModel,
    scrollState: ScrollState?,
    end: Dp,
    bottom: Dp,
) {
    val userPreferences by viewModel.userPreferences.collectAsStateWithLifecycle()
    val speed = userPreferences?.autoScrollSpeed ?: UserPreferences.DEFAULT_AUTO_SCROLL_SPEED
    var isRunning by remember { mutableStateOf(false) }
    // A page of another song is another scroll: a run never carries over from one to the next.
    LaunchedEffect(scrollState) { isRunning = false }
    AutoScrollEffect(
        isRunning = isRunning,
        scrollState = scrollState,
        dpPerSecond = autoScrollDpPerSecond(speed, userPreferences?.fontScale ?: UserPreferences.DEFAULT_FONT_SCALE),
        onFinished = { isRunning = false },
    )
    val hasSomethingToScroll = (scrollState?.maxValue ?: 0) > 0
    val externalPlayer = rememberExternalPlayerControl()
    AnimatedVisibility(
        modifier = Modifier.align(Alignment.BottomEnd).padding(end = end, bottom = bottom),
        visible = hasSomethingToScroll || isRunning || externalPlayer != null,
        enter = scaleIn() + fadeIn(),
        exit = scaleOut() + fadeOut(),
    ) {
        Column(
            verticalArrangement = Arrangement.spacedBy(AUTO_SCROLL_BUTTON_GAP),
            horizontalAlignment = Alignment.CenterHorizontally,
        ) {
            AnimatedVisibility(
                visible = isRunning,
                enter = expandVertically() + fadeIn(),
                exit = shrinkVertically() + fadeOut(),
            ) {
                val speedDescription = stringResource(Res.string.song_details_auto_scroll_speed, speed)
                Column(
                    verticalArrangement = Arrangement.spacedBy(AUTO_SCROLL_BUTTON_GAP),
                    horizontalAlignment = Alignment.CenterHorizontally,
                ) {
                    AutoScrollButton(
                        icon = painterResource(Res.drawable.ic_add),
                        contentDescription = stringResource(Res.string.song_details_auto_scroll_faster),
                        onClick = { viewModel.preferencesController.setAutoScrollSpeed(speed + 1) },
                    )
                    Text(
                        modifier = Modifier.semantics { contentDescription = speedDescription },
                        text = speed.toString(),
                        style = MaterialTheme.typography.labelLarge,
                        color = MaterialTheme.colorScheme.onSurfaceVariant,
                    )
                    AutoScrollButton(
                        icon = painterResource(Res.drawable.ic_subtract),
                        contentDescription = stringResource(Res.string.song_details_auto_scroll_slower),
                        onClick = { viewModel.preferencesController.setAutoScrollSpeed(speed - 1) },
                    )
                }
            }
            if (hasSomethingToScroll || isRunning) {
                AutoScrollButton(
                    icon = painterResource(if (isRunning) Res.drawable.ic_pause else Res.drawable.ic_play),
                    contentDescription = stringResource(
                        if (isRunning) Res.string.song_details_auto_scroll_stop else Res.string.song_details_auto_scroll_start,
                    ),
                    onClick = { isRunning = !isRunning },
                )
            }
            // The first of the column's buttons, apart from the song's own scroll: the player the musician plays along with.
            externalPlayer?.let { player ->
                AutoScrollButton(
                    icon = painterResource(Res.drawable.ic_media_play_pause),
                    contentDescription = stringResource(Res.string.song_details_external_player),
                    onClick = player::playPause,
                )
            }
        }
    }
}

@Composable
private fun AutoScrollButton(
    icon: Painter,
    contentDescription: String,
    onClick: () -> Unit,
) = SmallFloatingActionButton(
    // The buttons take no focus, as the step buttons do: a button that held it would be the target of every arrow
    // key after it, which scroll the song and step it instead.
    modifier = Modifier.size(STEP_BUTTON_SIZE).focusProperties { canFocus = false },
    onClick = onClick,
) {
    Icon(painter = icon, contentDescription = contentDescription)
}

private val AUTO_SCROLL_BUTTON_GAP = 4.dp

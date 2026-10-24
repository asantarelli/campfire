/*
 * This file is part of Campfire.
 * Copyright (c) Pandula Péter 2017-2026.
 * https://github.com/pandulapeter/campfire
 *
 * This Source Code Form is subject to the terms of the Mozilla Public License, v. 2.0.
 * If a copy of the MPL was not distributed with this file, You can obtain one at
 * https://mozilla.org/MPL/2.0/.
 */
package com.pandulapeter.campfire.presentation.ui.screens.settings

import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.lifecycle.Lifecycle
import androidx.lifecycle.compose.LifecycleEventEffect
import com.pandulapeter.campfire.presentation.localization.stringResource
import com.pandulapeter.campfire.presentation.resources.Res
import com.pandulapeter.campfire.presentation.resources.ic_media_restart
import com.pandulapeter.campfire.presentation.resources.settings_external_player
import com.pandulapeter.campfire.presentation.resources.settings_external_player_allow
import com.pandulapeter.campfire.presentation.resources.settings_external_player_allowed
import com.pandulapeter.campfire.presentation.resources.settings_external_player_description
import com.pandulapeter.campfire.presentation.ui.components.ActionListItem
import com.pandulapeter.campfire.presentation.ui.platform.rememberExternalPlayerControl
import com.pandulapeter.campfire.presentation.ui.settingsPage.SettingsSubsection
import org.jetbrains.compose.resources.painterResource

/**
 * What the system has to allow for the song's button that takes the music player back to the start to ask the player
 * for its position, shown only where there is something to allow. The answer is read again whenever the app comes back
 * to the front, since it is given on a screen of the system's own.
 */
@Composable
internal fun ExternalPlayerSettings() {
    val access = rememberExternalPlayerControl()?.restartAccess ?: return
    var isGranted by remember(access) { mutableStateOf(access.isGranted) }
    LifecycleEventEffect(Lifecycle.Event.ON_RESUME) { isGranted = access.isGranted }
    SettingsSubsection(
        title = stringResource(Res.string.settings_external_player),
        description = stringResource(Res.string.settings_external_player_description),
    ) {
        ActionListItem(
            title = stringResource(if (isGranted) Res.string.settings_external_player_allowed else Res.string.settings_external_player_allow),
            icon = painterResource(Res.drawable.ic_media_restart),
            isEmphasized = false,
            onClick = access::open,
        )
    }
}

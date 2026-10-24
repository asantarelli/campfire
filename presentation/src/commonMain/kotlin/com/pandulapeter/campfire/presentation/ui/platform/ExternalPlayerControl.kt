/*
 * This file is part of Campfire.
 * Copyright (c) Pandula Péter 2017-2026.
 * https://github.com/pandulapeter/campfire
 *
 * This Source Code Form is subject to the terms of the Mozilla Public License, v. 2.0.
 * If a copy of the MPL was not distributed with this file, You can obtain one at
 * https://mozilla.org/MPL/2.0/.
 */
package com.pandulapeter.campfire.presentation.ui.platform

import androidx.compose.runtime.Composable

/**
 * The play and pause of whatever is playing music on the device, which is a musician's own player (Spotify, YouTube Music,
 * a recording on the phone) next to the song being read, not the app's own: the song is read with a hand on the
 * instrument, and the button that stops the recording is the one nearest to it.
 */
internal interface ExternalPlayerControl {

    fun playPause()

    /**
     * Takes the player back to the start of what it is playing: the previous-track key, which a player that is past the
     * first seconds of a track answers by starting it over, and one that is not by going on to the track before it.
     */
    fun restart()
}

/**
 * The control of the device's media player, or null where the app cannot reach one: only Android and Windows let an app
 * press the media keys, whoever owns the player that is active, and that is the only thing they are used for.
 */
@Composable
internal expect fun rememberExternalPlayerControl(): ExternalPlayerControl?

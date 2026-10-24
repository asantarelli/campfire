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

    /** What the system has to allow for [restart] to put the player at the very start, or null where it needs nothing. */
    val restartAccess: RestartAccess?
        get() = null

    fun playPause()

    /**
     * Takes the player back to the start of what it is playing and plays it from there, and says whether it could be asked
     * to: a player may offer neither a position nor a previous track to another app - Spotify's free account does not - and
     * then there is nothing to ask. Where the system lets the app ask the player for a position that is what it does; where
     * it does not, it presses the previous-track key, which a player past the first seconds of a track answers by starting it
     * over, and many answer by doing nothing.
     */
    fun restart(): Boolean
}

/** The system's permission for [ExternalPlayerControl.restart] to ask the player for a position. */
internal interface RestartAccess {

    val isGranted: Boolean

    /** Opens the screen of the system's settings where it is given. */
    fun open()
}

/**
 * The control of the device's media player, or null where the app cannot reach one: only Android and Windows let an app
 * reach the player that is active, whoever owns it, and that is the only thing they are used for.
 */
@Composable
internal expect fun rememberExternalPlayerControl(): ExternalPlayerControl?

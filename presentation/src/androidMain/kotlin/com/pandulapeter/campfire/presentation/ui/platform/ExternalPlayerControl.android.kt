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

import android.content.ComponentName
import android.content.Context
import android.content.Intent
import android.media.AudioManager
import android.media.session.MediaController
import android.media.session.MediaSessionManager
import android.media.session.PlaybackState
import android.os.SystemClock
import android.provider.Settings
import android.view.KeyEvent
import androidx.compose.runtime.Composable
import androidx.compose.runtime.remember
import androidx.compose.ui.platform.LocalContext

/**
 * Reaches the media session that is active, which the system hands to the player that is playing or was playing last,
 * as a headset button does. Play and pause are the media key and need no permission. The start of a track is asked of the
 * player itself, with its position, where the user has allowed it - Android gives the sessions of other apps only to a
 * notification listener - and is the previous-track key otherwise, which a player that is not past the first seconds of its
 * track, or has no track before it, answers with something else or nothing. The app learns neither what is playing nor
 * whether anything is.
 */
@Composable
internal actual fun rememberExternalPlayerControl(): ExternalPlayerControl? {
    val context = LocalContext.current
    return remember(context) {
        (context.getSystemService(Context.AUDIO_SERVICE) as? AudioManager)?.let { AndroidPlayerControl(context.applicationContext, it) }
    }
}

private class AndroidPlayerControl(
    private val context: Context,
    private val audioManager: AudioManager,
) : ExternalPlayerControl {

    override val restartAccess: RestartAccess = object : RestartAccess {
        override val isGranted get() = Settings.Secure.getString(context.contentResolver, ENABLED_LISTENERS)?.contains(context.packageName) == true

        override fun open() {
            context.startActivity(Intent(Settings.ACTION_NOTIFICATION_LISTENER_SETTINGS).addFlags(Intent.FLAG_ACTIVITY_NEW_TASK))
        }
    }

    override fun playPause() = press(KeyEvent.KEYCODE_MEDIA_PLAY_PAUSE)

    override fun restart() {
        val player = activePlayer()
        if (player == null) {
            press(KeyEvent.KEYCODE_MEDIA_PREVIOUS)
        } else {
            player.transportControls.seekTo(0L)
            player.transportControls.play()
        }
    }

    /** The session of the player the musician plays along with: the one playing, else the one that played last; never the app's own. */
    private fun activePlayer(): MediaController? {
        if (!restartAccess.isGranted) return null
        val sessions = runCatching {
            val manager = context.getSystemService(Context.MEDIA_SESSION_SERVICE) as MediaSessionManager
            manager.getActiveSessions(ComponentName(context, PlayerSessionsService::class.java))
        }.getOrNull().orEmpty().filter { it.packageName != context.packageName }
        return sessions.firstOrNull { it.playbackState?.state == PlaybackState.STATE_PLAYING } ?: sessions.firstOrNull()
    }

    private fun press(keyCode: Int) {
        val now = SystemClock.uptimeMillis()
        audioManager.dispatchMediaKeyEvent(KeyEvent(now, now, KeyEvent.ACTION_DOWN, keyCode, 0))
        audioManager.dispatchMediaKeyEvent(KeyEvent(now, now, KeyEvent.ACTION_UP, keyCode, 0))
    }

    private companion object {
        const val ENABLED_LISTENERS = "enabled_notification_listeners"
    }
}

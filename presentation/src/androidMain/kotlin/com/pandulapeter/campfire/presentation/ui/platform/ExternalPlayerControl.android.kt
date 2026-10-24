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

import android.content.Context
import android.media.AudioManager
import android.os.SystemClock
import android.view.KeyEvent
import androidx.compose.runtime.Composable
import androidx.compose.runtime.remember
import androidx.compose.ui.platform.LocalContext

/**
 * Presses the media keys of the session that is active, which the system hands to the player that is playing or was
 * playing last, as a headset button does. It needs no permission and reads nothing: the app learns neither what is
 * playing nor whether anything is.
 */
@Composable
internal actual fun rememberExternalPlayerControl(): ExternalPlayerControl? {
    val context = LocalContext.current
    return remember(context) {
        (context.getSystemService(Context.AUDIO_SERVICE) as? AudioManager)?.let(::AudioManagerPlayerControl)
    }
}

private class AudioManagerPlayerControl(private val audioManager: AudioManager) : ExternalPlayerControl {

    override fun playPause() = press(KeyEvent.KEYCODE_MEDIA_PLAY_PAUSE)

    override fun restart() = press(KeyEvent.KEYCODE_MEDIA_PREVIOUS)

    private fun press(keyCode: Int) {
        val now = SystemClock.uptimeMillis()
        audioManager.dispatchMediaKeyEvent(KeyEvent(now, now, KeyEvent.ACTION_DOWN, keyCode, 0))
        audioManager.dispatchMediaKeyEvent(KeyEvent(now, now, KeyEvent.ACTION_UP, keyCode, 0))
    }
}

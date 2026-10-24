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
import androidx.compose.runtime.remember
import java.util.Base64

/**
 * The media keys of the keyboard, pressed through the system on Windows, which hands them to the player that is playing or
 * was playing last. Nowhere else can an app press them, so the other systems have no control.
 */
@Composable
internal actual fun rememberExternalPlayerControl(): ExternalPlayerControl? = remember {
    if (System.getProperty("os.name").orEmpty().startsWith("Windows")) WindowsPlayerControl else null
}

private object WindowsPlayerControl : ExternalPlayerControl {

    override fun playPause() = press(KEY_PLAY_PAUSE)

    override fun restart() = press(KEY_PREVIOUS_TRACK)

    /**
     * Through a PowerShell that calls `keybd_event`, since the JVM has no way to press a media key and the app carries no
     * native library for it. The script travels encoded, so that no quote of it is lost between the app and the command line.
     */
    private fun press(virtualKey: Int) {
        val script = "Add-Type -Namespace Campfire -Name Keys -MemberDefinition " +
            "'[DllImport(\"user32.dll\")] public static extern void keybd_event(byte key, byte scan, uint flags, UIntPtr extra);'; " +
            "[Campfire.Keys]::keybd_event($virtualKey, 0, 0, [UIntPtr]::Zero); " +
            "[Campfire.Keys]::keybd_event($virtualKey, 0, 2, [UIntPtr]::Zero)"
        val encoded = Base64.getEncoder().encodeToString(script.toByteArray(Charsets.UTF_16LE))
        runCatching {
            ProcessBuilder("powershell.exe", "-NoProfile", "-NonInteractive", "-WindowStyle", "Hidden", "-EncodedCommand", encoded)
                .redirectErrorStream(true)
                .start()
        }
    }

    private const val KEY_PREVIOUS_TRACK = 0xB1
    private const val KEY_PLAY_PAUSE = 0xB3
}

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
 * The player that is active on Windows, which hands the media keys and the playback position to the one that is playing
 * or was playing last. Nowhere else can an app reach it, so the other systems have no control.
 */
@Composable
internal actual fun rememberExternalPlayerControl(): ExternalPlayerControl? = remember {
    if (System.getProperty("os.name").orEmpty().startsWith("Windows")) WindowsPlayerControl else null
}

private object WindowsPlayerControl : ExternalPlayerControl {

    override fun playPause() = run(PLAY_PAUSE_SCRIPT)

    override fun restart(): Boolean {
        run(RESTART_SCRIPT)
        return true
    }

    /**
     * Through a PowerShell, since the JVM can neither press a media key nor ask the system for the player, and the app
     * carries no native library for it. The script travels encoded, so that no quote of it is lost on the way.
     */
    private fun run(script: String) {
        val encoded = Base64.getEncoder().encodeToString(script.replace(DOLLAR_PLACEHOLDER, '$').toByteArray(Charsets.UTF_16LE))
        runCatching {
            ProcessBuilder("powershell.exe", "-NoProfile", "-NonInteractive", "-WindowStyle", "Hidden", "-EncodedCommand", encoded)
                .redirectErrorStream(true)
                .start()
        }
    }

    /** Written with § where PowerShell has its $, which Kotlin would read as a template. */
    private const val DOLLAR_PLACEHOLDER = '§'

    private const val KEYS = """
        Add-Type -Namespace Campfire -Name Keys -MemberDefinition '[DllImport("user32.dll")] public static extern void keybd_event(byte key, byte scan, uint flags, UIntPtr extra);'
        function Press(§key) { [Campfire.Keys]::keybd_event(§key, 0, 0, [UIntPtr]::Zero); [Campfire.Keys]::keybd_event(§key, 0, 2, [UIntPtr]::Zero) }
    """

    private const val PLAY_PAUSE_SCRIPT = KEYS + "Press 0xB3"

    /**
     * Asks the session the system calls current for the start of its track and for it to play, which a browser or a player
     * that gives its position answers; where there is none, or it refuses, or the system has no such thing (before Windows 10),
     * the previous-track key is pressed, which is what the player does with it.
     */
    private const val RESTART_SCRIPT = KEYS + """
        try {
            Add-Type -AssemblyName System.Runtime.WindowsRuntime
            §asTask = [System.WindowsRuntimeSystemExtensions].GetMethods() | Where-Object { §_.Name -eq 'AsTask' -and §_.GetParameters().Count -eq 1 -and §_.GetParameters()[0].ParameterType.Name -eq 'IAsyncOperation`1' } | Select-Object -First 1
            function Await(§operation, §type) { §task = §asTask.MakeGenericMethod(§type).Invoke(§null, @(§operation)); §task.Wait(-1) | Out-Null; §task.Result }
            §managerType = [Windows.Media.Control.GlobalSystemMediaTransportControlsSessionManager, Windows.Media.Control, ContentType = WindowsRuntime]
            §manager = Await (§managerType::RequestAsync()) §managerType
            §session = §manager.GetCurrentSession()
            if (§session -eq §null) { throw 'No session' }
            if (-not (Await (§session.TryChangePlaybackPositionAsync(0)) ([bool]))) { throw 'Refused' }
            Await (§session.TryPlayAsync()) ([bool]) | Out-Null
        } catch { Press 0xB1 }
    """
}

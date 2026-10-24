// This file is part of Campfire.
// Copyright (c) Pandula Péter 2017-2026.
// https://github.com/pandulapeter/campfire
//
// This Source Code Form is subject to the terms of the Mozilla Public License, v. 2.0.
// If a copy of the MPL was not distributed with this file, You can obtain one at
// https://mozilla.org/MPL/2.0/.

//! The play and pause of whatever is playing music on the computer: the player the musician plays along with, next to
//! the song being read, which the key of a keyboard's media row works and so does this.

/// Presses and lets go the play/pause key of the media row, which Windows hands to the player that is playing or was
/// playing last. It reads nothing: the app learns neither what is playing nor whether anything is.
pub fn play_pause() {
    press(0xB3);
}

/// Takes the player back to the start of what it is playing, and plays it from there. The session Windows calls current is
/// asked for the position, which a browser or a player that gives it answers; where there is none, or it refuses, or the
/// system has no such thing (before Windows 10), the previous-track key is pressed, which is what the player does with it.
pub fn restart() {
    run_script(RESTART_SCRIPT);
}

#[cfg(windows)]
const RESTART_SCRIPT: &str = r#"
Add-Type -Namespace Campfire -Name Keys -MemberDefinition '[DllImport("user32.dll")] public static extern void keybd_event(byte key, byte scan, uint flags, UIntPtr extra);'
function Press($key) { [Campfire.Keys]::keybd_event($key, 0, 0, [UIntPtr]::Zero); [Campfire.Keys]::keybd_event($key, 0, 2, [UIntPtr]::Zero) }
try {
    Add-Type -AssemblyName System.Runtime.WindowsRuntime
    $asTask = [System.WindowsRuntimeSystemExtensions].GetMethods() | Where-Object { $_.Name -eq 'AsTask' -and $_.GetParameters().Count -eq 1 -and $_.GetParameters()[0].ParameterType.Name -eq 'IAsyncOperation`1' } | Select-Object -First 1
    function Await($operation, $type) { $task = $asTask.MakeGenericMethod($type).Invoke($null, @($operation)); $task.Wait(-1) | Out-Null; $task.Result }
    $managerType = [Windows.Media.Control.GlobalSystemMediaTransportControlsSessionManager, Windows.Media.Control, ContentType = WindowsRuntime]
    $manager = Await ($managerType::RequestAsync()) $managerType
    $session = $manager.GetCurrentSession()
    if ($session -eq $null) { throw 'No session' }
    if (-not (Await ($session.TryChangePlaybackPositionAsync(0)) ([bool]))) { throw 'Refused' }
    Await ($session.TryPlayAsync()) ([bool]) | Out-Null
} catch { Press 0xB1 }
"#;

/// Through a PowerShell that reads the script from its input, since the app has no way to the system's players of its own.
/// Without a window of its own, and never waited for: the answer of the player is not something the app reads.
#[cfg(windows)]
fn run_script(script: &str) {
    use std::io::Write;
    use std::os::windows::process::CommandExt;
    use std::process::{Command, Stdio};
    const CREATE_NO_WINDOW: u32 = 0x0800_0000;
    let Ok(mut child) = Command::new("powershell.exe")
        .args(["-NoProfile", "-NonInteractive", "-Command", "-"])
        .creation_flags(CREATE_NO_WINDOW)
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
    else {
        return;
    };
    if let Some(mut input) = child.stdin.take() {
        let _ = input.write_all(script.as_bytes());
    }
    // Reaped on a thread of its own, so that the process does not stay a zombie and the window does not wait for it.
    std::thread::spawn(move || {
        let _ = child.wait();
    });
}

#[cfg(windows)]
fn press(virtual_key: u8) {
    const KEYEVENTF_KEYUP: u32 = 0x0002;
    #[link(name = "user32")]
    extern "system" {
        fn keybd_event(key: u8, scan: u8, flags: u32, extra: usize);
    }
    // SAFETY: a call of the system with plain numbers, which holds on to nothing.
    unsafe {
        keybd_event(virtual_key, 0, 0, 0);
        keybd_event(virtual_key, 0, KEYEVENTF_KEYUP, 0);
    }
}

/// Nothing outside of Windows, where the app is not run.
#[cfg(not(windows))]
fn press(_virtual_key: u8) {}

#[cfg(not(windows))]
const RESTART_SCRIPT: &str = "";

#[cfg(not(windows))]
fn run_script(_script: &str) {}

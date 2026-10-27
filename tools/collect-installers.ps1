# This file is part of Campfire.
# Copyright (c) Pandula Péter 2017-2026.
# https://github.com/pandulapeter/campfire
#
# This Source Code Form is subject to the terms of the Mozilla Public License, v. 2.0.
# If a copy of the MPL was not distributed with this file, You can obtain one at
# https://mozilla.org/MPL/2.0/.

<#
.SYNOPSIS
Copies the installers that were just built into Install/, each named with the version it is, and removes the ones of any
other version, so that what is in the folder is always one build and says which.

.DESCRIPTION
The version is campfire.versionName of gradle.properties, which the Android build, the Windows installer and the Rust app
(the version of rust/Cargo.toml, kept the same) are all made with. Raise it for every build that is handed out: the Windows
installer refuses to install over the same version, and a file name that repeats says nothing about which build it is.

Build first: `./gradlew :app:android:assembleRelease :app:desktop:packageDistributionForCurrentOS`, then
`cargo build --release --target i686-pc-windows-msvc -p sdcancionero` and `rust/installer/build.ps1` from rust/.
#>
param(
    [string]$Destination = (Join-Path $PSScriptRoot '..\Install')
)

$ErrorActionPreference = 'Stop'
$root = Resolve-Path (Join-Path $PSScriptRoot '..')
$version = (Select-String -Path (Join-Path $root 'gradle.properties') -Pattern '^campfire\.versionName=(.+)$').Matches[0].Groups[1].Value.Trim()
$rustVersion = (Select-String -Path (Join-Path $root 'rust\Cargo.toml') -Pattern '^version = "(.+)"$').Matches[0].Groups[1].Value.Trim()
if ($rustVersion -ne $version) {
    throw "The Rust app is $rustVersion and the rest is ${version}: set the version of rust/Cargo.toml to ${version} and build it again."
}

$files = [ordered]@{
    "SDCancionero-$version-android.apk"      = 'app\android\build\outputs\apk\release\android-release.apk'
    "SDCancionero-$version-x64.msi"          = "app\desktop\build\compose\binaries\main\msi\SDCancionero-$version.msi"
    "SDCancionero-$version-32bits.exe"       = 'rust\target\i686-pc-windows-msvc\release\sdcancionero.exe'
    "SDCancionero-$version-32bits-setup.exe" = "rust\dist\SDCancionero-Rust-Setup-$version.exe"
}

New-Item -ItemType Directory -Force $Destination | Out-Null
Get-ChildItem $Destination -File | Remove-Item -Force
foreach ($name in $files.Keys) {
    $source = Join-Path $root $files[$name]
    if (-not (Test-Path $source)) { throw "Missing ${source}: it has not been built for ${version}." }
    Copy-Item $source (Join-Path $Destination $name)
}
Get-ChildItem $Destination | Select-Object Name, Length

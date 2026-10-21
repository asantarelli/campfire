# This file is part of Campfire.
# Copyright (c) Pandula Péter 2017-2026.
# https://github.com/pandulapeter/campfire
#
# This Source Code Form is subject to the terms of the Mozilla Public License, v. 2.0.
# If a copy of the MPL was not distributed with this file, You can obtain one at
# https://mozilla.org/MPL/2.0/.

<#
.SYNOPSIS
Builds the Rust build for Windows computers of 32 bits (unless -SkipBuild) and packs it with NSIS into
SDCancionero-Rust-Setup-<version>.exe in -OutputDirectory.

.PARAMETER OutputDirectory
Where the installer is written. By default rust/dist.

.PARAMETER SkipBuild
Packs the executable that is already built instead of building it again.
#>
param(
    [string]$OutputDirectory = (Join-Path $PSScriptRoot '..\dist'),
    [switch]$SkipBuild
)

$ErrorActionPreference = 'Stop'

$workspace = Resolve-Path (Join-Path $PSScriptRoot '..')
$exe = Join-Path $workspace 'target-installer\i686-pc-windows-msvc\release\sdcancionero.exe'

if (-not $SkipBuild) {
    $env:Path = "$env:USERPROFILE\.cargo\bin;" + $env:Path
    Push-Location $workspace
    try {
        cargo build --release --target i686-pc-windows-msvc -p sdcancionero --target-dir target-installer
        if ($LASTEXITCODE -ne 0) { throw "cargo terminó con el código $LASTEXITCODE" }
    } finally {
        Pop-Location
    }
}
if (-not (Test-Path $exe)) { throw "No hay una compilación en '$exe'." }

$makensis = @(
    (Get-Command makensis.exe -ErrorAction SilentlyContinue).Source,
    'C:\Program Files (x86)\NSIS\makensis.exe',
    'C:\Program Files\NSIS\makensis.exe'
) | Where-Object { $_ -and (Test-Path $_) } | Select-Object -First 1
if (-not $makensis) { throw 'No se encontró NSIS (makensis.exe).' }

$version = (Select-String -Path (Join-Path $workspace 'Cargo.toml') -Pattern '^version = "(.+)"$').Matches[0].Groups[1].Value.Trim()
New-Item -ItemType Directory -Force $OutputDirectory | Out-Null
$output = Join-Path (Resolve-Path $OutputDirectory) "SDCancionero-Rust-Setup-$version.exe"
$icon = Join-Path $workspace 'sdcancionero\assets\icon.ico'

& $makensis /V2 "/DVERSION=$version" "/DEXE=$exe" "/DICON=$icon" "/DOUTFILE=$output" (Join-Path $PSScriptRoot 'installer.nsi')
if ($LASTEXITCODE -ne 0) { throw "makensis terminó con el código $LASTEXITCODE" }
Write-Host ('Listo: {0} ({1:N1} MB)' -f $output, ((Get-Item $output).Length / 1MB))

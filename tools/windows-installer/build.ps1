# This file is part of Campfire.
# Copyright (c) Pandula Péter 2017-2026.
# https://github.com/pandulapeter/campfire
#
# This Source Code Form is subject to the terms of the Mozilla Public License, v. 2.0.
# If a copy of the MPL was not distributed with this file, You can obtain one at
# https://mozilla.org/MPL/2.0/.

<#
.SYNOPSIS
Builds the installer of the Windows desktop build with NSIS, from the folder `./gradlew :app:desktop:createDistributable`
writes, and leaves it as SDCancionero-Setup-<version>.exe in -OutputDirectory.

.PARAMETER OutputDirectory
Where the installer is written. By default the folder above the repository.
#>
param(
    [string]$OutputDirectory = (Join-Path $PSScriptRoot '..\..\..')
)

$ErrorActionPreference = 'Stop'

$repository = Resolve-Path (Join-Path $PSScriptRoot '..\..')
$source = Join-Path $repository 'app\desktop\build\compose\binaries\main\app\SDCancionero'
if (-not (Test-Path (Join-Path $source 'SDCancionero.exe'))) {
    throw "No hay una compilación en '$source'. Primero: ./gradlew :app:desktop:createDistributable"
}

$makensis = @(
    (Get-Command makensis.exe -ErrorAction SilentlyContinue).Source,
    'C:\Program Files (x86)\NSIS\makensis.exe',
    'C:\Program Files\NSIS\makensis.exe'
) | Where-Object { $_ -and (Test-Path $_) } | Select-Object -First 1
if (-not $makensis) { throw 'No se encontró NSIS (makensis.exe).' }

$version = (Select-String -Path (Join-Path $repository 'gradle.properties') -Pattern '^campfire\.versionName=(.+)$').Matches[0].Groups[1].Value.Trim()
$output = Join-Path (Resolve-Path $OutputDirectory) "SDCancionero-Setup-$version.exe"
$icon = Join-Path $repository 'app\icons\appIcon.ico'

& $makensis /V2 "/DVERSION=$version" "/DSOURCE_DIR=$source" "/DICON=$icon" "/DOUTFILE=$output" (Join-Path $PSScriptRoot 'installer.nsi')
if ($LASTEXITCODE -ne 0) { throw "makensis terminó con el código $LASTEXITCODE" }
Write-Host ('Listo: {0} ({1:N1} MB)' -f $output, ((Get-Item $output).Length / 1MB))

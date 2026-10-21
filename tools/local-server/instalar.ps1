# This file is part of Campfire.
# Copyright (c) Pandula Péter 2017-2026.
# https://github.com/pandulapeter/campfire
#
# This Source Code Form is subject to the terms of the Mozilla Public License, v. 2.0.
# If a copy of the MPL was not distributed with this file, You can obtain one at
# https://mozilla.org/MPL/2.0/.

<#
.SYNOPSIS
Makes the computer serve SDCancionero by itself from now on: a shortcut in the Startup folder starts serve.ps1, hidden,
every time this user signs in, and it is started now as well.
#>
param(
    [int]$Port = 8080
)

$ErrorActionPreference = 'Stop'

$server = Join-Path $PSScriptRoot 'serve.ps1'
$startup = [Environment]::GetFolderPath('Startup')
$shortcutPath = Join-Path $startup 'SDCancionero (servidor).lnk'
$arguments = "-NoProfile -ExecutionPolicy Bypass -WindowStyle Hidden -File `"$server`" -Port $Port"

$shell = New-Object -ComObject WScript.Shell
$shortcut = $shell.CreateShortcut($shortcutPath)
$shortcut.TargetPath = (Join-Path $env:SystemRoot 'System32\WindowsPowerShell\v1.0\powershell.exe')
$shortcut.Arguments = $arguments
$shortcut.WorkingDirectory = $PSScriptRoot
$shortcut.WindowStyle = 7
$shortcut.Description = 'Sirve SDCancionero en http://localhost:' + $Port
$shortcut.Save()

Start-Process -FilePath $shortcut.TargetPath -ArgumentList $arguments -WindowStyle Hidden
Write-Host "Listo. SDCancionero se va a servir solo en http://localhost:$Port/ cada vez que inicies sesión."
Write-Host "Ábrelo en Vivaldi y usa 'Instalar SDCancionero' del menú de la página si querés un acceso como aplicación."

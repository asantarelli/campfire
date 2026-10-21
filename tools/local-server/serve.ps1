# This file is part of Campfire.
# Copyright (c) Pandula Péter 2017-2026.
# https://github.com/pandulapeter/campfire
#
# This Source Code Form is subject to the terms of the Mozilla Public License, v. 2.0.
# If a copy of the MPL was not distributed with this file, You can obtain one at
# https://mozilla.org/MPL/2.0/.

<#
.SYNOPSIS
Serves the web build of SDCancionero on this computer, with nothing to install: it needs only the PowerShell that every
Windows 10 ships, 32-bit included.

.DESCRIPTION
The page has to be opened from http://localhost, which browsers treat as a secure context: the library lives in the
browser's private file system, and that and installing the page as an app both need one. A file opened with file:// reads
no library at all.

An address that is not a file (/songs, /settings/library) is answered the way GitHub Pages' 404 page does, by sending
the browser to /?/songs. A plain "serve index.html everywhere" would break the page: it reads that form to learn which
screen to open on and which folder every relative address is resolved against (see app/web/CLAUDE.md).

.PARAMETER Root
The folder with index.html, which is the build's productionExecutable folder. By default the "app" folder beside this script.

.PARAMETER Port
The port on localhost. It is part of the app's address, and so of where the library is kept: changing it later starts an
empty library.
#>
param(
    [string]$Root = (Join-Path $PSScriptRoot 'app'),
    [int]$Port = 8080
)

$ErrorActionPreference = 'Stop'

$root = [System.IO.Path]::GetFullPath($Root).TrimEnd('\')
if (-not (Test-Path (Join-Path $root 'index.html'))) {
    throw "No hay un index.html en '$root'. Copia ahí el contenido de la carpeta productionExecutable de la compilación web."
}

# The type the browser needs for every file of the build. A .wasm without its own type is not compiled while it downloads.
$types = @{
    '.html' = 'text/html; charset=utf-8'
    '.js' = 'text/javascript; charset=utf-8'
    '.mjs' = 'text/javascript; charset=utf-8'
    '.css' = 'text/css; charset=utf-8'
    '.json' = 'application/json; charset=utf-8'
    '.wasm' = 'application/wasm'
    '.txt' = 'text/plain; charset=utf-8'
    '.xml' = 'application/xml; charset=utf-8'
    '.cho' = 'text/plain; charset=utf-8'
    '.png' = 'image/png'
    '.webp' = 'image/webp'
    '.jpg' = 'image/jpeg'
    '.svg' = 'image/svg+xml'
    '.ico' = 'image/x-icon'
    '.ttf' = 'font/ttf'
    '.otf' = 'font/otf'
    '.woff' = 'font/woff'
    '.woff2' = 'font/woff2'
}

# What has to be asked again every time, so that a new build is noticed as soon as the page is opened.
$alwaysFresh = @('index.html', 'service-worker.js', 'build.json')

$listener = New-Object System.Net.HttpListener
$listener.Prefixes.Add("http://localhost:$Port/")
$listener.Prefixes.Add("http://127.0.0.1:$Port/")
try {
    $listener.Start()
} catch {
    # Already running (a second copy was started at login, or by hand): the first one keeps serving.
    Write-Host "El puerto $Port ya está en uso, así que SDCancionero ya se está sirviendo."
    exit 0
}
Write-Host "SDCancionero en http://localhost:$Port/  (carpeta: $root)"

function Send-Redirect($response, [string]$location) {
    $response.StatusCode = 302
    $response.RedirectLocation = $location
    $response.Headers.Add('Cache-Control', 'no-store')
}

while ($listener.IsListening) {
    $context = $listener.GetContext()
    $response = $context.Response
    try {
        $request = $context.Request
        $path = [System.Uri]::UnescapeDataString($request.Url.AbsolutePath)
        if ($path -eq '/' -or $path -eq '') { $path = '/index.html' }

        $candidate = [System.IO.Path]::GetFullPath((Join-Path $root $path.TrimStart('/').Replace('/', '\')))
        $isInside = $candidate.StartsWith($root + '\', [System.StringComparison]::OrdinalIgnoreCase)

        if ($isInside -and (Test-Path -LiteralPath $candidate -PathType Leaf)) {
            $extension = [System.IO.Path]::GetExtension($candidate).ToLowerInvariant()
            $response.ContentType = if ($types.ContainsKey($extension)) { $types[$extension] } else { 'application/octet-stream' }
            if ($alwaysFresh -contains [System.IO.Path]::GetFileName($candidate)) {
                $response.Headers.Add('Cache-Control', 'no-cache')
            }
            $stream = [System.IO.File]::OpenRead($candidate)
            try {
                $response.ContentLength64 = $stream.Length
                $stream.CopyTo($response.OutputStream)
            } finally {
                $stream.Dispose()
            }
        } elseif ($path.Contains('.') -and [System.IO.Path]::GetExtension($path) -ne '') {
            # A file the build does not have: a real 404, not a screen of the app.
            $response.StatusCode = 404
        } else {
            # A screen of the app: /songs?x=1&y=2 goes to /?/songs&x=1~and~y=2, which is the form the page reads.
            $screen = $path.TrimStart('/').Replace('&', '~and~')
            $query = $request.Url.Query.TrimStart('?').Replace('&', '~and~')
            $target = "/?/$screen"
            if ($query -ne '') { $target += "&$query" }
            Send-Redirect $response $target
        }
    } catch {
        # A browser that gave up on a download (a navigation, a closed tab) is not a failure of the server.
        try { $response.StatusCode = 500 } catch { }
    } finally {
        try { $response.Close() } catch { }
    }
}

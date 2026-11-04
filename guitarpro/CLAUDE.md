<!--
 * This file is part of Campfire.
 * Copyright (c) Pandula Péter 2017-2026.
 * https://github.com/pandulapeter/campfire
 *
 * This Source Code Form is subject to the terms of the Mozilla Public License, v. 2.0.
 * If a copy of the MPL was not distributed with this file, You can obtain one at
 * https://mozilla.org/MPL/2.0/.
-->
# :guitarpro

Guitar Pro 3, 4 and 5 files (`.gp3`, `.gp4`, `.gp5`) read into a score and set on a page as marks to paint. **No
dependencies at all**, explicit API mode, stateless `object`s called by `:presentation`. Pure and tested; the Rust twin in
`rust/guitarpro` is the same code in another language, so a change to one is made in the other.

## `reader/`

- A file is one long run of little-endian numbers, flags and strings with no index: the reader knows the layout of every
  piece to find where the next begins, so what a score has no use for (the sound, the page, the mixer) is skipped, never
  kept. A layout it gets wrong shows as `UNEXPECTED_END` or a wrong count far from the mistake.
- **The extension says nothing about the version**: editors save a Guitar Pro 5 file as `.gp3`; the first string names the
  version. Guitar Pro 5 writes the strum's two speeds the other way round and keeps the key and the endings elsewhere;
  every such branch is `major`/`isAfter5Dot0`.
- **The last measure of a Guitar Pro 5 file may lack its closing byte** (`ByteCursor.skipIfAny`), which Guitar Pro reads
  past; a stricter reader refuses real files.
- Text is UTF-8 where it reads as that, else Windows-1252.
- Every count is bounded (`MAX_COUNT`), so a damaged file is refused rather than allocated. Everything that goes wrong is a
  `GuitarProException` with a `Reason`, never a message: the screen words it.
- Fixtures are written by PyGuitarPro (`pip install pyguitarpro`, used only to write them and as the oracle the Rust
  tests compare against; no code of it is here). Only the desktop source set reads files.

## `song/`

- **A score is kept as a ChordPro song** (`ScoreSongText`): the file in Base64 in `{meta: score-data …}` lines, a marker
  `{meta: score guitarpro}`, and the `{title}` and `{artist}` of the score. Every other ChordPro program keeps custom
  metadata and ignores it, and the library's parser, prettifier and header edits keep it (a test in `:presentation` holds
  that), so nothing of the library knows about scores but the import and the screen that opens one.
- The title and the artist are the song's, which the player changes in About the song; the file inside is never changed.

## `layout/` and `draw/`

- `ScoreLayouter` answers a list of `Draw`s in **staff spaces** (the distance between two lines), one `ScoreSystem` per line
  of the page; nothing names a color, a font or a size, so a window and a page of paper paint the same layout. A `Tone`
  says how much a mark matters.
- Signs are strokes and ellipses (`Glyphs`), not a font: nothing to bundle and no license. The staff is always a treble
  clef with an 8 under it, since a guitar sounds an octave below what it is written.
- **A chord is its diagram, which a file may hold without a name** (an editor writes the diagram and leaves the name
  empty): the reader keeps both (`Beat.chordShape`), and a system that has a diagram takes a band more above its staff
  (`DIAGRAM_BAND`). Frets are kept as the neck counts them, or from the diagram's first fret; `chordDiagram` tells which.
- A tie, a slur and a slide join notes of different measures, so one `TabPainter` lives for the whole track.
- Measures are stretched alike to the width of the system; the last system is stretched by 1.4 at most, so a short last line
  does not spread across the page.

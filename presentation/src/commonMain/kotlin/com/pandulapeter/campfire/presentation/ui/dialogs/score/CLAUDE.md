<!--
 * This file is part of Campfire.
 * Copyright (c) Pandula Péter 2017-2026.
 * https://github.com/pandulapeter/campfire
 *
 * This Source Code Form is subject to the terms of the Mozilla Public License, v. 2.0.
 * If a copy of the MPL was not distributed with this file, You can obtain one at
 * https://mozilla.org/MPL/2.0/.
-->
# :presentation — ui/dialogs/score

- **A Guitar Pro score is a song like any other in the library.** The import turns a `.gp3`/`.gp4`/`.gp5` into a `.cho`
  that holds the file in `{meta}` lines under a title and an artist (`ScoreSongText`, in
  `PrepareImportUseCaseImpl`), so naming, collisions, tags, export and sync need nothing of their own. `Song.isScore`
  says it, and the song list marks every song with a letter in front of its title: G for a score, C for a chord sheet
  (`SongKindBadge`).
- **Opening a score song shows this sheet instead of a page** (`Navigator.openSongDetails` and `openImportedSong`, whatever
  reached them: the list, a setlist, an import's snackbar). `ScoreOpener` reads the file out of the song's text; a song
  that says it is a score but holds none that reads is a snackbar (`Message.ScoreUnreadable`), never an empty page. The
  sheet is read-only, and it is titled with the song's own title and artist.
- A score has no text to type into, so its menu entry is About the song instead of Edit song file: the title and the artist
  are what the player changes, written into the file's lines like any song's. Scores are edited in a score editor and
  imported again.
- The score is read when the sheet composes (`readScore`, which never throws: a failure is a reason, worded from strings).
  No `SavedDialog`: a process Android restores does not reopen it.
- The page is one canvas to a line (`ScorePage`), so a long score draws only what is in sight; the layout is set again
  when the sheet's width or what is shown changes, and a change of what is shown crossfades (a resize does not: it
  happens on every frame).
- The size is whole dp steps (`ScoreSizes`), each a new layout. One of the staff and the tablature is always shown; the
  rhythm under the tablature goes with it.
- Colors follow `ScorePalette`: ink, muted and the theme's primary for markers, chords and tempo; the text on a string of
  the tablature masks the line behind it with the sheet's own color.

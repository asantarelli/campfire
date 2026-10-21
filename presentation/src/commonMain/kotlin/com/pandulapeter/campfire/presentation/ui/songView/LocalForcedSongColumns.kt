/*
 * This file is part of Campfire.
 * Copyright (c) Pandula Péter 2017-2026.
 * https://github.com/pandulapeter/campfire
 *
 * This Source Code Form is subject to the terms of the Mozilla Public License, v. 2.0.
 * If a copy of the MPL was not distributed with this file, You can obtain one at
 * https://mozilla.org/MPL/2.0/.
 */
package com.pandulapeter.campfire.presentation.ui.songView

import androidx.compose.runtime.compositionLocalOf
import com.pandulapeter.campfire.data.model.preferences.UserPreferences

/**
 * How many columns the reader asked every song to be set in, [UserPreferences.AUTOMATIC_SONG_COLUMNS] where they left
 * that to the width of the window. Provided once for the whole app rather than handed down the song's tree, since it
 * is a preference of the reader's that only the grid search reads; a song that is set in one column whatever the width
 * (the editor's preview) ignores it.
 */
internal val LocalForcedSongColumns = compositionLocalOf { UserPreferences.AUTOMATIC_SONG_COLUMNS }

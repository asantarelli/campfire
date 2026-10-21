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
 * Where the reader asked the diagrams of a song's chords to stand: among the song's sections, or pinned at the top or on
 * the right of the page. Provided once for the whole app, like [LocalForcedSongColumns], since it is a preference only
 * the song details screen reads.
 */
internal val LocalPinnedChordsPosition = compositionLocalOf { UserPreferences.PinnedChordsPosition.IN_SONG }

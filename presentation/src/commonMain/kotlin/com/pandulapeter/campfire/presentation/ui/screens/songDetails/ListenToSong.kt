/*
 * This file is part of Campfire.
 * Copyright (c) Pandula Péter 2017-2026.
 * https://github.com/pandulapeter/campfire
 *
 * This Source Code Form is subject to the terms of the Mozilla Public License, v. 2.0.
 * If a copy of the MPL was not distributed with this file, You can obtain one at
 * https://mozilla.org/MPL/2.0/.
 */
package com.pandulapeter.campfire.presentation.ui.screens.songDetails

import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import com.pandulapeter.campfire.data.model.preferences.UserPreferences
import com.pandulapeter.campfire.data.model.song.Song
import com.pandulapeter.campfire.presentation.localization.stringResource
import com.pandulapeter.campfire.presentation.resources.Res
import com.pandulapeter.campfire.presentation.resources.ic_listen
import com.pandulapeter.campfire.presentation.resources.music_search_spotify
import com.pandulapeter.campfire.presentation.resources.music_search_youtube
import com.pandulapeter.campfire.presentation.resources.music_search_youtube_music
import com.pandulapeter.campfire.presentation.resources.song_details_listen
import com.pandulapeter.campfire.presentation.ui.components.menu.ActionsMenuItem
import org.jetbrains.compose.resources.painterResource

/**
 * The button that looks the song up to listen to it, which is what a musician reaches for to learn it or to check a
 * tempo: it opens the search of the player's service in the browser, with the artist and the title already typed.
 */
@Composable
internal fun ListenToSongButton(
    modifier: Modifier = Modifier,
    song: Song,
    service: UserPreferences.MusicSearchService,
    urlOpener: (String) -> Unit,
) = IconButton(
    modifier = modifier,
    onClick = { urlOpener(musicSearchUrl(service, title = song.title, artist = song.artist)) },
) {
    Icon(
        painter = painterResource(Res.drawable.ic_listen),
        contentDescription = stringResource(Res.string.song_details_listen, musicSearchServiceName(service)),
    )
}

/** The same thing as an entry of the song's menu, where the bar has no room for the button. */
@Composable
internal fun listenToSongAction(
    song: Song,
    service: UserPreferences.MusicSearchService,
    urlOpener: (String) -> Unit,
) = ActionsMenuItem(
    title = stringResource(Res.string.song_details_listen, musicSearchServiceName(service)),
    icon = painterResource(Res.drawable.ic_listen),
    key = LISTEN_ACTION_KEY,
    onClick = { urlOpener(musicSearchUrl(service, title = song.title, artist = song.artist)) },
)

/** What the service is called, which is the same in every language. */
@Composable
private fun musicSearchServiceName(service: UserPreferences.MusicSearchService) = stringResource(
    when (service) {
        UserPreferences.MusicSearchService.YOUTUBE -> Res.string.music_search_youtube
        UserPreferences.MusicSearchService.YOUTUBE_MUSIC -> Res.string.music_search_youtube_music
        UserPreferences.MusicSearchService.SPOTIFY -> Res.string.music_search_spotify
    },
)

private const val LISTEN_ACTION_KEY = "listen"

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

import com.pandulapeter.campfire.data.model.preferences.UserPreferences
import kotlin.test.Test
import kotlin.test.assertEquals

class MusicSearchTest {

    @Test
    fun `a search names the artist first and then the title`() {
        assertEquals(
            "https://www.youtube.com/results?search_query=Traditional%20American%20House%20of%20the%20Rising%20Sun",
            musicSearchUrl(UserPreferences.MusicSearchService.YOUTUBE, title = "House of the Rising Sun", artist = "Traditional American"),
        )
    }

    @Test
    fun `a song without an artist is searched by its title alone`() {
        assertEquals(
            "https://music.youtube.com/search?q=Yesterday",
            musicSearchUrl(UserPreferences.MusicSearchService.YOUTUBE_MUSIC, title = "Yesterday", artist = "  "),
        )
    }

    @Test
    fun `accents and symbols are written as the bytes of their UTF-8`() {
        assertEquals(
            "https://open.spotify.com/search/Caf%C3%A9%20%26%20Co%C3%B1ac",
            musicSearchUrl(UserPreferences.MusicSearchService.SPOTIFY, title = "Café & Coñac", artist = ""),
        )
    }

    @Test
    fun `characters that need no escaping are left as they are`() {
        assertEquals(
            "https://www.youtube.com/results?search_query=A-b_c.d~e1",
            musicSearchUrl(UserPreferences.MusicSearchService.YOUTUBE, title = "A-b_c.d~e1", artist = ""),
        )
    }
}

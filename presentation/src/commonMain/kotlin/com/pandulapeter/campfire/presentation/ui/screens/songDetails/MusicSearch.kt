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

/**
 * The address that searches [service] for the song, which the platform's browser then opens: the app itself asks the
 * service for nothing, it only writes the question. The artist comes first, the way people type it, and is left out
 * where the song has none.
 */
internal fun musicSearchUrl(service: UserPreferences.MusicSearchService, title: String, artist: String): String {
    val terms = percentEncoded(listOf(artist, title).map { it.trim() }.filter { it.isNotEmpty() }.joinToString(" "))
    return when (service) {
        UserPreferences.MusicSearchService.YOUTUBE -> "https://www.youtube.com/results?search_query=$terms"
        UserPreferences.MusicSearchService.YOUTUBE_MUSIC -> "https://music.youtube.com/search?q=$terms"
        UserPreferences.MusicSearchService.SPOTIFY -> "https://open.spotify.com/search/$terms"
    }
}

/** [text] as a part of an address: every byte of its UTF-8 that is not an unreserved character written as `%XX`. */
private fun percentEncoded(text: String): String = buildString {
    text.encodeToByteArray().forEach { byte ->
        val value = byte.toInt() and BYTE_MASK
        val character = value.toChar()
        if (character in 'A'..'Z' || character in 'a'..'z' || character in '0'..'9' || character in UNRESERVED_MARKS) {
            append(character)
        } else {
            append('%')
            append(HEX_DIGITS[value shr HEX_SHIFT])
            append(HEX_DIGITS[value and HEX_MASK])
        }
    }
}

private const val UNRESERVED_MARKS = "-._~"
private const val HEX_DIGITS = "0123456789ABCDEF"
private const val BYTE_MASK = 0xFF
private const val HEX_SHIFT = 4
private const val HEX_MASK = 0x0F

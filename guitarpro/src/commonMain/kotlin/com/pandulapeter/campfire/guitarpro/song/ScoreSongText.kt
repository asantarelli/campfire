/*
 * This file is part of Campfire.
 * Copyright (c) Pandula Péter 2017-2026.
 * https://github.com/pandulapeter/campfire
 *
 * This Source Code Form is subject to the terms of the Mozilla Public License, v. 2.0.
 * If a copy of the MPL was not distributed with this file, You can obtain one at
 * https://mozilla.org/MPL/2.0/.
 */
package com.pandulapeter.campfire.guitarpro.song

import com.pandulapeter.campfire.guitarpro.reader.GuitarProReader
import kotlin.io.encoding.Base64
import kotlin.io.encoding.ExperimentalEncodingApi

/**
 * A Guitar Pro score as a ChordPro song, which is how a library holds one: the file itself, in `{meta}` lines, under a title and an
 * artist the player can change like any song's. Every other ChordPro program keeps custom metadata and ignores it, and everything
 * the library does to a song file (naming, tags, export, sync) then works on a score with nothing of its own.
 *
 * ```
 * {title: Exercises}
 * {artist: Somebody}
 * {meta: score guitarpro}
 * {meta: score-data UEsDBBQ...}
 * ```
 *
 * The data is the file in Base64, a line to every [LINE_LENGTH] characters so that no line is long enough to trouble an editor.
 */
@OptIn(ExperimentalEncodingApi::class)
public object ScoreSongText {

    /** The `{meta}` key of the marker line, which says the song is a score; its value names the format. */
    public const val MARKER_NAME: String = "score"

    /** The `{meta}` key of every line of the file's data. */
    public const val DATA_NAME: String = "score-data"

    private const val FORMAT = "guitarpro"
    private const val LINE_LENGTH = 76

    /**
     * The song text holding the score of [bytes], titled by the file and credited to the artist it names, or [fallbackTitle] for a
     * title it names none; null where the bytes are not a score that can be read, which is not worth keeping.
     */
    public fun of(bytes: ByteArray, fallbackTitle: String): String? {
        val score = try {
            GuitarProReader.read(bytes)
        } catch (_: Exception) {
            return null
        }
        return encode(title = score.title.ifBlank { fallbackTitle }, artist = score.artist, bytes = bytes)
    }

    /** The song text of [bytes] under [title] and [artist], either of which may be blank, whatever the bytes are. */
    public fun encode(title: String, artist: String, bytes: ByteArray): String {
        val cleanTitle = title.directiveText()
        val cleanArtist = artist.directiveText()
        return buildString {
            if (cleanTitle.isNotEmpty()) append("{title: ").append(cleanTitle).append("}\n")
            if (cleanArtist.isNotEmpty()) append("{artist: ").append(cleanArtist).append("}\n")
            append("{meta: ").append(MARKER_NAME).append(' ').append(FORMAT).append("}\n")
            Base64.Default.encode(bytes).chunked(LINE_LENGTH).forEach { append("{meta: ").append(DATA_NAME).append(' ').append(it).append("}\n") }
        }
    }

    /** The file a song text holds, from the custom metadata of its header (`ChordProMetadata.custom`); null for a song that holds none. */
    public fun bytesOf(custom: Map<String, List<String>>): ByteArray? {
        if (custom[MARKER_NAME].isNullOrEmpty()) return null
        val data = custom[DATA_NAME]?.joinToString("") { it.trim() }?.takeIf { it.isNotEmpty() } ?: return null
        return try {
            Base64.Default.decode(data)
        } catch (_: IllegalArgumentException) {
            null
        }
    }

    /** Whether the custom metadata of a song says it is a score. */
    public fun isScore(custom: Map<String, List<String>>): Boolean = !custom[MARKER_NAME].isNullOrEmpty()

    /** A value a directive can hold: on one line, and with nothing that would close the directive early. */
    private fun String.directiveText() = map { if (it == '{' || it == '}' || it.isISOControl()) ' ' else it }.joinToString("").trim()
}

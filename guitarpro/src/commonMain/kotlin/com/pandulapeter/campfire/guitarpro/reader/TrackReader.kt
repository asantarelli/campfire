/*
 * This file is part of Campfire.
 * Copyright (c) Pandula Péter 2017-2026.
 * https://github.com/pandulapeter/campfire
 *
 * This Source Code Form is subject to the terms of the Mozilla Public License, v. 2.0.
 * If a copy of the MPL was not distributed with this file, You can obtain one at
 * https://mozilla.org/MPL/2.0/.
 */
package com.pandulapeter.campfire.guitarpro.reader

import com.pandulapeter.campfire.guitarpro.model.Track

/** The description of a track: its name, its tuning and its capo, without its measures. */
internal class TrackReader(
    private val cursor: ByteCursor,
    private val version: GuitarProVersion,
) {

    fun track(number: Int): Track {
        val isVersion5 = version.major == 5
        if (isVersion5 && (number == 1 || !version.isAfter5Dot0)) {
            cursor.skip(1)
        }
        val flags = cursor.u8()
        val name = cursor.byteSizeString(40)
        val stringCount = cursor.count().coerceAtMost(MAX_STRINGS)
        // The tuning of seven strings is always written, whatever the instrument has.
        val tunings = List(MAX_STRINGS) { cursor.i32() }
        cursor.skip(4) // The port.
        val channel = cursor.i32()
        cursor.skip(4) // The channel of the effects.
        cursor.skip(4) // How many frets the instrument has.
        val capo = cursor.i32()
        cursor.skip(4) // Its color.
        if (isVersion5) {
            cursor.skip(2 + 1 + 1 + 1 + 4 + 4 + 4 + 12)
            cursor.skip(4 + 4 + 4) // The instrument of the sound engine.
            cursor.skip(if (version.isAfter5Dot0) 4 else 3)
            if (version.isAfter5Dot0) {
                cursor.skip(4) // Its equalizer.
                cursor.intByteSizeString()
                cursor.intByteSizeString()
            }
        }
        return Track(
            name = name,
            strings = tunings.take(stringCount),
            capo = capo,
            isPercussion = flags and 0x01 != 0 || channel == PERCUSSION_CHANNEL,
            measures = emptyList(),
        )
    }

    private companion object {
        const val MAX_STRINGS = 7
        const val PERCUSSION_CHANNEL = 10
    }
}

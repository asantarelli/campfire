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

import com.pandulapeter.campfire.guitarpro.model.Score
import com.pandulapeter.campfire.guitarpro.reader.GuitarProException.Reason

/**
 * Reads the files Guitar Pro 3, 4 and 5 write (`.gp3`, `.gp4`, `.gp5`). The pieces a score has no use for (the sound, the page, the
 * mixer) are walked past rather than kept; the extension of a file says nothing, since other editors save a Guitar Pro 5 file as `.gp3`.
 */
public object GuitarProReader {

    /** @throws GuitarProException where the bytes are not a score this reader can read. */
    public fun read(bytes: ByteArray): Score {
        val cursor = ByteCursor(bytes)
        // A file too short to name a version is not one of Guitar Pro's.
        val version = try {
            GuitarProVersion.of(cursor.byteSizeString(30))
        } catch (exception: GuitarProException) {
            throw if (exception.reason == Reason.UNEXPECTED_END) GuitarProException(Reason.NOT_GUITAR_PRO) else exception
        }
        return ScoreReader(cursor, version).read()
    }
}

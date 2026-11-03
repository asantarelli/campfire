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

import com.pandulapeter.campfire.guitarpro.reader.GuitarProException.Reason

/** The version of a file, as `5.10` is (5, 10). */
internal data class GuitarProVersion(val major: Int, val minor: Int) {

    val isAfter5Dot0 get() = major == 5 && minor > 0

    companion object {

        private const val NAME = "FICHIER GUITAR PRO"

        /** The version a file's first string names: `FICHIER GUITAR PRO v5.10` is 5.10. */
        fun of(text: String): GuitarProVersion {
            val name = text.trim()
            if (!name.startsWith(NAME)) throw GuitarProException(Reason.NOT_GUITAR_PRO)
            val digits = name.substringAfterLast('v', missingDelimiterValue = "")
            val (major, minor) = digits.split('.').takeIf { it.size == 2 }?.map { it.trim().toIntOrNull() }
                ?: throw GuitarProException(Reason.NOT_GUITAR_PRO)
            if (major == null || minor == null) throw GuitarProException(Reason.NOT_GUITAR_PRO)
            if (major !in 3..5) throw GuitarProException(Reason.UNSUPPORTED_VERSION)
            return GuitarProVersion(major, minor)
        }
    }
}

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

/** A file that is not a score the reader can read; [reason] says why, for whoever words it for the player. */
public class GuitarProException(public val reason: Reason) : Exception(reason.name) {

    public enum class Reason {
        /** The file does not start with the name of a version of Guitar Pro. */
        NOT_GUITAR_PRO,

        /** A version of Guitar Pro the reader does not know how to read. */
        UNSUPPORTED_VERSION,

        /** The file ends before the score does. */
        UNEXPECTED_END,

        /** A number in the file that no score has. */
        CORRUPT,
    }
}

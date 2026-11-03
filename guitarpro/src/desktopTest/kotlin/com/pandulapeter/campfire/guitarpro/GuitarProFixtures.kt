/*
 * This file is part of Campfire.
 * Copyright (c) Pandula Péter 2017-2026.
 * https://github.com/pandulapeter/campfire
 *
 * This Source Code Form is subject to the terms of the Mozilla Public License, v. 2.0.
 * If a copy of the MPL was not distributed with this file, You can obtain one at
 * https://mozilla.org/MPL/2.0/.
 */
package com.pandulapeter.campfire.guitarpro

/**
 * The same short song written by PyGuitarPro in the four versions the reader knows, the only files of the module's tests: common
 * code cannot open a file, so the tests that read them live in the desktop source set. The Rust twin of this module checks each file
 * against what PyGuitarPro itself makes of it.
 */
internal object GuitarProFixtures {

    val NAMES = listOf("song.gp3", "song.gp4", "song.gp5", "song51.gp5")

    fun bytesOf(name: String): ByteArray =
        checkNotNull(GuitarProFixtures::class.java.classLoader.getResourceAsStream(name)) { "Missing fixture $name." }.use { it.readBytes() }
}

/*
 * This file is part of Campfire.
 * Copyright (c) Pandula Péter 2017-2026.
 * https://github.com/pandulapeter/campfire
 *
 * This Source Code Form is subject to the terms of the Mozilla Public License, v. 2.0.
 * If a copy of the MPL was not distributed with this file, You can obtain one at
 * https://mozilla.org/MPL/2.0/.
 */
package com.pandulapeter.campfire.guitarpro.model

public data class Note(
    /** The string, 1 being the highest. */
    val string: Int,
    val fret: Int,
    val kind: NoteKind,
    val effects: NoteEffects,
)

public enum class NoteKind {
    NORMAL,

    /** Held from the note before it, on the same string. */
    TIE,
    DEAD,
}

public data class NoteEffects(
    /** Hammer-on or pull-off to the next note. */
    val hammer: Boolean = false,
    val slide: Slide? = null,
    val bend: Bend? = null,
    val harmonic: Boolean = false,
    val vibrato: Boolean = false,
    val palmMute: Boolean = false,
    val letRing: Boolean = false,
    val ghost: Boolean = false,
    val accent: Boolean = false,
    val heavyAccent: Boolean = false,
    /** The fret of the grace note played just before it. */
    val grace: Int? = null,
    val trill: Boolean = false,
)

public enum class Slide {
    /** To the next note, which is played again. */
    SHIFT,

    /** To the next note, which is not. */
    LEGATO,
    OUT_DOWN,
    OUT_UP,
    IN_FROM_BELOW,
    IN_FROM_ABOVE,
}

/** A bend: the points of the string's rise and fall, each a place along the note (0 to 12) and a height in quarters of a tone. */
public data class Bend(val points: List<BendPoint>)

public data class BendPoint(val position: Int, val height: Int)

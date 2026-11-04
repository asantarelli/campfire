/*
 * This file is part of Campfire.
 * Copyright (c) Pandula Péter 2017-2026.
 * https://github.com/pandulapeter/campfire
 *
 * This Source Code Form is subject to the terms of the Mozilla Public License, v. 2.0.
 * If a copy of the MPL was not distributed with this file, You can obtain one at
 * https://mozilla.org/MPL/2.0/.
 */
package com.pandulapeter.campfire.presentation.ui.dialogs.training

import com.pandulapeter.campfire.chordpro.chords.ChordNotation
import com.pandulapeter.campfire.presentation.ui.chords.shownChordName

/** One of the twelve answers of the practice: a note as a pitch class from 0 (C), written the way the player reads it. */
internal data class NoteChoice(val pitchClass: Int, val label: String)

private val SHARP_NAMES = listOf("C", "C#", "D", "D#", "E", "F", "F#", "G", "G#", "A", "A#", "B")
private val FLAT_NAMES = listOf("C", "Db", "D", "Eb", "E", "F", "Gb", "G", "Ab", "A", "Bb", "B")

/**
 * The twelve notes to answer with, in [notation]: a black key is named both ways ("C♯/D♭"), since which of the two a neck or a chord
 * means is for the spelling to say, and a question has one answer to both.
 */
internal fun noteChoices(notation: ChordNotation): List<NoteChoice> = SHARP_NAMES.indices.map { pitchClass ->
    val sharp = shownChordName(SHARP_NAMES[pitchClass], notation)
    val flat = shownChordName(FLAT_NAMES[pitchClass], notation)
    NoteChoice(pitchClass, if (sharp == flat) sharp else "$sharp/$flat")
}

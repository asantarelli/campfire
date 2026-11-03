/*
 * This file is part of Campfire.
 * Copyright (c) Pandula Péter 2017-2026.
 * https://github.com/pandulapeter/campfire
 *
 * This Source Code Form is subject to the terms of the Mozilla Public License, v. 2.0.
 * If a copy of the MPL was not distributed with this file, You can obtain one at
 * https://mozilla.org/MPL/2.0/.
 */
package com.pandulapeter.campfire.presentation.ui.chords

import androidx.compose.runtime.Immutable
import androidx.compose.runtime.compositionLocalOf
import androidx.compose.ui.graphics.Color
import com.pandulapeter.campfire.data.model.preferences.UserPreferences

/**
 * What a player asked the diagrams to say beyond the shape: [labels] on the dots and over the open strings, and whether
 * the dots are [isColoredByFunction]. Provided once for the whole app, like the song's font, since every diagram of the
 * Chords section, the pinned ones and the chord's sheet follow it.
 */
@Immutable
internal data class ChordDiagramStyle(
    val labels: UserPreferences.ChordDotLabels = UserPreferences.ChordDotLabels.NONE,
    val isColoredByFunction: Boolean = false,
)

internal val LocalChordDiagramStyle = compositionLocalOf { ChordDiagramStyle() }

/** The part a note plays in a chord, which is what a color by function tells apart: the root has the accent of its own. */
internal enum class ChordFunction(val color: Color) {
    THIRD(Color(0xFFFFA726)),
    FIFTH(Color(0xFF66BB6A)),
    SEVENTH(Color(0xFFAB47BC)),
}

/** The function of the note [semitones] above the root: the thirds, the fifths (diminished and augmented too) and the sevenths and the sixth; none for the root and the other notes. */
internal fun chordFunctionOf(semitones: Int): ChordFunction? = when (semitones.mod(12)) {
    3, 4 -> ChordFunction.THIRD
    6, 7, 8 -> ChordFunction.FIFTH
    9, 10, 11 -> ChordFunction.SEVENTH
    else -> null
}

private val ROOT_NAMES = listOf("C", "D♭", "D", "E♭", "E", "F", "F♯", "G", "A♭", "A", "B♭", "B")
private const val LETTERS = "CDEFGAB"
private val NATURAL_PITCH_CLASSES = intArrayOf(0, 2, 4, 5, 7, 9, 11)

/** How many letters above the root's the note [semitones] above it is spelled, and its place in the chord (`♭3`, `5`, `♭7`). */
private class Step(val letters: Int, val degree: String)

private val STEPS = listOf(
    Step(0, "R"), Step(1, "♭2"), Step(1, "2"), Step(2, "♭3"), Step(2, "3"), Step(3, "4"),
    Step(4, "♭5"), Step(4, "5"), Step(4, "♯5"), Step(5, "6"), Step(6, "♭7"), Step(6, "7"),
)

/**
 * The name of the note of [pitchClass] as the chord on [root] spells it: by its place above the root, so that the minor
 * third of a G is a B♭ and the third of a D an F♯, never the other spelling of the same key. The root is spelled the usual
 * way (D♭ and not C♯, F♯ and not G♭). Without a root (a shape of the song's own name) the notes are spelled with sharps.
 */
internal fun noteLabelOf(pitchClass: Int, root: Int): String {
    val pitch = pitchClass.mod(12)
    if (root !in 0..11) return listOf("C", "C♯", "D", "D♯", "E", "F", "F♯", "G", "G♯", "A", "A♯", "B")[pitch]
    val interval = (pitch - root).mod(12)
    val rootLetter = ROOT_NAMES[root][0]
    val letterIndex = (LETTERS.indexOf(rootLetter) + STEPS[interval].letters) % LETTERS.length
    val offset = (pitch - NATURAL_PITCH_CLASSES[letterIndex] + 6).mod(12) - 6
    val accidental = when (offset) {
        0 -> ""
        1 -> "♯"
        -1 -> "♭"
        2 -> "𝄪"
        -2 -> "𝄫"
        else -> ""
    }
    return LETTERS[letterIndex] + accidental
}

/** The place of the note of [pitchClass] in the chord on [root]: `R`, `♭3`, `5`, `♭7`; null without a root. */
internal fun degreeLabelOf(pitchClass: Int, root: Int): String? = if (root in 0..11) STEPS[(pitchClass - root).mod(12)].degree else null

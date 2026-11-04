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

import com.pandulapeter.campfire.presentation.ui.chords.ChordDiagramGeometry

/**
 * A dot of a chord diagram the study mode can ask about: where it is, the [finger] that holds it where the shape names one, and the
 * [pitchClass] of the note it sounds.
 */
internal data class StudyTarget(val string: Int, val row: Int, val finger: Int?, val pitchClass: Int)

/**
 * Every dot of [geometry] that has a note to ask about, and a barre once, at its first string, since a finger across several strings
 * is one finger and its lowest string is the note a player reads it by.
 */
internal fun studyTargetsOf(geometry: ChordDiagramGeometry.Fretted): List<StudyTarget> {
    val dots = geometry.dots.mapNotNull { dot ->
        geometry.pitchClasses.getOrNull(dot.string)?.let { StudyTarget(dot.string, dot.row, dot.finger, it) }
    }
    val barres = geometry.barres.mapNotNull { barre ->
        geometry.pitchClasses.getOrNull(barre.fromString)?.let { StudyTarget(barre.fromString, barre.row, barre.finger, it) }
    }
    return (dots + barres).distinctBy { it.string to it.row }
}

/** The distinct notes a shape sounds, in the order of its strings, which is what the help of the study mode lists. */
internal fun notesOf(geometry: ChordDiagramGeometry.Fretted): List<Int> = geometry.pitchClasses.filterNotNull().distinct()

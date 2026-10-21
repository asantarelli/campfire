/*
 * This file is part of Campfire.
 * Copyright (c) Pandula Péter 2017-2026.
 * https://github.com/pandulapeter/campfire
 *
 * This Source Code Form is subject to the terms of the Mozilla Public License, v. 2.0.
 * If a copy of the MPL was not distributed with this file, You can obtain one at
 * https://mozilla.org/MPL/2.0/.
 */
package com.pandulapeter.campfire.presentation.ui.screens.songDetails

import androidx.compose.ui.unit.Dp
import androidx.compose.ui.unit.dp
import com.pandulapeter.campfire.chordpro.model.ChordTuning
import com.pandulapeter.campfire.data.model.preferences.UserPreferences
import com.pandulapeter.campfire.presentation.ui.chords.PINNED_CHORDS_STRIP_HEIGHT
import com.pandulapeter.campfire.presentation.ui.chords.SongChord
import com.pandulapeter.campfire.presentation.ui.chords.pinnedChordsPanelWidth
import com.pandulapeter.campfire.presentation.ui.songView.model.ChordDiagrams
import com.pandulapeter.campfire.presentation.ui.songView.model.SongLyricsModel

/**
 * The chords of a page that stand pinned beside it instead of among its sections, and what pinning them takes from the
 * song: [topInset] of its height where they are across the top, [endInset] of its width where they are down the right,
 * both left to the song's own padding so that every measure the page makes of itself (how many rows fit, where a step
 * lands) already counts them out.
 */
internal class PinnedChords(
    val position: UserPreferences.PinnedChordsPosition,
    val chords: List<SongChord>,
    val tuning: ChordTuning,
    val storedShapes: Map<String, String>,
    val onChordClicked: ((String) -> Unit)?,
) {
    val topInset: Dp = if (position == UserPreferences.PinnedChordsPosition.TOP) PINNED_CHORDS_STRIP_HEIGHT else 0.dp
    val endInset: Dp = if (position == UserPreferences.PinnedChordsPosition.RIGHT) pinnedChordsPanelWidth(tuning.instrument) else 0.dp
}

/**
 * What the page pins, or null where it pins nothing: the reader keeps the chords among the sections, the song has no
 * chords to draw yet (they are collected away from the main thread and may not be there), or the diagrams are of the
 * song's own definitions alone, which are a section of their own whatever else.
 */
internal fun pinnedChordsOf(
    position: UserPreferences.PinnedChordsPosition,
    chordDiagrams: ChordDiagrams?,
    model: SongLyricsModel,
): PinnedChords? {
    val tuning = chordDiagrams?.tuning ?: return null
    if (position == UserPreferences.PinnedChordsPosition.IN_SONG || chordDiagrams.showsDefinitionsOnly) return null
    if (model.chordTuning != tuning || model.chords.isEmpty()) return null
    return PinnedChords(
        position = position,
        chords = model.chords,
        tuning = tuning,
        storedShapes = chordDiagrams.storedShapes,
        onChordClicked = chordDiagrams.onChordClicked,
    )
}

/** The height [this] takes from the top of the song, none where nothing is pinned. */
internal val PinnedChords?.heightInset: Dp get() = this?.topInset ?: 0.dp

/** The width [this] takes from the end of the song, none where nothing is pinned. */
internal val PinnedChords?.widthInset: Dp get() = this?.endInset ?: 0.dp

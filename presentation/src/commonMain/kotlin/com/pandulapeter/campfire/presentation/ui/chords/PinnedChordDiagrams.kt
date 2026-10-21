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

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.lazy.LazyRow
import androidx.compose.foundation.lazy.grid.GridCells
import androidx.compose.foundation.lazy.grid.LazyVerticalGrid
import androidx.compose.foundation.lazy.grid.items
import androidx.compose.foundation.lazy.items
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Surface
import androidx.compose.runtime.Composable
import androidx.compose.runtime.remember
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.Dp
import androidx.compose.ui.unit.dp
import com.pandulapeter.campfire.chordpro.model.ChordInstrument
import com.pandulapeter.campfire.chordpro.model.ChordTuning
import com.pandulapeter.campfire.data.model.preferences.UserPreferences

/**
 * The diagrams of a song's chords pinned beside the song, which the song scrolls under or past: a strip across the top
 * ([UserPreferences.PinnedChordsPosition.TOP], one row that scrolls sideways) or a panel down the right
 * ([UserPreferences.PinnedChordsPosition.RIGHT], one or two columns that scroll), so that the shape of a chord is in
 * view wherever in the song it comes. Each cell is the Chords tab's, with the shape the song itself would draw it with
 * ([selectShape]), and opens the chord's sheet.
 */
@Composable
internal fun PinnedChordDiagrams(
    modifier: Modifier = Modifier,
    position: UserPreferences.PinnedChordsPosition,
    chords: List<SongChord>,
    tuning: ChordTuning,
    storedShapes: Map<String, String>,
    onChordClicked: ((String) -> Unit)?,
) {
    val cells = remember(chords, tuning, storedShapes) { chords.map { it to selectShape(it, tuning, storedShapes) } }
    val cellWidth = chordCellMinWidth(tuning.instrument)
    Surface(modifier = modifier, color = MaterialTheme.colorScheme.background) {
        when (position) {
            UserPreferences.PinnedChordsPosition.TOP -> LazyRow(
                contentPadding = PaddingValues(horizontal = PINNED_CHORDS_GAP, vertical = PINNED_CHORDS_GAP / 2),
                horizontalArrangement = Arrangement.spacedBy(PINNED_CHORDS_GAP),
            ) {
                items(items = cells, key = { (chord, _) -> chord.name }) { (chord, selection) ->
                    ChordLibraryCell(
                        modifier = Modifier.width(cellWidth),
                        chord = chord,
                        tuning = tuning,
                        selection = selection,
                        onClick = { onChordClicked?.invoke(chord.name) },
                    )
                }
            }

            else -> LazyVerticalGrid(
                columns = GridCells.Fixed(pinnedChordsColumns(tuning.instrument)),
                contentPadding = PaddingValues(PINNED_CHORDS_GAP),
                horizontalArrangement = Arrangement.spacedBy(PINNED_CHORDS_GAP),
                verticalArrangement = Arrangement.spacedBy(PINNED_CHORDS_GAP),
            ) {
                items(items = cells, key = { (chord, _) -> chord.name }) { (chord, selection) ->
                    ChordLibraryCell(
                        chord = chord,
                        tuning = tuning,
                        selection = selection,
                        onClick = { onChordClicked?.invoke(chord.name) },
                    )
                }
            }
        }
    }
}

/** How many cells stand side by side in the panel on the right: two diagrams of a neck fit it, one of a keyboard does. */
internal fun pinnedChordsColumns(instrument: ChordInstrument) = if (instrument.isFretted) 2 else 1

/** The width of the panel on the right for [instrument]: its columns of cells and the gaps between and around them. */
internal fun pinnedChordsPanelWidth(instrument: ChordInstrument): Dp {
    val columns = pinnedChordsColumns(instrument)
    return chordCellMinWidth(instrument) * columns + PINNED_CHORDS_GAP * (columns + 1)
}

/** The height of the strip across the top: one cell (a name over its diagram) and the gaps above and below it. */
internal val PINNED_CHORDS_STRIP_HEIGHT = 156.dp

private val PINNED_CHORDS_GAP = 8.dp

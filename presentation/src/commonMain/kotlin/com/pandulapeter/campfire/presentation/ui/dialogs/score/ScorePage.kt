/*
 * This file is part of Campfire.
 * Copyright (c) Pandula Péter 2017-2026.
 * https://github.com/pandulapeter/campfire
 *
 * This Source Code Form is subject to the terms of the Mozilla Public License, v. 2.0.
 * If a copy of the MPL was not distributed with this file, You can obtain one at
 * https://mozilla.org/MPL/2.0/.
 */
package com.pandulapeter.campfire.presentation.ui.dialogs.score

import androidx.compose.animation.Crossfade
import androidx.compose.foundation.Canvas
import androidx.compose.foundation.layout.BoxWithConstraints
import androidx.compose.foundation.layout.ColumnScope
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.itemsIndexed
import androidx.compose.foundation.lazy.rememberLazyListState
import androidx.compose.material3.MaterialTheme
import androidx.compose.runtime.Composable
import androidx.compose.runtime.remember
import androidx.compose.ui.Modifier
import androidx.compose.ui.text.rememberTextMeasurer
import androidx.compose.ui.unit.dp
import com.pandulapeter.campfire.guitarpro.layout.LayoutOptions
import com.pandulapeter.campfire.guitarpro.layout.ScoreLayouter
import com.pandulapeter.campfire.guitarpro.model.Score
import com.pandulapeter.campfire.presentation.ui.components.bounceScrollableContent
import com.pandulapeter.campfire.presentation.ui.components.fadingTopEdge
import com.pandulapeter.campfire.presentation.ui.dialogs.sheet.sheetContainerColor

/**
 * One track of a score set in lines as wide as the sheet, one canvas to a line so that only the lines in sight are drawn. A change of
 * what is shown crossfades; a change of the sheet's width lays the lines out again, which is not animated since it happens on every
 * frame of a resize.
 */
@Composable
internal fun ColumnScope.ScorePage(
    score: Score,
    view: ScoreView,
    contentPadding: PaddingValues,
) {
    val listState = rememberLazyListState()
    val background = sheetContainerColor()
    val colors = MaterialTheme.colorScheme
    val palette = remember(colors, background) {
        ScorePalette(ink = colors.onSurface, muted = colors.onSurfaceVariant, accent = colors.primary, background = background)
    }
    val textMeasurer = rememberTextMeasurer()
    val textStyle = MaterialTheme.typography.bodyMedium
    BoxWithConstraints(modifier = Modifier.weight(1f, fill = false).fillMaxWidth()) {
        Crossfade(targetState = view) { shown ->
            val layout = remember(score, shown, maxWidth) {
                ScoreLayouter.layout(
                    score = score,
                    trackIndex = shown.track,
                    options = LayoutOptions(
                        width = ScoreSizes.systemWidth(maxWidth.value, shown.space),
                        showNotation = shown.showNotation,
                        showTab = shown.showTab,
                        showRhythm = shown.showRhythm,
                    ),
                )
            }
            LazyColumn(
                modifier = Modifier.bounceScrollableContent(listState).fadingTopEdge(listState, background),
                state = listState,
                contentPadding = contentPadding,
            ) {
                itemsIndexed(layout.systems) { index, system ->
                    val pitch = (layout.systems.getOrNull(index + 1)?.top ?: layout.height) - system.top
                    Canvas(modifier = Modifier.fillMaxWidth().height((pitch * shown.space).dp).padding(horizontal = ScoreSizes.PAGE_PADDING.dp)) {
                        drawScoreSystem(system = system, space = shown.space.dp.toPx(), palette = palette, textMeasurer = textMeasurer, textStyle = textStyle)
                    }
                }
            }
        }
    }
}

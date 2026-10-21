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

import androidx.compose.foundation.ScrollState
import androidx.compose.foundation.gestures.scrollBy
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.rememberUpdatedState
import androidx.compose.runtime.withFrameNanos
import androidx.compose.ui.platform.LocalDensity
import androidx.compose.ui.unit.dp

/**
 * Scrolls [scrollState] on its own while [isRunning], at [dpPerSecond], a frame at a time.
 *
 * It lets go of the scroll the moment anything else holds it - a drag, the wheel, a step button, a key - rather than
 * fighting it, and at the end of the song, calling [onFinished] either way so that the button that started it goes back
 * to saying play. The pixels a frame moves are counted whole and the rest carried to the next one, since a slow song
 * moves less than a pixel in a frame and would otherwise never move. Anything but a running state costs nothing.
 */
@Composable
internal fun AutoScrollEffect(
    isRunning: Boolean,
    scrollState: ScrollState?,
    dpPerSecond: Float,
    onFinished: () -> Unit,
) {
    val density = LocalDensity.current
    val pixelsPerSecond by rememberUpdatedState(with(density) { dpPerSecond.dp.toPx() })
    val currentOnFinished by rememberUpdatedState(onFinished)
    LaunchedEffect(isRunning, scrollState) {
        if (!isRunning || scrollState == null) return@LaunchedEffect
        var previousFrame = withFrameNanos { it }
        var carried = 0f
        while (true) {
            val frame = withFrameNanos { it }
            carried += pixelsPerSecond * (frame - previousFrame) / NANOS_PER_SECOND
            previousFrame = frame
            if (scrollState.isScrollInProgress) {
                currentOnFinished()
                return@LaunchedEffect
            }
            val whole = carried.toInt()
            if (whole == 0) continue
            carried -= whole
            if (scrollState.scrollBy(whole.toFloat()) < whole - HALF_A_PIXEL) {
                currentOnFinished()
                return@LaunchedEffect
            }
        }
    }
}

private const val NANOS_PER_SECOND = 1_000_000_000f
private const val HALF_A_PIXEL = 0.5f

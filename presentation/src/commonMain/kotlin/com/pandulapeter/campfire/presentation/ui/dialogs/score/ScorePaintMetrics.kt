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

import androidx.compose.ui.unit.Density

/**
 * The smallest a score is painted at, which depends on what it is painted on: a screen's pixels, or a page's points, where a
 * pixel of the file is a third of one. [textDensity] is the density the texts are measured at where it is not the scope's own.
 */
internal class ScorePaintMetrics(
    val minimumStroke: Float,
    val minimumText: Float,
    val textDensity: Density? = null,
) {

    companion object {

        /** On a screen: a thin line is a pixel, and no text is smaller than seven. */
        val Screen = ScorePaintMetrics(minimumStroke = 1f, minimumText = 7f)
    }
}

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

import androidx.compose.ui.graphics.Color
import com.pandulapeter.campfire.guitarpro.draw.Tone

/** What each [Tone] of a laid out score looks like, and the color of the page a text that masks the lines behind it is set on. */
internal class ScorePalette(
    private val ink: Color,
    private val muted: Color,
    private val accent: Color,
    val background: Color,
) {

    fun colorOf(tone: Tone) = when (tone) {
        Tone.INK -> ink
        Tone.MUTED -> muted
        Tone.ACCENT -> accent
    }
}

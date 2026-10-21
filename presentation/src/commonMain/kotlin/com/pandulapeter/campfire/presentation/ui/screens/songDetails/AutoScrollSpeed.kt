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

import com.pandulapeter.campfire.data.model.preferences.UserPreferences

/**
 * How far a song scrolls itself in a second, in dp, at the reader's [speed] (a step of
 * [UserPreferences.MIN_AUTO_SCROLL_SPEED] to [UserPreferences.MAX_AUTO_SCROLL_SPEED]) and with the song drawn at
 * [fontScale]: a song set larger has taller lines, and the same number of lines a second is a longer way, so the
 * speed is counted in lines rather than in distance. A step is a few dp a second, which is the pace of a slow song
 * (about a line every four seconds at the middle of the range) up to that of a quick one.
 */
internal fun autoScrollDpPerSecond(speed: Int, fontScale: Float): Float =
    AUTO_SCROLL_DP_PER_SPEED_STEP * speed.coerceIn(UserPreferences.MIN_AUTO_SCROLL_SPEED, UserPreferences.MAX_AUTO_SCROLL_SPEED) * fontScale

private const val AUTO_SCROLL_DP_PER_SPEED_STEP = 3f

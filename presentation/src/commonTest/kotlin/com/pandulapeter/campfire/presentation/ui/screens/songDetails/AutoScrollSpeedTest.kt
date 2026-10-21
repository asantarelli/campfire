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
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertTrue

class AutoScrollSpeedTest {

    @Test
    fun `every step of the speed is faster than the one before`() {
        val speeds = (UserPreferences.MIN_AUTO_SCROLL_SPEED..UserPreferences.MAX_AUTO_SCROLL_SPEED).map { autoScrollDpPerSecond(it, fontScale = 1f) }
        assertEquals(speeds.sorted(), speeds)
        assertEquals(speeds.size, speeds.distinct().size)
    }

    @Test
    fun `a song set twice as large scrolls twice as far in a second`() {
        assertEquals(2 * autoScrollDpPerSecond(4, fontScale = 1f), autoScrollDpPerSecond(4, fontScale = 2f))
    }

    @Test
    fun `a speed outside the range is taken as the nearest one inside it`() {
        assertEquals(autoScrollDpPerSecond(UserPreferences.MAX_AUTO_SCROLL_SPEED, 1f), autoScrollDpPerSecond(99, 1f))
        assertEquals(autoScrollDpPerSecond(UserPreferences.MIN_AUTO_SCROLL_SPEED, 1f), autoScrollDpPerSecond(-5, 1f))
        assertTrue(autoScrollDpPerSecond(UserPreferences.MIN_AUTO_SCROLL_SPEED, 1f) > 0f)
    }
}

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

/**
 * How many pixels (dp) the space between two lines of a staff takes. A slider of whole steps rather than a continuous one, since every
 * step lays the score out again.
 */
internal object ScoreSizes {

    const val MIN = 6
    const val MAX = 16
    const val DEFAULT = 10

    /** How many dp the page is padded by on each side, which the room the systems are set in leaves out. */
    const val PAGE_PADDING = 12

    /** How wide a system is, in staff spaces, on a page [widthDp] wide where a space is [space] dp. */
    fun systemWidth(widthDp: Float, space: Int) = ((widthDp - 2 * PAGE_PADDING) / space).coerceAtLeast(MIN_SYSTEM_WIDTH)

    private const val MIN_SYSTEM_WIDTH = 30f
}

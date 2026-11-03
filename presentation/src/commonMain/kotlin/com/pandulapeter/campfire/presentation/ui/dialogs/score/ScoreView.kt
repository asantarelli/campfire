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

/** What the player asked of a score: which track, what to show of it, and how big. [space] is dp between two lines of a staff. */
internal data class ScoreView(
    val track: Int,
    val showNotation: Boolean,
    val showTab: Boolean,
    val showRhythm: Boolean,
    val space: Int,
)

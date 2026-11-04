/*
 * This file is part of Campfire.
 * Copyright (c) Pandula Péter 2017-2026.
 * https://github.com/pandulapeter/campfire
 *
 * This Source Code Form is subject to the terms of the Mozilla Public License, v. 2.0.
 * If a copy of the MPL was not distributed with this file, You can obtain one at
 * https://mozilla.org/MPL/2.0/.
 */
package com.pandulapeter.campfire.presentation.ui.components

import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import com.pandulapeter.campfire.presentation.resources.Res
import com.pandulapeter.campfire.presentation.resources.ic_practice
import org.jetbrains.compose.resources.painterResource

/** The icon button beside a tool's search that opens its practice sheet: the neck trainer on the Scales tab, the study mode on the Chords tab. */
@Composable
internal fun PracticeButton(
    modifier: Modifier = Modifier,
    description: String,
    onClick: () -> Unit,
) = IconButton(modifier = modifier, onClick = onClick) {
    Icon(painter = painterResource(Res.drawable.ic_practice), contentDescription = description)
}

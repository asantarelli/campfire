/*
 * This file is part of Campfire.
 * Copyright (c) Pandula Péter 2017-2026.
 * https://github.com/pandulapeter/campfire
 *
 * This Source Code Form is subject to the terms of the Mozilla Public License, v. 2.0.
 * If a copy of the MPL was not distributed with this file, You can obtain one at
 * https://mozilla.org/MPL/2.0/.
 */
package com.pandulapeter.campfire.presentation.ui.components.songCard

import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.semantics.contentDescription
import androidx.compose.ui.semantics.semantics
import androidx.compose.ui.unit.dp
import com.pandulapeter.campfire.presentation.localization.stringResource
import com.pandulapeter.campfire.presentation.resources.Res
import com.pandulapeter.campfire.presentation.resources.songs_kind_chordpro
import com.pandulapeter.campfire.presentation.resources.songs_kind_guitar_pro

/**
 * The letter in front of a song's title that says what the file is: a G for a Guitar Pro score, which opens as a staff and a
 * tablature, a C for a ChordPro song, which opens as a page of chords. The letters are the formats' own initials in every language.
 */
@Composable
internal fun SongKindBadge(
    modifier: Modifier = Modifier,
    isScore: Boolean,
) {
    val description = stringResource(if (isScore) Res.string.songs_kind_guitar_pro else Res.string.songs_kind_chordpro)
    Surface(
        modifier = modifier.size(BADGE_SIZE).semantics { contentDescription = description },
        shape = RoundedCornerShape(6.dp),
        color = if (isScore) MaterialTheme.colorScheme.tertiaryContainer else MaterialTheme.colorScheme.secondaryContainer,
        contentColor = if (isScore) MaterialTheme.colorScheme.onTertiaryContainer else MaterialTheme.colorScheme.onSecondaryContainer,
    ) {
        Box(contentAlignment = Alignment.Center) {
            Text(text = if (isScore) "G" else "C", style = MaterialTheme.typography.labelSmall)
        }
    }
}

private val BADGE_SIZE = 20.dp

/*
 * This file is part of Campfire.
 * Copyright (c) Pandula Péter 2017-2026.
 * https://github.com/pandulapeter/campfire
 *
 * This Source Code Form is subject to the terms of the Mozilla Public License, v. 2.0.
 * If a copy of the MPL was not distributed with this file, You can obtain one at
 * https://mozilla.org/MPL/2.0/.
 */
package com.pandulapeter.campfire.presentation.ui.screens.songEditor.reference

import androidx.compose.animation.AnimatedVisibility
import androidx.compose.animation.fadeIn
import androidx.compose.animation.fadeOut
import androidx.compose.animation.slideInHorizontally
import androidx.compose.animation.slideOutHorizontally
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxHeight
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.dp
import com.pandulapeter.campfire.presentation.localization.stringResource
import com.pandulapeter.campfire.presentation.resources.Res
import com.pandulapeter.campfire.presentation.resources.chordpro_reference_full
import com.pandulapeter.campfire.presentation.resources.close
import com.pandulapeter.campfire.presentation.resources.ic_clear
import com.pandulapeter.campfire.presentation.resources.ic_open_in_new
import com.pandulapeter.campfire.presentation.resources.song_editor_chordpro_reference
import com.pandulapeter.campfire.presentation.ui.components.list.LIST_ITEM_KEYLINE
import com.pandulapeter.campfire.presentation.ui.screens.songEditor.toolbar.EditorInsertion
import com.pandulapeter.campfire.presentation.ui.settingsPage.SettingsSectionTitle
import org.jetbrains.compose.resources.painterResource

/**
 * The reference of everything the text can say, as a panel on the right edge of the panes: it is not modal and takes
 * no scrim, so the text stays within reach of the keyboard and the pointer while it is open, which is the point of
 * having it beside the text rather than in front of it. A row's button writes into the field the way the toolbar's
 * does; the last row leads to the full reference on the ChordPro site, for the directives the app does not write.
 *
 * It comes and goes by sliding, like everything the user opens, and stands over the preview where there is one.
 */
@Composable
internal fun ChordProReferenceSheet(
    modifier: Modifier = Modifier,
    isVisible: Boolean,
    onInsert: (EditorInsertion) -> Unit,
    onOpenFullReference: () -> Unit,
    onClose: () -> Unit,
) = AnimatedVisibility(
    modifier = modifier,
    visible = isVisible,
    enter = slideInHorizontally { it } + fadeIn(),
    exit = slideOutHorizontally { it } + fadeOut(),
) {
    Surface(
        modifier = Modifier.fillMaxHeight().width(SHEET_WIDTH),
        color = MaterialTheme.colorScheme.surfaceContainerHigh,
        shape = RoundedCornerShape(topStart = SHEET_CORNER, bottomStart = SHEET_CORNER),
    ) {
        Column {
            ChordProReferenceHeader(onClose = onClose)
            val groups = chordProReferenceGroups()
            LazyColumn(modifier = Modifier.fillMaxWidth().weight(1f)) {
                groups.forEach { group ->
                    item(key = group.title, contentType = HEADING) {
                        SettingsSectionTitle(text = group.title)
                    }
                    items(items = group.entries, key = { "${group.title}/${it.syntax}" }, contentType = { ENTRY }) { entry ->
                        ChordProReferenceRow(entry = entry, onInsert = onInsert)
                    }
                }
                item(key = FULL_REFERENCE, contentType = FULL_REFERENCE) {
                    ChordProFullReferenceLink(onClick = onOpenFullReference)
                }
            }
        }
    }
}

@Composable
private fun ChordProReferenceHeader(
    onClose: () -> Unit,
) = Row(
    modifier = Modifier.fillMaxWidth().padding(start = LIST_ITEM_KEYLINE, end = 4.dp, top = 8.dp),
    verticalAlignment = Alignment.CenterVertically,
) {
    Text(
        modifier = Modifier.weight(1f),
        text = stringResource(Res.string.song_editor_chordpro_reference),
        style = MaterialTheme.typography.titleLarge,
    )
    IconButton(onClick = onClose) {
        Icon(
            painter = painterResource(Res.drawable.ic_clear),
            contentDescription = stringResource(Res.string.close),
        )
    }
}

@Composable
private fun ChordProFullReferenceLink(
    onClick: () -> Unit,
) = Row(
    modifier = Modifier.fillMaxWidth().padding(horizontal = 4.dp, vertical = 8.dp),
    horizontalArrangement = Arrangement.Start,
) {
    TextButton(onClick = onClick) {
        Icon(
            painter = painterResource(Res.drawable.ic_open_in_new),
            contentDescription = null,
        )
        Text(
            modifier = Modifier.padding(start = 8.dp),
            text = stringResource(Res.string.chordpro_reference_full),
        )
    }
}

private val SHEET_WIDTH = 340.dp
private val SHEET_CORNER = 28.dp
private const val HEADING = "heading"
private const val ENTRY = "entry"
private const val FULL_REFERENCE = "full_reference"

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

import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.semantics.contentDescription
import androidx.compose.ui.semantics.semantics
import androidx.compose.ui.unit.dp
import com.pandulapeter.campfire.presentation.localization.stringResource
import com.pandulapeter.campfire.presentation.resources.Res
import com.pandulapeter.campfire.presentation.resources.chordpro_reference_insert
import com.pandulapeter.campfire.presentation.resources.chordpro_reference_insert_description
import com.pandulapeter.campfire.presentation.ui.components.list.LIST_ITEM_KEYLINE
import com.pandulapeter.campfire.presentation.ui.screens.songEditor.toolbar.EditorInsertion
import com.pandulapeter.campfire.presentation.ui.theme.LocalMonospaceFontFamily

/**
 * What one thing the text can say is called, how it is written and what it does, with a button that writes it at the
 * caret. The syntax is in the monospace face the editor's own text is set in, since it is a piece of that text.
 */
@Composable
internal fun ChordProReferenceRow(
    modifier: Modifier = Modifier,
    entry: ChordProReferenceEntry,
    onInsert: (EditorInsertion) -> Unit,
) = Column(
    modifier = modifier.fillMaxWidth().padding(horizontal = LIST_ITEM_KEYLINE, vertical = 8.dp),
    verticalArrangement = Arrangement.spacedBy(4.dp),
) {
    Row(verticalAlignment = Alignment.CenterVertically) {
        Text(
            modifier = Modifier.weight(1f),
            text = entry.title,
            style = MaterialTheme.typography.titleMedium,
        )
        val insertDescription = stringResource(Res.string.chordpro_reference_insert_description, entry.title)
        TextButton(
            modifier = Modifier.semantics { contentDescription = insertDescription },
            onClick = { onInsert(entry.insertion) },
        ) {
            Text(text = stringResource(Res.string.chordpro_reference_insert))
        }
    }
    Text(
        modifier = Modifier
            .clip(MaterialTheme.shapes.small)
            .background(MaterialTheme.colorScheme.surfaceContainerHighest)
            .padding(horizontal = 8.dp, vertical = 4.dp),
        text = entry.syntax,
        style = MaterialTheme.typography.bodySmall.copy(fontFamily = LocalMonospaceFontFamily.current),
        color = MaterialTheme.colorScheme.onSurfaceVariant,
    )
    Text(
        text = entry.description,
        style = MaterialTheme.typography.bodyMedium,
        color = MaterialTheme.colorScheme.onSurfaceVariant,
    )
}

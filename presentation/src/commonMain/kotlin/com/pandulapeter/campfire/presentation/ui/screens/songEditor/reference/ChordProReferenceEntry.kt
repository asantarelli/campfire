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

import com.pandulapeter.campfire.presentation.ui.screens.songEditor.toolbar.EditorInsertion

/**
 * One thing the editor's text can say, as the reference sheet lists it.
 *
 * @param title What the toolbar calls it, so that a button there and a row here are recognizably the same thing.
 * @param syntax How it is written, a literal that is never translated: it is the ChordPro the file holds.
 * @param description What it does, in a sentence.
 * @param insertion What the row's button writes, which is exactly what the toolbar button of the same name writes.
 */
internal data class ChordProReferenceEntry(
    val title: String,
    val syntax: String,
    val description: String,
    val insertion: EditorInsertion,
)

/** The entries that belong together under one heading. */
internal data class ChordProReferenceGroup(
    val title: String,
    val entries: List<ChordProReferenceEntry>,
)

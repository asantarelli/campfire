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

import androidx.compose.runtime.Composable
import com.pandulapeter.campfire.presentation.localization.stringResource
import com.pandulapeter.campfire.presentation.resources.Res
import com.pandulapeter.campfire.presentation.resources.chordpro_reference_album
import com.pandulapeter.campfire.presentation.resources.chordpro_reference_annotation
import com.pandulapeter.campfire.presentation.resources.chordpro_reference_artist
import com.pandulapeter.campfire.presentation.resources.chordpro_reference_capo
import com.pandulapeter.campfire.presentation.resources.chordpro_reference_chord
import com.pandulapeter.campfire.presentation.resources.chordpro_reference_chorus_recall
import com.pandulapeter.campfire.presentation.resources.chordpro_reference_comment
import com.pandulapeter.campfire.presentation.resources.chordpro_reference_comment_box
import com.pandulapeter.campfire.presentation.resources.chordpro_reference_comment_italic
import com.pandulapeter.campfire.presentation.resources.chordpro_reference_composer
import com.pandulapeter.campfire.presentation.resources.chordpro_reference_cover
import com.pandulapeter.campfire.presentation.resources.chordpro_reference_duration
import com.pandulapeter.campfire.presentation.resources.chordpro_reference_grid
import com.pandulapeter.campfire.presentation.resources.chordpro_reference_group_changes
import com.pandulapeter.campfire.presentation.resources.chordpro_reference_group_comments
import com.pandulapeter.campfire.presentation.resources.chordpro_reference_group_line
import com.pandulapeter.campfire.presentation.resources.chordpro_reference_group_sections
import com.pandulapeter.campfire.presentation.resources.chordpro_reference_group_song
import com.pandulapeter.campfire.presentation.resources.chordpro_reference_key
import com.pandulapeter.campfire.presentation.resources.chordpro_reference_language
import com.pandulapeter.campfire.presentation.resources.chordpro_reference_link
import com.pandulapeter.campfire.presentation.resources.chordpro_reference_lyricist
import com.pandulapeter.campfire.presentation.resources.chordpro_reference_section
import com.pandulapeter.campfire.presentation.resources.chordpro_reference_subtitle
import com.pandulapeter.campfire.presentation.resources.chordpro_reference_tab
import com.pandulapeter.campfire.presentation.resources.chordpro_reference_tag
import com.pandulapeter.campfire.presentation.resources.chordpro_reference_tempo
import com.pandulapeter.campfire.presentation.resources.chordpro_reference_time
import com.pandulapeter.campfire.presentation.resources.chordpro_reference_title
import com.pandulapeter.campfire.presentation.resources.chordpro_reference_transpose
import com.pandulapeter.campfire.presentation.resources.chordpro_reference_year
import com.pandulapeter.campfire.presentation.resources.song_editor_insert_album
import com.pandulapeter.campfire.presentation.resources.song_editor_insert_annotation
import com.pandulapeter.campfire.presentation.resources.song_editor_insert_artist
import com.pandulapeter.campfire.presentation.resources.song_editor_insert_capo
import com.pandulapeter.campfire.presentation.resources.song_editor_insert_chord
import com.pandulapeter.campfire.presentation.resources.song_editor_insert_chorus_recall
import com.pandulapeter.campfire.presentation.resources.song_editor_insert_comment
import com.pandulapeter.campfire.presentation.resources.song_editor_insert_comment_box
import com.pandulapeter.campfire.presentation.resources.song_editor_insert_comment_italic
import com.pandulapeter.campfire.presentation.resources.song_editor_insert_composer
import com.pandulapeter.campfire.presentation.resources.song_editor_insert_cover_art
import com.pandulapeter.campfire.presentation.resources.song_editor_insert_duration
import com.pandulapeter.campfire.presentation.resources.song_editor_insert_key
import com.pandulapeter.campfire.presentation.resources.song_editor_insert_key_change
import com.pandulapeter.campfire.presentation.resources.song_editor_insert_language
import com.pandulapeter.campfire.presentation.resources.song_editor_insert_link
import com.pandulapeter.campfire.presentation.resources.song_editor_insert_lyricist
import com.pandulapeter.campfire.presentation.resources.song_editor_insert_subtitle
import com.pandulapeter.campfire.presentation.resources.song_editor_insert_tag
import com.pandulapeter.campfire.presentation.resources.song_editor_insert_tempo
import com.pandulapeter.campfire.presentation.resources.song_editor_insert_time
import com.pandulapeter.campfire.presentation.resources.song_editor_insert_title
import com.pandulapeter.campfire.presentation.resources.song_editor_insert_year
import com.pandulapeter.campfire.presentation.resources.song_editor_section_bridge
import com.pandulapeter.campfire.presentation.resources.song_editor_section_chorus
import com.pandulapeter.campfire.presentation.resources.song_editor_section_grid
import com.pandulapeter.campfire.presentation.resources.song_editor_section_intro
import com.pandulapeter.campfire.presentation.resources.song_editor_section_outro
import com.pandulapeter.campfire.presentation.resources.song_editor_section_pre_chorus
import com.pandulapeter.campfire.presentation.resources.song_editor_section_solo
import com.pandulapeter.campfire.presentation.resources.song_editor_section_tab
import com.pandulapeter.campfire.presentation.resources.song_editor_section_verse
import com.pandulapeter.campfire.presentation.ui.screens.songEditor.toolbar.EditorInsertion
import org.jetbrains.compose.resources.StringResource

/**
 * Everything the toolbar writes, explained: what describes the song, what goes inside a line, the sections, the
 * comments and the changes a song makes as it goes. The titles are the toolbar's own, so a row and its button are
 * recognizably one thing, and the insertions are built the way the toolbar builds them, so that a button here writes
 * what the one there writes.
 *
 * `{new_page}`, `{column_break}` and `{new_song}` are left out for the reasons the toolbar leaves them out: the reader
 * flows the sections into columns by itself, and the editor only ever holds one song. The tab is the plain one, for any
 * instrument: the toolbar's tab button is the one that asks which instrument it is for and draws its staff.
 */
@Composable
internal fun chordProReferenceGroups(): List<ChordProReferenceGroup> = listOf(
    ChordProReferenceGroup(stringResource(Res.string.chordpro_reference_group_song), songEntries()),
    ChordProReferenceGroup(stringResource(Res.string.chordpro_reference_group_line), lineEntries()),
    ChordProReferenceGroup(stringResource(Res.string.chordpro_reference_group_sections), sectionEntries()),
    ChordProReferenceGroup(stringResource(Res.string.chordpro_reference_group_comments), commentEntries()),
    ChordProReferenceGroup(stringResource(Res.string.chordpro_reference_group_changes), changeEntries()),
)

@Composable
private fun songEntries() = listOf(
    metadata(Res.string.song_editor_insert_title, Res.string.chordpro_reference_title, name = "title", example = "Home on the Range"),
    metadata(Res.string.song_editor_insert_subtitle, Res.string.chordpro_reference_subtitle, name = "subtitle", example = "My Western Home"),
    metadata(Res.string.song_editor_insert_artist, Res.string.chordpro_reference_artist, name = "artist", example = "Traditional American"),
    metadata(Res.string.song_editor_insert_composer, Res.string.chordpro_reference_composer, name = "composer", example = "Daniel E. Kelley"),
    metadata(Res.string.song_editor_insert_lyricist, Res.string.chordpro_reference_lyricist, name = "lyricist", example = "Brewster M. Higley"),
    metadata(Res.string.song_editor_insert_album, Res.string.chordpro_reference_album, name = "album", example = "Campfire Songs"),
    metadata(Res.string.song_editor_insert_year, Res.string.chordpro_reference_year, name = "year", example = "1872"),
    metadata(Res.string.song_editor_insert_key, Res.string.chordpro_reference_key, name = "key", example = "G"),
    metadata(Res.string.song_editor_insert_capo, Res.string.chordpro_reference_capo, name = "capo", example = "2"),
    metadata(Res.string.song_editor_insert_tempo, Res.string.chordpro_reference_tempo, name = "tempo", example = "100"),
    metadata(Res.string.song_editor_insert_time, Res.string.chordpro_reference_time, name = "time", example = "3/4"),
    metadata(Res.string.song_editor_insert_duration, Res.string.chordpro_reference_duration, name = "duration", example = "3:45"),
    metadata(Res.string.song_editor_insert_tag, Res.string.chordpro_reference_tag, name = "tag", example = "Easy"),
    meta(Res.string.song_editor_insert_language, Res.string.chordpro_reference_language, key = "language", example = "es"),
    meta(Res.string.song_editor_insert_cover_art, Res.string.chordpro_reference_cover, key = "cover", example = "https://…"),
    meta(Res.string.song_editor_insert_link, Res.string.chordpro_reference_link, key = "link", example = "https://… Name"),
)

@Composable
private fun lineEntries() = listOf(
    entry(
        label = Res.string.song_editor_insert_chord,
        description = Res.string.chordpro_reference_chord,
        syntax = "[Am]",
        insertion = { EditorInsertion(it, prefix = "[", suffix = "]", isOwnLine = false) },
    ),
    entry(
        label = Res.string.song_editor_insert_annotation,
        description = Res.string.chordpro_reference_annotation,
        syntax = "[*Softly]",
        insertion = { EditorInsertion(it, prefix = "[*", suffix = "]", isOwnLine = false) },
    ),
)

@Composable
private fun sectionEntries() = listOf(
    Res.string.song_editor_section_intro to "intro",
    Res.string.song_editor_section_verse to "verse",
    Res.string.song_editor_section_pre_chorus to "pre-chorus",
    Res.string.song_editor_section_chorus to "chorus",
    Res.string.song_editor_section_bridge to "bridge",
    Res.string.song_editor_section_solo to "solo",
    Res.string.song_editor_section_outro to "outro",
).map { (label, name) -> section(label, Res.string.chordpro_reference_section, name) } + listOf(
    entry(
        label = Res.string.song_editor_insert_chorus_recall,
        description = Res.string.chordpro_reference_chorus_recall,
        syntax = "{chorus}",
        insertion = { EditorInsertion.directive(it, name = "chorus", hasValue = false) },
    ),
    section(Res.string.song_editor_section_tab, Res.string.chordpro_reference_tab, name = "tab"),
    section(Res.string.song_editor_section_grid, Res.string.chordpro_reference_grid, name = "grid"),
)

@Composable
private fun commentEntries() = listOf(
    directive(Res.string.song_editor_insert_comment, Res.string.chordpro_reference_comment, name = "comment", example = "Repeat twice"),
    directive(Res.string.song_editor_insert_comment_italic, Res.string.chordpro_reference_comment_italic, name = "comment_italic", example = "Softly"),
    directive(Res.string.song_editor_insert_comment_box, Res.string.chordpro_reference_comment_box, name = "comment_box", example = "Last time"),
)

@Composable
private fun changeEntries() = listOf(
    directive(Res.string.song_editor_insert_key_change, Res.string.chordpro_reference_transpose, name = "transpose", example = "2"),
)

@Composable
private fun entry(
    label: StringResource,
    description: StringResource,
    syntax: String,
    insertion: (String) -> EditorInsertion,
): ChordProReferenceEntry {
    val title = stringResource(label)
    return ChordProReferenceEntry(
        title = title,
        syntax = syntax,
        description = stringResource(description),
        insertion = insertion(title),
    )
}

@Composable
private fun metadata(label: StringResource, description: StringResource, name: String, example: String) = entry(
    label = label,
    description = description,
    syntax = "{$name: $example}",
    insertion = { EditorInsertion.metadata(it, name = name) },
)

@Composable
private fun meta(label: StringResource, description: StringResource, key: String, example: String) = entry(
    label = label,
    description = description,
    syntax = "{meta: $key $example}",
    insertion = { EditorInsertion.meta(it, key = key) },
)

@Composable
private fun directive(label: StringResource, description: StringResource, name: String, example: String) = entry(
    label = label,
    description = description,
    syntax = "{$name: $example}",
    insertion = { EditorInsertion.directive(it, name = name) },
)

@Composable
private fun section(label: StringResource, description: StringResource, name: String) = entry(
    label = label,
    description = description,
    syntax = "{start_of_$name} … {end_of_$name}",
    insertion = { EditorInsertion.section(it, name = name) },
)

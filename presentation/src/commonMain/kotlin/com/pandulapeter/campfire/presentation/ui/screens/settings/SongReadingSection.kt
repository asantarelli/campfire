/*
 * This file is part of Campfire.
 * Copyright (c) Pandula Péter 2017-2026.
 * https://github.com/pandulapeter/campfire
 *
 * This Source Code Form is subject to the terms of the Mozilla Public License, v. 2.0.
 * If a copy of the MPL was not distributed with this file, You can obtain one at
 * https://mozilla.org/MPL/2.0/.
 */
package com.pandulapeter.campfire.presentation.ui.screens.settings

import androidx.compose.runtime.Composable
import com.pandulapeter.campfire.data.model.preferences.UserPreferences
import com.pandulapeter.campfire.presentation.localization.stringResource
import com.pandulapeter.campfire.presentation.resources.Res
import com.pandulapeter.campfire.presentation.resources.music_search_spotify
import com.pandulapeter.campfire.presentation.resources.music_search_youtube
import com.pandulapeter.campfire.presentation.resources.music_search_youtube_music
import com.pandulapeter.campfire.presentation.resources.settings_group_song_view
import com.pandulapeter.campfire.presentation.resources.settings_music_search
import com.pandulapeter.campfire.presentation.resources.settings_music_search_description
import com.pandulapeter.campfire.presentation.resources.settings_pinned_chords
import com.pandulapeter.campfire.presentation.resources.settings_pinned_chords_description
import com.pandulapeter.campfire.presentation.resources.settings_pinned_chords_in_song
import com.pandulapeter.campfire.presentation.resources.settings_pinned_chords_right
import com.pandulapeter.campfire.presentation.resources.settings_pinned_chords_top
import com.pandulapeter.campfire.presentation.resources.settings_song_columns
import com.pandulapeter.campfire.presentation.resources.settings_song_columns_automatic
import com.pandulapeter.campfire.presentation.resources.settings_song_columns_description
import com.pandulapeter.campfire.presentation.resources.settings_song_font
import com.pandulapeter.campfire.presentation.resources.settings_song_font_default
import com.pandulapeter.campfire.presentation.resources.settings_song_font_description
import com.pandulapeter.campfire.presentation.resources.settings_song_font_monospace
import com.pandulapeter.campfire.presentation.ui.CampfireViewModel
import com.pandulapeter.campfire.presentation.ui.components.controls.SegmentedChoice
import com.pandulapeter.campfire.presentation.ui.settingsPage.SettingsSection
import com.pandulapeter.campfire.presentation.ui.settingsPage.SettingsSubsection

/**
 * The third section of the Preferences tab: how a song is laid out while it is read - its columns and where the
 * diagrams of its chords stand - and which service the song's listen button looks it up on. Only there while the songs
 * are switched on, like the section before it.
 */
@Composable
internal fun SongReadingSection(
    viewModel: CampfireViewModel,
    userPreferences: UserPreferences?,
) = SettingsSection(title = stringResource(Res.string.settings_group_song_view)) {
    SettingsSubsection(
        title = stringResource(Res.string.settings_song_columns),
        description = stringResource(Res.string.settings_song_columns_description),
    ) {
        SegmentedChoice(
            options = listOf(UserPreferences.AUTOMATIC_SONG_COLUMNS to stringResource(Res.string.settings_song_columns_automatic)) +
                (1..UserPreferences.MAX_SONG_COLUMNS).map { it to it.toString() },
            selected = userPreferences?.songColumns ?: UserPreferences.AUTOMATIC_SONG_COLUMNS,
            onSelected = viewModel.preferencesController::setSongColumns,
        )
    }
    SettingsSubsection(
        title = stringResource(Res.string.settings_song_font),
        description = stringResource(Res.string.settings_song_font_description),
    ) {
        SegmentedChoice(
            options = listOf(
                UserPreferences.SongFont.DEFAULT to stringResource(Res.string.settings_song_font_default),
                UserPreferences.SongFont.MONOSPACE to stringResource(Res.string.settings_song_font_monospace),
            ),
            selected = userPreferences?.songFont ?: UserPreferences.SongFont.DEFAULT,
            onSelected = viewModel.preferencesController::setSongFont,
        )
    }
    SettingsSubsection(
        title = stringResource(Res.string.settings_pinned_chords),
        description = stringResource(Res.string.settings_pinned_chords_description),
        isEnabled = userPreferences?.areChordsEnabled != false && userPreferences?.areChordDiagramsEnabled != false,
    ) {
        SegmentedChoice(
            options = listOf(
                UserPreferences.PinnedChordsPosition.IN_SONG to stringResource(Res.string.settings_pinned_chords_in_song),
                UserPreferences.PinnedChordsPosition.TOP to stringResource(Res.string.settings_pinned_chords_top),
                UserPreferences.PinnedChordsPosition.RIGHT to stringResource(Res.string.settings_pinned_chords_right),
            ),
            selected = userPreferences?.pinnedChordsPosition ?: UserPreferences.PinnedChordsPosition.RIGHT,
            isEnabled = userPreferences?.areChordsEnabled != false && userPreferences?.areChordDiagramsEnabled != false,
            onSelected = viewModel.preferencesController::setPinnedChordsPosition,
        )
    }
    SettingsSubsection(
        title = stringResource(Res.string.settings_music_search),
        description = stringResource(Res.string.settings_music_search_description),
    ) {
        SegmentedChoice(
            options = listOf(
                UserPreferences.MusicSearchService.YOUTUBE to stringResource(Res.string.music_search_youtube),
                UserPreferences.MusicSearchService.YOUTUBE_MUSIC to stringResource(Res.string.music_search_youtube_music),
                UserPreferences.MusicSearchService.SPOTIFY to stringResource(Res.string.music_search_spotify),
            ),
            selected = userPreferences?.musicSearchService ?: UserPreferences.MusicSearchService.YOUTUBE,
            onSelected = viewModel.preferencesController::setMusicSearchService,
        )
    }
}

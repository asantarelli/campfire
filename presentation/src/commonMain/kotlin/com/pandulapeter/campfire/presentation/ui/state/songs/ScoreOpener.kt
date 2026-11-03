/*
 * This file is part of Campfire.
 * Copyright (c) Pandula Péter 2017-2026.
 * https://github.com/pandulapeter/campfire
 *
 * This Source Code Form is subject to the terms of the Mozilla Public License, v. 2.0.
 * If a copy of the MPL was not distributed with this file, You can obtain one at
 * https://mozilla.org/MPL/2.0/.
 */
package com.pandulapeter.campfire.presentation.ui.state.songs

import com.pandulapeter.campfire.chordpro.ChordProParser
import com.pandulapeter.campfire.data.model.song.Song
import com.pandulapeter.campfire.domain.api.songs.GetSongContentUseCase
import com.pandulapeter.campfire.guitarpro.song.ScoreSongText
import com.pandulapeter.campfire.presentation.ui.dialogs.DialogHost
import com.pandulapeter.campfire.presentation.ui.dialogs.DialogType
import com.pandulapeter.campfire.presentation.ui.messages.Message
import com.pandulapeter.campfire.presentation.ui.messages.MessageSink
import kotlinx.coroutines.CancellationException
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.launch

/**
 * Opens a song that holds a Guitar Pro score: the score is read out of the song's file and shown on its sheet in place of the
 * song's page, which a score has none of. Where it cannot be read out (the file is gone, or something edited its lines) the
 * player is told rather than shown an empty page.
 */
internal class ScoreOpener(
    private val scope: CoroutineScope,
    private val dialogHost: DialogHost,
    private val messageSink: MessageSink,
    private val getSongContent: GetSongContentUseCase,
    private val song: (fileName: String) -> Song?,
) {

    fun open(fileName: String) {
        val song = song(fileName) ?: return
        scope.launch {
            val bytes = try {
                getSongContent(fileName)?.let { ScoreSongText.bytesOf(ChordProParser.parseMetadata(it.text).custom) }
            } catch (exception: CancellationException) {
                throw exception
            } catch (exception: Exception) {
                null
            }
            if (bytes == null) messageSink.sendMessage(Message.ScoreUnreadable) else dialogHost.showDialog(DialogType.GuitarProScore(song = song, bytes = bytes))
        }
    }
}

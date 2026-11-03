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

import com.pandulapeter.campfire.data.model.importing.ImportedFile
import com.pandulapeter.campfire.data.model.library.LibraryFiles
import com.pandulapeter.campfire.guitarpro.model.Score
import com.pandulapeter.campfire.guitarpro.reader.GuitarProException
import com.pandulapeter.campfire.guitarpro.reader.GuitarProReader

/** Whether the file is named as a Guitar Pro score, which is shown as it is rather than imported into the library. */
internal val ImportedFile.isGuitarProScore: Boolean
    get() = LibraryFiles.isScoreFileName(name)

/** What reading the bytes of a Guitar Pro file came to. */
internal sealed interface ScoreReading {

    data class Read(val score: Score) : ScoreReading

    data class Failed(val reason: GuitarProException.Reason) : ScoreReading
}

/**
 * Reads a score, never failing: whatever is wrong with the file is a reason to word for the player. A file the reader trips over
 * without meaning to (a number it did not expect, a list shorter than it says) is damaged as far as anyone is concerned.
 */
internal fun readScore(bytes: ByteArray): ScoreReading = try {
    ScoreReading.Read(GuitarProReader.read(bytes))
} catch (exception: GuitarProException) {
    ScoreReading.Failed(exception.reason)
} catch (_: RuntimeException) {
    ScoreReading.Failed(GuitarProException.Reason.CORRUPT)
}

/** The track a score opens on: the first that is not a drum kit, which has no strings to read. */
internal fun Score.firstReadableTrack() = tracks.indexOfFirst { !it.isPercussion }.coerceAtLeast(0)

/*
 * This file is part of Campfire.
 * Copyright (c) Pandula Péter 2017-2026.
 * https://github.com/pandulapeter/campfire
 *
 * This Source Code Form is subject to the terms of the Mozilla Public License, v. 2.0.
 * If a copy of the MPL was not distributed with this file, You can obtain one at
 * https://mozilla.org/MPL/2.0/.
 */
package com.pandulapeter.campfire.guitarpro.reader

import com.pandulapeter.campfire.guitarpro.model.Beat
import com.pandulapeter.campfire.guitarpro.model.BeatStatus
import com.pandulapeter.campfire.guitarpro.model.Measure
import com.pandulapeter.campfire.guitarpro.model.MeasureHeader
import com.pandulapeter.campfire.guitarpro.model.QUARTER_TICKS
import com.pandulapeter.campfire.guitarpro.model.Score
import com.pandulapeter.campfire.guitarpro.model.Track

/** The score of a file whose version is known: its info, its measures' headers, its tracks and the beats of every measure. */
internal class ScoreReader(
    private val cursor: ByteCursor,
    private val version: GuitarProVersion,
) {
    private val major get() = version.major

    fun read(): Score {
        var score = info()
        if (major <= 4) {
            cursor.boolean() // The triplet feel.
        }
        if (major >= 4) {
            lyrics()
        }
        if (major == 5) {
            if (version.isAfter5Dot0) {
                cursor.skip(4 + 4 + 11) // The master volume and its equalizer.
            }
            cursor.skip(4 * 2 + 4 * 4 + 4 + 2) // The page: its size, margins, scale and what is shown on it.
            repeat(10) { cursor.intByteSizeString() }
            cursor.intByteSizeString() // The name of the tempo.
        }
        val tempo = cursor.i32()
        val key: Int
        if (major == 5) {
            if (version.isAfter5Dot0) {
                cursor.boolean() // Whether the tempo is hidden.
            }
            key = cursor.i8()
            cursor.skip(4) // The octave.
        } else {
            key = cursor.i32()
            if (major == 4) {
                cursor.skip(1) // The octave.
            }
        }
        cursor.skip(64 * 12) // The 64 channels of the mixer.
        if (major == 5) {
            cursor.skip(19 * 2) // The signs that say where to jump.
            cursor.skip(4) // The master reverb.
        }
        val measureCount = cursor.count()
        val trackCount = cursor.count()
        val headers = headers(measureCount)
        val trackReader = TrackReader(cursor, version)
        val tracks = (1..trackCount).map { trackReader.track(it) }
        if (major == 5) {
            cursor.skip(if (version.isAfter5Dot0) 1 else 2)
        }
        score = score.copy(tempo = tempo, key = key, headers = headers)
        return score.copy(tracks = measures(score, tracks))
    }

    private fun info(): Score {
        val title = cursor.intByteSizeString()
        val subtitle = cursor.intByteSizeString()
        val artist = cursor.intByteSizeString()
        val album = cursor.intByteSizeString()
        val words = cursor.intByteSizeString()
        val music = if (major == 5) cursor.intByteSizeString() else words
        val copyright = cursor.intByteSizeString()
        val tabbedBy = cursor.intByteSizeString()
        cursor.intByteSizeString() // The instructions.
        repeat(cursor.count()) { cursor.intByteSizeString() } // The notices.
        return Score(title, subtitle, artist, album, words, music, copyright, tabbedBy)
    }

    private fun lyrics() {
        cursor.skip(4) // The track they belong to.
        repeat(5) {
            cursor.skip(4)
            cursor.intSizeString()
        }
    }

    private fun headers(count: Int): List<MeasureHeader> {
        val headers = mutableListOf<MeasureHeader>()
        var start = QUARTER_TICKS
        for (number in 0 until count) {
            if (major == 5 && number > 0) {
                cursor.skip(1)
            }
            val header = header(headers.lastOrNull()).copy(start = start)
            start += header.length
            headers += header
        }
        return headers
    }

    private fun header(previous: MeasureHeader?): MeasureHeader {
        val flags = cursor.u8()
        val isVersion5 = major == 5
        val numerator = if (flags and 0x01 != 0) cursor.i8() else previous?.numerator ?: 4
        val denominator = if (flags and 0x02 != 0) cursor.i8() else previous?.denominator ?: 4
        var repeatClose = -1
        var repeatAlternative = 0
        var marker: String? = null
        var keyChange: Int? = null
        if (flags and 0x08 != 0) {
            repeatClose = cursor.i8()
        }
        // Where the number of the ending is, and the marker and the key, is not the same in a Guitar Pro 5 file as in the others.
        if (!isVersion5 && flags and 0x10 != 0) {
            repeatAlternative = cursor.u8()
        }
        if (flags and 0x20 != 0) {
            marker = cursor.intByteSizeString()
            cursor.skip(4) // Its color.
        }
        if (flags and 0x40 != 0) {
            keyChange = cursor.i8()
            cursor.i8() // Whether it is minor.
        }
        if (isVersion5) {
            if (flags and 0x10 != 0) {
                repeatAlternative = cursor.u8()
            }
            if (repeatClose > -1) {
                repeatClose -= 1
            }
            if (flags and 0x03 != 0) {
                cursor.skip(4) // How the beats of the measure are beamed.
            }
            if (flags and 0x10 == 0) {
                cursor.skip(1)
            }
            cursor.skip(1) // The triplet feel.
        }
        return MeasureHeader(
            numerator = numerator.coerceAtLeast(1),
            denominator = denominator.coerceAtLeast(1),
            isRepeatOpen = flags and 0x04 != 0,
            repeatClose = repeatClose,
            repeatAlternative = repeatAlternative,
            marker = marker,
            keyChange = keyChange,
            hasDoubleBar = flags and 0x80 != 0,
        )
    }

    /** The tracks with the measures of each: the file writes the first measure of every track, then the second of every track... */
    private fun measures(score: Score, tracks: List<Track>): List<Track> {
        val voiceCount = if (major == 5) 2 else 1
        val beatReader = BeatReader(cursor, version)
        val lastFrets = tracks.map { IntArray(it.strings.size + 1) { -1 } }
        val measures = tracks.map { mutableListOf<Measure>() }
        for (header in score.headers) {
            for ((trackIndex, track) in tracks.withIndex()) {
                val voices = List(voiceCount) { voice(beatReader, header.start, track.strings.size, lastFrets[trackIndex]) }
                if (major == 5) {
                    cursor.skipIfAny() // Where the line breaks, which the last measure of a file may lack.
                }
                measures[trackIndex] += Measure(voices)
            }
        }
        return tracks.mapIndexed { index, track -> track.copy(measures = measures[index]) }
    }

    private fun voice(beatReader: BeatReader, start: Int, strings: Int, lastFrets: IntArray): List<Beat> {
        val beats = mutableListOf<Beat>()
        var at = start
        repeat(cursor.count()) {
            val beat = beatReader.beat(at, strings, lastFrets)
            if (beat.status != BeatStatus.EMPTY) {
                at += beat.duration.ticks
            }
            beats += beat
        }
        return beats
    }
}

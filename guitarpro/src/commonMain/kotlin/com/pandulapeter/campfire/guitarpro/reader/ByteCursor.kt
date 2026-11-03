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

import com.pandulapeter.campfire.guitarpro.reader.GuitarProException.Reason

/** How many of anything a file may claim to hold, so that a damaged one is refused rather than allocated. */
private const val MAX_COUNT = 100_000

/**
 * The numbers, flags and strings of a Guitar Pro file, which are one long run with no index in front of anything: a reader walks it
 * from the start and every piece that is not read is skipped.
 */
internal class ByteCursor(private val data: ByteArray) {

    private var position = 0

    fun skip(count: Int) {
        if (count < 0 || position + count > data.size) throw GuitarProException(Reason.UNEXPECTED_END)
        position += count
    }

    /** Moves one byte on, or stops at the end of the file, which is where Guitar Pro itself stops when a last byte is missing. */
    fun skipIfAny() {
        position = (position + 1).coerceAtMost(data.size)
    }

    fun u8(): Int {
        if (position >= data.size) throw GuitarProException(Reason.UNEXPECTED_END)
        return data[position++].toInt() and 0xFF
    }

    fun i8(): Int = u8().toByte().toInt()

    fun boolean(): Boolean = u8() != 0

    fun i16(): Int = (u8() or (u8() shl 8)).toShort().toInt()

    fun i32(): Int = u8() or (u8() shl 8) or (u8() shl 16) or (u8() shl 24)

    /** A count the file claims, which is never negative and never more than a score could hold. */
    fun count(): Int {
        val count = i32()
        if (count !in 0..MAX_COUNT) throw GuitarProException(Reason.CORRUPT)
        return count
    }

    /** A length of one byte and then `room` bytes, of which the length are the text. */
    fun byteSizeString(room: Int): String {
        val length = u8()
        val start = position
        skip(room)
        return textOf(start, length.coerceAtMost(room))
    }

    /** A length of four bytes, and then that many bytes of text. */
    fun intSizeString(): String {
        val length = count()
        val start = position
        skip(length)
        return textOf(start, length)
    }

    /** A number of four bytes that is one more than the room of a [byteSizeString] that follows. */
    fun intByteSizeString(): String {
        val length = i32()
        if (length <= 0) return ""
        if (length > MAX_COUNT) throw GuitarProException(Reason.CORRUPT)
        return byteSizeString(length - 1)
    }

    /** Text in UTF-8 where it reads as that, else in Windows-1252, which is what Guitar Pro itself writes. */
    private fun textOf(start: Int, length: Int): String {
        val bytes = data.copyOfRange(start, start + length)
        return try {
            bytes.decodeToString(throwOnInvalidSequence = true)
        } catch (_: CharacterCodingException) {
            bytes.map { (it.toInt() and 0xFF).toChar() }.joinToString("")
        }
    }
}

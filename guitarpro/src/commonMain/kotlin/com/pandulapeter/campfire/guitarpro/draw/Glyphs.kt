/*
 * This file is part of Campfire.
 * Copyright (c) Pandula Péter 2017-2026.
 * https://github.com/pandulapeter/campfire
 *
 * This Source Code Form is subject to the terms of the Mozilla Public License, v. 2.0.
 * If a copy of the MPL was not distributed with this file, You can obtain one at
 * https://mozilla.org/MPL/2.0/.
 */
package com.pandulapeter.campfire.guitarpro.draw

import kotlin.math.absoluteValue

/**
 * The signs of music written down, drawn with strokes and ellipses rather than set in a font, so that a score looks the same wherever
 * it is painted and nothing has to be bundled for it. Every shape is in units of the space between two lines of a staff, with the
 * staff's top line at y = 0 and its bottom one at y = 4.
 */
internal object Glyphs {

    /** The thickness of the thin lines of a staff, a stem and a bar line. */
    const val LINE = 0.1f

    /** How thick a stem is. */
    const val STEM = 0.12f

    /** How thick a beam is. */
    const val BEAM = 0.5f

    /** Half the width of a notehead. */
    const val HEAD_RX = 0.62f

    /** A notehead at ([x], [y]): filled for a quarter note and shorter, hollow for a half note and a whole one. */
    fun notehead(x: Float, y: Float, value: Int): Draw = when (value) {
        1 -> Draw.Ellipse(x, y, HEAD_RX + 0.1f, 0.5f, 0f, isFilled = false, width = 0.2f)
        2 -> Draw.Ellipse(x, y, HEAD_RX - 0.05f, 0.42f, -0.45f, isFilled = false, width = 0.14f)
        else -> Draw.Ellipse(x, y, HEAD_RX, 0.5f, -0.4f, isFilled = true, width = 0f)
    }

    /** The flags of a stem that ends at ([x], [tip]): [count] of them, hanging from the stem's end towards its notes. */
    fun flags(x: Float, tip: Float, isUp: Boolean, count: Int): List<Draw> {
        val sign = if (isUp) 1f else -1f
        return (0 until count).map { index ->
            val start = tip + sign * index * 0.85f
            Draw.Polyline(
                bezier(Point(x, start), Point(x + 0.1f, start + sign * 0.9f), Point(x + 1.2f, start + sign * 1.3f), Point(x + 0.75f, start + sign * 2.5f)),
                0.26f,
            )
        }
    }

    /** A rest of [value] (1 whole, 2 half, 4 quarter...) at [x], centered on the staff. */
    fun rest(x: Float, value: Int): List<Draw> = when (value) {
        1 -> listOf(polygonOf((x - 0.6f) to 1f, (x + 0.6f) to 1f, (x + 0.6f) to 1.5f, (x - 0.6f) to 1.5f))
        2 -> listOf(polygonOf((x - 0.6f) to 1.5f, (x + 0.6f) to 1.5f, (x + 0.6f) to 2f, (x - 0.6f) to 2f))
        4 -> listOf(
            Draw.Polyline(
                listOf(
                    Point(x - 0.25f, 0.65f),
                    Point(x + 0.35f, 1.2f),
                    Point(x - 0.35f, 1.9f),
                    Point(x + 0.35f, 2.6f),
                    Point(x - 0.1f, 3.1f),
                    Point(x + 0.2f, 3.5f),
                ),
                0.3f,
            ),
        )
        else -> shortRest(x, flagCountOf(value))
    }

    /** The stem of a short rest slants from the top right to the bottom left, and each flag is a dot with a hook to the stem. */
    private fun shortRest(x: Float, count: Int): List<Draw> {
        val top = 1f - (count - 1f) * 0.45f
        val bottom = top + 1.5f + (count - 1f) * 0.9f
        val draws = mutableListOf<Draw>(Draw.Line(x + 0.45f, top, x - 0.2f, bottom, 0.14f))
        for (index in 0 until count) {
            val y = top + 0.3f + index * 0.9f
            val stemX = x + 0.45f - (y - top) * 0.43f * 0.5f
            draws += dot(stemX - 0.3f, y + 0.25f, 0.26f)
            draws += Draw.Polyline(
                bezier(Point(stemX - 0.3f, y + 0.3f), Point(stemX, y + 0.25f), Point(stemX + 0.2f, y + 0.05f), Point(stemX + 0.4f, y - 0.15f)),
                0.14f,
            )
        }
        return draws
    }

    /** A sharp centered at ([x], [y]). */
    fun sharp(x: Float, y: Float): List<Draw> = listOf(
        Draw.Line(x - 0.2f, y - 1f, x - 0.2f, y + 1.1f, 0.1f),
        Draw.Line(x + 0.2f, y - 1.1f, x + 0.2f, y + 1f, 0.1f),
        Draw.Line(x - 0.5f, y + 0.15f, x + 0.5f, y - 0.25f, 0.26f),
        Draw.Line(x - 0.5f, y + 0.7f, x + 0.5f, y + 0.3f, 0.26f),
    )

    /** A flat centered at ([x], [y]), its belly on the line the note sits on. */
    fun flat(x: Float, y: Float): List<Draw> {
        val belly = bezier(Point(x - 0.25f, y + 0.5f), Point(x + 0.2f, y + 0.15f), Point(x + 0.65f, y - 0.15f), Point(x + 0.5f, y - 0.45f)) +
            bezier(Point(x + 0.5f, y - 0.45f), Point(x + 0.4f, y - 0.8f), Point(x + 0.1f, y - 0.8f), Point(x - 0.25f, y - 0.45f))
        return listOf(Draw.Line(x - 0.25f, y - 1.5f, x - 0.25f, y + 0.5f, 0.1f), Draw.Polyline(belly, 0.2f))
    }

    /** A natural centered at ([x], [y]). */
    fun natural(x: Float, y: Float): List<Draw> = listOf(
        Draw.Line(x - 0.25f, y - 1.2f, x - 0.25f, y + 0.55f, 0.1f),
        Draw.Line(x + 0.25f, y - 0.55f, x + 0.25f, y + 1.2f, 0.1f),
        Draw.Line(x - 0.25f, y - 0.1f, x + 0.25f, y - 0.4f, 0.24f),
        Draw.Line(x - 0.25f, y + 0.45f, x + 0.25f, y + 0.15f, 0.24f),
    )

    /** The treble clef, its bowl around the second line from the bottom, which is where the G is: the clef a guitar is written in. */
    fun trebleClef(x: Float): List<Draw> {
        // One stroke: from the tail at the bottom up the spine, round the top loop, down across the spine and round the bowl.
        val path = CLEF_PATH.map { (px, py) -> Point(px, py) }
        val points = smooth(path).map { Point(x + it.x, it.y) }
        return listOf(Draw.Polyline(points, 0.17f), dot(x + 0.55f, 5.85f, 0.2f))
    }

    /** A repeat's two dots, on the spaces either side of the middle line, at [x]. */
    fun repeatDots(x: Float): List<Draw> = listOf(dot(x, 1.5f, 0.2f), dot(x, 2.5f, 0.2f))

    /**
     * The sharps or the flats of a key signature, as many as [fifths] says (positive for sharps, negative for flats) from [x], spaced
     * [gap] apart: each on the line or the space it always stands on.
     */
    fun keySignature(x: Float, fifths: Int, gap: Float): List<Draw> {
        val count = fifths.absoluteValue.coerceAtMost(7)
        return (0 until count).flatMap { index ->
            val signX = x + index * gap
            if (fifths > 0) sharp(signX, SHARPS[index]) else flat(signX, FLATS[index])
        }
    }

    /** How many flags or beams a note of [value] has. */
    fun flagCountOf(value: Int): Int = when (value) {
        8 -> 1
        16 -> 2
        32 -> 3
        else -> 4
    }

    // The staff positions in half spaces below the top line, in the order the signs are added.
    private val SHARPS = listOf(0f, 1.5f, -0.5f, 1f, 2.5f, 0.5f, 2f)
    private val FLATS = listOf(2f, 0.5f, 2.5f, 1f, 3f, 1.5f, 3.5f)

    private val CLEF_PATH = listOf(
        0.6f to 5.95f, 0.95f to 6.1f, 1.25f to 5.85f, 1.27f to 5.3f, 1.22f to 4.2f, 1.25f to 3f, 1.3f to 1.6f, 1.36f to 0.3f, 1.46f to -0.9f,
        1.56f to -1.7f, 1.36f to -2.05f, 0.95f to -1.7f, 0.6f to -0.8f, 0.55f to 0.3f, 0.75f to 1.5f, 1.2f to 2.4f, 1.75f to 3.1f,
        2.05f to 3.8f, 1.85f to 4.5f, 1.3f to 4.8f, 0.75f to 4.5f, 0.6f to 3.9f, 0.85f to 3.35f, 1.35f to 3.3f, 1.55f to 3.7f, 1.3f to 4f,
    )
}

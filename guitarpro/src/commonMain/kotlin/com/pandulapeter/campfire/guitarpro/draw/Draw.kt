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

/**
 * What a laid out score is drawn with: lines, curves, ellipses and texts, in units of the space between two lines of a staff, so that
 * whoever paints it (a window, a page) only has to scale it. Nothing in it names a color or a font: a [Tone] says how much a mark
 * matters and the painter picks what that looks like.
 */
public sealed interface Draw {

    public val tone: Tone

    /** A straight line of a thickness. */
    public data class Line(
        val x1: Float,
        val y1: Float,
        val x2: Float,
        val y2: Float,
        val width: Float,
        override val tone: Tone = Tone.INK,
    ) : Draw

    /** A line through points, drawn smooth enough for a curve to look like one. */
    public data class Polyline(
        val points: List<Point>,
        val width: Float,
        override val tone: Tone = Tone.INK,
    ) : Draw

    /** A filled shape with no hollow and no dent. */
    public data class Polygon(
        val points: List<Point>,
        override val tone: Tone = Tone.INK,
    ) : Draw

    /** An ellipse turned by [angle] (radians), filled or outlined. */
    public data class Ellipse(
        val x: Float,
        val y: Float,
        val rx: Float,
        val ry: Float,
        val angle: Float,
        val isFilled: Boolean,
        val width: Float,
        override val tone: Tone = Tone.INK,
    ) : Draw

    /**
     * A text centered on [y], [size] units tall, over a patch of the background where it [masks] the lines behind it (the number on a
     * string of tablature).
     */
    public data class Text(
        val x: Float,
        val y: Float,
        val text: String,
        val size: Float,
        val anchor: Anchor,
        val style: TextStyle,
        val masks: Boolean = false,
        override val tone: Tone = Tone.INK,
    ) : Draw
}

/** A point of a shape. */
public data class Point(val x: Float, val y: Float)

/** How much a mark stands out. */
public enum class Tone {
    /** The score itself: notes, lines, numbers. */
    INK,

    /** What stands beside it: measure numbers, the names of the strings. */
    MUTED,

    /** What the reader looks for: markers, chords, tempo. */
    ACCENT,
}

/** Where a text is anchored on the point it is given. */
public enum class Anchor {
    START,
    CENTER,
    END,
}

/** How a text is set. */
public enum class TextStyle {
    NORMAL,
    BOLD,
    ITALIC,
}

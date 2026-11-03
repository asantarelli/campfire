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

import kotlin.math.max

/** A filled dot. */
internal fun dot(x: Float, y: Float, radius: Float): Draw =
    Draw.Ellipse(x = x, y = y, rx = radius, ry = radius, angle = 0f, isFilled = true, width = 0f)

internal fun polygonOf(vararg points: Pair<Float, Float>): Draw = Draw.Polygon(points.map { Point(it.first, it.second) })

/** A text of the given look. */
internal fun textOf(
    x: Float,
    y: Float,
    text: String,
    size: Float,
    anchor: Anchor,
    style: TextStyle,
    tone: Tone,
): Draw = Draw.Text(x = x, y = y, text = text, size = size, anchor = anchor, style = style, tone = tone)

/** The same moved by ([dx], [dy]). */
internal fun Draw.moved(dx: Float, dy: Float): Draw = when (this) {
    is Draw.Line -> copy(x1 = x1 + dx, y1 = y1 + dy, x2 = x2 + dx, y2 = y2 + dy)
    is Draw.Polyline -> copy(points = points.map { Point(it.x + dx, it.y + dy) })
    is Draw.Polygon -> copy(points = points.map { Point(it.x + dx, it.y + dy) })
    is Draw.Ellipse -> copy(x = x + dx, y = y + dy)
    is Draw.Text -> copy(x = x + dx, y = y + dy)
}

/** Points along a cubic curve, enough of them for it to look smooth. */
internal fun bezier(p0: Point, p1: Point, p2: Point, p3: Point): List<Point> = (0..BEZIER_STEPS).map { step ->
    val t = step.toFloat() / BEZIER_STEPS
    val u = 1f - t
    val (a, b, c, d) = listOf(u * u * u, 3f * u * u * t, 3f * u * t * t, t * t * t)
    Point(a * p0.x + b * p1.x + c * p2.x + d * p3.x, a * p0.y + b * p1.y + c * p2.y + d * p3.y)
}

/** A smooth line through [points], which is how a shape drawn by hand (a clef) is kept to a short list of them. */
internal fun smooth(points: List<Point>): List<Point> {
    if (points.size < 3) return points
    val result = mutableListOf<Point>()
    for (index in 0 until points.size - 1) {
        val p0 = points[max(index - 1, 0)]
        val p1 = points[index]
        val p2 = points[index + 1]
        val p3 = points[minOf(index + 2, points.size - 1)]
        for (step in 0 until SMOOTH_STEPS) {
            val t = step.toFloat() / SMOOTH_STEPS
            result += Point(blend(p0.x, p1.x, p2.x, p3.x, t), blend(p0.y, p1.y, p2.y, p3.y, t))
        }
    }
    return result + points.last()
}

private fun blend(a: Float, b: Float, c: Float, d: Float, t: Float): Float {
    val t2 = t * t
    val t3 = t2 * t
    return 0.5f * (2f * b + (-a + c) * t + (2f * a - 5f * b + 4f * c - d) * t2 + (-a + 3f * b - 3f * c + d) * t3)
}

/**
 * An arc over two points, [height] high at its middle (negative above the line between them, as the y axis points down): a tie or a
 * slur.
 */
internal fun arc(x1: Float, y1: Float, x2: Float, y2: Float, height: Float, width: Float): Draw {
    val control = Point((x1 + x2) / 2f, (y1 + y2) / 2f + height * 1.33f)
    // A quadratic curve written as a cubic one.
    val c1 = Point(x1 + (control.x - x1) * 2f / 3f, y1 + (control.y - y1) * 2f / 3f)
    val c2 = Point(x2 + (control.x - x2) * 2f / 3f, y2 + (control.y - y2) * 2f / 3f)
    return Draw.Polyline(bezier(Point(x1, y1), c1, c2, Point(x2, y2)), width)
}

private const val BEZIER_STEPS = 14
private const val SMOOTH_STEPS = 8

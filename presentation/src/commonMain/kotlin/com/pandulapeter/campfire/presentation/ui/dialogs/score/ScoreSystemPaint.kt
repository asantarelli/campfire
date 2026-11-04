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

import androidx.compose.ui.geometry.Offset
import androidx.compose.ui.geometry.Size
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.Path
import androidx.compose.ui.graphics.StrokeCap
import androidx.compose.ui.graphics.StrokeJoin
import androidx.compose.ui.graphics.drawscope.DrawScope
import androidx.compose.ui.graphics.drawscope.Fill
import androidx.compose.ui.graphics.drawscope.Stroke
import androidx.compose.ui.graphics.drawscope.translate
import androidx.compose.ui.text.TextMeasurer
import androidx.compose.ui.text.TextStyle
import androidx.compose.ui.text.font.FontStyle
import androidx.compose.ui.text.drawText
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.LayoutDirection
import androidx.compose.ui.unit.sp
import com.pandulapeter.campfire.guitarpro.draw.Anchor
import com.pandulapeter.campfire.guitarpro.draw.Draw
import com.pandulapeter.campfire.guitarpro.draw.Point
import com.pandulapeter.campfire.guitarpro.draw.TextStyle as ScoreTextStyle
import com.pandulapeter.campfire.guitarpro.layout.ScoreSystem
import kotlin.math.PI
import kotlin.math.cos
import kotlin.math.max
import kotlin.math.sin

/** How many pixels the thinnest line takes, so that a thin line of a small page does not vanish. */
private const val MIN_STROKE = 1f

/** The least a text is set at, in pixels. */
private const val MIN_TEXT = 7f

private const val ELLIPSE_POINTS = 28

private val TWO_PI = (2 * PI).toFloat()

/** The room a text's mask takes around it, in pixels. */
private const val MASK_PADDING = 1.5f

/**
 * Paints one system of a laid out score. Its marks are in staff spaces counted from the top of the page, so the canvas is moved up by
 * where the system starts; [space] is how many pixels a space takes.
 */
internal fun DrawScope.drawScoreSystem(
    system: ScoreSystem,
    space: Float,
    palette: ScorePalette,
    textMeasurer: TextMeasurer,
    textStyle: TextStyle,
    metrics: ScorePaintMetrics = ScorePaintMetrics.Screen,
) {
    translate(left = 0f, top = -system.top * space) {
        for (draw in system.draws) {
            val color = palette.colorOf(draw.tone)
            when (draw) {
                is Draw.Line -> drawLine(
                    color = color,
                    start = Offset(draw.x1 * space, draw.y1 * space),
                    end = Offset(draw.x2 * space, draw.y2 * space),
                    strokeWidth = max(draw.width * space, metrics.minimumStroke),
                )

                is Draw.Polyline -> drawPath(
                    path = pathOf(draw.points, space, isClosed = false),
                    color = color,
                    style = Stroke(width = max(draw.width * space, metrics.minimumStroke), cap = StrokeCap.Round, join = StrokeJoin.Round),
                )

                is Draw.Polygon -> drawPath(path = pathOf(draw.points, space, isClosed = true), color = color, style = Fill)
                is Draw.Ellipse -> drawPath(
                    path = pathOf(ellipsePoints(draw), space, isClosed = true),
                    color = color,
                    style = if (draw.isFilled) Fill else Stroke(width = max(draw.width * space, metrics.minimumStroke)),
                )

                is Draw.Text -> drawScoreText(draw, space, color, palette, textMeasurer, textStyle, metrics)
            }
        }
    }
}

private fun pathOf(points: List<Point>, space: Float, isClosed: Boolean) = Path().apply {
    points.forEachIndexed { index, point ->
        if (index == 0) moveTo(point.x * space, point.y * space) else lineTo(point.x * space, point.y * space)
    }
    if (isClosed) close()
}

/** The outline of an ellipse turned by its angle, which is how a notehead leans. */
private fun ellipsePoints(ellipse: Draw.Ellipse): List<Point> {
    val turnSin = sin(ellipse.angle)
    val turnCos = cos(ellipse.angle)
    return (0 until ELLIPSE_POINTS).map { step ->
        val t = step.toFloat() / ELLIPSE_POINTS * TWO_PI
        val px = ellipse.rx * cos(t)
        val py = ellipse.ry * sin(t)
        Point(ellipse.x + px * turnCos - py * turnSin, ellipse.y + px * turnSin + py * turnCos)
    }
}

private fun DrawScope.drawScoreText(
    draw: Draw.Text,
    space: Float,
    color: Color,
    palette: ScorePalette,
    textMeasurer: TextMeasurer,
    textStyle: TextStyle,
    metrics: ScorePaintMetrics,
) {
    val size = max(draw.size * space, metrics.minimumText)
    val layout = textMeasurer.measure(
        text = draw.text,
        density = metrics.textDensity ?: this,
        layoutDirection = LayoutDirection.Ltr,
        style = textStyle.copy(
            color = color,
            // Where the text is measured at a density of its own (a page, whose unit is the point) the size is that unit, not a pixel count.
            fontSize = if (metrics.textDensity == null) size.toSp() else size.sp,
            fontWeight = if (draw.style == ScoreTextStyle.BOLD) FontWeight.Bold else FontWeight.Normal,
            fontStyle = if (draw.style == ScoreTextStyle.ITALIC) FontStyle.Italic else FontStyle.Normal,
        ),
        maxLines = 1,
        softWrap = false,
    )
    val width = layout.size.width.toFloat()
    val height = layout.size.height.toFloat()
    val left = draw.x * space - when (draw.anchor) {
        Anchor.START -> 0f
        Anchor.CENTER -> width / 2f
        Anchor.END -> width
    }
    val top = draw.y * space - height / 2f
    if (draw.masks) {
        drawRect(
            color = palette.background,
            topLeft = Offset(left - MASK_PADDING, top - MASK_PADDING),
            size = Size(width + 2 * MASK_PADDING, height + 2 * MASK_PADDING),
        )
    }
    drawText(textLayoutResult = layout, topLeft = Offset(left, top))
}

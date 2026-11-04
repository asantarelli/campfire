/*
 * This file is part of Campfire.
 * Copyright (c) Pandula Péter 2017-2026.
 * https://github.com/pandulapeter/campfire
 *
 * This Source Code Form is subject to the terms of the Mozilla Public License, v. 2.0.
 * If a copy of the MPL was not distributed with this file, You can obtain one at
 * https://mozilla.org/MPL/2.0/.
 */
package com.pandulapeter.campfire.chordpro.training

import kotlin.random.Random

/**
 * The practice's way of asking more of what is missed: the index of one of [weights], chosen in proportion to it, never [excluded]
 * unless it is the only one there is, so that a question is not asked twice in a row.
 */
internal fun chooseWeighted(weights: List<Double>, excluded: Int?, random: Random): Int {
    val candidates = weights.indices.filter { it != excluded }.ifEmpty { weights.indices.toList() }
    var point = random.nextDouble() * candidates.sumOf { weights[it] }
    for (candidate in candidates) {
        point -= weights[candidate]
        if (point < 0.0) return candidate
    }
    return candidates.last()
}

/** The weight of what was just answered: what was missed is asked about more, what was known less, within bounds. */
internal fun adjustedWeight(weight: Double, isHit: Boolean): Double =
    if (isHit) (weight * HIT_FACTOR).coerceAtLeast(MIN_WEIGHT) else (weight * MISS_FACTOR + MISS_BONUS).coerceAtMost(MAX_WEIGHT)

internal const val INITIAL_WEIGHT = 1.0
private const val HIT_FACTOR = 0.6
private const val MISS_FACTOR = 2.0
private const val MISS_BONUS = 1.0
private const val MIN_WEIGHT = 0.3
private const val MAX_WEIGHT = 16.0

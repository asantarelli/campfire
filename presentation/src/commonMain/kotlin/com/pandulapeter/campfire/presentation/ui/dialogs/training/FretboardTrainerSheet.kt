/*
 * This file is part of Campfire.
 * Copyright (c) Pandula Péter 2017-2026.
 * https://github.com/pandulapeter/campfire
 *
 * This Source Code Form is subject to the terms of the Mozilla Public License, v. 2.0.
 * If a copy of the MPL was not distributed with this file, You can obtain one at
 * https://mozilla.org/MPL/2.0/.
 */
package com.pandulapeter.campfire.presentation.ui.dialogs.training

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.rememberScrollState
import androidx.compose.material3.Button
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableIntStateOf
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.unit.dp
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import com.pandulapeter.campfire.chordpro.scales.Scales
import com.pandulapeter.campfire.chordpro.training.FretPosition
import com.pandulapeter.campfire.chordpro.training.FretboardTrainer
import com.pandulapeter.campfire.data.model.preferences.UserPreferences
import com.pandulapeter.campfire.presentation.localization.stringResource
import com.pandulapeter.campfire.presentation.mapper.toChordNotation
import com.pandulapeter.campfire.presentation.resources.Res
import com.pandulapeter.campfire.presentation.resources.training_correct
import com.pandulapeter.campfire.presentation.resources.training_needs_strings
import com.pandulapeter.campfire.presentation.resources.training_neck_description
import com.pandulapeter.campfire.presentation.resources.training_neck_question
import com.pandulapeter.campfire.presentation.resources.training_neck_title
import com.pandulapeter.campfire.presentation.resources.training_next
import com.pandulapeter.campfire.presentation.resources.training_wrong
import com.pandulapeter.campfire.presentation.ui.CampfireViewModel
import com.pandulapeter.campfire.presentation.ui.components.bounceVerticalScroll
import com.pandulapeter.campfire.presentation.ui.components.fadingTopEdge
import com.pandulapeter.campfire.presentation.ui.dialogs.DialogType
import com.pandulapeter.campfire.presentation.ui.dialogs.sheet.CampfireBottomSheet
import com.pandulapeter.campfire.presentation.ui.dialogs.sheet.sheetContainerColor
import com.pandulapeter.campfire.presentation.ui.instrument.instrumentTuning
import com.pandulapeter.campfire.presentation.ui.scales.ScaleDiagram
import com.pandulapeter.campfire.presentation.ui.scales.ScaleDiagramGeometry
import com.pandulapeter.campfire.presentation.ui.scales.scaleDiagramGeometryOf

/**
 * The neck trainer: a neck with one dot on it and a question mark in the dot, and the twelve notes to answer with. What is missed comes
 * back sooner (`FretboardTrainer`), and the answers are counted for this session and, in the preferences, for every one.
 */
@Composable
internal fun FretboardTrainerSheet(
    viewModel: CampfireViewModel,
    dialog: DialogType.FretboardTrainer,
) {
    val userPreferences by viewModel.userPreferences.collectAsStateWithLifecycle()
    val notation = (userPreferences?.chordSpelling ?: UserPreferences.ChordSpelling.Default).notation.toChordNotation()
    val instrument = userPreferences?.instrument ?: UserPreferences.Instrument.GUITAR
    // The tuner lists the strings from the lowest, and a neck is drawn from the highest.
    val strings = userPreferences?.instrumentTuning?.strings
    val choices = remember(notation) { noteChoices(notation) }
    CampfireBottomSheet(
        title = stringResource(Res.string.training_neck_title),
        onDismiss = { viewModel.dialogHost.dismissSheet(dialog) },
    ) { contentPadding ->
        val scrollState = rememberScrollState()
        Column(
            modifier = Modifier
                .weight(1f, fill = false)
                .fillMaxWidth()
                .fadingTopEdge(scrollState, sheetContainerColor())
                .bounceVerticalScroll(scrollState)
                .padding(contentPadding)
                .padding(top = 8.dp, bottom = 16.dp, start = 16.dp, end = 16.dp),
            verticalArrangement = Arrangement.spacedBy(16.dp),
        ) {
            val isFretted = !strings.isNullOrEmpty() && instrument !in UNTRAINED_INSTRUMENTS
            if (!isFretted || strings == null) {
                Text(
                    text = stringResource(Res.string.training_needs_strings),
                    style = MaterialTheme.typography.bodyMedium,
                    color = MaterialTheme.colorScheme.onSurfaceVariant,
                )
                return@Column
            }
            val trainer = remember(strings) { FretboardTrainer(openStrings = strings.reversed(), maxFret = TRAINED_FRETS) }
            var position by remember(trainer) { mutableStateOf(trainer.next()) }
            var answered by remember(trainer) { mutableStateOf<Int?>(null) }
            // The trainer's counters are plain fields, so what reads them is told to read them again by this.
            var revision by remember(trainer) { mutableIntStateOf(0) }
            val correct = trainer.pitchClassAt(position)
            val correctLabel = choices.first { it.pitchClass == correct }.label
            val geometry = remember(strings, instrument, position) { questionGeometry(strings, instrument, position) }
            Text(
                modifier = Modifier.fillMaxWidth(),
                text = stringResource(Res.string.training_neck_description),
                style = MaterialTheme.typography.bodyMedium,
                color = MaterialTheme.colorScheme.onSurfaceVariant,
                textAlign = TextAlign.Center,
            )
            ScaleDiagram(
                modifier = Modifier.fillMaxWidth().height((16 + 22 * strings.size).dp),
                geometry = geometry,
                labels = listOf(if (answered == null) "?" else correctLabel.substringBefore('/')),
                description = stringResource(Res.string.training_neck_question),
            )
            Text(
                modifier = Modifier.fillMaxWidth(),
                text = when (answered) {
                    null -> stringResource(Res.string.training_neck_question)
                    correct -> stringResource(Res.string.training_correct)
                    else -> stringResource(Res.string.training_wrong, correctLabel)
                },
                style = MaterialTheme.typography.titleMedium,
                color = if (answered != null && answered != correct) MaterialTheme.colorScheme.error else MaterialTheme.colorScheme.onSurface,
                textAlign = TextAlign.Center,
            )
            NoteAnswerGrid(
                choices = choices,
                answered = answered,
                correct = correct,
                onAnswered = { pitchClass ->
                    val isHit = trainer.answer(position, pitchClass)
                    answered = pitchClass
                    revision++
                    viewModel.preferencesController.recordNeckTraining(isHit)
                },
            )
            Button(
                modifier = Modifier.align(Alignment.CenterHorizontally),
                enabled = answered != null,
                onClick = {
                    position = trainer.next()
                    answered = null
                },
            ) {
                Text(stringResource(Res.string.training_next))
            }
            // Read for the revision, which is what keeps the score drawn up to date.
            TrainingScore(
                sessionHits = revision.let { trainer.hits },
                sessionAttempts = trainer.attempts,
                streak = trainer.streak,
                bestStreak = trainer.bestStreak,
                totalHits = userPreferences?.neckTrainerHits ?: 0,
                totalAttempts = userPreferences?.neckTrainerAttempts ?: 0,
            )
        }
    }
}

/** The neck of [instrument] with one dot on it, at [position], which the dot's label asks about. */
private fun questionGeometry(strings: List<Int>, instrument: UserPreferences.Instrument, position: FretPosition): ScaleDiagramGeometry {
    val neck = scaleDiagramGeometryOf(Scales.all.first { it.intervals.size == CHROMATIC_NOTES }, root = 0, strings = strings, instrument = instrument)
    return if (neck is ScaleDiagramGeometry.Fretboard) {
        neck.copy(notes = listOf(ScaleDiagramGeometry.Fretboard.Note(row = position.row, fret = position.fret, degree = 0, isRoot = true)))
    } else {
        neck
    }
}

/** The instruments with no neck of frets to practice on: a keyboard has none, a banjo's drone string starts at the fifth fret and a violin has no frets. */
private val UNTRAINED_INSTRUMENTS = setOf(
    UserPreferences.Instrument.KEYBOARD,
    UserPreferences.Instrument.OTHER,
    UserPreferences.Instrument.BANJO,
    UserPreferences.Instrument.VIOLIN,
)

/** How many notes the chromatic scale has, which is every one: the scale that puts a dot wherever the practice asks. */
private const val CHROMATIC_NOTES = 12

/** How far up the neck the practice goes: the twelfth fret, from where the notes repeat. */
private const val TRAINED_FRETS = 12

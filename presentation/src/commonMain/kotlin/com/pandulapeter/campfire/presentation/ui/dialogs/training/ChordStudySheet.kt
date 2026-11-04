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
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.rememberScrollState
import androidx.compose.material3.Button
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.CompositionLocalProvider
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
import com.pandulapeter.campfire.chordpro.model.ChordTuning
import com.pandulapeter.campfire.chordpro.training.ChordStudy
import com.pandulapeter.campfire.chordpro.training.StudyHelp
import com.pandulapeter.campfire.data.model.preferences.UserPreferences
import com.pandulapeter.campfire.presentation.localization.stringResource
import com.pandulapeter.campfire.presentation.mapper.toChordNotation
import com.pandulapeter.campfire.presentation.resources.Res
import com.pandulapeter.campfire.presentation.resources.training_correct
import com.pandulapeter.campfire.presentation.resources.training_help_blank
import com.pandulapeter.campfire.presentation.resources.training_help_fingers
import com.pandulapeter.campfire.presentation.resources.training_next
import com.pandulapeter.campfire.presentation.resources.training_study_description
import com.pandulapeter.campfire.presentation.resources.training_study_empty
import com.pandulapeter.campfire.presentation.resources.training_study_notes
import com.pandulapeter.campfire.presentation.resources.training_study_question_dot
import com.pandulapeter.campfire.presentation.resources.training_study_question_finger
import com.pandulapeter.campfire.presentation.resources.training_study_title
import com.pandulapeter.campfire.presentation.resources.training_wrong
import com.pandulapeter.campfire.presentation.ui.CampfireViewModel
import com.pandulapeter.campfire.presentation.ui.chords.ChordDiagram
import com.pandulapeter.campfire.presentation.ui.chords.ChordDiagramGeometry
import com.pandulapeter.campfire.presentation.ui.chords.ChordStudyView
import com.pandulapeter.campfire.presentation.ui.chords.LocalChordDiagramStyle
import com.pandulapeter.campfire.presentation.ui.chords.chordDiagramGeometryOf
import com.pandulapeter.campfire.presentation.ui.chords.libraryKeys
import com.pandulapeter.campfire.presentation.ui.chords.selectShape
import com.pandulapeter.campfire.presentation.ui.components.bounceVerticalScroll
import com.pandulapeter.campfire.presentation.ui.components.fadingTopEdge
import com.pandulapeter.campfire.presentation.ui.dialogs.DialogType
import com.pandulapeter.campfire.presentation.ui.dialogs.sheet.CampfireBottomSheet
import com.pandulapeter.campfire.presentation.ui.dialogs.sheet.sheetContainerColor
import com.pandulapeter.campfire.presentation.ui.instrument.chordTuning
import com.pandulapeter.campfire.presentation.ui.instrument.storedChordShapes
import com.pandulapeter.campfire.presentation.ui.screens.chords.ChordLibraryResults
import com.pandulapeter.campfire.presentation.ui.screens.chords.chordLibraryResults
import kotlin.random.Random

/** One chord of the study: its name as the Chords tab writes it and the shape it draws, with the dots the questions are about. */
private class StudyChord(val name: String, val geometry: ChordDiagramGeometry.Fretted, val targets: List<StudyTarget>)

/**
 * The study mode: the diagram of a chord of the key the Chords tab is on, with one dot marked, and the question of which note it holds.
 * The help goes as the player gets a chord right and comes back where they do not (`ChordStudy`), and a chord that is missed is asked
 * again sooner. The answers are counted for this session and, in the preferences, for every one.
 */
@Composable
internal fun ChordStudySheet(
    viewModel: CampfireViewModel,
    dialog: DialogType.ChordStudy,
) {
    val userPreferences by viewModel.userPreferences.collectAsStateWithLifecycle()
    val spelling = userPreferences?.chordSpelling ?: UserPreferences.ChordSpelling.Default
    val notation = spelling.notation.toChordNotation()
    val tuning = userPreferences?.chordTuning()
    val keys = remember(spelling.accidentals) { libraryKeys(spelling.accidentals) }
    val keyIndex = viewModel.chordLibrary.keyIndex
    val storedShapes = tuning?.let { userPreferences?.storedChordShapes(it) }.orEmpty()
    val chords = remember(keys, keyIndex, notation, tuning, storedShapes) { tuning?.let { studyChordsOf(keys, keyIndex, notation, it, storedShapes) }.orEmpty() }
    val choices = remember(notation) { noteChoices(notation) }
    CampfireBottomSheet(
        title = stringResource(Res.string.training_study_title),
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
            if (chords.isEmpty()) {
                Text(
                    text = stringResource(Res.string.training_study_empty),
                    style = MaterialTheme.typography.bodyMedium,
                    color = MaterialTheme.colorScheme.onSurfaceVariant,
                )
                return@Column
            }
            val study = remember(chords) { ChordStudy(chordCount = chords.size) }
            var chordIndex by remember(study) { mutableIntStateOf(study.nextChord()) }
            var target by remember(study) { mutableStateOf(chords[chordIndex].targets.random(Random.Default)) }
            var answered by remember(study) { mutableStateOf<Int?>(null) }
            var revision by remember(study) { mutableIntStateOf(0) }
            val chord = chords[chordIndex]
            val help = study.helpFor(chordIndex)
            val correct = target.pitchClass
            val correctLabel = choices.first { it.pitchClass == correct }.label
            Text(
                modifier = Modifier.fillMaxWidth(),
                text = stringResource(Res.string.training_study_description),
                style = MaterialTheme.typography.bodyMedium,
                color = MaterialTheme.colorScheme.onSurfaceVariant,
                textAlign = TextAlign.Center,
            )
            Text(
                modifier = Modifier.fillMaxWidth(),
                text = chord.name,
                style = MaterialTheme.typography.headlineSmall,
                color = MaterialTheme.colorScheme.onSurface,
                textAlign = TextAlign.Center,
            )
            // What the dots say is the study's, whatever the player set for the chords in the songs.
            CompositionLocalProvider(
                LocalChordDiagramStyle provides LocalChordDiagramStyle.current.copy(study = ChordStudyView(target.string, target.row, help)),
            ) {
                ChordDiagram(
                    modifier = Modifier.align(Alignment.CenterHorizontally).width(DIAGRAM_WIDTH).height(DIAGRAM_HEIGHT),
                    geometry = chord.geometry,
                    description = chord.name,
                )
            }
            Text(
                modifier = Modifier.fillMaxWidth(),
                text = when {
                    help != StudyHelp.BLANK && target.finger != null -> stringResource(Res.string.training_study_question_finger, target.finger!!)
                    else -> stringResource(Res.string.training_study_question_dot)
                },
                style = MaterialTheme.typography.titleMedium,
                textAlign = TextAlign.Center,
            )
            Text(
                modifier = Modifier.fillMaxWidth(),
                text = when (help) {
                    StudyHelp.NOTES -> stringResource(Res.string.training_study_notes, notesLine(chord, choices))
                    StudyHelp.FINGERS -> stringResource(Res.string.training_help_fingers)
                    StudyHelp.BLANK -> stringResource(Res.string.training_help_blank)
                },
                style = MaterialTheme.typography.bodyMedium,
                color = MaterialTheme.colorScheme.onSurfaceVariant,
                textAlign = TextAlign.Center,
            )
            if (answered != null) {
                Text(
                    modifier = Modifier.fillMaxWidth(),
                    text = if (answered == correct) stringResource(Res.string.training_correct) else stringResource(Res.string.training_wrong, correctLabel),
                    style = MaterialTheme.typography.titleMedium,
                    color = if (answered == correct) MaterialTheme.colorScheme.onSurface else MaterialTheme.colorScheme.error,
                    textAlign = TextAlign.Center,
                )
            }
            NoteAnswerGrid(
                choices = choices,
                answered = answered,
                correct = correct,
                onAnswered = { pitchClass ->
                    val isHit = pitchClass == correct
                    study.record(chordIndex, isHit)
                    answered = pitchClass
                    revision++
                    viewModel.preferencesController.recordChordStudy(isHit)
                },
            )
            Button(
                modifier = Modifier.align(Alignment.CenterHorizontally),
                enabled = answered != null,
                onClick = {
                    chordIndex = study.nextChord()
                    target = chords[chordIndex].targets.random(Random.Default)
                    answered = null
                },
            ) {
                Text(stringResource(Res.string.training_next))
            }
            // Read for the revision, which is what keeps the score drawn up to date.
            TrainingScore(
                sessionHits = revision.let { study.hits },
                sessionAttempts = study.attempts,
                streak = study.streak,
                bestStreak = study.bestStreak,
                totalHits = userPreferences?.chordStudyHits ?: 0,
                totalAttempts = userPreferences?.chordStudyAttempts ?: 0,
            )
        }
    }
}

/** The chords the Chords tab lists for its key that have a fretted shape with something to ask about. */
private fun studyChordsOf(
    keys: List<String>,
    keyIndex: Int,
    notation: com.pandulapeter.campfire.chordpro.chords.ChordNotation,
    tuning: ChordTuning,
    storedShapes: Map<String, String>,
): List<StudyChord> {
    val results = chordLibraryResults(keys, keyIndex, query = "", notation = notation, tuning = tuning, searchesShapes = false) as? ChordLibraryResults.Chords
    // The tab lists a key's common chords first, which are the ones worth knowing by heart; the rare ones come after them.
    return results?.chords.orEmpty().take(STUDIED_CHORDS).mapNotNull { chord ->
        val shape = selectShape(chord, tuning, storedShapes).shape ?: return@mapNotNull null
        val geometry = chordDiagramGeometryOf(shape, tuning, chord.diagramRoot) as? ChordDiagramGeometry.Fretted ?: return@mapNotNull null
        val targets = studyTargetsOf(geometry)
        if (targets.isEmpty()) null else StudyChord(chord.name, geometry, targets)
    }
}

/** The notes of a chord as the help lists them, in the order of its strings. */
private fun notesLine(chord: StudyChord, choices: List<NoteChoice>) =
    notesOf(chord.geometry).joinToString(" ") { pitchClass -> choices.first { it.pitchClass == pitchClass }.label.substringBefore('/') }

/** How many of the key's chords are studied: the major, minor, seventh, major seventh, minor seventh and the two suspended ones. */
private const val STUDIED_CHORDS = 7

private val DIAGRAM_WIDTH = 180.dp
private val DIAGRAM_HEIGHT = 220.dp

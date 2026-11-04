<!--
 * This file is part of Campfire.
 * Copyright (c) Pandula Péter 2017-2026.
 * https://github.com/pandulapeter/campfire
 *
 * This Source Code Form is subject to the terms of the Mozilla Public License, v. 2.0.
 * If a copy of the MPL was not distributed with this file, You can obtain one at
 * https://mozilla.org/MPL/2.0/.
-->
# :presentation — ui/dialogs/training

Two practice sheets, each opened by `PracticeButton` beside a tool's search: the neck trainer from the Scales tab
(`FretboardTrainerSheet`) and the study mode from the Chords tab (`ChordStudySheet`). What is asked, and what is asked more
of, is `:chordpro`'s `training` package; a sheet draws the question and takes the answer.

- **The neck trainer** puts one dot on the neck (the scale diagram's own, with one note) and asks its note among twelve
  buttons (`NoteAnswerGrid`). It is for the instruments with a neck of frets: not a keyboard, a banjo (its drone string
  starts at the fifth fret) or a violin.
- **The study mode asks about the dots of a chord of the key the Chords tab is on**, the seven the tab lists first (the
  ones worth knowing by heart). The question is the marked dot's note. The diagram says no more than the help allows
  (`ChordStudyView` in `ChordDiagramStyle`): the root's color, the notes and the colors by function are all left out, or the
  diagram would answer. The help is the chord's: two right answers take one away, a wrong one gives one back
  (`StudyHelp`: the fingers and the chord's notes listed, the fingers alone, nothing).
- A black key is answered by both of its names (`noteChoices`), since a question has one answer to both spellings.
- Every answer is counted for the session (the sheet's own state, gone with it) and added to the totals the preferences keep
  (`neckTrainerHits`, `chordStudyAttempts` and their pairs, `recordNeckTraining`), which are never exported or synced.

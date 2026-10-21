// This file is part of Campfire.
// Copyright (c) Pandula Péter 2017-2026.
// https://github.com/pandulapeter/campfire
//
// This Source Code Form is subject to the terms of the Mozilla Public License, v. 2.0.
// If a copy of the MPL was not distributed with this file, You can obtain one at
// https://mozilla.org/MPL/2.0/.

//! The song model: what a ChordPro file says, never how it looks. Immutable once parsed.
//!
//! A port of the Kotlin `model` package, without the parts the Rust build leaves out (chord definitions, the tab retuner's
//! notices and mid-song tempo changes). Positions are counted in characters, not in UTF-16 units.

use std::collections::BTreeMap;

/// A parsed ChordPro song: the directives that describe it plus the blocks that make up its body.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Song {
    pub metadata: Metadata,
    pub blocks: Vec<Block>,
}

impl Song {
    /// True if any lyrics or grid line contains at least one real chord (annotations don't count), or a tab holds a staff.
    pub fn has_chords(&self) -> bool {
        self.blocks.iter().any(|block| match block {
            Block::Section(section) => section.lines.iter().any(|line| match line {
                Line::Lyrics(lyrics) => lyrics.chords.iter().any(|chord| !chord.is_annotation),
                Line::Grid(grid) => grid.tokens.iter().any(|token| matches!(token, GridToken::Chord(_))),
                Line::Tab(tab) => crate::tokens::is_staff_line(&tab.text),
                Line::Blank => false,
            }),
            _ => false,
        })
    }
}

/// The metadata directives of a song. Everything is optional; fallbacks (such as the file name becoming the title) are the
/// caller's job, not the parser's.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Metadata {
    pub title: Option<String>,
    pub subtitle: Option<String>,
    pub artist: Option<String>,
    pub composer: Option<String>,
    pub lyricist: Option<String>,
    pub album: Option<String>,
    pub year: Option<String>,
    /// The address of the song's cover image, from the first `{meta: cover https://…}` directive that holds one.
    pub cover_art: Option<String>,
    /// `{key}`, kept as written, e.g. `Am` or `Bb`.
    pub key: Option<String>,
    pub capo: Option<i32>,
    pub tempo: Option<String>,
    /// `{time}`, e.g. `3/4`.
    pub time: Option<String>,
    pub duration: Option<String>,
    /// The `{transpose}` before the song's first line, applied by the renderer on top of the user's transposition.
    pub transpose: i32,
    /// The labels the song was filed under, in file order, without duplicates.
    pub tags: Vec<String>,
    /// The languages the song is sung in, as ISO codes folded to lowercase and cut down to their primary subtag.
    pub languages: Vec<String>,
    /// Pages about the song, one per `{meta: link https://… Optional name}` directive, each address once.
    pub links: Vec<Link>,
    /// `{meta: name value}` and unknown `x_*` directives, in order.
    pub custom: BTreeMap<String, Vec<String>>,
}

impl Metadata {
    /// The title as the app shows it, wherever a song is named: `{title}`, or `fallback` where the file declares none, with
    /// `{subtitle}` after it in parentheses. A subtitle that only repeats the title is dropped.
    pub fn display_title(&self, fallback: &str) -> String {
        let name = self.title.as_deref().filter(|title| !title.trim().is_empty()).unwrap_or(fallback);
        match self.subtitle.as_deref().filter(|subtitle| !subtitle.trim().is_empty() && subtitle.to_lowercase() != name.to_lowercase()) {
            Some(suffix) => format!("{name} ({suffix})"),
            None => name.to_string(),
        }
    }
}

/// A page about a song, carried as `{meta: link https://… Optional name}`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Link {
    pub url: String,
    pub name: Option<String>,
}

/// One top-level element of a song body.
#[derive(Debug, Clone, PartialEq)]
pub enum Block {
    /// An environment or an implicit paragraph. Its lines may switch between lyrics, tablature and grids as the song does.
    Section(Section),
    /// `{chorus}`: repeat the most recent chorus. `blocks` is that chorus as the parser found it, or empty where no chorus
    /// came before.
    ChorusRecall { label: Option<String>, blocks: Vec<Block> },
    /// `{comment}` and its styled forms.
    Comment(Comment),
    /// `{transpose: N}` somewhere after the song has begun: from here on the chords are read `semitones` away from where the
    /// song's own transposition puts them. `key` is the song's `{key}`, for a viewer to name the change by.
    Transpose { semitones: i32, key: Option<String> },
    /// `{column_break}` / `{new_page}` and friends: a hint that the layout may break here.
    Break,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Section {
    pub section_type: SectionType,
    /// `Verse 1` from `{start_of_verse: Verse 1}`.
    pub label: Option<String>,
    pub lines: Vec<Line>,
    /// Whether this is the rest of the section before it, which a comment, a break or a chorus recall standing inside that
    /// section cut in two. It has no heading of its own to show.
    pub is_continuation: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Comment {
    pub text: String,
    pub style: CommentStyle,
    pub placement: CommentPlacement,
    /// Whether it was written inside a `{start_of_tab}` or a `{start_of_grid}` that has a line of its own.
    pub is_in_tab_or_grid: bool,
    /// Whether that environment is a `{start_of_tab}`.
    pub is_in_tab: bool,
    /// The selector of that `{start_of_tab}`.
    pub tab_selector: Option<String>,
}

impl Comment {
    pub fn new(text: String, style: CommentStyle) -> Self {
        Comment { text, style, placement: CommentPlacement::BetweenSections, is_in_tab_or_grid: false, is_in_tab: false, tab_selector: None }
    }
}

/// What a section of a song *is*, not how its lines are written down.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SectionType {
    Verse,
    Chorus,
    Bridge,
    /// `{start_of_<name>}` for any other name, e.g. `intro`, `solo`, `outro`, `pre-chorus`.
    Custom(String),
    /// Lines outside any environment, grouped by blank lines. Rendered like a verse without a label.
    Paragraph,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CommentStyle {
    Plain,
    Italic,
    Box,
}

/// Which section a [`Comment`] was written in.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CommentPlacement {
    /// Outside every environment, where it belongs to no section.
    BetweenSections,
    /// Inside a section before its first line, so it belongs to the section that follows it.
    StartOfSection,
    /// Inside a section after a line of it, so it belongs to the section before it.
    InSection,
}

/// One line inside a [`Block::Section`].
#[derive(Debug, Clone, PartialEq)]
pub enum Line {
    /// A lyrics line: text with chords anchored at character offsets.
    Lyrics(Lyrics),
    /// One line inside `{start_of_tab}`: monospaced, transposed on the frets, not on the notes.
    Tab(Tab),
    /// One line inside `{start_of_grid}`: tokens separated by whitespace.
    Grid(Grid),
    /// An empty line inside an environment.
    Blank,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Lyrics {
    pub text: String,
    pub chords: Vec<LyricChord>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LyricChord {
    /// Offset into the text, in characters, where the chord sits.
    pub position: usize,
    /// `Am7`, `N.C.`, …
    pub name: String,
    /// `[*text]` annotations: shown like a chord, never transposed.
    pub is_annotation: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Tab {
    pub text: String,
    /// Whether the tab line before this one in the section was written in the same `{start_of_tab}` environment.
    pub continues_environment: bool,
    /// The label of the `{start_of_tab}` the line was written in, on every line of it.
    pub label: Option<String>,
    /// The selector of the `{start_of_tab}` the line was written in: the instrument it is for.
    pub selector: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Grid {
    pub tokens: Vec<GridToken>,
    /// The label of the `{start_of_grid}` the line was written in, on every line of it.
    pub label: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GridToken {
    /// `|`, `||`, `|.`, `|:`, `:|`, `:|:`, voltas such as `|1` and `:|2>`.
    Bar(String),
    /// `Am`, or `C~G` for several chords in one cell.
    Chord(String),
    /// `.`
    Beat,
    /// `%` (repeat previous cell), `%%`.
    Repeat(String),
    /// A margin label before the first bar, a comment after the last one, or a `/` chord position.
    Text(String),
}

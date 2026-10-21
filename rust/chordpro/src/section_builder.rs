// This file is part of Campfire.
// Copyright (c) Pandula Péter 2017-2026.
// https://github.com/pandulapeter/campfire
//
// This Source Code Form is subject to the terms of the Mozilla Public License, v. 2.0.
// If a copy of the MPL was not distributed with this file, You can obtain one at
// https://mozilla.org/MPL/2.0/.

//! Accumulates the lines of the section that is currently open.
//!
//! Sections are appended to the blocks when they close, so that a comment or a break appearing inside one can be emitted in
//! the right order.

use crate::model::{Block, Comment, CommentPlacement, Grid, Line, SectionType, Tab};
use crate::parser;
use crate::tokens;

/// How the lines of an environment are written down, which says nothing about what section they are in.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum LineMode {
    Tab,
    Grid,
    Verbatim,
}

pub(crate) struct SectionBuilder {
    pub(crate) blocks: Vec<Block>,
    section_type: Option<SectionType>,
    label: Option<String>,
    line_mode: Option<LineMode>,
    /// The label of the tab or grid environment that is open, which each of its lines carries.
    line_mode_label: Option<String>,
    /// The selector of the tab environment that is open, which each of its lines and comments carries.
    line_mode_selector: Option<String>,
    lines: Vec<Line>,
    /// Whether `lines` holds anything but blank lines, kept as they are added.
    pub(crate) has_content_line: bool,
    /// The `{comment}` a legacy heading section was opened by, which is what is left of it if no line ever follows.
    heading_text: Option<String>,
    pub(crate) is_explicit: bool,
    is_continuation: bool,
    /// Whether the tab environment that is open has had a line yet.
    has_tab_line: bool,
    /// Whether a part of this section was emitted already, so that a block added now stands inside it.
    has_emitted_lines: bool,
    /// Where in `blocks` the comments this section opened with are, which `close` moves out of it if no line follows.
    opening_comments: Vec<usize>,
    /// Whether the tab or grid environment that is open has had a line that is not blank.
    has_line_mode_line: bool,
    line_mode_comments: Vec<usize>,
    /// Whether the running paragraph was opened by the tab or grid environment that is open.
    is_opened_by_line_mode: bool,
}

impl SectionBuilder {
    pub(crate) fn new() -> Self {
        SectionBuilder {
            blocks: Vec::new(),
            section_type: None,
            label: None,
            line_mode: None,
            line_mode_label: None,
            line_mode_selector: None,
            lines: Vec::new(),
            has_content_line: false,
            heading_text: None,
            is_explicit: false,
            is_continuation: false,
            has_tab_line: false,
            has_emitted_lines: false,
            opening_comments: Vec::new(),
            has_line_mode_line: false,
            line_mode_comments: Vec::new(),
            is_opened_by_line_mode: false,
        }
    }

    /// Whether a `{start_of_tab}`, a `{start_of_grid}` or one of the verbatim environments is open.
    pub(crate) fn is_in_line_mode(&self) -> bool {
        self.line_mode.is_some()
    }

    pub(crate) fn open(&mut self, section_type: SectionType, label: Option<String>, is_explicit: bool, heading_text: Option<String>, is_continuation: bool) {
        self.section_type = Some(section_type);
        self.label = label;
        self.is_explicit = is_explicit;
        self.heading_text = heading_text;
        self.is_continuation = is_continuation;
        self.has_emitted_lines = is_continuation;
        self.has_tab_line = false;
        self.is_opened_by_line_mode = false;
        self.opening_comments.clear();
        self.lines.clear();
        self.has_content_line = false;
    }

    /// A tab or grid environment starts. It never opens a section of its own, but it does need one to live in, so a file that
    /// puts tablature outside every environment gets the same implicit paragraph a bare line of lyrics would get — carrying
    /// the environment's own label as its heading. Inside a section that is already running the section's own heading wins,
    /// and the environment's label is carried by its lines instead.
    pub(crate) fn open_line_mode(&mut self, mode: LineMode, label: Option<String>, selector: Option<String>) {
        if self.section_type.is_none() {
            self.open(SectionType::Paragraph, label.clone(), false, None, false);
            self.is_opened_by_line_mode = true;
        }
        self.line_mode = Some(mode);
        self.line_mode_label = label;
        self.line_mode_selector = if mode == LineMode::Tab { selector } else { None };
        self.has_tab_line = false;
        self.has_line_mode_line = false;
        self.line_mode_comments.clear();
    }

    /// An environment that opened a paragraph of its own and wrote nothing in it leaves nothing behind: its label names that
    /// environment, and a line after it would otherwise be headed by it.
    pub(crate) fn close_line_mode(&mut self) {
        self.finish_line_mode();
        self.line_mode = None;
        self.line_mode_label = None;
        self.line_mode_selector = None;
        if self.is_opened_by_line_mode && self.lines.iter().all(|line| *line == Line::Blank) {
            self.close();
        }
        self.is_opened_by_line_mode = false;
    }

    pub(crate) fn close(&mut self) {
        self.finish_line_mode();
        self.line_mode = None;
        self.line_mode_label = None;
        self.line_mode_selector = None;
        let Some(section_type) = self.section_type.take() else {
            return;
        };
        while self.lines.last() == Some(&Line::Blank) {
            self.lines.pop();
        }
        if !self.lines.is_empty() {
            self.blocks.push(Block::Section(crate::model::Section {
                section_type,
                label: self.label.take(),
                lines: std::mem::take(&mut self.lines),
                is_continuation: self.is_continuation,
            }));
        } else {
            if !self.has_emitted_lines {
                for &index in &self.opening_comments {
                    if let Block::Comment(comment) = &mut self.blocks[index] {
                        comment.placement = CommentPlacement::BetweenSections;
                    }
                }
            }
            if let Some(heading) = self.heading_text.take() {
                self.blocks.push(Block::Comment(Comment::new(heading, crate::model::CommentStyle::Plain)));
            }
        }
        self.opening_comments.clear();
        self.label = None;
        self.is_explicit = false;
        self.is_continuation = false;
        self.has_tab_line = false;
        self.is_opened_by_line_mode = false;
        self.has_emitted_lines = false;
        self.heading_text = None;
        self.lines.clear();
        self.has_content_line = false;
    }

    /// A comment in a tab or grid environment that never had a line is not a note about one: there is nothing it would be
    /// hidden with, and nothing for the serializer to write it back inside.
    fn finish_line_mode(&mut self) {
        if self.line_mode.is_none() {
            return;
        }
        if !self.has_line_mode_line {
            for &index in &self.line_mode_comments {
                if let Block::Comment(comment) = &mut self.blocks[index] {
                    comment.is_in_tab_or_grid = false;
                    comment.is_in_tab = false;
                    comment.tab_selector = None;
                }
            }
        }
        self.line_mode_comments.clear();
    }

    /// Emits a standalone block without losing the section around it: the section is flushed and then reopened. Only a
    /// section with lines in it is flushed. A tab or grid environment that is open carries on in the reopened half, which is
    /// marked as the continuation it is, so that it is neither headed a second time nor left out of a recall of its chorus.
    pub(crate) fn add_block(&mut self, block: Block) {
        // Blank lines alone are not a part of the section yet: flushed, they would be trimmed away, and the lines after the
        // block would be the continuation of a section that never started.
        let is_cut = self.section_type.is_some() && self.has_content_line;
        let placed = match block {
            Block::Comment(comment) => Block::Comment(self.placed(comment, is_cut)),
            other => other,
        };
        if is_cut {
            let section_type = self.section_type.clone().expect("a cut section is open");
            let label = self.label.clone();
            let is_explicit = self.is_explicit;
            // The block interrupts the section and not the tab or grid environment the file is in the middle of: that one
            // ends at its own `{end_of_…}`.
            let line_mode = self.line_mode.take();
            let line_mode_label = self.line_mode_label.clone();
            let line_mode_selector = self.line_mode_selector.clone();
            let has_tab_line = self.has_tab_line;
            self.close();
            self.blocks.push(placed.clone());
            self.open(section_type, label, is_explicit, None, true);
            self.line_mode = line_mode;
            self.line_mode_label = line_mode_label;
            self.line_mode_selector = line_mode_selector;
            self.has_tab_line = has_tab_line;
        } else {
            self.blocks.push(placed.clone());
        }
        if let Block::Comment(comment) = &placed {
            let index = self.blocks.len() - 1;
            if comment.placement == CommentPlacement::StartOfSection {
                self.opening_comments.push(index);
            }
            if comment.is_in_tab_or_grid {
                self.line_mode_comments.push(index);
            }
        }
    }

    /// Where a comment written now stands. Only an environment is a section to the reader: the lines of an implicit paragraph
    /// are just lines, unless a legacy heading named them.
    fn placed(&self, mut comment: Comment, is_cut: bool) -> Comment {
        let is_in_section = self
            .section_type
            .as_ref()
            .is_some_and(|section_type| self.is_explicit || self.line_mode.is_some() || *section_type != SectionType::Paragraph);
        comment.placement = if !is_in_section {
            CommentPlacement::BetweenSections
        } else if is_cut || self.has_emitted_lines {
            CommentPlacement::InSection
        } else {
            CommentPlacement::StartOfSection
        };
        comment.is_in_tab_or_grid = matches!(self.line_mode, Some(LineMode::Tab | LineMode::Grid));
        comment.is_in_tab = self.line_mode == Some(LineMode::Tab);
        comment.tab_selector = self.line_mode_selector.clone();
        comment
    }

    pub(crate) fn add_content(&mut self, raw_line: &str, trimmed_line: &str) {
        if trimmed_line.is_empty() {
            if self.section_type.is_some() {
                if self.is_explicit || self.line_mode.is_some() {
                    self.lines.push(Line::Blank);
                } else {
                    // A blank line ends an implicit paragraph or a legacy heading section.
                    self.close();
                }
            }
            return;
        }
        if self.section_type.is_none() {
            self.open(SectionType::Paragraph, None, false, None, false);
        }
        if matches!(self.line_mode, Some(LineMode::Tab | LineMode::Grid)) {
            self.has_line_mode_line = true;
        }
        let line = match self.line_mode {
            Some(LineMode::Tab) => {
                let line = Line::Tab(Tab {
                    text: raw_line.to_string(),
                    continues_environment: self.has_tab_line,
                    label: self.line_mode_label.clone(),
                    selector: self.line_mode_selector.clone(),
                });
                self.has_tab_line = true;
                line
            }
            Some(LineMode::Grid) => Line::Grid(Grid { tokens: tokens::parse_grid_tokens(trimmed_line), label: self.line_mode_label.clone() }),
            Some(LineMode::Verbatim) => Line::Lyrics(crate::model::Lyrics { text: raw_line.to_string(), chords: Vec::new() }),
            None => Line::Lyrics(parser::parse_lyrics(raw_line)),
        };
        self.lines.push(line);
        self.has_content_line = true;
    }
}

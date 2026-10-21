// This file is part of Campfire.
// Copyright (c) Pandula Péter 2017-2026.
// https://github.com/pandulapeter/campfire
//
// This Source Code Form is subject to the terms of the Mozilla Public License, v. 2.0.
// If a copy of the MPL was not distributed with this file, You can obtain one at
// https://mozilla.org/MPL/2.0/.

//! Resolves what every `{chorus}` of a parsed song repeats.
//!
//! The recalls of a song together repeat at most [`RECALL_BUDGET_FLOOR`] plus [`RECALL_BUDGET_PER_WEIGHT`] times the weight
//! of what the song writes down itself; a recall past that carries no chorus and is drawn as its heading alone. Every later
//! step expands each recall on its own, so without a bound a small file of recalls is millions of lines to all of them.

use crate::model::{Block, CommentPlacement, Line, SectionType};

const RECALL_BUDGET_FLOOR: usize = 64_000;
const RECALL_BUDGET_PER_WEIGHT: usize = 2;
/// What a line, a comment or a break weighs before its characters: the cost of a row, however short.
const RECALL_PIECE_WEIGHT: usize = 8;

/// `blocks` with every recall carrying the chorus it repeats: the last one that was over by the time the recall is reached,
/// all of it — a chorus cut in two by a comment is its first section, the comment and the continuation. A recall standing
/// inside a chorus repeats the chorus before that one, since the one it stands in is not over yet. A recall inside the
/// recalled chorus is left out of the copy, and so is a `{transpose}`: a recalled chorus is played at the transposition of
/// the place it is recalled at.
pub(crate) fn with_choruses_recalled(blocks: Vec<Block>) -> Vec<Block> {
    if !blocks.iter().any(|block| matches!(block, Block::ChorusRecall { .. })) {
        return blocks;
    }
    // Every chorus, by the index of its last piece; ascending, since they are found in order.
    let mut choruses: Vec<(usize, Vec<Block>, usize)> = Vec::new();
    let mut pieces: Option<Vec<Block>> = None;
    let mut last_piece_index: Option<usize> = None;
    for (index, block) in blocks.iter().enumerate() {
        let Block::Section(section) = block else {
            continue;
        };
        if section.is_continuation && pieces.is_some() {
            let chorus = pieces.as_mut().expect("checked above");
            let from = last_piece_index.map_or(0, |last| last + 1);
            chorus.extend(
                blocks[from..index]
                    .iter()
                    .filter(|piece| !matches!(piece, Block::ChorusRecall { .. } | Block::Transpose { .. }))
                    .cloned(),
            );
            chorus.push(block.clone());
        } else {
            if let (Some(chorus), Some(last)) = (pieces.take(), last_piece_index) {
                choruses.push(recalled_chorus(last, chorus, &blocks));
            }
            pieces = (section.section_type == SectionType::Chorus).then(|| {
                let mut opening = comments_opening(&blocks, index);
                opening.push(block.clone());
                opening
            });
        }
        last_piece_index = Some(index);
    }
    if let (Some(chorus), Some(last)) = (pieces.take(), last_piece_index) {
        choruses.push(recalled_chorus(last, chorus, &blocks));
    }
    let mut budget = RECALL_BUDGET_FLOOR + RECALL_BUDGET_PER_WEIGHT * blocks.iter().map(recall_weight).sum::<usize>();
    let mut chorus_index: Option<usize> = None;
    blocks
        .iter()
        .enumerate()
        .map(|(index, block)| {
            let Block::ChorusRecall { label, .. } = block else {
                return block.clone();
            };
            let mut next = chorus_index.map_or(0, |current| current + 1);
            while next < choruses.len() && choruses[next].0 < index {
                chorus_index = Some(next);
                next += 1;
            }
            let empty = || Block::ChorusRecall { label: label.clone(), blocks: Vec::new() };
            let Some((_, chorus, weight)) = chorus_index.and_then(|current| choruses.get(current)) else {
                return empty();
            };
            // A recall past the budget keeps its heading and repeats nothing; it is never cut in the middle, and a later,
            // smaller chorus may still fit.
            if *weight > budget {
                return empty();
            }
            budget -= weight;
            Block::ChorusRecall { label: label.clone(), blocks: chorus.clone() }
        })
        .collect()
}

/// A chorus whose last piece is at `last_piece_index`, with its weight, weighed once however often it is recalled.
fn recalled_chorus(last_piece_index: usize, mut pieces: Vec<Block>, blocks: &[Block]) -> (usize, Vec<Block>, usize) {
    pieces.extend(comments_ending(blocks, last_piece_index));
    let weight = pieces.iter().map(recall_weight).sum();
    (last_piece_index, pieces, weight)
}

/// What a recall of this block makes every later step do: its rows, its characters and its chords.
fn recall_weight(block: &Block) -> usize {
    match block {
        Block::Section(section) => section.label.as_ref().map_or(0, |label| label.chars().count()) + section.lines.iter().map(line_weight).sum::<usize>(),
        Block::Comment(comment) => RECALL_PIECE_WEIGHT + comment.text.chars().count(),
        // Its own line, not what it repeats, which is what the budget is spent on.
        Block::ChorusRecall { label, .. } => RECALL_PIECE_WEIGHT + label.as_ref().map_or(0, |label| label.chars().count()),
        _ => RECALL_PIECE_WEIGHT,
    }
}

fn line_weight(line: &Line) -> usize {
    RECALL_PIECE_WEIGHT
        + match line {
            Line::Lyrics(lyrics) => lyrics.text.chars().count() + lyrics.chords.iter().map(|chord| chord.name.chars().count()).sum::<usize>(),
            Line::Tab(tab) => tab.text.chars().count(),
            Line::Grid(grid) => grid.tokens.len(),
            Line::Blank => 0,
        }
}

/// The comments the section at `index` opens with, which are part of the chorus a recall repeats.
fn comments_opening(blocks: &[Block], index: usize) -> Vec<Block> {
    let before = &blocks[..index];
    let count = before
        .iter()
        .rev()
        .take_while(|block| matches!(block, Block::Comment(comment) if comment.placement == CommentPlacement::StartOfSection))
        .count();
    before[before.len() - count..].to_vec()
}

/// The comments the section whose last piece is at `index` ends with, which are part of the chorus a recall repeats.
fn comments_ending(blocks: &[Block], index: usize) -> Vec<Block> {
    blocks[index + 1..]
        .iter()
        .take_while(|block| {
            matches!(block, Block::Break | Block::Transpose { .. })
                || matches!(block, Block::Comment(comment) if comment.placement == CommentPlacement::InSection)
        })
        .filter(|block| matches!(block, Block::Comment(_)))
        .cloned()
        .collect()
}

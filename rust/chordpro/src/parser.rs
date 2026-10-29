// This file is part of Campfire.
// Copyright (c) Pandula Péter 2017-2026.
// https://github.com/pandulapeter/campfire
//
// This Source Code Form is subject to the terms of the Mozilla Public License, v. 2.0.
// If a copy of the MPL was not distributed with this file, You can obtain one at
// https://mozilla.org/MPL/2.0/.

//! Turns ChordPro text into a [`Song`].
//!
//! A port of the Kotlin `ChordProParser`. The parser is total: it never fails or rejects, since the file is the user's and
//! half of it may be under the caret.

use crate::chorus_recalls;
use crate::directives::{self, Directive};
use crate::metadata_builder::MetadataBuilder;
use crate::model::{Block, Comment, CommentStyle, LyricChord, Lyrics, Metadata, SectionType, Song};
use crate::notation::ChordNotation;
use crate::section_builder::{LineMode, SectionBuilder};
use crate::{normalize, values};

/// The directives that make a block of their own and so cut whatever section they stand in into two.
const BLOCK_NAMES: [&str; 14] = [
    "chorus", "comment", "c", "comment_italic", "ci", "comment_box", "cb", "highlight", "new_page", "np", "new_physical_page", "npp",
    "column_break", "colb",
];

/// The song in the notation the app works in, `text` being written in `notation`: a file is in the standard one, which an `H`
/// chord anywhere in it overrules, and the editor's field in the reader's own.
pub fn parse(text: &str, notation: ChordNotation) -> Song {
    normalize::normalized(parse_as_written(text), notation)
}

/// The song with every chord spelled the way its file spells it.
pub fn parse_as_written(text: &str) -> Song {
    let mut metadata = MetadataBuilder::new();
    let mut section = SectionBuilder::new();
    let mut transposition = Transposition::new();
    for line in directives::scan(text) {
        if line.is_source_comment {
            continue;
        }
        match &line.directive {
            None => {
                if !line.trimmed.is_empty() {
                    transposition.start_body();
                }
                section.add_content(&line.raw, &line.trimmed);
            }
            Some(directive) => handle_directive(directive, &mut metadata, &mut section, &mut transposition),
        }
    }
    section.close();
    transposition.finish();
    let declared = metadata.build(transposition.whole_song);
    // Only known once the whole file has been read, since a song may name its key under its last line.
    let key = declared.key.clone().filter(|key| !key.trim().is_empty());
    let blocks = section
        .blocks
        .into_iter()
        .map(|block| match block {
            Block::Transpose { semitones, .. } => Block::Transpose { semitones, key: key.clone() },
            other => other,
        })
        .collect();
    Song { metadata: declared, blocks: chorus_recalls::with_choruses_recalled(blocks) }
}

/// The directives of a song, read without the body.
pub fn parse_metadata(text: &str) -> Metadata {
    let mut metadata = MetadataBuilder::new();
    let mut transposition = Transposition::new();
    for line in directives::scan(text) {
        if line.is_source_comment {
            continue;
        }
        let Some(directive) = line.directive else {
            if !line.trimmed.is_empty() {
                transposition.start_body();
            }
            continue;
        };
        if directives::has_selector_suffix(&directive.name) {
            continue;
        }
        if directive.name == "transpose" {
            transposition.consume(directive.value.as_deref());
        } else if starts_body(&directive) {
            transposition.start_body();
        }
        metadata.consume(&directive, transposition.is_in_body);
    }
    transposition.finish();
    metadata.build(transposition.whole_song)
}

/// The semitones a non-empty `{transpose}` value asks for, or `None` where it is not a number. `2s` and `-3f` ask for sharps
/// or flats as well; the reader's own spelling preference decides that here.
pub fn transpose_semitones(written: &str) -> Option<i32> {
    let trimmed = written.trim();
    let trimmed = trimmed.strip_prefix('+').unwrap_or(trimmed);
    let without_suffix = match trimmed.chars().last() {
        Some(last) if matches!(last.to_ascii_lowercase(), 's' | 'f') => &trimmed[..trimmed.len() - last.len_utf8()],
        _ => trimmed,
    };
    without_suffix.trim().parse().ok()
}

/// Whether a directive begins the body of a song.
pub(crate) fn starts_body(directive: &Directive) -> bool {
    directives::start_of_environment(&directive.name).is_some() || BLOCK_NAMES.contains(&directive.name.as_str())
}

fn handle_directive(directive: &Directive, metadata: &mut MetadataBuilder, section: &mut SectionBuilder, transposition: &mut Transposition) {
    let name = directive.name.as_str();
    // A definition's shape is kept for the diagram of a chord the app's own tables have no name for; `{chord}` only says which
    // chord to draw, which the app already does.
    let base = name.split('-').next().unwrap_or(name);
    if base == "define" {
        if let Some(definition) = directive.value.as_deref().and_then(crate::definitions::parse) {
            metadata.add_definition(definition);
        }
        return;
    }
    if base == "chord" {
        return;
    }
    if directives::has_selector_suffix(name) {
        return;
    }
    if name == "transpose" {
        // Inside a section the modulation cuts it in two, the way a comment does, and the rest is its continuation.
        if let Some(semitones) = transposition.consume(directive.value.as_deref()) {
            section.add_block(Block::Transpose { semitones, key: None });
        }
        return;
    }
    if starts_body(directive) {
        transposition.start_body();
    }
    if let Some(environment) = directives::start_of_environment(name) {
        // Tablature and grids are how the next few lines are written, not a section of their own: they open inside whatever
        // section is running.
        if let Some(mode) = line_mode(&environment) {
            section.open_line_mode(mode, directives::label(directive.value.as_deref()), directives::selector_of(name));
            return;
        }
        section.close();
        section.open(section_type(&environment), directives::label(directive.value.as_deref()), true, None, false);
        if directives::is_delegate_environment(&environment.to_lowercase()) {
            section.open_line_mode(LineMode::Verbatim, None, None);
        }
        return;
    }
    if let Some(environment) = directives::end_of_environment(name) {
        if line_mode(&environment).is_none() {
            section.close();
        } else {
            section.close_line_mode();
        }
        return;
    }
    let value = directive.value.as_deref().unwrap_or("").trim();
    match name {
        "chorus" => {
            let recall = Block::ChorusRecall { label: directives::label(directive.value.as_deref()), blocks: Vec::new() };
            if section.is_in_line_mode() {
                // The environment still has lines to come, so the recall interrupts it the way a comment does.
                section.add_block(recall);
            } else {
                section.close();
                section.blocks.push(recall);
            }
        }
        "comment" | "c" => handle_comment(value, CommentStyle::Plain, section),
        "comment_italic" | "ci" => handle_comment(value, CommentStyle::Italic, section),
        "comment_box" | "cb" => handle_comment(value, CommentStyle::Box, section),
        // Never a Campfire 3 heading: that dialect wrote its headings as `{comment}` only.
        "highlight" => section.add_block(Block::Comment(Comment::new(value.to_string(), CommentStyle::Plain))),
        "new_page" | "np" | "new_physical_page" | "npp" | "column_break" | "colb" => section.add_block(Block::Break),
        // Splitting is the splitter's job.
        "new_song" | "ns" => {}
        _ => metadata.consume(directive, transposition.is_in_body),
    }
}

fn handle_comment(text: &str, style: CommentStyle, section: &mut SectionBuilder) {
    // Inside a tab or a grid a comment is a note to the player: a Campfire 3 file had its headings between its sections, and
    // taking this one for a heading would end the environment that is still open around it.
    if style == CommentStyle::Plain && !section.is_explicit && !section.is_in_line_mode() {
        if let Some(section_type) = legacy_heading(text) {
            section.close();
            section.open(section_type, Some(text.to_string()), false, Some(text.to_string()), false);
            return;
        }
    }
    section.add_block(Block::Comment(Comment::new(text.to_string(), style)));
}

/// Whether a plain `{comment}` reading `text` is a Campfire 3 heading, which opens a section rather than cutting one.
pub fn is_legacy_heading(text: &str) -> bool {
    legacy_heading(text.trim()).is_some()
}

/// The section names the Campfire 3 dialect used as `{comment}` headings.
fn legacy_heading(text: &str) -> Option<SectionType> {
    let first_word: String = text.chars().take_while(|c| c.is_alphabetic() || *c == '-').collect();
    Some(match first_word.to_lowercase().as_str() {
        "intro" => SectionType::Custom("intro".into()),
        "verse" => SectionType::Verse,
        "pre-chorus" => SectionType::Custom("pre-chorus".into()),
        "chorus" => SectionType::Chorus,
        "bridge" => SectionType::Bridge,
        "solo" => SectionType::Custom("solo".into()),
        "outro" => SectionType::Custom("outro".into()),
        _ => return None,
    })
}

fn section_type(environment: &str) -> SectionType {
    match environment.to_lowercase().as_str() {
        "verse" => SectionType::Verse,
        "chorus" => SectionType::Chorus,
        "bridge" => SectionType::Bridge,
        other => SectionType::Custom(other.to_string()),
    }
}

/// The way an environment says its lines are written, or `None` for the environments that are sections.
fn line_mode(environment: &str) -> Option<LineMode> {
    match environment.to_lowercase().as_str() {
        "tab" => Some(LineMode::Tab),
        "grid" => Some(LineMode::Grid),
        _ => None,
    }
}

/// A lyrics line: its text with the chords in brackets taken out and anchored at the character offset they stood at.
pub fn parse_lyrics(raw_line: &str) -> Lyrics {
    let chars: Vec<char> = raw_line.chars().collect();
    let mut text = String::new();
    let mut text_length = 0;
    let mut chords = Vec::new();
    let mut consumed_until = 0;
    for bracket in directives::brackets(raw_line) {
        let before: String = chars[consumed_until..bracket.open].iter().collect();
        text_length += bracket.open - consumed_until;
        text.push_str(&before);
        let content = bracket.content.trim();
        if !content.is_empty() {
            let is_annotation = content.starts_with('*');
            chords.push(LyricChord {
                position: text_length,
                name: if is_annotation { content[1..].to_string() } else { content.to_string() },
                is_annotation,
            });
        }
        consumed_until = bracket.close + 1;
    }
    text.extend(&chars[consumed_until..]);
    Lyrics { text, chords }
}

/// The `{transpose}` directives of a song, the way the spec reads them: each one is the transposition of the rest of the song
/// (not an addition to the one before it), and one with no value goes back to the one before it. What a song opens with —
/// every `{transpose}` before its first line — is the whole song's; a later one is a modulation.
struct Transposition {
    restorable: Vec<i32>,
    current: i32,
    last_reported: i32,
    /// The transposition of the whole song: what was in effect when the body began, or at the end of a song with none.
    whole_song: i32,
    /// Whether the body has begun, before which every directive is part of the header.
    is_in_body: bool,
}

impl Transposition {
    fn new() -> Self {
        Transposition { restorable: Vec::new(), current: 0, last_reported: 0, whole_song: 0, is_in_body: false }
    }

    fn start_body(&mut self) {
        if self.is_in_body {
            return;
        }
        self.is_in_body = true;
        self.whole_song = self.current;
        self.last_reported = self.current;
    }

    /// Reads one directive; the offset from the whole song's the chords after it are at, where that changed in the body.
    fn consume(&mut self, value: Option<&str>) -> Option<i32> {
        let written = value.map(str::trim).unwrap_or("");
        if written.is_empty() {
            self.current = self.restorable.pop().unwrap_or(0);
        } else {
            let semitones = transpose_semitones(written)?;
            self.restorable.push(self.current);
            self.current = semitones;
        }
        if !self.is_in_body || self.current == self.last_reported {
            return None;
        }
        self.last_reported = self.current;
        Some(self.current - self.whole_song)
    }

    fn finish(&mut self) {
        self.start_body();
    }
}

/// Whether `text` holds a readable tempo, a helper the editor uses to mark an unreadable one.
pub fn is_readable_tempo(text: &str) -> bool {
    values::parse_tempo(text).is_some()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{Line, Section};

    fn song(text: &str) -> Song {
        parse(text, ChordNotation::Standard)
    }

    fn sections(song: &Song) -> Vec<&Section> {
        song.blocks.iter().filter_map(|block| if let Block::Section(section) = block { Some(section) } else { None }).collect()
    }

    fn lyrics(line: &Line) -> &Lyrics {
        match line {
            Line::Lyrics(lyrics) => lyrics,
            other => panic!("{other:?} is no lyrics line"),
        }
    }

    #[test]
    fn chords_are_taken_out_of_the_text_and_anchored() {
        let song = song("[Am]Hello [G]world\nplain\n");
        let section = &sections(&song)[0];
        assert_eq!(section.section_type, SectionType::Paragraph);
        let first = lyrics(&section.lines[0]);
        assert_eq!(first.text, "Hello world");
        assert_eq!(first.chords.iter().map(|c| (c.position, c.name.as_str())).collect::<Vec<_>>(), vec![(0, "Am"), (6, "G")]);
        assert_eq!(lyrics(&section.lines[1]).chords.len(), 0);
    }

    #[test]
    fn an_annotation_is_marked_and_an_empty_bracket_is_dropped() {
        let song = song("[*softly]Hi [] there\n");
        let line = lyrics(&sections(&song)[0].lines[0]);
        assert_eq!(line.text, "Hi  there");
        assert_eq!(line.chords.len(), 1);
        assert!(line.chords[0].is_annotation);
        assert_eq!(line.chords[0].name, "softly");
    }

    #[test]
    fn positions_count_characters_not_bytes() {
        let song = song("Canción [Am]muy ñoño\n");
        let line = lyrics(&sections(&song)[0].lines[0]);
        assert_eq!(line.text, "Canción muy ñoño");
        assert_eq!(line.chords[0].position, 8);
    }

    #[test]
    fn a_header_and_two_environments_make_a_song() {
        let song = song(
            "{title: Test}\n{artist: Someone}\n{key: G}\n{start_of_verse: Verse 1}\n[G]One\n{end_of_verse}\n{soc}\n[C]Two\n{eoc}\n{chorus}\n",
        );
        assert_eq!(song.metadata.title.as_deref(), Some("Test"));
        assert_eq!(song.metadata.artist.as_deref(), Some("Someone"));
        assert_eq!(song.metadata.key.as_deref(), Some("G"));
        let found = sections(&song);
        assert_eq!(found.len(), 2);
        assert_eq!(found[0].section_type, SectionType::Verse);
        assert_eq!(found[0].label.as_deref(), Some("Verse 1"));
        assert_eq!(found[1].section_type, SectionType::Chorus);
        match &song.blocks[2] {
            Block::ChorusRecall { blocks, .. } => assert_eq!(blocks.len(), 1),
            other => panic!("{other:?}"),
        }
    }

    #[test]
    fn a_blank_line_ends_an_implicit_paragraph_but_not_an_environment() {
        let song = song("one\n\ntwo\n{sov}\nthree\n\nfour\n{eov}\n");
        let found = sections(&song);
        assert_eq!(found.len(), 3);
        assert_eq!(found[2].lines.len(), 3);
        assert_eq!(found[2].lines[1], Line::Blank);
    }

    #[test]
    fn a_comment_cuts_its_section_and_the_rest_is_a_continuation() {
        let song = song("{sov}\n[G]one\n{c: Slowly}\n[C]two\n{eov}\n");
        assert_eq!(song.blocks.len(), 3);
        match &song.blocks[1] {
            Block::Comment(comment) => assert_eq!(comment.placement, crate::model::CommentPlacement::InSection),
            other => panic!("{other:?}"),
        }
        let found = sections(&song);
        assert!(!found[0].is_continuation);
        assert!(found[1].is_continuation);
    }

    #[test]
    fn a_comment_before_any_line_belongs_to_the_next_section() {
        let song = song("{sov}\n{c: Softly}\n[G]one\n{eov}\n");
        match &song.blocks[0] {
            Block::Comment(comment) => assert_eq!(comment.placement, crate::model::CommentPlacement::StartOfSection),
            other => panic!("{other:?}"),
        }
        let loose = self::song("{c: Hello}\n[G]one\n");
        match &loose.blocks[0] {
            Block::Comment(comment) => assert_eq!(comment.placement, crate::model::CommentPlacement::BetweenSections),
            other => panic!("{other:?}"),
        }
    }

    #[test]
    fn tablature_and_grids_live_inside_the_running_section() {
        let song = song("{sov}\n[Am]riff\n{sot: Riff}\ne|--0--|\ne|--3--|\n{eot}\n{sog}\n| Am . | C . |\n{eog}\n{eov}\n");
        let found = sections(&song);
        assert_eq!(found.len(), 1);
        let kinds: Vec<&str> = found[0]
            .lines
            .iter()
            .map(|line| match line {
                Line::Lyrics(_) => "lyrics",
                Line::Tab(_) => "tab",
                Line::Grid(_) => "grid",
                Line::Blank => "blank",
            })
            .collect();
        assert_eq!(kinds, ["lyrics", "tab", "tab", "grid"]);
        match (&found[0].lines[1], &found[0].lines[2]) {
            (Line::Tab(first), Line::Tab(second)) => {
                assert!(!first.continues_environment);
                assert!(second.continues_environment);
                assert_eq!(first.label.as_deref(), Some("Riff"));
            }
            other => panic!("{other:?}"),
        }
    }

    #[test]
    fn delegated_environments_keep_their_text_and_their_braces() {
        let song = song("{start_of_abc}\nX:1\n[CEG]\n{ c d e }\n{end_of_abc}\n");
        let found = sections(&song);
        assert_eq!(found.len(), 1);
        assert_eq!(found[0].section_type, SectionType::Custom("abc".into()));
        assert_eq!(lyrics(&found[0].lines[1]).text, "[CEG]");
        assert_eq!(lyrics(&found[0].lines[2]).text, "{ c d e }");
        assert!(!song.has_chords());
    }

    #[test]
    fn a_legacy_heading_opens_a_section() {
        let song = song("{comment: Verse 1}\n[G]one\n\n{comment: Chorus}\n[C]two\n");
        let found = sections(&song);
        assert_eq!(found[0].section_type, SectionType::Verse);
        assert_eq!(found[0].label.as_deref(), Some("Verse 1"));
        assert_eq!(found[1].section_type, SectionType::Chorus);
        assert!(is_legacy_heading("Bridge 2"));
        assert!(!is_legacy_heading("Slowly"));
    }

    #[test]
    fn a_transposition_before_the_first_line_is_the_songs_and_a_later_one_a_modulation() {
        let song = song("{transpose: 2}\n{key: G}\n[G]one\n{transpose: 4}\n[C]two\n{transpose}\n[D]three\n");
        assert_eq!(song.metadata.transpose, 2);
        let modulations: Vec<i32> = song
            .blocks
            .iter()
            .filter_map(|block| if let Block::Transpose { semitones, key } = block { assert_eq!(key.as_deref(), Some("G")); Some(*semitones) } else { None })
            .collect();
        assert_eq!(modulations, vec![2, 0]);
    }

    #[test]
    fn a_transpose_value_may_ask_for_a_spelling() {
        assert_eq!(transpose_semitones("+2"), Some(2));
        assert_eq!(transpose_semitones("-3f"), Some(-3));
        assert_eq!(transpose_semitones("2S"), Some(2));
        assert_eq!(transpose_semitones("two"), None);
    }

    #[test]
    fn source_comments_and_unknown_directives_never_reach_the_song() {
        let song = song("# a note to self\n{unknown: x}\n{title: T}\n[G]sing\n");
        assert_eq!(sections(&song).len(), 1);
        assert_eq!(song.metadata.title.as_deref(), Some("T"));
        assert!(song.metadata.custom.is_empty());
    }

    #[test]
    fn selector_suffixes_and_definitions_are_read_past() {
        let song = song("{title: T}\n{title-guitar: Other}\n{define: Am base-fret 1 frets 0 2 2 1 0 0}\n{chord: G}\n[G]sing\n");
        assert_eq!(song.metadata.title.as_deref(), Some("T"));
        assert_eq!(sections(&song).len(), 1);
    }

    #[test]
    fn the_metadata_alone_is_read_without_the_body() {
        let metadata = parse_metadata("{t: A}\n{st: B}\n[G]body\n{key: Z}\n");
        assert_eq!(metadata.title.as_deref(), Some("A"));
        assert_eq!(metadata.subtitle.as_deref(), Some("B"));
        assert_eq!(metadata.display_title("file"), "A (B)");
        assert_eq!(metadata.key.as_deref(), Some("Z"));
    }

    #[test]
    fn it_never_fails_on_what_is_not_a_song() {
        for text in ["", "{", "}", "[", "{{{", "[[[]]]", "{start_of_verse", "{end_of_verse}\n{eoc}", "\u{feff}", "{sot}\n{c: x}\n"] {
            let _ = song(text);
        }
    }

    #[test]
    fn an_empty_environment_leaves_nothing_behind() {
        let song = song("{sot: Riff}\n{eot}\n[G]after\n");
        let found = sections(&song);
        assert_eq!(found.len(), 1);
        assert_eq!(found[0].label, None);
    }
}

// This file is part of Campfire.
// Copyright (c) Pandula Péter 2017-2026.
// https://github.com/pandulapeter/campfire
//
// This Source Code Form is subject to the terms of the Mozilla Public License, v. 2.0.
// If a copy of the MPL was not distributed with this file, You can obtain one at
// https://mozilla.org/MPL/2.0/.

//! How a line is read as a directive, where the chords in brackets are, and the environments directives open.
//!
//! A port of the Kotlin `ChordProDirectives`, `ChordProEnvironments` and `ChordProLineScanner`, which are the one definition
//! of the dialect.

const START_OF_PREFIX: &str = "start_of_";
const END_OF_PREFIX: &str = "end_of_";
const CUSTOM_PREFIX: &str = "x_";

/// Every directive name ChordPro defines, used to detect (and drop) selector suffixes such as `title-guitar` and to accept a
/// value separated from the name by whitespace alone.
const KNOWN_NAMES: &[&str] = &[
    "title", "t", "subtitle", "st", "artist", "composer", "lyricist", "album", "year", "key", "capo", "tempo", "time", "duration",
    "transpose", "tag", "language", "lang", "meta", "chorus", "comment", "c", "comment_italic", "ci", "comment_box", "cb",
    "new_page", "np", "new_physical_page", "npp", "column_break", "colb", "new_song", "ns", "define", "chord", "image", "columns",
    "col", "highlight", "pagetype", "titles", "sorttitle", "arranger", "copyright", "grid", "g", "no_grid", "ng", "diagrams",
    "textfont", "tf", "textsize", "ts", "textcolour", "chordfont", "cf", "chordsize", "cs", "chordcolour",
    "tabfont", "tabsize", "tabcolour", "gridfont", "gridsize", "gridcolour", "titlefont", "titlesize", "titlecolour",
];

const START_SHORT_NAMES: [(&str, &str); 5] = [("soc", "chorus"), ("sov", "verse"), ("sob", "bridge"), ("sot", "tab"), ("sog", "grid")];
const END_SHORT_NAMES: [(&str, &str); 5] = [("eoc", "chorus"), ("eov", "verse"), ("eob", "bridge"), ("eot", "tab"), ("eog", "grid")];

/// The environments ChordPro hands to another program — ABC and LilyPond notation, SVG, a block of preformatted text. Their
/// lines are that program's input rather than lyrics, kept line for line, and nothing in them is a chord.
pub const DELEGATE_ENVIRONMENTS: [&str; 4] = ["abc", "ly", "svg", "textblock"];

const SELECTABLE_ENVIRONMENTS: [&str; 9] = ["chorus", "verse", "bridge", "tab", "grid", "abc", "ly", "svg", "textblock"];

/// A directive as a line spells it: its lowercase name and its value, `None` where it has none.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Directive {
    pub name: String,
    pub value: Option<String>,
}

/// One closed bracket pair and its untrimmed content, as character offsets of its brackets.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Bracket {
    pub open: usize,
    pub close: usize,
    pub content: String,
}

pub fn is_delegate_environment(environment: &str) -> bool {
    DELEGATE_ENVIRONMENTS.contains(&environment)
}

/// [`match_directive`] for a line of an environment ChordPro hands to another program, whose own syntax is full of braces
/// and `#`: only the `{end_of_…}` that closes it and a directive written with a colon are directives there.
pub fn match_delegated_directive(trimmed_line: &str) -> Option<Directive> {
    let (name, value_start) = walk_directive(trimmed_line)?;
    let chars: Vec<char> = trimmed_line.chars().collect();
    let has_colon = value_start.is_some_and(|start| start > 0 && chars[start - 1] == ':');
    if end_of_environment(&name).is_some() || has_colon {
        match_directive(trimmed_line)
    } else {
        None
    }
}

/// Where the value of the directive on `trimmed_line` starts, as a character index: right after its colon, or at the first
/// character after the whitespace that separates it from a name written without one. `None` for a line that is not a directive
/// and for one with no value.
pub fn value_start(trimmed_line: &str) -> Option<usize> {
    walk_directive(trimmed_line)?.1
}

/// Whether the value of the directive `name` is text the song shows (a comment, the label of a section or of a chorus recall)
/// rather than a setting. Its brackets are then chords, as they are in a line of lyrics: that is where an intro is written
/// down as a row of chords.
pub fn has_chords_in_value(name: &str) -> bool {
    matches!(name, "chorus" | "comment" | "c" | "comment_italic" | "ci" | "comment_box" | "cb" | "highlight")
        || start_of_environment(name).is_some_and(|environment| !is_delegate_environment(&environment.to_lowercase()))
}

/// Matches a directive in one linear walk, which keeps malformed input cheap while the user types.
pub fn match_directive(trimmed_line: &str) -> Option<Directive> {
    let (name, value_start) = walk_directive(trimmed_line)?;
    let chars: Vec<char> = trimmed_line.chars().collect();
    let value = value_start.map(|start| chars[start..chars.len() - 1].iter().collect::<String>().trim().to_string());
    Some(Directive { name, value })
}

/// The lowercase name of the directive on `trimmed_line` and the character index its value starts at, `None` where it has
/// none, or `None` altogether where the line is no directive.
///
/// ChordPro separates a value from the name with a colon "and/or whitespace". Whitespace alone is only taken for a name the
/// app knows, though: a line in braces such as `{Verse 2}` has always been shown as the lyrics it is.
fn walk_directive(trimmed_line: &str) -> Option<(String, Option<usize>)> {
    let chars: Vec<char> = trimmed_line.chars().collect();
    let close = chars.len().checked_sub(1)?;
    if close < 1 || chars[0] != '{' || chars[close] != '}' {
        return None;
    }
    let mut index = 1;
    while index < close && chars[index].is_whitespace() {
        index += 1;
    }
    let name_start = index;
    while index < close && is_directive_name_character(chars[index]) {
        index += 1;
    }
    if index == name_start {
        return None;
    }
    let name_end = index;
    let written: String = chars[name_start..name_end].iter().collect::<String>().to_lowercase();
    // A negated selector holds wherever nothing matches the selector, which in Campfire is everywhere, so the directive is
    // read as the one it is written on. A `!` anywhere else is not part of a directive name.
    let name = if written.contains('!') { negated_selector_base(&written)? } else { written };
    while index < close && chars[index].is_whitespace() {
        index += 1;
    }
    if index == close {
        Some((name, None))
    } else if chars[index] == ':' {
        Some((name, Some(index + 1)))
    } else if index > name_end && is_known_name(&name) {
        Some((name, Some(index)))
    } else {
        None
    }
}

fn is_directive_name_character(c: char) -> bool {
    c.is_ascii_alphanumeric() || c == '_' || c == '-' || c == '!'
}

/// `title` for `title-guitar!`: the known directive a negated selector is written on, or `None`. A tab's selector is the one
/// Campfire matches, so `start_of_tab-guitar!` is kept whole for the parser to read.
fn negated_selector_base(name: &str) -> Option<String> {
    if !name.ends_with('!') || name.matches('!').count() != 1 {
        return None;
    }
    if start_of_environment(name).is_some_and(|environment| environment.to_lowercase() == "tab") {
        return Some(name.to_string());
    }
    let without_bang = &name[..name.len() - 1];
    let base = without_bang.rfind('-').map(|index| &without_bang[..index]).unwrap_or("");
    (!base.is_empty() && is_known_name(base)).then(|| base.to_string())
}

/// Whether `name` is a directive ChordPro defines, a custom `x_` one or one of those with a selector suffix.
fn is_known_name(name: &str) -> bool {
    KNOWN_NAMES.contains(&name)
        || start_of_environment(name).is_some()
        || end_of_environment(name).is_some()
        || name.starts_with(CUSTOM_PREFIX)
        || has_selector_suffix(name)
}

/// True for names such as `title-guitar`: a known directive with a (non-negated) selector suffix, which Campfire matches
/// nothing against and drops. Environments are the exception.
pub fn has_selector_suffix(name: &str) -> bool {
    if name.starts_with(START_OF_PREFIX) || name.starts_with(END_OF_PREFIX) {
        return false;
    }
    match name.rfind('-') {
        Some(index) if index > 0 => KNOWN_NAMES.contains(&&name[..index]),
        _ => false,
    }
}

/// Every closed bracket pair from left to right; an unclosed opening bracket ends the walk.
pub fn brackets(line: &str) -> Vec<Bracket> {
    let chars: Vec<char> = line.chars().collect();
    let mut brackets = Vec::new();
    let mut index = 0;
    while let Some(open) = chars[index..].iter().position(|&c| c == '[').map(|offset| index + offset) {
        let Some(close) = chars[open + 1..].iter().position(|&c| c == ']').map(|offset| open + 1 + offset) else {
            break;
        };
        brackets.push(Bracket { open, close, content: chars[open + 1..close].iter().collect() });
        index = close + 1;
    }
    brackets
}

/// Whether `line` contains at least one closed bracket pair.
pub fn has_brackets(line: &str) -> bool {
    match line.find('[') {
        Some(open) => line[open + 1..].contains(']'),
        None => false,
    }
}

/// The environment name of a `{start_of_x}` / `{soc}` style directive, or `None` if this is not one.
pub fn start_of_environment(name: &str) -> Option<String> {
    environment_of(name, &START_SHORT_NAMES, START_OF_PREFIX)
}

/// The environment name of an `{end_of_x}` / `{eoc}` style directive, or `None` if this is not one.
pub fn end_of_environment(name: &str) -> Option<String> {
    environment_of(name, &END_SHORT_NAMES, END_OF_PREFIX)
}

fn environment_of(name: &str, short_names: &[(&str, &str)], prefix: &str) -> Option<String> {
    if let Some((_, environment)) = short_names.iter().find(|(short, _)| *short == name) {
        return Some((*environment).to_string());
    }
    let rest = name.strip_prefix(prefix).filter(|rest| !rest.is_empty())?;
    Some(without_selector(rest.strip_suffix('!').unwrap_or(rest)))
}

/// `environment` without a selector suffix. An environment is part of the song itself, so `{start_of_chorus-guitar}` is read
/// as the chorus it selects rather than left out. A tab's selector is kept on its lines by [`selector_of`].
fn without_selector(environment: &str) -> String {
    let base = environment.rfind('-').map(|index| &environment[..index]).unwrap_or(environment);
    if SELECTABLE_ENVIRONMENTS.contains(&base) {
        base.to_string()
    } else {
        environment.to_string()
    }
}

/// The selector of a `{start_of_x-selector}` directive named `name` (`guitar` for `start_of_tab-guitar`, `guitar!` for the
/// negated `start_of_tab-guitar!`), or `None` for one without a selector or no start of an environment at all.
pub fn selector_of(name: &str) -> Option<String> {
    let environment = name.strip_prefix(START_OF_PREFIX)?;
    let separator = environment.rfind('-').filter(|&index| index > 0)?;
    if !SELECTABLE_ENVIRONMENTS.contains(&&environment[..separator]) {
        return None;
    }
    let selector = &environment[separator + 1..];
    (!selector.is_empty() && selector != "!").then(|| selector.to_string())
}

/// The label an environment directive gives its section: the whole value (`{sov: Verse 1}`), or its `label` attribute where
/// the value is written as attributes (`{sov label="Verse 1"}`, in either quotes). A value made of attributes that has no label
/// gives none, rather than showing the attributes as a heading. An empty label is `None`.
pub fn label(value: Option<&str>) -> Option<String> {
    let trimmed = value?.trim();
    if trimmed.is_empty() {
        return None;
    }
    match attributes(trimmed) {
        None => Some(trimmed.to_string()),
        Some(attributes) => attributes.into_iter().find(|(name, _)| name == "label").map(|(_, value)| value).filter(|value| !value.is_empty()),
    }
}

/// `trimmed` as a run of `name="value"` attributes, or `None` where it is anything else.
fn attributes(trimmed: &str) -> Option<Vec<(String, String)>> {
    let chars: Vec<char> = trimmed.chars().collect();
    let mut attributes = Vec::new();
    let mut index = 0;
    while index < chars.len() {
        while chars.get(index).is_some_and(|c| c.is_whitespace()) {
            index += 1;
        }
        let name_start = index;
        if !chars.get(index).is_some_and(|c| c.is_ascii_alphabetic() || *c == '_') {
            return None;
        }
        while chars.get(index).is_some_and(|c| c.is_ascii_alphanumeric() || *c == '_' || *c == '-') {
            index += 1;
        }
        let name: String = chars[name_start..index].iter().collect();
        while chars.get(index).is_some_and(|c| c.is_whitespace()) {
            index += 1;
        }
        if chars.get(index) != Some(&'=') {
            return None;
        }
        index += 1;
        while chars.get(index).is_some_and(|c| c.is_whitespace()) {
            index += 1;
        }
        let quote = *chars.get(index).filter(|&&c| c == '"' || c == '\'')?;
        let value_start = index + 1;
        let value_end = chars[value_start..].iter().position(|&c| c == quote).map(|offset| value_start + offset)?;
        attributes.push((name, chars[value_start..value_end].iter().collect()));
        index = value_end + 1;
    }
    Some(attributes)
}

/// Splits a file into its lines accepting `\r\n`, a bare `\r` and `\n`; a trailing line break does not create an extra line.
pub fn split_lines(text: &str) -> Vec<String> {
    let normalized = text.replace("\r\n", "\n").replace('\r', "\n");
    let mut lines: Vec<String> = normalized.split('\n').map(str::to_string).collect();
    if lines.len() > 1 && lines.last().is_some_and(String::is_empty) {
        lines.pop();
    }
    lines
}

/// One line of the text and how the parser reads it.
#[derive(Debug, Clone)]
pub struct ScannedLine {
    pub raw: String,
    pub trimmed: String,
    /// The lowercase environment open *before* this line (`tab`, `grid`, `abc`, …), or `None`.
    pub environment: Option<String>,
    /// A `#` line outside an environment handed to another program, which never reaches the song.
    pub is_source_comment: bool,
    /// The directive this line is, read by the rule of its environment.
    pub directive: Option<Directive>,
}

/// Every line of `text`, with the environment each stands in: a `{start_of_…}` opens its environment and any `{end_of_…}`
/// closes the one that is open, whichever environment it names. Inside an environment handed to another program a `#` and a
/// brace are that program's syntax.
pub fn scan(text: &str) -> Vec<ScannedLine> {
    let mut environment: Option<String> = None;
    split_lines(text)
        .into_iter()
        .map(|raw| {
            let trimmed = raw.trim().to_string();
            let is_delegated = environment.as_deref().is_some_and(is_delegate_environment);
            let is_source_comment = !is_delegated && trimmed.starts_with('#');
            let directive = if is_source_comment {
                None
            } else if is_delegated {
                match_delegated_directive(&trimmed)
            } else {
                match_directive(&trimmed)
            };
            let before = environment.clone();
            if let Some(directive) = &directive {
                if let Some(opened) = start_of_environment(&directive.name) {
                    environment = Some(opened.to_lowercase());
                } else if end_of_environment(&directive.name).is_some() {
                    environment = None;
                }
            }
            ScannedLine { raw, trimmed, environment: before, is_source_comment, directive }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn directive(line: &str) -> Option<(String, Option<String>)> {
        match_directive(line).map(|directive| (directive.name, directive.value))
    }

    #[test]
    fn a_directive_has_a_name_and_a_value() {
        assert_eq!(directive("{title: Amazing Grace}"), Some(("title".into(), Some("Amazing Grace".into()))));
        assert_eq!(directive("{t:X}"), Some(("t".into(), Some("X".into()))));
        assert_eq!(directive("{Title: X}"), Some(("title".into(), Some("X".into()))));
        assert_eq!(directive("{title X}"), Some(("title".into(), Some("X".into()))));
        assert_eq!(directive("{chorus}"), Some(("chorus".into(), None)));
        assert_eq!(directive("{ soc }"), Some(("soc".into(), None)));
    }

    #[test]
    fn braces_with_prose_are_no_directive() {
        for line in ["{Verse 2}", "{}", "{ : x}", "not {a} directive", "{unknown: value}x"] {
            assert_eq!(directive(line), None, "{line}");
        }
        // An unknown name with a colon is still read, and dropped by the parser.
        assert_eq!(directive("{unknown: value}"), Some(("unknown".into(), Some("value".into()))));
    }

    #[test]
    fn a_negated_selector_is_read_as_the_directive_it_is_written_on() {
        assert_eq!(directive("{title-guitar!: X}"), Some(("title".into(), Some("X".into()))));
        assert_eq!(directive("{start_of_tab-guitar!}").map(|d| d.0), Some("start_of_tab-guitar!".into()));
        assert_eq!(directive("{a!b: x}"), None);
    }

    #[test]
    fn environments_are_named_with_or_without_a_selector() {
        assert_eq!(start_of_environment("sov").as_deref(), Some("verse"));
        assert_eq!(start_of_environment("start_of_chorus-guitar").as_deref(), Some("chorus"));
        assert_eq!(start_of_environment("start_of_pre-chorus").as_deref(), Some("pre-chorus"));
        assert_eq!(end_of_environment("eot").as_deref(), Some("tab"));
        assert_eq!(selector_of("start_of_tab-guitar").as_deref(), Some("guitar"));
        assert_eq!(selector_of("start_of_tab-guitar!").as_deref(), Some("guitar!"));
        assert_eq!(selector_of("start_of_pre-chorus"), None);
        assert_eq!(selector_of("start_of_tab"), None);
    }

    #[test]
    fn a_label_is_the_value_or_its_label_attribute() {
        assert_eq!(label(Some("Verse 1")).as_deref(), Some("Verse 1"));
        assert_eq!(label(Some("label=\"Verse 1\"")).as_deref(), Some("Verse 1"));
        assert_eq!(label(Some("label='Verse 1' shape=\"1+4x2\"")).as_deref(), Some("Verse 1"));
        assert_eq!(label(Some("shape=\"1+4x2+4\"")), None);
        assert_eq!(label(Some("  ")), None);
        assert_eq!(label(None), None);
    }

    #[test]
    fn brackets_are_closed_pairs_and_an_unclosed_one_ends_the_walk() {
        let found = brackets("[Am]Hello [G]world [C");
        assert_eq!(found.len(), 2);
        assert_eq!((found[0].open, found[0].close, found[0].content.as_str()), (0, 3, "Am"));
        assert_eq!(found[1].content, "G");
        assert!(has_brackets("a [b] c"));
        assert!(!has_brackets("a ] b ["));
    }

    #[test]
    fn the_scanner_follows_environments_and_leaves_delegated_text_alone() {
        let lines = scan("# source\n{sot}\ne|--0--|\n{eot}\n{start_of_abc}\n# not a comment\n{ c d e }\n{end_of_abc}\nlast\n");
        assert!(lines[0].is_source_comment);
        assert_eq!(lines[2].environment.as_deref(), Some("tab"));
        assert_eq!(lines[4].directive.as_ref().map(|d| d.name.as_str()), Some("start_of_abc"));
        assert!(!lines[5].is_source_comment);
        assert_eq!(lines[6].directive, None);
        assert_eq!(lines[7].directive.as_ref().map(|d| d.name.as_str()), Some("end_of_abc"));
        assert_eq!(lines[8].environment, None);
        assert_eq!(lines.len(), 9);
    }

    #[test]
    fn lines_are_split_on_every_kind_of_break() {
        assert_eq!(split_lines("a\r\nb\rc\nd\n"), vec!["a", "b", "c", "d"]);
        assert_eq!(split_lines(""), vec![""]);
        assert_eq!(split_lines("\n"), vec![""]);
    }
}

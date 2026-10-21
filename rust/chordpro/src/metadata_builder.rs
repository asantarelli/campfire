// This file is part of Campfire.
// Copyright (c) Pandula Péter 2017-2026.
// https://github.com/pandulapeter/campfire
//
// This Source Code Form is subject to the terms of the Mozilla Public License, v. 2.0.
// If a copy of the MPL was not distributed with this file, You can obtain one at
// https://mozilla.org/MPL/2.0/.

//! Collects the metadata directives of a song while the parser walks it, with the rules of which line of a field wins.

use std::collections::{BTreeMap, HashSet};

use crate::directives::Directive;
use crate::model::{Link, Metadata};
use crate::values;

/// The value of a field a song can change mid-song — its key, tempo or time signature — of which a later line is a change
/// from where it stands rather than a second value: the song starts in the first one its header names. An empty line says
/// nothing rather than taking back what another one said, but it still keeps the body out of the header's place. A line in
/// the body counts only for a song whose header has no line of that field at all, so a file that writes its metadata at the
/// bottom is still read.
pub(crate) struct ChangeableValue {
    value: Option<String>,
    has_header_line: bool,
    is_readable: fn(&str) -> bool,
}

impl ChangeableValue {
    fn new(is_readable: fn(&str) -> bool) -> Self {
        ChangeableValue { value: None, has_header_line: false, is_readable }
    }

    fn consume(&mut self, written: &str, is_in_body: bool) {
        if !is_in_body {
            self.has_header_line = true;
        }
        if self.takes(written, is_in_body) {
            self.value = Some(written.to_string());
        }
    }

    fn takes(&self, written: &str, is_in_body: bool) -> bool {
        if is_in_body && self.has_header_line {
            return false;
        }
        match &self.value {
            None => true,
            Some(current) => current.is_empty() || (!(self.is_readable)(current) && (self.is_readable)(written)),
        }
    }
}

/// The value of a field a song can only say once — its title, its year, its capo — which is the first line of it that says
/// anything, wherever in the file it stands. An empty line or one the song cannot read is as good as missing, and the next
/// one that says something stands in.
struct OnceValue {
    value: Option<String>,
    is_readable: fn(&str) -> bool,
}

impl OnceValue {
    fn new(is_readable: fn(&str) -> bool) -> Self {
        OnceValue { value: None, is_readable }
    }

    fn consume(&mut self, written: &str) {
        let replaces = match &self.value {
            None => true,
            Some(current) => current.is_empty() || (!(self.is_readable)(current) && (self.is_readable)(written)),
        };
        if replaces {
            self.value = Some(written.to_string());
        }
    }
}

fn always(_: &str) -> bool {
    true
}

fn readable_tempo(value: &str) -> bool {
    values::parse_tempo(value).is_some()
}

fn readable_time(value: &str) -> bool {
    values::parse_time(value).is_some()
}

fn readable_duration(value: &str) -> bool {
    values::parse_duration(value).is_some()
}

fn readable_capo(value: &str) -> bool {
    value.parse::<i32>().is_ok_and(|capo| capo >= 0)
}

/// The short names of the metadata directives, under the long one they stand for.
fn long_name(name: &str) -> &str {
    match name {
        "t" => "title",
        "st" => "subtitle",
        "lang" => "language",
        other => other,
    }
}

/// The names `{meta: name value}` is defined to mean exactly what the standalone `{name: value}` means.
const STANDARD_META_NAMES: [&str; 11] =
    ["title", "subtitle", "artist", "composer", "lyricist", "album", "year", "key", "capo", "tempo", "time"];

pub(crate) struct MetadataBuilder {
    title: OnceValue,
    subtitle: OnceValue,
    artist: OnceValue,
    composer: OnceValue,
    lyricist: OnceValue,
    album: OnceValue,
    year: OnceValue,
    cover_art: Option<String>,
    key: ChangeableValue,
    capo: Option<i32>,
    pub(crate) tempo: ChangeableValue,
    pub(crate) time: ChangeableValue,
    duration: OnceValue,
    tags: Vec<String>,
    tag_keys: HashSet<String>,
    languages: Vec<String>,
    links: Vec<Link>,
    custom: BTreeMap<String, Vec<String>>,
}

impl MetadataBuilder {
    pub(crate) fn new() -> Self {
        MetadataBuilder {
            title: OnceValue::new(always),
            subtitle: OnceValue::new(always),
            artist: OnceValue::new(always),
            composer: OnceValue::new(always),
            lyricist: OnceValue::new(always),
            album: OnceValue::new(always),
            year: OnceValue::new(always),
            cover_art: None,
            key: ChangeableValue::new(always),
            capo: None,
            tempo: ChangeableValue::new(readable_tempo),
            time: ChangeableValue::new(readable_time),
            duration: OnceValue::new(readable_duration),
            tags: Vec::new(),
            tag_keys: HashSet::new(),
            languages: Vec::new(),
            links: Vec::new(),
            custom: BTreeMap::new(),
        }
    }

    /// `is_in_body`: whether `directive` stands in the body of the song rather than in its header.
    pub(crate) fn consume(&mut self, directive: &Directive, is_in_body: bool) {
        let value = directive.value.as_deref().map(str::trim).unwrap_or("").to_string();
        match long_name(&directive.name) {
            "title" => self.title.consume(&value),
            "subtitle" => self.subtitle.consume(&value),
            "artist" => self.artist.consume(&value),
            "composer" => self.composer.consume(&value),
            "lyricist" => self.lyricist.consume(&value),
            "album" => self.album.consume(&value),
            "year" => self.year.consume(&value),
            "key" => self.key.consume(&value, is_in_body),
            "capo" => {
                if self.capo.is_none() && readable_capo(&value) {
                    self.capo = value.parse().ok();
                }
            }
            "tempo" => self.tempo.consume(&value, is_in_body),
            "time" => self.time.consume(&value, is_in_body),
            "duration" => self.duration.consume(&value),
            "tag" => {
                if let Some(tag) = tag(directive) {
                    self.add_tag(tag);
                }
            }
            "language" => {
                if let Some(language) = language(directive) {
                    self.add_language(language);
                }
            }
            "meta" => self.consume_meta(directive, &value, is_in_body),
            _ => {
                if directive.name.starts_with("x_") && directive.name.len() > 2 {
                    self.custom.entry(directive.name.clone()).or_default().push(value);
                }
            }
        }
    }

    fn consume_meta(&mut self, directive: &Directive, value: &str, is_in_body: bool) {
        // The spec defines these as the standalone directive, so they are read as one: a song whose header is all
        // `{meta: title …}` lines is titled, named and keyed by it like any other.
        if let Some(standard) = standard_meta(directive) {
            self.consume(&standard, is_in_body);
            return;
        }
        let name = value.split(' ').next().unwrap_or("").trim();
        if name.eq_ignore_ascii_case("tag") {
            if let Some(tag) = tag(directive) {
                self.add_tag(tag);
            }
        } else if is_meta_key(directive, "language") || is_meta_key(directive, "lang") {
            // Read as its own thing rather than as one more custom item, so that it is not carried twice.
            if let Some(language) = language(directive) {
                self.add_language(language);
            }
        } else if is_meta_key(directive, "cover") {
            if self.cover_art.is_none() {
                self.cover_art = meta_value(directive).and_then(|value| web_url(&value));
            }
        } else if is_meta_key(directive, "link") {
            if let Some(link) = link(directive) {
                if self.links.iter().all(|existing| existing.url != link.url) {
                    self.links.push(link);
                }
            }
        } else if !name.is_empty() {
            let rest = value.split_once(' ').map(|(_, rest)| rest.trim()).unwrap_or("");
            self.custom.entry(name.to_string()).or_default().push(rest.to_string());
        }
    }

    /// A tag the song already carries in another spelling is not a second tag.
    fn add_tag(&mut self, value: String) {
        if self.tag_keys.insert(case_insensitive_key(&value)) {
            self.tags.push(value);
        }
    }

    fn add_language(&mut self, value: String) {
        if !self.languages.contains(&value) {
            self.languages.push(value);
        }
    }

    pub(crate) fn build(&self, transpose: i32) -> Metadata {
        Metadata {
            title: self.title.value.clone(),
            subtitle: self.subtitle.value.clone(),
            artist: self.artist.value.clone(),
            composer: self.composer.value.clone(),
            lyricist: self.lyricist.value.clone(),
            album: self.album.value.clone(),
            year: self.year.value.clone(),
            cover_art: self.cover_art.clone(),
            key: self.key.value.clone(),
            capo: self.capo,
            tempo: self.tempo.value.clone(),
            time: self.time.value.clone(),
            duration: self.duration.value.clone(),
            transpose,
            tags: self.tags.clone(),
            languages: self.languages.clone(),
            links: self.links.clone(),
            custom: self.custom.clone(),
        }
    }
}

/// Equal for exactly the strings a case-insensitive comparison calls equal, so that it can key a hash set.
fn case_insensitive_key(value: &str) -> String {
    value.chars().map(|c| c.to_uppercase().next().unwrap_or(c).to_lowercase().next().unwrap_or(c)).collect()
}

/// The key of a `{meta}` directive, the first word of its value.
fn meta_key(directive: &Directive) -> Option<String> {
    if directive.name != "meta" {
        return None;
    }
    directive.value.as_deref().map(|value| value.trim().split(' ').next().unwrap_or("").trim().to_string())
}

fn is_meta_key(directive: &Directive, name: &str) -> bool {
    meta_key(directive).is_some_and(|key| key.eq_ignore_ascii_case(name))
}

/// What a `{meta}` directive holds after its key, or an empty string where it holds only the key.
fn meta_value(directive: &Directive) -> Option<String> {
    directive.value.as_deref().map(|value| value.trim().split_once(' ').map(|(_, rest)| rest.to_string()).unwrap_or_default())
}

/// The standalone directive a `{meta}` one stands for (`{meta: title Amazing Grace}` is `{title: Amazing Grace}`), or `None`.
fn standard_meta(directive: &Directive) -> Option<Directive> {
    if directive.name != "meta" {
        return None;
    }
    let value = directive.value.as_deref()?.trim();
    let name = value.split(' ').next().unwrap_or("").trim().to_lowercase();
    if STANDARD_META_NAMES.contains(&name.as_str()) {
        let rest = value.split_once(' ').map(|(_, rest)| rest.trim()).unwrap_or("");
        Some(Directive { name, value: Some(rest.to_string()) })
    } else {
        None
    }
}

/// The tag a directive carries: `{tag: Needs study}` and `{meta: tag Needs study}` are the same thing. The value is taken
/// whole, since the spec calls a tag arbitrary text.
fn tag(directive: &Directive) -> Option<String> {
    let value = match directive.name.as_str() {
        "tag" => directive.value.as_deref().map(|value| value.trim().to_string()),
        "meta" if is_meta_key(directive, "tag") => meta_value(directive).map(|value| value.trim().to_string()),
        _ => None,
    }?;
    (!value.is_empty()).then_some(value)
}

/// The language a directive declares, normalized: the primary subtag, lowercase. `{meta: language en}`, `{meta: lang en}`,
/// `{language: en}` and `{lang: en}` are read as the same thing.
fn language(directive: &Directive) -> Option<String> {
    let value = match directive.name.as_str() {
        "language" | "lang" => directive.value.clone(),
        "meta" if is_meta_key(directive, "language") || is_meta_key(directive, "lang") => meta_value(directive),
        _ => None,
    }?;
    language_code(&value)
}

/// A language value as the library files it under: the primary subtag, lowercase. The two ISO codes that mean "there is no
/// language here" (`und`, `zxx`) are read as none.
pub fn language_code(value: &str) -> Option<String> {
    let primary = value.trim().split('-').next().unwrap_or("").split('_').next().unwrap_or("").to_lowercase();
    (!primary.is_empty() && primary != "und" && primary != "zxx").then_some(primary)
}

/// The address and optional name a `{meta: link …}` directive holds, or `None` where it names nothing a browser opens.
fn link(directive: &Directive) -> Option<Link> {
    if !is_meta_key(directive, "link") {
        return None;
    }
    let value = meta_value(directive)?;
    let value = value.trim();
    let address: String = value.chars().take_while(|c| !c.is_whitespace()).collect();
    let url = web_url(&address)?;
    let name: String = value.chars().skip(address.chars().count()).collect::<String>().trim().to_string();
    Some(Link { url, name: (!name.is_empty()).then_some(name) })
}

/// `value` as a web address, or `None` where it is not one: trimmed, and taken only when it is an `http` or `https` URL with
/// no whitespace inside it.
pub fn web_url(value: &str) -> Option<String> {
    let url = value.trim();
    let lower = url.to_lowercase();
    let has_scheme = lower.starts_with("https://") || lower.starts_with("http://");
    let after_scheme = url.split_once("//").map(|(_, rest)| rest).unwrap_or("");
    (has_scheme && !after_scheme.is_empty() && !url.chars().any(char::is_whitespace)).then(|| url.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn directive(name: &str, value: &str) -> Directive {
        Directive { name: name.to_string(), value: Some(value.to_string()) }
    }

    #[test]
    fn the_first_line_of_a_field_wins() {
        let mut builder = MetadataBuilder::new();
        builder.consume(&directive("title", "First"), false);
        builder.consume(&directive("t", "Second"), false);
        builder.consume(&directive("key", "G"), false);
        builder.consume(&directive("key", "A"), true);
        assert_eq!(builder.build(0).title.as_deref(), Some("First"));
        assert_eq!(builder.build(0).key.as_deref(), Some("G"));
    }

    #[test]
    fn an_empty_line_gives_way_to_the_next_one() {
        let mut builder = MetadataBuilder::new();
        builder.consume(&directive("title", ""), false);
        builder.consume(&directive("title", "Real"), false);
        builder.consume(&directive("tempo", "fast"), false);
        builder.consume(&directive("tempo", "96"), false);
        let metadata = builder.build(0);
        assert_eq!(metadata.title.as_deref(), Some("Real"));
        assert_eq!(metadata.tempo.as_deref(), Some("96"));
    }

    #[test]
    fn a_body_line_counts_only_where_the_header_has_none() {
        let mut builder = MetadataBuilder::new();
        builder.consume(&directive("key", "D"), true);
        assert_eq!(builder.build(0).key.as_deref(), Some("D"));
    }

    #[test]
    fn meta_items_are_read_as_what_they_stand_for() {
        let mut builder = MetadataBuilder::new();
        builder.consume(&directive("meta", "title Amazing Grace"), false);
        builder.consume(&directive("meta", "tag Needs study"), false);
        builder.consume(&directive("tag", "needs STUDY"), false);
        builder.consume(&directive("meta", "language en-US"), false);
        builder.consume(&directive("lang", "EN"), false);
        builder.consume(&directive("meta", "cover https://example.com/a.png"), false);
        builder.consume(&directive("meta", "cover file:///x"), false);
        builder.consume(&directive("meta", "link https://example.com/watch Live version"), false);
        builder.consume(&directive("meta", "link https://example.com/watch Other"), false);
        builder.consume(&directive("meta", "mood calm"), false);
        builder.consume(&directive("x_source", "book"), false);
        let metadata = builder.build(0);
        assert_eq!(metadata.title.as_deref(), Some("Amazing Grace"));
        assert_eq!(metadata.tags, vec!["Needs study"]);
        assert_eq!(metadata.languages, vec!["en"]);
        assert_eq!(metadata.cover_art.as_deref(), Some("https://example.com/a.png"));
        assert_eq!(metadata.links, vec![Link { url: "https://example.com/watch".into(), name: Some("Live version".into()) }]);
        assert_eq!(metadata.custom["mood"], vec!["calm"]);
        assert_eq!(metadata.custom["x_source"], vec!["book"]);
    }

    #[test]
    fn a_capo_must_be_a_number() {
        let mut builder = MetadataBuilder::new();
        builder.consume(&directive("capo", "two"), false);
        builder.consume(&directive("capo", "2"), false);
        builder.consume(&directive("capo", "5"), false);
        assert_eq!(builder.build(0).capo, Some(2));
    }
}

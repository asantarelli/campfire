// This file is part of Campfire.
// Copyright (c) Pandula Péter 2017-2026.
// https://github.com/pandulapeter/campfire
//
// This Source Code Form is subject to the terms of the Mozilla Public License, v. 2.0.
// If a copy of the MPL was not distributed with this file, You can obtain one at
// https://mozilla.org/MPL/2.0/.

//! How the list of songs is ordered, and the rows it is shown in.

use library::SongEntry;

use crate::settings::SongOrder;

/// What the heading of the songs that have no artist says.
const NO_ARTIST: &str = "Sin artista";

/// One row of the list: a heading over the songs of an artist, or a song by its place in the list.
#[derive(Debug, Clone, PartialEq)]
pub enum ListRow {
    Header(String),
    Song(usize),
}

fn artist_of(entry: &SongEntry) -> Option<&str> {
    entry.metadata.artist.as_deref().map(str::trim).filter(|artist| !artist.is_empty())
}

/// Puts the songs in the order asked for: by title, or by artist and then by title, with the songs that have no artist last.
pub fn sort(songs: &mut [SongEntry], order: SongOrder) {
    match order {
        SongOrder::Title => songs.sort_by_key(|entry| entry.display_title().to_lowercase()),
        SongOrder::Artist | SongOrder::ArtistGroups => {
            songs.sort_by_key(|entry| (artist_of(entry).is_none(), artist_of(entry).unwrap_or_default().to_lowercase(), entry.display_title().to_lowercase()));
        }
    }
}

/// The rows of the songs at `matching`, in the order of the list: with a heading in front of the songs of each artist where
/// they are grouped.
pub fn rows(songs: &[SongEntry], matching: &[usize], order: SongOrder) -> Vec<ListRow> {
    if order != SongOrder::ArtistGroups {
        return matching.iter().map(|&index| ListRow::Song(index)).collect();
    }
    let mut rows = Vec::new();
    let mut current: Option<Option<String>> = None;
    for &index in matching {
        let artist = artist_of(&songs[index]);
        let key = artist.map(str::to_lowercase);
        if current.as_ref() != Some(&key) {
            rows.push(ListRow::Header(artist.unwrap_or(NO_ARTIST).to_string()));
            current = Some(key);
        }
        rows.push(ListRow::Song(index));
    }
    rows
}

#[cfg(test)]
mod tests {
    use super::*;
    use chordpro::model::Metadata;

    fn entry(file_name: &str, title: &str, artist: Option<&str>) -> SongEntry {
        let metadata = Metadata { title: Some(title.to_string()), artist: artist.map(str::to_string), ..Metadata::default() };
        SongEntry { file_name: file_name.to_string(), metadata, has_chords: true }
    }

    fn titles(songs: &[SongEntry]) -> Vec<String> {
        songs.iter().map(SongEntry::display_title).collect()
    }

    fn library() -> Vec<SongEntry> {
        vec![entry("a.cho", "Zamba", Some("Soda")), entry("b.cho", "alba", None), entry("c.cho", "Beso", Some("soda")), entry("d.cho", "Cielo", Some("Abba"))]
    }

    #[test]
    fn songs_are_ordered_by_title_whatever_the_case() {
        let mut songs = library();
        sort(&mut songs, SongOrder::Title);
        assert_eq!(titles(&songs), ["alba", "Beso", "Cielo", "Zamba"]);
    }

    #[test]
    fn songs_are_ordered_by_artist_then_title_with_the_ones_without_an_artist_last() {
        let mut songs = library();
        sort(&mut songs, SongOrder::Artist);
        assert_eq!(titles(&songs), ["Cielo", "Beso", "Zamba", "alba"]);
    }

    #[test]
    fn artists_are_grouped_under_a_heading_each_and_the_songs_without_one_under_a_last_heading() {
        let mut songs = library();
        sort(&mut songs, SongOrder::ArtistGroups);
        let matching: Vec<usize> = (0..songs.len()).collect();
        assert_eq!(
            rows(&songs, &matching, SongOrder::ArtistGroups),
            [
                ListRow::Header("Abba".into()),
                ListRow::Song(0),
                ListRow::Header("soda".into()),
                ListRow::Song(1),
                ListRow::Song(2),
                ListRow::Header(NO_ARTIST.into()),
                ListRow::Song(3),
            ]
        );
    }

    #[test]
    fn a_list_that_is_not_grouped_has_no_headings_and_a_search_keeps_only_its_songs() {
        let songs = library();
        assert_eq!(rows(&songs, &[2, 0], SongOrder::Title), [ListRow::Song(2), ListRow::Song(0)]);
        assert_eq!(rows(&songs, &[2], SongOrder::ArtistGroups), [ListRow::Header("soda".into()), ListRow::Song(2)]);
    }
}

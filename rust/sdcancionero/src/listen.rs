// This file is part of Campfire.
// Copyright (c) Pandula Péter 2017-2026.
// https://github.com/pandulapeter/campfire
//
// This Source Code Form is subject to the terms of the Mozilla Public License, v. 2.0.
// If a copy of the MPL was not distributed with this file, You can obtain one at
// https://mozilla.org/MPL/2.0/.

//! Looking a song up where it can be heard: the search of YouTube, YouTube Music or Spotify for its title and artist, opened in
//! the browser. Nothing is sent from here; the address is the whole of it.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Service {
    YouTube,
    YouTubeMusic,
    Spotify,
}

impl Service {
    pub const ALL: [Service; 3] = [Service::YouTube, Service::YouTubeMusic, Service::Spotify];

    pub fn name(self) -> &'static str {
        match self {
            Service::YouTube => "YouTube",
            Service::YouTubeMusic => "YouTube Music",
            Service::Spotify => "Spotify",
        }
    }

    /// The address of this service's search for `query`.
    pub fn search_url(self, query: &str) -> String {
        let encoded = percent_encode(query);
        match self {
            Service::YouTube => format!("https://www.youtube.com/results?search_query={encoded}"),
            Service::YouTubeMusic => format!("https://music.youtube.com/search?q={encoded}"),
            Service::Spotify => format!("https://open.spotify.com/search/{encoded}"),
        }
    }
}

/// What a song is searched by: its title and its artist.
pub fn search_query(title: &str, artist: Option<&str>) -> String {
    match artist.map(str::trim).filter(|artist| !artist.is_empty()) {
        Some(artist) => format!("{artist} {title}"),
        None => title.to_string(),
    }
}

/// `text` as a part of an address: every byte but the unreserved ones as `%XX`, a space as `%20`.
pub fn percent_encode(text: &str) -> String {
    let mut encoded = String::new();
    for byte in text.trim().bytes() {
        if byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.' | b'~') {
            encoded.push(byte as char);
        } else {
            encoded.push_str(&format!("%{byte:02X}"));
        }
    }
    encoded
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_query_is_encoded_for_an_address() {
        assert_eq!(percent_encode("Gracias a la Vida"), "Gracias%20a%20la%20Vida");
        assert_eq!(percent_encode("Canción & más"), "Canci%C3%B3n%20%26%20m%C3%A1s");
        assert_eq!(percent_encode("a-b_c.d~e"), "a-b_c.d~e");
    }

    #[test]
    fn each_service_searches_by_title_and_artist() {
        let query = search_query("Gracias a la Vida", Some("Mercedes Sosa"));
        assert_eq!(query, "Mercedes Sosa Gracias a la Vida");
        assert_eq!(Service::YouTube.search_url(&query), "https://www.youtube.com/results?search_query=Mercedes%20Sosa%20Gracias%20a%20la%20Vida");
        assert!(Service::YouTubeMusic.search_url(&query).starts_with("https://music.youtube.com/search?q="));
        assert!(Service::Spotify.search_url(&query).starts_with("https://open.spotify.com/search/"));
        assert_eq!(search_query("Sola", None), "Sola");
        assert_eq!(search_query("Sola", Some("  ")), "Sola");
    }
}

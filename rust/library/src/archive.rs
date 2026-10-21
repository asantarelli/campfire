// This file is part of Campfire.
// Copyright (c) Pandula Péter 2017-2026.
// https://github.com/pandulapeter/campfire
//
// This Source Code Form is subject to the terms of the Mozilla Public License, v. 2.0.
// If a copy of the MPL was not distributed with this file, You can obtain one at
// https://mozilla.org/MPL/2.0/.

//! The zip archives that carry a library between devices: `songs/<name>.cho` and `setlists/<name>.setlist.json`, which is
//! what the Kotlin app writes, and any zip of such files, flat or in folders, which is what it reads.

use std::io::{Cursor, Read, Write};

use zip::write::SimpleFileOptions;
use zip::{CompressionMethod, ZipArchive, ZipWriter};

use crate::names;

/// The name the Kotlin app gives an exported library.
pub const ARCHIVE_NAME: &str = "campfire_library.zip";
pub const SONGS_DIRECTORY: &str = "songs";
pub const SETLISTS_DIRECTORY: &str = "setlists";

/// The most a text file may weigh, and the most one import may inflate to, however many archives it is spread over.
pub const MAX_TEXT_FILE_SIZE: u64 = 8 << 20;
pub const MAX_IMPORT_SIZE: u64 = 24 << 20;
/// How many archives deep an archive in an archive is opened.
const MAX_DEPTH: usize = 3;

const BINARY_EXTENSIONS: &[&str] = &[
    ".png", ".jpg", ".jpeg", ".gif", ".heic", ".heif", ".webp", ".bmp", ".tif", ".tiff", ".ico", ".mp3", ".m4a", ".aac", ".wav", ".flac",
    ".ogg", ".opus", ".aif", ".aiff", ".mid", ".midi", ".mp4", ".m4v", ".mov", ".avi", ".mkv", ".webm", ".db", ".sqlite", ".exe", ".dll",
    ".apk",
];

/// A file on its way into the library: the name it came under, folders left out, and its bytes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImportedFile {
    pub name: String,
    pub bytes: Vec<u8>,
}

/// What an archive held.
#[derive(Debug, Default)]
pub struct Unpacked {
    pub files: Vec<ImportedFile>,
    /// The names of the entries that were not read: too large, or an archive that could not be opened.
    pub unread: Vec<String>,
}

#[derive(Debug)]
pub enum ArchiveError {
    /// The bytes are no zip archive.
    NotAnArchive(String),
    /// Packing failed.
    Write(String),
}

impl std::fmt::Display for ArchiveError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ArchiveError::NotAnArchive(reason) => write!(formatter, "el archivo no es un zip válido ({reason})"),
            ArchiveError::Write(reason) => write!(formatter, "no se pudo armar el zip ({reason})"),
        }
    }
}

impl std::error::Error for ArchiveError {}

/// Whether the start of a file is the start of a zip archive.
pub fn is_zip_archive(prefix: &[u8]) -> bool {
    prefix.starts_with(b"PK\x03\x04") || prefix.starts_with(b"PK\x05\x06")
}

/// The archive of `files`, each entry named by the key it is given (`songs/x.cho`).
pub fn pack(files: &[(String, Vec<u8>)]) -> Result<Vec<u8>, ArchiveError> {
    let mut writer = ZipWriter::new(Cursor::new(Vec::new()));
    let options = SimpleFileOptions::default().compression_method(CompressionMethod::Deflated);
    for (name, bytes) in files {
        writer.start_file(name.as_str(), options).map_err(|error| ArchiveError::Write(error.to_string()))?;
        writer.write_all(bytes).map_err(|error| ArchiveError::Write(error.to_string()))?;
    }
    writer.finish().map(Cursor::into_inner).map_err(|error| ArchiveError::Write(error.to_string()))
}

/// Opens `archive`: its files by the name they have without their folders (the library is flat, so `songs/x.cho` and `x.cho`
/// are one file), an archive inside it opened in turn. What the archiving tool wrote for itself (`._x.cho`, `.DS_Store`, a
/// hidden folder) and what is never text by its name is not read, and costs nothing. `max_total` bounds what the whole import
/// may inflate to.
pub fn unpack(archive: &[u8], max_total: u64) -> Result<Unpacked, ArchiveError> {
    let mut inflated = 0;
    let mut unpacked = Unpacked::default();
    unpack_into(archive, 1, max_total, &mut inflated, &mut unpacked)?;
    Ok(unpacked)
}

fn unpack_into(archive: &[u8], depth: usize, max_total: u64, inflated: &mut u64, unpacked: &mut Unpacked) -> Result<(), ArchiveError> {
    let mut zip = ZipArchive::new(Cursor::new(archive)).map_err(|error| ArchiveError::NotAnArchive(error.to_string()))?;
    for index in 0..zip.len() {
        let Ok(mut entry) = zip.by_index(index) else {
            continue;
        };
        if entry.is_dir() {
            continue;
        }
        let path = entry.name().to_string();
        let name = path.rsplit(['/', '\\']).next().unwrap_or("").to_string();
        if name.is_empty() || path.split(['/', '\\']).any(|segment| segment != "." && names::is_hidden_file_name(segment)) {
            continue;
        }
        let lower = name.to_lowercase();
        if BINARY_EXTENSIONS.iter().any(|extension| lower.ends_with(extension)) {
            continue;
        }
        let is_archive = names::is_archive_file_name(&name);
        let limit = if is_archive { max_total.saturating_sub(*inflated) } else { MAX_TEXT_FILE_SIZE.min(max_total.saturating_sub(*inflated)) };
        if entry.size() > limit {
            unpacked.unread.push(name);
            continue;
        }
        // The declared size is the archive's own word, so the read is bounded by the limit as well.
        let mut bytes = Vec::new();
        if entry.by_ref().take(limit + 1).read_to_end(&mut bytes).is_err() || bytes.len() as u64 > limit {
            unpacked.unread.push(name);
            continue;
        }
        *inflated += bytes.len() as u64;
        if depth < MAX_DEPTH && (is_archive || is_zip_archive(&bytes)) && !names::is_song_file_name(&name) {
            // A nested archive that cannot be read is reported rather than failing the files next to it.
            if unpack_into(&bytes, depth + 1, max_total, inflated, unpacked).is_err() {
                unpacked.unread.push(name);
            }
        } else {
            unpacked.files.push(ImportedFile { name, bytes });
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn entry(name: &str, text: &str) -> (String, Vec<u8>) {
        (name.to_string(), text.as_bytes().to_vec())
    }

    #[test]
    fn what_is_packed_is_unpacked_without_its_folders() {
        let bytes = pack(&[entry("songs/a.cho", "{title: A}"), entry("setlists/s.setlist.json", "{}")]).unwrap();
        assert!(is_zip_archive(&bytes));
        let unpacked = unpack(&bytes, MAX_IMPORT_SIZE).unwrap();
        let names: Vec<&str> = unpacked.files.iter().map(|file| file.name.as_str()).collect();
        assert_eq!(names, ["a.cho", "s.setlist.json"]);
        assert_eq!(unpacked.files[0].bytes, b"{title: A}");
        assert!(unpacked.unread.is_empty());
    }

    #[test]
    fn the_leftovers_of_an_archiving_tool_are_never_read() {
        let bytes = pack(&[
            entry("a.cho", "x"),
            entry("._a.cho", "binary"),
            entry(".DS_Store", "x"),
            entry(".Trash/old.cho", "x"),
            entry("cover.png", "x"),
            entry("./b.cho", "y"),
            entry("dir\\c.cho", "z"),
        ])
        .unwrap();
        let unpacked = unpack(&bytes, MAX_IMPORT_SIZE).unwrap();
        let names: Vec<&str> = unpacked.files.iter().map(|file| file.name.as_str()).collect();
        assert_eq!(names, ["a.cho", "b.cho", "c.cho"]);
    }

    #[test]
    fn an_archive_in_an_archive_is_opened() {
        let inner = pack(&[entry("deep.cho", "x")]).unwrap();
        let outer = pack(&[("inner.zip".to_string(), inner), entry("top.cho", "y")]).unwrap();
        let unpacked = unpack(&outer, MAX_IMPORT_SIZE).unwrap();
        let mut names: Vec<&str> = unpacked.files.iter().map(|file| file.name.as_str()).collect();
        names.sort();
        assert_eq!(names, ["deep.cho", "top.cho"]);
    }

    #[test]
    fn a_file_over_the_limit_is_reported_and_not_read() {
        let bytes = pack(&[entry("big.cho", &"x".repeat(1000)), entry("small.cho", "y")]).unwrap();
        let unpacked = unpack(&bytes, 500).unwrap();
        assert_eq!(unpacked.unread, ["big.cho"]);
        assert_eq!(unpacked.files.len(), 1);
    }

    #[test]
    fn bytes_that_are_no_archive_are_refused() {
        assert!(matches!(unpack(b"{title: x}", MAX_IMPORT_SIZE), Err(ArchiveError::NotAnArchive(_))));
        assert!(!is_zip_archive(b"{title"));
    }
}

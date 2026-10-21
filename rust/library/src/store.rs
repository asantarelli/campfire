// This file is part of Campfire.
// Copyright (c) Pandula Péter 2017-2026.
// https://github.com/pandulapeter/campfire
//
// This Source Code Form is subject to the terms of the Mozilla Public License, v. 2.0.
// If a copy of the MPL was not distributed with this file, You can obtain one at
// https://mozilla.org/MPL/2.0/.

//! The library folder: `songs/*.cho` and `setlists/*.setlist.json`, the same layout the Kotlin app keeps, so that one
//! folder can be used by both.
//!
//! A read returns `None` **only** for a file that is not there; a file that cannot be read is an error, since a missing file
//! is a deletion to anything that syncs the folder.

use std::collections::HashSet;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use chordpro::model::Metadata;
use chordpro::notation::ChordNotation;
use chordpro::parser;

use crate::archive::{self, ArchiveError, ImportedFile, ARCHIVE_NAME, SETLISTS_DIRECTORY, SONGS_DIRECTORY};
use crate::names;
use crate::setlist::Setlist;
use crate::text::decode_library_text;

static TEMPORARY_FILE_COUNTER: AtomicU64 = AtomicU64::new(0);

/// A song of the library as a list shows it: its file and what its header says, without keeping its text.
#[derive(Debug, Clone, PartialEq)]
pub struct SongEntry {
    pub file_name: String,
    pub metadata: Metadata,
    pub has_chords: bool,
}

impl SongEntry {
    /// The title the song is listed under: its `{title}` (and `{subtitle}`), else its file name without the extension.
    pub fn display_title(&self) -> String {
        let fallback = self.file_name.rsplit_once('.').map(|(stem, _)| stem).unwrap_or(&self.file_name);
        self.metadata.display_title(fallback)
    }
}

/// What to do with an incoming file whose name is taken by one with other content.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Collision {
    /// Leave the file in the library as it is.
    Skip,
    /// Keep both: the incoming one under a numbered name.
    Number,
    /// Put the incoming one in place of the one in the library.
    Replace,
}

/// What an import would do, decided before anything is written.
#[derive(Debug, Default)]
pub struct ImportPlan {
    /// Files whose name is free.
    pub new: Vec<ImportedFile>,
    /// Files the library already has, with the same content.
    pub same: Vec<String>,
    /// Files whose name is taken by one with other content.
    pub conflicts: Vec<ImportedFile>,
    /// What cannot be imported: not a song or a setlist, too large, or an archive that could not be opened.
    pub skipped: Vec<String>,
}

/// What an import did.
#[derive(Debug, Default, PartialEq, Eq)]
pub struct ImportReport {
    pub added: Vec<String>,
    pub replaced: Vec<String>,
    pub numbered: Vec<String>,
    pub unchanged: Vec<String>,
    pub skipped: Vec<String>,
}

pub struct Library {
    root: PathBuf,
}

/// Where the library lives for the current user: `%LOCALAPPDATA%\SDCancionero\library` on Windows, a folder in the home
/// directory elsewhere.
pub fn default_root() -> PathBuf {
    if let Some(local) = std::env::var_os("LOCALAPPDATA") {
        return Path::new(&local).join("SDCancionero").join("library");
    }
    let home = std::env::var_os("HOME").or_else(|| std::env::var_os("USERPROFILE")).map(PathBuf::from).unwrap_or_else(|| PathBuf::from("."));
    home.join(".sdcancionero").join("library")
}

impl Library {
    /// Opens the library at `root`, creating its folders where they are missing.
    pub fn open(root: impl Into<PathBuf>) -> io::Result<Library> {
        let library = Library { root: root.into() };
        fs::create_dir_all(library.songs_directory())?;
        fs::create_dir_all(library.setlists_directory())?;
        Ok(library)
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    fn songs_directory(&self) -> PathBuf {
        self.root.join(SONGS_DIRECTORY)
    }

    fn setlists_directory(&self) -> PathBuf {
        self.root.join(SETLISTS_DIRECTORY)
    }

    fn names_in(directory: &Path, accepts: fn(&str) -> bool) -> io::Result<Vec<String>> {
        let mut found = Vec::new();
        for entry in fs::read_dir(directory)? {
            let entry = entry?;
            let Some(name) = entry.file_name().to_str().map(str::to_string) else {
                continue;
            };
            if accepts(&name) && entry.file_type()?.is_file() {
                found.push(name);
            }
        }
        found.sort_by_key(|name| names::identity_key(name));
        Ok(found)
    }

    pub fn song_names(&self) -> io::Result<Vec<String>> {
        Self::names_in(&self.songs_directory(), names::is_song_file_name)
    }

    pub fn setlist_names(&self) -> io::Result<Vec<String>> {
        Self::names_in(&self.setlists_directory(), names::is_setlist_file_name)
    }

    /// The text of the song file `name`, `None` where there is no such file.
    pub fn read_song(&self, name: &str) -> io::Result<Option<String>> {
        Self::read_text(&self.songs_directory(), name)
    }

    fn read_text(directory: &Path, name: &str) -> io::Result<Option<String>> {
        if !names::can_hold_file_name(name) {
            return Ok(None);
        }
        match fs::read(directory.join(name)) {
            Ok(bytes) => Ok(Some(decode_library_text(&bytes))),
            Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(None),
            Err(error) => Err(error),
        }
    }

    /// Writes the song file `name` as UTF-8, whole or not at all.
    pub fn write_song(&self, name: &str, text: &str) -> io::Result<()> {
        Self::write_bytes(&self.songs_directory(), name, text.as_bytes())
    }

    fn write_bytes(directory: &Path, name: &str, bytes: &[u8]) -> io::Result<()> {
        if !names::can_hold_file_name(name) {
            return Err(io::Error::new(io::ErrorKind::InvalidInput, format!("\"{name}\" no es un nombre de archivo válido")));
        }
        // Written to a file of its own, named without the target so that a name at the length limit still saves, and moved
        // into place: a crash leaves the old file or the new one, never half of one.
        let temporary = directory.join(format!(".campfire-{}-{}.tmp", std::process::id(), TEMPORARY_FILE_COUNTER.fetch_add(1, Ordering::Relaxed)));
        fs::write(&temporary, bytes)?;
        fs::rename(&temporary, directory.join(name)).inspect_err(|_| {
            let _ = fs::remove_file(&temporary);
        })
    }

    pub fn delete_song(&self, name: &str) -> io::Result<()> {
        Self::delete(&self.songs_directory(), name)
    }

    fn delete(directory: &Path, name: &str) -> io::Result<()> {
        if !names::can_hold_file_name(name) {
            return Ok(());
        }
        match fs::remove_file(directory.join(name)) {
            Err(error) if error.kind() != io::ErrorKind::NotFound => Err(error),
            _ => Ok(()),
        }
    }

    /// Moves a song to another name: the new file is written before the old one is removed, and every setlist that named the
    /// old one names the new one. A name that differs only by case is a move too.
    pub fn rename_song(&self, from: &str, to: &str) -> io::Result<()> {
        if from == to {
            return Ok(());
        }
        let Some(text) = self.read_song(from)? else {
            return Err(io::Error::new(io::ErrorKind::NotFound, format!("no existe \"{from}\"")));
        };
        let is_same_file = names::identity_key(from) == names::identity_key(to);
        if is_same_file {
            // Windows ignores the case of a name: the old file has to be gone before the new spelling is written.
            self.delete_song(from)?;
            self.write_song(to, &text)?;
        } else {
            self.write_song(to, &text)?;
            self.delete_song(from)?;
        }
        for setlist_name in self.setlist_names()? {
            if let Some(setlist) = self.read_setlist(&setlist_name)? {
                if setlist.songs.iter().any(|song| song.file == from) {
                    self.write_setlist(&setlist_name, &setlist.with_song_renamed(from, to))?;
                }
            }
        }
        Ok(())
    }

    /// Deletes a song and takes it out of every setlist that named it.
    pub fn remove_song(&self, name: &str) -> io::Result<()> {
        self.delete_song(name)?;
        for setlist_name in self.setlist_names()? {
            if let Some(setlist) = self.read_setlist(&setlist_name)? {
                if setlist.songs.iter().any(|song| song.file == name) {
                    self.write_setlist(&setlist_name, &setlist.without_song(name))?;
                }
            }
        }
        Ok(())
    }

    pub fn read_setlist(&self, name: &str) -> io::Result<Option<Setlist>> {
        Ok(Self::read_text(&self.setlists_directory(), name)?.and_then(|text| Setlist::decode(&text)))
    }

    pub fn write_setlist(&self, name: &str, setlist: &Setlist) -> io::Result<()> {
        Self::write_bytes(&self.setlists_directory(), name, setlist.encode().as_bytes())
    }

    pub fn delete_setlist(&self, name: &str) -> io::Result<()> {
        Self::delete(&self.setlists_directory(), name)
    }

    /// A free name for a new song of `artist` and `title`, numbered where another song has the one it would get.
    pub fn free_song_name(&self, artist: Option<&str>, title: &str) -> io::Result<String> {
        let taken: HashSet<String> = self.song_names()?.iter().map(|name| names::identity_key(name)).collect();
        Ok(names::unique_name(&taken, &names::song_file_name(artist, title)))
    }

    /// A free name for a new setlist called `title`.
    pub fn free_setlist_name(&self, title: &str) -> io::Result<String> {
        let taken: HashSet<String> = self.setlist_names()?.iter().map(|name| names::identity_key(name)).collect();
        Ok(names::unique_name(&taken, &names::setlist_file_name(title)))
    }

    /// Every song of the library as a list shows it, in the order of their file names. A file that cannot be read is left out
    /// of the list, the others are not held up by it.
    pub fn scan_songs(&self) -> io::Result<Vec<SongEntry>> {
        let mut entries = Vec::new();
        for file_name in self.song_names()? {
            let Ok(Some(text)) = self.read_song(&file_name) else {
                continue;
            };
            let song = parser::parse(&text, ChordNotation::Standard);
            entries.push(SongEntry { file_name, has_chords: song.has_chords(), metadata: song.metadata });
        }
        Ok(entries)
    }

    /// The whole library as the zip the Kotlin app exports: `songs/<name>` and `setlists/<name>`. `None` where there is
    /// nothing to export.
    pub fn export_archive(&self) -> Result<Option<Vec<u8>>, ExportError> {
        let mut files = Vec::new();
        for name in self.song_names()? {
            if let Some(text) = self.read_song(&name)? {
                files.push((format!("{SONGS_DIRECTORY}/{name}"), text.into_bytes()));
            }
        }
        for name in self.setlist_names()? {
            if let Some(text) = Self::read_text(&self.setlists_directory(), &name)? {
                files.push((format!("{SETLISTS_DIRECTORY}/{name}"), text.into_bytes()));
            }
        }
        if files.is_empty() {
            return Ok(None);
        }
        Ok(Some(archive::pack(&files)?))
    }

    /// What importing `files` would do. A file is a song or a setlist by its name; anything else is skipped.
    pub fn plan_import(&self, files: Vec<ImportedFile>, unread: Vec<String>) -> io::Result<ImportPlan> {
        let mut plan = ImportPlan { skipped: unread, ..ImportPlan::default() };
        let mut seen: HashSet<String> = HashSet::new();
        for file in files {
            let directory = if names::is_song_file_name(&file.name) {
                self.songs_directory()
            } else if names::is_setlist_file_name(&file.name) {
                self.setlists_directory()
            } else {
                plan.skipped.push(file.name);
                continue;
            };
            // The same name twice in one import is one file, the first of them.
            if !seen.insert(names::identity_key(&file.name)) || !names::can_hold_file_name(&file.name) {
                plan.skipped.push(file.name);
                continue;
            }
            let existing = Self::existing_file(&directory, &file.name)?;
            match existing {
                None => plan.new.push(file),
                Some(bytes) if decode_library_text(&bytes) == decode_library_text(&file.bytes) => plan.same.push(file.name),
                Some(_) => plan.conflicts.push(file),
            }
        }
        Ok(plan)
    }

    /// The bytes of the file whose name is `name` under any case, which is what Windows would call the same file.
    fn existing_file(directory: &Path, name: &str) -> io::Result<Option<Vec<u8>>> {
        match fs::read(directory.join(name)) {
            Ok(bytes) => Ok(Some(bytes)),
            Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(None),
            Err(error) => Err(error),
        }
    }

    /// Writes what `plan` holds: the new files as they are, the conflicts as `collision` says.
    pub fn apply_import(&self, plan: ImportPlan, collision: Collision) -> io::Result<ImportReport> {
        let mut report = ImportReport { unchanged: plan.same, skipped: plan.skipped, ..ImportReport::default() };
        for file in plan.new {
            self.write_imported(&file.name, &file.bytes)?;
            report.added.push(file.name);
        }
        let mut taken_songs: HashSet<String> = self.song_names()?.iter().map(|name| names::identity_key(name)).collect();
        let mut taken_setlists: HashSet<String> = self.setlist_names()?.iter().map(|name| names::identity_key(name)).collect();
        for file in plan.conflicts {
            match collision {
                Collision::Skip => report.skipped.push(file.name),
                Collision::Replace => {
                    self.write_imported(&file.name, &file.bytes)?;
                    report.replaced.push(file.name);
                }
                Collision::Number => {
                    let taken = if names::is_song_file_name(&file.name) { &mut taken_songs } else { &mut taken_setlists };
                    let name = names::unique_name(taken, &file.name);
                    taken.insert(names::identity_key(&name));
                    self.write_imported(&name, &file.bytes)?;
                    report.numbered.push(name);
                }
            }
        }
        Ok(report)
    }

    /// A song is written as UTF-8 whatever it came in; a setlist document as it came, which is JSON.
    fn write_imported(&self, name: &str, bytes: &[u8]) -> io::Result<()> {
        if names::is_setlist_file_name(name) {
            Self::write_bytes(&self.setlists_directory(), name, bytes)
        } else {
            Self::write_bytes(&self.songs_directory(), name, decode_library_text(bytes).as_bytes())
        }
    }

    /// Imports a zip whole: every song and setlist in it, the conflicts as `collision` says.
    pub fn import_archive(&self, archive: &[u8], collision: Collision) -> Result<ImportReport, ExportError> {
        let unpacked = archive::unpack(archive, archive::MAX_IMPORT_SIZE)?;
        let plan = self.plan_import(unpacked.files, unpacked.unread)?;
        Ok(self.apply_import(plan, collision)?)
    }

    /// The name an exported archive is saved under.
    pub fn archive_name() -> &'static str {
        ARCHIVE_NAME
    }
}

/// What can go wrong between the disk and an archive.
#[derive(Debug)]
pub enum ExportError {
    Io(io::Error),
    Archive(ArchiveError),
}

impl From<io::Error> for ExportError {
    fn from(error: io::Error) -> Self {
        ExportError::Io(error)
    }
}

impl From<ArchiveError> for ExportError {
    fn from(error: ArchiveError) -> Self {
        ExportError::Archive(error)
    }
}

impl std::fmt::Display for ExportError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ExportError::Io(error) => write!(formatter, "{error}"),
            ExportError::Archive(error) => write!(formatter, "{error}"),
        }
    }
}

impl std::error::Error for ExportError {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::setlist::SetlistSong;

    fn library() -> (tempfile::TempDir, Library) {
        let directory = tempfile::tempdir().unwrap();
        let library = Library::open(directory.path().join("library")).unwrap();
        (directory, library)
    }

    #[test]
    fn a_song_is_written_read_listed_and_deleted() {
        let (_guard, library) = library();
        assert_eq!(library.read_song("a.cho").unwrap(), None);
        library.write_song("a.cho", "{title: Canción}\n[Am]hola\n").unwrap();
        library.write_song("b.cho", "{title: B}").unwrap();
        library.write_song("notes.txt", "x").unwrap();
        assert_eq!(library.read_song("a.cho").unwrap().as_deref(), Some("{title: Canción}\n[Am]hola\n"));
        assert_eq!(library.song_names().unwrap(), ["a.cho", "b.cho"]);
        library.delete_song("a.cho").unwrap();
        library.delete_song("a.cho").unwrap();
        assert_eq!(library.song_names().unwrap(), ["b.cho"]);
    }

    #[test]
    fn a_file_in_the_wrong_code_page_is_read_and_written_back_as_utf8() {
        let (_guard, library) = library();
        fs::write(library.songs_directory().join("old.cho"), b"{title: Canci\xF3n}").unwrap();
        assert_eq!(library.read_song("old.cho").unwrap().as_deref(), Some("{title: Canción}"));
    }

    #[test]
    fn a_name_that_is_a_path_is_refused() {
        let (_guard, library) = library();
        assert!(library.write_song("../evil.cho", "x").is_err());
        assert_eq!(library.read_song("../evil.cho").unwrap(), None);
    }

    #[test]
    fn the_list_shows_titles_and_whether_there_are_chords() {
        let (_guard, library) = library();
        library.write_song("x-with.cho", "{title: With}\n{subtitle: Live}\n[G]hi").unwrap();
        library.write_song("y-plain.cho", "just words").unwrap();
        let entries = library.scan_songs().unwrap();
        assert_eq!(entries[0].display_title(), "With (Live)");
        assert!(entries[0].has_chords);
        assert_eq!(entries[1].display_title(), "y-plain");
        assert!(!entries[1].has_chords);
    }

    #[test]
    fn a_new_song_gets_a_free_name() {
        let (_guard, library) = library();
        let first = library.free_song_name(Some("Mercedes Sosa"), "Gracias a la Vida").unwrap();
        assert_eq!(first, "mercedes_sosa-gracias_a_la_vida.cho");
        library.write_song(&first, "x").unwrap();
        assert_eq!(library.free_song_name(Some("Mercedes Sosa"), "Gracias a la Vida").unwrap(), "mercedes_sosa-gracias_a_la_vida_2.cho");
    }

    #[test]
    fn a_renamed_song_stays_in_its_setlists_and_a_removed_one_leaves_them() {
        let (_guard, library) = library();
        library.write_song("a.cho", "x").unwrap();
        library.write_song("b.cho", "y").unwrap();
        let setlist = Setlist {
            title: "S".into(),
            songs: vec![SetlistSong { file: "a.cho".into(), transposition: 2, ..Default::default() }, SetlistSong { file: "b.cho".into(), ..Default::default() }],
            ..Default::default()
        };
        library.write_setlist("s.setlist.json", &setlist).unwrap();
        library.rename_song("a.cho", "z.cho").unwrap();
        assert_eq!(library.song_names().unwrap(), ["b.cho", "z.cho"]);
        let renamed = library.read_setlist("s.setlist.json").unwrap().unwrap();
        assert_eq!(renamed.songs[0].file, "z.cho");
        assert_eq!(renamed.songs[0].transposition, 2);
        library.remove_song("b.cho").unwrap();
        assert_eq!(library.read_setlist("s.setlist.json").unwrap().unwrap().songs.len(), 1);
    }

    #[test]
    fn a_rename_that_only_changes_the_case_keeps_the_file() {
        let (_guard, library) = library();
        library.write_song("abc.cho", "x").unwrap();
        library.rename_song("abc.cho", "ABC.cho").unwrap();
        assert_eq!(library.song_names().unwrap(), ["ABC.cho"]);
        assert_eq!(library.read_song("ABC.cho").unwrap().as_deref(), Some("x"));
    }

    #[test]
    fn the_exported_zip_imports_into_another_library() {
        let (_one, source) = library();
        source.write_song("a.cho", "{title: A}\n[Am]x").unwrap();
        source.write_setlist("s.setlist.json", &Setlist { title: "Lista".into(), ..Default::default() }).unwrap();
        let archive = source.export_archive().unwrap().expect("something to export");
        let (_two, target) = library();
        let report = target.import_archive(&archive, Collision::Number).unwrap();
        assert_eq!(report.added, ["a.cho", "s.setlist.json"]);
        assert_eq!(target.read_song("a.cho").unwrap().as_deref(), Some("{title: A}\n[Am]x"));
        assert_eq!(target.read_setlist("s.setlist.json").unwrap().unwrap().title, "Lista");
        // Importing the same archive again changes nothing.
        let again = target.import_archive(&archive, Collision::Number).unwrap();
        assert_eq!(again.unchanged.len(), 2);
        assert!(again.added.is_empty() && again.numbered.is_empty());
    }

    #[test]
    fn an_empty_library_exports_nothing() {
        let (_guard, library) = library();
        assert!(library.export_archive().unwrap().is_none());
    }

    #[test]
    fn a_conflict_is_skipped_numbered_or_replaced_as_asked() {
        let (_guard, library) = library();
        library.write_song("a.cho", "old").unwrap();
        let incoming = || vec![ImportedFile { name: "a.cho".into(), bytes: b"new".to_vec() }, ImportedFile { name: "notes.txt".into(), bytes: b"x".to_vec() }];
        let plan = library.plan_import(incoming(), Vec::new()).unwrap();
        assert_eq!(plan.conflicts.len(), 1);
        assert_eq!(plan.skipped, ["notes.txt"]);
        let skipped = library.apply_import(plan, Collision::Skip).unwrap();
        assert_eq!(skipped.skipped, ["notes.txt", "a.cho"]);
        assert_eq!(library.read_song("a.cho").unwrap().as_deref(), Some("old"));
        let numbered = library.apply_import(library.plan_import(incoming(), Vec::new()).unwrap(), Collision::Number).unwrap();
        assert_eq!(numbered.numbered, ["a_2.cho"]);
        assert_eq!(library.read_song("a_2.cho").unwrap().as_deref(), Some("new"));
        let replaced = library.apply_import(library.plan_import(incoming(), Vec::new()).unwrap(), Collision::Replace).unwrap();
        assert_eq!(replaced.replaced, ["a.cho"]);
        assert_eq!(library.read_song("a.cho").unwrap().as_deref(), Some("new"));
    }

    #[test]
    fn the_same_name_twice_in_one_import_is_one_file() {
        let (_guard, library) = library();
        let files = vec![ImportedFile { name: "a.cho".into(), bytes: b"1".to_vec() }, ImportedFile { name: "A.CHO".into(), bytes: b"2".to_vec() }];
        let plan = library.plan_import(files, Vec::new()).unwrap();
        assert_eq!(plan.new.len(), 1);
        assert_eq!(plan.skipped, ["A.CHO"]);
    }
}

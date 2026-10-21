// This file is part of Campfire.
// Copyright (c) Pandula Péter 2017-2026.
// https://github.com/pandulapeter/campfire
//
// This Source Code Form is subject to the terms of the Mozilla Public License, v. 2.0.
// If a copy of the MPL was not distributed with this file, You can obtain one at
// https://mozilla.org/MPL/2.0/.

//! The window: the list of songs, a song, and the settings, with the library behind them.

use std::path::PathBuf;
use std::time::{Duration, Instant};

use chordpro::model::Song;
use chordpro::notation::ChordNotation;
use chordpro::{parser, transposer};
use eframe::egui::{self, RichText, ScrollArea, Vec2};
use library::{Collision, Library, SongEntry};

use crate::editor::{Editor, EditorAction};
use crate::listen::{self, Service};
use crate::settings::{self, Instrument, Settings, NOTATIONS};
use crate::tools::{self, SongMetronome, SongTiming, ToolTab, Tools};

mod setlists;
use setlists::{SetlistState, SongsTab};
use crate::theme::{self, SongColors};
use crate::viewer::{self, ViewOptions};

/// The songs of a fresh library.
const DEMO_SONGS: [(&str, &str); 2] = [
    (
        "traditional_american-home_on_the_range_my_western_home.cho",
        include_str!("../assets/demo/traditional_american-home_on_the_range_my_western_home.cho"),
    ),
    ("traditional_american-house_of_the_rising_sun.cho", include_str!("../assets/demo/traditional_american-house_of_the_rising_sun.cho")),
];

/// How long a message in the status line stays up.
const STATUS_DURATION: Duration = Duration::from_secs(6);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Screen {
    Songs,
    Song,
    Editor,
    Tools,
    Setlist,
    Settings,
}

/// What the menu of a song in the list asks for.
enum SongAction {
    Open(String),
    Edit(String),
    AddToSetlist(String, String),
    Export(String),
    Duplicate(String),
    Delete(String),
}

/// The song on the screen, as it is on disk and as it is shown.
struct OpenSong {
    file_name: String,
    song: Song,
    /// How far the player moved the song from the key its file puts it in, a tone and a half either way at the most.
    transposition: i32,
    /// `song` moved by its own `{transpose}` and by `transposition`, kept so that a frame does not move it again.
    shown: Song,
    is_auto_scrolling: bool,
    scroll_offset: f32,
}

impl OpenSong {
    fn new(file_name: String, text: &str, prefer_flats: Option<bool>) -> OpenSong {
        let song = parser::parse(text, ChordNotation::Standard);
        let mut open = OpenSong { file_name, shown: song.clone(), song, transposition: 0, is_auto_scrolling: false, scroll_offset: 0.0 };
        open.move_song(prefer_flats);
        open
    }

    /// Moves the song to where its file and the player put it.
    fn move_song(&mut self, prefer_flats: Option<bool>) {
        self.shown = transposer::transpose_song(&self.song, self.song.metadata.transpose + self.transposition, prefer_flats);
    }

    /// Moves the song by `change` semitones, wrapping from the highest to the lowest so that every key is reachable.
    fn transpose_by(&mut self, change: i32, prefer_flats: Option<bool>) {
        self.transposition = (self.transposition + change + 5).rem_euclid(12) - 5;
        self.move_song(prefer_flats);
    }
}

pub struct App {
    library: Library,
    settings: Settings,
    settings_path: PathBuf,
    songs: Vec<SongEntry>,
    search: String,
    screen: Screen,
    open: Option<OpenSong>,
    editor: Option<Editor>,
    tools: Option<Tools>,
    song_metronome: Option<SongMetronome>,
    songs_tab: SongsTab,
    setlists: Vec<(String, library::Setlist)>,
    new_setlist_title: String,
    current_setlist: Option<SetlistState>,
    /// The repertoire and the place in it of the song on the screen, where it was opened from one.
    from_setlist: Option<(String, usize)>,
    /// The chord whose shapes are shown over the song, as the file writes it.
    chord_window: Option<String>,
    /// The song the player asked to delete, until they confirm.
    song_to_delete: Option<String>,
    status: Option<(String, Instant)>,
    is_dark_applied: Option<bool>,
    fonts_applied: Option<(String, String)>,
}

impl App {
    pub fn new(context: &egui::Context) -> App {
        let _ = context;
        let root = library::default_root();
        let settings_path = Settings::path_for(&root);
        let library = Library::open(&root).unwrap_or_else(|error| panic!("No se pudo abrir la biblioteca en {}: {error}", root.display()));
        let settings = Settings::load(&settings_path);
        let mut app = App {
            library,
            settings,
            settings_path,
            songs: Vec::new(),
            search: String::new(),
            screen: Screen::Songs,
            open: None,
            editor: None,
            tools: None,
            song_metronome: None,
            songs_tab: SongsTab::Songs,
            setlists: Vec::new(),
            new_setlist_title: String::new(),
            current_setlist: None,
            from_setlist: None,
            chord_window: None,
            song_to_delete: None,
            status: None,
            is_dark_applied: None,
            fonts_applied: None,
        };
        app.plant_demo_library();
        app.reload_songs();
        app.reload_setlists();
        app
    }

    /// The two songs a fresh install opens with, planted once: a library the player emptied on purpose stays empty.
    fn plant_demo_library(&mut self) {
        let marker = self.settings_path.with_file_name("demo-planted");
        if marker.exists() || self.library.song_names().map(|names| !names.is_empty()).unwrap_or(true) {
            return;
        }
        for (name, text) in DEMO_SONGS {
            if let Err(error) = self.library.write_song(name, text) {
                self.say(format!("No se pudo preparar la canción de ejemplo: {error}"));
            }
        }
        let _ = std::fs::write(marker, "");
    }

    fn say(&mut self, message: impl Into<String>) {
        self.status = Some((message.into(), Instant::now()));
    }

    fn reload_songs(&mut self) {
        match self.library.scan_songs() {
            Ok(mut songs) => {
                songs.sort_by_key(|entry| entry.display_title().to_lowercase());
                self.songs = songs;
            }
            Err(error) => self.say(format!("No se pudo leer la biblioteca: {error}")),
        }
    }

    fn save_settings(&mut self) {
        if let Err(error) = self.settings.save(&self.settings_path) {
            self.say(format!("No se pudieron guardar los ajustes: {error}"));
        }
    }

    fn open_song(&mut self, file_name: &str) {
        match self.library.read_song(file_name) {
            Ok(Some(text)) => {
                self.open = Some(OpenSong::new(file_name.to_string(), &text, self.settings.prefer_flats));
                self.from_setlist = None;
                self.song_metronome = None;
                self.screen = Screen::Song;
            }
            Ok(None) => self.say("La canción ya no existe."),
            Err(error) => self.say(format!("No se pudo abrir la canción: {error}")),
        }
    }

    fn edit_song(&mut self, file_name: &str) {
        match self.library.read_song(file_name) {
            Ok(Some(text)) => {
                self.editor = Some(Editor::new(Some(file_name.to_string()), text));
                self.screen = Screen::Editor;
            }
            Ok(None) => self.say("La canción ya no existe."),
            Err(error) => self.say(format!("No se pudo abrir la canción: {error}")),
        }
    }

    fn new_song(&mut self) {
        self.editor = Some(Editor::new(None, Editor::NEW_SONG.to_string()));
        self.screen = Screen::Editor;
    }

    /// Writes the song of the editor to its file; a new song is named after its title and its artist, which it must have.
    fn save_editor(&mut self) -> bool {
        let Some(editor) = &self.editor else {
            return false;
        };
        let text = editor.text.clone();
        let file_name = match editor.file_name.clone() {
            Some(name) => name,
            None => {
                let metadata = parser::parse_metadata(&text);
                let Some(title) = metadata.title.as_deref().map(str::trim).filter(|title| !title.is_empty()) else {
                    self.say("Para guardar la canción escribí un título: {title: …}");
                    return false;
                };
                match self.library.free_song_name(metadata.artist.as_deref(), title) {
                    Ok(name) => name,
                    Err(error) => {
                        self.say(format!("No se pudo elegir un nombre: {error}"));
                        return false;
                    }
                }
            }
        };
        if let Err(error) = self.library.write_song(&file_name, &text) {
            self.say(format!("No se pudo guardar: {error}"));
            return false;
        }
        if let Some(editor) = &mut self.editor {
            editor.mark_saved(file_name.clone());
        }
        if self.open.as_ref().is_some_and(|open| open.file_name == file_name) {
            let flats = self.settings.prefer_flats;
            if let Some(open) = &mut self.open {
                let (transposition, is_auto_scrolling, scroll_offset) = (open.transposition, open.is_auto_scrolling, open.scroll_offset);
                *open = OpenSong::new(file_name.clone(), &text, flats);
                open.transposition = transposition;
                open.move_song(flats);
                open.is_auto_scrolling = is_auto_scrolling;
                open.scroll_offset = scroll_offset;
            }
        }
        self.reload_songs();
        self.say("Canción guardada.");
        true
    }

    /// Renames the file of the song being edited to what its title and its artist say, and every setlist with it.
    fn update_editor_file_name(&mut self) {
        if self.editor.as_ref().is_some_and(Editor::is_dirty) && !self.save_editor() {
            return;
        }
        let Some(current) = self.editor.as_ref().and_then(|editor| editor.file_name.clone()) else {
            return;
        };
        let text = self.editor.as_ref().map(|editor| editor.text.clone()).unwrap_or_default();
        let metadata = parser::parse_metadata(&text);
        let Some(title) = metadata.title.as_deref().map(str::trim).filter(|title| !title.is_empty()) else {
            self.say("La canción no tiene título: no hay de dónde sacar el nombre.");
            return;
        };
        let wanted = library::names::song_file_name(metadata.artist.as_deref(), title);
        let taken: std::collections::HashSet<String> = match self.library.song_names() {
            Ok(names) => names.iter().filter(|name| **name != current).map(|name| library::names::identity_key(name)).collect(),
            Err(error) => {
                self.say(format!("No se pudo leer la biblioteca: {error}"));
                return;
            }
        };
        let new_name = library::names::unique_name(&taken, &wanted);
        if new_name == current {
            self.say("El nombre del archivo ya es el que corresponde.");
            return;
        }
        match self.library.rename_song(&current, &new_name) {
            Ok(()) => {
                if let Some(editor) = &mut self.editor {
                    editor.file_name = Some(new_name.clone());
                }
                if let Some(open) = &mut self.open {
                    if open.file_name == current {
                        open.file_name = new_name.clone();
                    }
                }
                self.reload_songs();
                self.say(format!("Archivo renombrado a {new_name}"));
            }
            Err(error) => self.say(format!("No se pudo renombrar: {error}")),
        }
    }

    /// Opens the tools on `tab`, the metronome following the song on the screen if there is one.
    fn open_tools(&mut self, tab: ToolTab) {
        let mut tools = self.tools.take().unwrap_or_else(|| Tools::new(tab));
        if let Some(open) = &self.open {
            let metadata = &open.shown.metadata;
            tools.follow(&SongTiming {
                tempo: metadata.tempo.as_deref().and_then(chordpro::values::parse_tempo).map(|tempo| tempo as u32),
                time: metadata.time.as_deref().and_then(chordpro::values::parse_time).map(|(beats, unit)| (beats as u32, unit as u32)),
            });
        }
        tools.tab = tab;
        self.tools = Some(tools);
        self.screen = Screen::Tools;
    }

    fn tools_screen(&mut self, ui: &mut egui::Ui) {
        let mut leave = false;
        ui.horizontal(|ui| {
            if ui.button("← Volver").clicked() {
                leave = true;
            }
            ui.heading("Herramientas");
        });
        let colors = SongColors::of(self.settings.is_dark);
        if let Some(tools) = &mut self.tools {
            tools.show(ui, &self.settings, &colors);
        }
        if leave {
            // The click never outlives its screen, and the microphone is closed with it.
            self.tools = None;
            self.screen = if self.open.is_some() { Screen::Song } else { Screen::Songs };
        }
    }

    fn close_editor(&mut self) {
        self.editor = None;
        self.screen = if self.open.is_some() { Screen::Song } else { Screen::Songs };
    }

    fn editor_screen(&mut self, ui: &mut egui::Ui) {
        let Some(editor) = &mut self.editor else {
            self.screen = Screen::Songs;
            return;
        };
        let colors = SongColors::of(self.settings.is_dark);
        match editor.show(ui, &colors, self.settings.editor_text_size) {
            EditorAction::None => {}
            EditorAction::Save => {
                self.save_editor();
            }
            EditorAction::Close | EditorAction::DiscardAndClose => self.close_editor(),
            EditorAction::UpdateFileName => self.update_editor_file_name(),
        }
    }

    fn transpose_open(&mut self, change: i32) {
        let flats = self.settings.prefer_flats;
        if let Some(open) = &mut self.open {
            open.transpose_by(change, flats);
        }
        self.remember_transposition();
    }

    fn import_zip(&mut self) {
        let Some(path) = rfd::FileDialog::new().set_title("Importar canciones").add_filter("Zip o canciones", &["zip", "cho", "chopro", "crd", "txt"]).pick_file() else {
            return;
        };
        let Ok(bytes) = std::fs::read(&path) else {
            self.say("No se pudo leer el archivo.");
            return;
        };
        let name = path.file_name().and_then(|name| name.to_str()).unwrap_or("").to_string();
        let result = if library::archive::is_zip_archive(&bytes) {
            self.library.import_archive(&bytes, Collision::Number)
        } else {
            let file = library::archive::ImportedFile { name, bytes };
            self.library.plan_import(vec![file], Vec::new()).and_then(|plan| self.library.apply_import(plan, Collision::Number)).map_err(library::ExportError::from)
        };
        match result {
            Ok(report) => {
                let added = report.added.len() + report.numbered.len();
                self.say(format!("Importación lista: {added} nuevas, {} ya estaban, {} omitidas.", report.unchanged.len(), report.skipped.len()));
                self.reload_songs();
            }
            Err(error) => self.say(format!("No se pudo importar: {error}")),
        }
    }

    fn export_zip(&mut self) {
        let archive = match self.library.export_archive() {
            Ok(Some(archive)) => archive,
            Ok(None) => {
                self.say("La biblioteca está vacía: no hay nada para exportar.");
                return;
            }
            Err(error) => {
                self.say(format!("No se pudo exportar: {error}"));
                return;
            }
        };
        let Some(path) = rfd::FileDialog::new().set_title("Exportar biblioteca").set_file_name(Library::archive_name()).add_filter("Zip", &["zip"]).save_file() else {
            return;
        };
        match std::fs::write(&path, archive) {
            Ok(()) => self.say(format!("Biblioteca exportada a {}", path.display())),
            Err(error) => self.say(format!("No se pudo guardar el zip: {error}")),
        }
    }

    /// A chord sheet pasted over the list of songs becomes a new song in the editor, with its chords set into the words.
    fn paste_into_list(&mut self, ui: &egui::Ui) {
        if ui.memory(|memory| memory.focused().is_some()) {
            return;
        }
        let pasted = ui.input_mut(|input| {
            let index = input.events.iter().position(|event| matches!(event, egui::Event::Paste(_)))?;
            match input.events.remove(index) {
                egui::Event::Paste(text) => Some(text),
                _ => None,
            }
        });
        let Some(text) = pasted else {
            return;
        };
        match chordpro::convert::convert_song(&text) {
            Some(song) => {
                self.editor = Some(Editor::new(None, song));
                self.screen = Screen::Editor;
                self.say("Hoja de acordes convertida a ChordPro. Revisá el título y guardá.");
            }
            None => {
                let song = format!("{{title: }}\n{{artist: }}\n{{key: }}\n\n{}", text.replace("\r\n", "\n"));
                self.editor = Some(Editor::new(None, song));
                self.screen = Screen::Editor;
                self.say("Texto pegado como canción nueva. No encontré acordes para colocar.");
            }
        }
    }

    fn songs_screen(&mut self, ui: &mut egui::Ui) {
        self.paste_into_list(ui);
        ui.horizontal(|ui| {
            ui.heading("SDCancionero");
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.button("Ajustes").clicked() {
                    self.screen = Screen::Settings;
                }
                if ui.button("Herramientas").on_hover_text("Metrónomo, afinador y acordes").clicked() {
                    self.open_tools(ToolTab::Metronome);
                }
                if ui.button("Exportar zip").clicked() {
                    self.export_zip();
                }
                if ui.button("Importar").clicked() {
                    self.import_zip();
                }
                if ui.button("+ Nueva").clicked() {
                    self.new_song();
                }
            });
        });
        ui.horizontal(|ui| {
            ui.selectable_value(&mut self.songs_tab, SongsTab::Songs, RichText::new("Canciones").size(16.0));
            ui.selectable_value(&mut self.songs_tab, SongsTab::Setlists, RichText::new("Repertorios").size(16.0));
        });
        ui.separator();
        if self.songs_tab == SongsTab::Setlists {
            self.setlists_tab(ui);
            return;
        }
        ui.add(egui::TextEdit::singleline(&mut self.search).hint_text("Buscar por título o artista").desired_width(f32::INFINITY));
        ui.add_space(6.0);
        let query = self.search.trim().to_lowercase();
        let matching: Vec<usize> = (0..self.songs.len())
            .filter(|&index| {
                let entry = &self.songs[index];
                query.is_empty()
                    || entry.display_title().to_lowercase().contains(&query)
                    || entry.metadata.artist.as_deref().is_some_and(|artist| artist.to_lowercase().contains(&query))
            })
            .collect();
        if self.songs.is_empty() {
            ui.add_space(24.0);
            ui.label("La biblioteca está vacía. Importá un zip o canciones en ChordPro con el botón «Importar».");
            return;
        }
        let mut chosen: Option<String> = None;
        let mut action: Option<SongAction> = None;
        let colors = SongColors::of(self.settings.is_dark);
        ScrollArea::vertical().auto_shrink([false, false]).show_rows(ui, 46.0, matching.len(), |ui, range| {
            for &index in &matching[range] {
                let entry = &self.songs[index];
                ui.horizontal(|ui| {
                    let response = ui.allocate_response(Vec2::new((ui.available_width() - 44.0).max(60.0), 42.0), egui::Sense::click());
                    let painter = ui.painter();
                    if response.hovered() {
                        painter.rect_filled(response.rect, 6.0, ui.visuals().widgets.hovered.bg_fill);
                    }
                    painter.text(response.rect.left_top() + Vec2::new(10.0, 4.0), egui::Align2::LEFT_TOP, entry.display_title(), egui::FontId::new(16.0, theme::bold()), colors.text);
                    let detail = entry.metadata.artist.clone().unwrap_or_default();
                    let detail = match (&entry.metadata.key, detail.is_empty()) {
                        (Some(key), false) => format!("{detail} · {key}"),
                        (Some(key), true) => key.clone(),
                        (None, _) => detail,
                    };
                    painter.text(response.rect.left_top() + Vec2::new(10.0, 24.0), egui::Align2::LEFT_TOP, detail, egui::FontId::proportional(12.5), colors.muted);
                    if response.clicked() {
                        chosen = Some(entry.file_name.clone());
                    }
                    // The menu of the song, three dots at the end of its row.
                    let (dots, button) = ui.allocate_exact_size(Vec2::new(36.0, 42.0), egui::Sense::click());
                    let button = button.on_hover_text("Acciones");
                    if button.hovered() {
                        ui.painter().rect_filled(dots, 18.0, ui.visuals().widgets.hovered.bg_fill);
                    }
                    for step in [-1.0, 0.0, 1.0] {
                        ui.painter().circle_filled(dots.center() + Vec2::new(0.0, step * 7.0), 2.2, colors.text);
                    }
                    egui::Popup::menu(&button).show(|ui| {
                        if ui.button("Editar").clicked() {
                            action = Some(SongAction::Edit(entry.file_name.clone()));
                            ui.close();
                        }
                        if ui.button("Duplicar").clicked() {
                            action = Some(SongAction::Duplicate(entry.file_name.clone()));
                            ui.close();
                        }
                        if ui.button("Exportar").clicked() {
                            action = Some(SongAction::Export(entry.file_name.clone()));
                            ui.close();
                        }
                        if ui.button(RichText::new("Eliminar").color(egui::Color32::from_rgb(0xE5, 0x57, 0x57))).clicked() {
                            action = Some(SongAction::Delete(entry.file_name.clone()));
                            ui.close();
                        }
                        ui.separator();
                        if ui.button("Abrir").clicked() {
                            action = Some(SongAction::Open(entry.file_name.clone()));
                            ui.close();
                        }
                        ui.menu_button("Agregar a un repertorio", |ui| {
                            if self.setlists.is_empty() {
                                ui.label("No hay repertorios todavía.");
                            }
                            for (setlist_file, setlist) in &self.setlists {
                                let title = if setlist.title.is_empty() { setlist_file.trim_end_matches(".setlist.json").to_string() } else { setlist.title.clone() };
                                if ui.button(title).clicked() {
                                    action = Some(SongAction::AddToSetlist(entry.file_name.clone(), setlist_file.clone()));
                                    ui.close();
                                }
                            }
                        });
                    });
                });
            }
        });
        if let Some(file_name) = chosen {
            self.open_song(&file_name);
        }
        match action {
            Some(SongAction::Open(file_name)) => self.open_song(&file_name),
            Some(SongAction::Edit(file_name)) => self.edit_song(&file_name),
            Some(SongAction::AddToSetlist(file_name, setlist_file)) => self.add_to_setlist(&file_name, &setlist_file),
            Some(SongAction::Export(file_name)) => self.export_song(&file_name),
            Some(SongAction::Duplicate(file_name)) => self.duplicate_song(&file_name),
            Some(SongAction::Delete(file_name)) => self.song_to_delete = Some(file_name),
            None => {}
        }
    }

    /// Adds a song to the end of a repertoire.
    fn add_to_setlist(&mut self, file_name: &str, setlist_file: &str) {
        match self.library.read_setlist(setlist_file) {
            Ok(Some(mut setlist)) => {
                if setlist.songs.iter().any(|song| song.file == file_name) {
                    self.say("La canción ya está en ese repertorio.");
                    return;
                }
                setlist.songs.push(library::SetlistSong { file: file_name.to_string(), ..library::SetlistSong::default() });
                match self.library.write_setlist(setlist_file, &setlist) {
                    Ok(()) => {
                        self.say(format!("Agregada a «{}».", setlist.title));
                        self.reload_setlists();
                    }
                    Err(error) => self.say(format!("No se pudo guardar el repertorio: {error}")),
                }
            }
            Ok(None) => self.say("El repertorio ya no existe."),
            Err(error) => self.say(format!("No se pudo abrir el repertorio: {error}")),
        }
    }

    /// Saves one song as the `.cho` file it is.
    fn export_song(&mut self, file_name: &str) {
        let text = match self.library.read_song(file_name) {
            Ok(Some(text)) => text,
            Ok(None) => {
                self.say("La canción ya no existe.");
                return;
            }
            Err(error) => {
                self.say(format!("No se pudo leer la canción: {error}"));
                return;
            }
        };
        let Some(path) = rfd::FileDialog::new().set_title("Exportar canción").set_file_name(file_name).add_filter("ChordPro", &["cho"]).save_file() else {
            return;
        };
        match std::fs::write(&path, text) {
            Ok(()) => self.say(format!("Canción exportada a {}", path.display())),
            Err(error) => self.say(format!("No se pudo guardar el archivo: {error}")),
        }
    }

    /// Makes a copy of a song under a free file name, its title ending in « (copia)» so that the two can be told apart.
    fn duplicate_song(&mut self, file_name: &str) {
        let text = match self.library.read_song(file_name) {
            Ok(Some(text)) => text,
            Ok(None) => {
                self.say("La canción ya no existe.");
                return;
            }
            Err(error) => {
                self.say(format!("No se pudo leer la canción: {error}"));
                return;
            }
        };
        let taken: std::collections::HashSet<String> = match self.library.song_names() {
            Ok(names) => names.iter().map(|name| library::names::identity_key(name)).collect(),
            Err(error) => {
                self.say(format!("No se pudo leer la biblioteca: {error}"));
                return;
            }
        };
        let new_name = library::names::unique_name(&taken, file_name);
        let copy = chordpro::text_rewriter::with_title_suffix(&text, " (copia)");
        match self.library.write_song(&new_name, &copy) {
            Ok(()) => {
                self.reload_songs();
                self.say(format!("Canción duplicada como {new_name}"));
            }
            Err(error) => self.say(format!("No se pudo duplicar: {error}")),
        }
    }

    /// Deletes a song, and takes it out of every repertoire that named it.
    fn delete_song(&mut self, file_name: &str) {
        match self.library.remove_song(file_name) {
            Ok(()) => {
                if self.open.as_ref().is_some_and(|open| open.file_name == file_name) {
                    self.open = None;
                }
                self.reload_songs();
                self.reload_setlists();
                self.say("Canción eliminada.");
            }
            Err(error) => self.say(format!("No se pudo eliminar: {error}")),
        }
    }

    fn song_screen(&mut self, ui: &mut egui::Ui) {
        let Some(open) = &self.open else {
            self.screen = Screen::Songs;
            return;
        };
        let title = open.shown.metadata.display_title(open.file_name.trim_end_matches(".cho"));
        let artist = open.shown.metadata.artist.clone();
        let key = open.shown.metadata.key.clone();
        if self.song_metronome.is_none() {
            if let Some(open) = &self.open {
                let metadata = &open.shown.metadata;
                self.song_metronome = Some(SongMetronome::for_song(&SongTiming {
                    tempo: metadata.tempo.as_deref().and_then(chordpro::values::parse_tempo).map(|tempo| tempo as u32),
                    time: metadata.time.as_deref().and_then(chordpro::values::parse_time).map(|(beats, unit)| (beats as u32, unit as u32)),
                }));
            }
        }
        let metronome_colors = SongColors::of(self.settings.is_dark);
        let mut step_in_setlist: Option<i32> = None;
        let mut leave = false;
        let mut edit = false;
        let mut open_tools = false;
        let mut transpose = 0;
        let mut toggle_scroll = false;
        let mut changed_settings = false;
        let speed = self.settings.autoscroll_speed;
        ui.horizontal_wrapped(|ui| {
            if ui.button("← Canciones").clicked() {
                leave = true;
            }
            if ui.button("Editar").clicked() {
                edit = true;
            }
            if ui.button("Herramientas").clicked() {
                open_tools = true;
            }
            if let (Some((_, index)), Some(state)) = (&self.from_setlist, &self.current_setlist) {
                let (index, count) = (*index, state.setlist.songs.len());
                if ui.add_enabled(index > 0, egui::Button::new("‹ Anterior")).clicked() {
                    step_in_setlist = Some(-1);
                }
                ui.label(format!("{}/{}", index + 1, count));
                if ui.add_enabled(index + 1 < count, egui::Button::new("Siguiente ›")).clicked() {
                    step_in_setlist = Some(1);
                }
            }
            ui.separator();
            if let Some(metronome) = &mut self.song_metronome {
                metronome.show(ui, &metronome_colors);
            }
            ui.separator();
            ui.menu_button("Escuchar ▾", |ui| {
                let query = listen::search_query(&title, open.shown.metadata.artist.as_deref());
                for service in Service::ALL {
                    if ui.button(service.name()).on_hover_text("Busca la canción en el navegador").clicked() {
                        ui.ctx().open_url(egui::OpenUrl::new_tab(service.search_url(&query)));
                        ui.close();
                    }
                }
            });
            if ui.checkbox(&mut self.settings.pin_chord_diagrams, "Diagramas").on_hover_text("Los acordes de la canción, dibujados al costado").changed() {
                changed_settings = true;
            }
            ui.separator();
            if ui.button("−").on_hover_text("Bajar un semitono").clicked() {
                transpose = -1;
            }
            let semitones = open.transposition;
            ui.label(if semitones == 0 { "Tono original".to_string() } else { format!("{semitones:+}") });
            if ui.button("+").on_hover_text("Subir un semitono").clicked() {
                transpose = 1;
            }
            ui.separator();
            if ui.checkbox(&mut self.settings.show_chords, "Acordes").changed() {
                changed_settings = true;
            }
            ui.label("Columnas");
            if ui.add(egui::DragValue::new(&mut self.settings.columns).range(1..=settings::MAX_COLUMNS)).changed() {
                changed_settings = true;
            }
            ui.label("Tamaño");
            if ui.add(egui::DragValue::new(&mut self.settings.text_size).range(settings::MIN_TEXT_SIZE..=settings::MAX_TEXT_SIZE).speed(0.5)).changed() {
                changed_settings = true;
            }
            ui.separator();
            let scroll_label = if open.is_auto_scrolling { "Detener" } else { "Autoscroll" };
            if ui.button(scroll_label).on_hover_text("Espacio").clicked() {
                toggle_scroll = true;
            }
            ui.label(format!("{speed:.0} px/s"));
        });
        ui.separator();
        if ui.input(|input| input.key_pressed(egui::Key::Space)) {
            toggle_scroll = true;
        }
        let (up, down) = ui.input(|input| (input.key_pressed(egui::Key::ArrowUp) && input.modifiers.ctrl, input.key_pressed(egui::Key::ArrowDown) && input.modifiers.ctrl));
        if up || down {
            self.settings.autoscroll_speed = (speed + if up { 5.0 } else { -5.0 }).clamp(settings::MIN_AUTOSCROLL_SPEED, settings::MAX_AUTOSCROLL_SPEED);
            changed_settings = true;
        }
        let is_dark = self.settings.is_dark;
        let colors = SongColors::of(is_dark);
        let options = ViewOptions {
            columns: self.settings.columns,
            text_size: self.settings.text_size,
            notation: self.settings.notation,
            accidental_signs: self.settings.accidental_signs,
            show_chords: self.settings.show_chords,
            colors: &colors,
        };
        let delta_seconds = ui.input(|input| input.stable_dt).min(0.1);
        let open = self.open.as_mut().expect("checked above");
        let mut area = ScrollArea::vertical().auto_shrink([false, false]);
        if open.is_auto_scrolling {
            area = area.vertical_scroll_offset(open.scroll_offset + self.settings.autoscroll_speed * delta_seconds);
        }
        let mut tapped_chord: Option<String> = None;
        let output = area.show(ui, |ui| {
            ui.heading(&title);
            let detail: Vec<String> = [artist, key.map(|key| format!("Tono: {key}"))].into_iter().flatten().collect();
            if !detail.is_empty() {
                ui.label(RichText::new(detail.join(" · ")).color(colors.muted));
            }
            ui.add_space(10.0);
            let hits = viewer::show(ui, &open.shown, &options);
            for (index, hit) in hits.iter().enumerate() {
                let response = ui.interact(hit.rect, egui::Id::unique(("song_chord", index)), egui::Sense::click());
                if response.on_hover_cursor(egui::CursorIcon::PointingHand).clicked() {
                    tapped_chord = Some(hit.name.clone());
                }
            }
        });
        let end = (output.content_size.y - output.inner_rect.height()).max(0.0);
        let reached_end = open.is_auto_scrolling && output.state.clamped_offset().y >= end && end > 0.0;
        open.scroll_offset = output.state.clamped_offset().y;
        if reached_end {
            open.is_auto_scrolling = false;
        }
        if open.is_auto_scrolling {
            ui.ctx().request_repaint();
        }
        if toggle_scroll {
            open.is_auto_scrolling = !open.is_auto_scrolling;
        }
        if leave {
            self.screen = if self.from_setlist.is_some() { Screen::Setlist } else { Screen::Songs };
            self.from_setlist = None;
        }
        if let (Some(step), Some((_, index))) = (step_in_setlist, self.from_setlist.clone()) {
            self.open_from_setlist((index as i32 + step).max(0) as usize);
        }
        if let Some(chord) = tapped_chord {
            self.chord_window = Some(chord);
        }
        if open_tools {
            self.open_tools(ToolTab::Metronome);
        }
        if edit {
            let file_name = self.open.as_ref().map(|open| open.file_name.clone());
            if let Some(file_name) = file_name {
                self.edit_song(&file_name);
            }
        }
        if transpose != 0 {
            self.transpose_open(transpose);
        }
        if changed_settings {
            self.save_settings();
        }
    }

    fn settings_screen(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            if ui.button("← Volver").clicked() {
                self.screen = if self.open.is_some() { Screen::Song } else { Screen::Songs };
            }
            ui.heading("Ajustes");
        });
        ui.separator();
        let mut changed = false;
        egui::Grid::new("settings").num_columns(2).spacing([16.0, 10.0]).show(ui, |ui| {
            ui.label("Tema");
            ui.horizontal(|ui| {
                changed |= ui.radio_value(&mut self.settings.is_dark, true, "Oscuro").changed();
                changed |= ui.radio_value(&mut self.settings.is_dark, false, "Claro").changed();
            });
            ui.end_row();
            ui.label("Notación de acordes");
            egui::ComboBox::from_id_salt("notation")
                .selected_text(NOTATIONS.iter().find(|(notation, _)| *notation == self.settings.notation).map(|(_, name)| *name).unwrap_or(""))
                .show_ui(ui, |ui| {
                    for (notation, name) in NOTATIONS {
                        changed |= ui.selectable_value(&mut self.settings.notation, notation, name).changed();
                    }
                });
            ui.end_row();
            ui.label("Alteraciones");
            changed |= ui.checkbox(&mut self.settings.accidental_signs, "Mostrar ♯ y ♭ en lugar de # y b").changed();
            ui.end_row();
            ui.label("Al transponer");
            ui.horizontal(|ui| {
                changed |= ui.radio_value(&mut self.settings.prefer_flats, None, "Según el tono").changed();
                changed |= ui.radio_value(&mut self.settings.prefer_flats, Some(false), "Sostenidos").changed();
                changed |= ui.radio_value(&mut self.settings.prefer_flats, Some(true), "Bemoles").changed();
            });
            ui.end_row();
            ui.label("Instrumento");
            ui.horizontal(|ui| {
                changed |= ui.radio_value(&mut self.settings.instrument, Instrument::Guitar, "Guitarra").changed();
                changed |= ui.radio_value(&mut self.settings.instrument, Instrument::Ukulele, "Ukelele").changed();
            });
            ui.end_row();
            ui.label("Diagramas de acordes");
            changed |= ui.checkbox(&mut self.settings.pin_chord_diagrams, "Fijarlos junto a la canción").changed();
            ui.end_row();
            ui.label("Tipografía de las canciones");
            egui::ComboBox::from_id_salt("view_font").selected_text(self.settings.view_font.clone()).show_ui(ui, |ui| {
                for name in theme::view_fonts() {
                    changed |= ui.selectable_value(&mut self.settings.view_font, name.to_string(), name).changed();
                }
            });
            ui.end_row();
            ui.label("Tipografía del editor");
            egui::ComboBox::from_id_salt("editor_font").selected_text(self.settings.editor_font.clone()).show_ui(ui, |ui| {
                for name in theme::editor_fonts() {
                    changed |= ui.selectable_value(&mut self.settings.editor_font, name.to_string(), name).changed();
                }
            });
            ui.end_row();
            ui.label("Tamaño del editor");
            changed |= ui.add(egui::Slider::new(&mut self.settings.editor_text_size, settings::MIN_TEXT_SIZE..=settings::MAX_TEXT_SIZE)).changed();
            ui.end_row();
            ui.label("Columnas");
            changed |= ui.add(egui::Slider::new(&mut self.settings.columns, 1..=settings::MAX_COLUMNS)).changed();
            ui.end_row();
            ui.label("Tamaño del texto");
            changed |= ui.add(egui::Slider::new(&mut self.settings.text_size, settings::MIN_TEXT_SIZE..=settings::MAX_TEXT_SIZE)).changed();
            ui.end_row();
            ui.label("Velocidad del autoscroll");
            changed |= ui.add(egui::Slider::new(&mut self.settings.autoscroll_speed, settings::MIN_AUTOSCROLL_SPEED..=settings::MAX_AUTOSCROLL_SPEED).suffix(" px/s")).changed();
            ui.end_row();
        });
        ui.add_space(16.0);
        ui.separator();
        ui.horizontal(|ui| {
            ui.label(RichText::new(format!("Biblioteca: {}", self.library.root().display())).small());
            if ui.small_button("Abrir carpeta").clicked() {
                let _ = std::process::Command::new("explorer").arg(self.library.root()).spawn();
            }
        });
        ui.horizontal(|ui| {
            if ui.button("Importar…").clicked() {
                self.import_zip();
            }
            if ui.button("Exportar zip").clicked() {
                self.export_zip();
            }
            if ui.button("Restablecer ajustes").on_hover_text("Vuelve todo lo de esta pantalla a como venía").clicked() {
                self.settings = Settings::default();
                changed = true;
            }
        });
        ui.add_space(8.0);
        ui.label(RichText::new(format!("SDCancionero {} · versión de Windows de 32 bits, hecha en Rust", env!("CARGO_PKG_VERSION"))).small().weak());
        if changed {
            self.save_settings();
            let flats = self.settings.prefer_flats;
            if let Some(open) = &mut self.open {
                open.move_song(flats);
            }
        }
    }
}

impl eframe::App for App {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let fonts = (self.settings.view_font.clone(), self.settings.editor_font.clone());
        if self.fonts_applied.as_ref() != Some(&fonts) {
            theme::install_fonts(ui.ctx(), &fonts.0, &fonts.1);
            self.fonts_applied = Some(fonts);
        }
        if self.is_dark_applied != Some(self.settings.is_dark) {
            theme::apply_visuals(ui.ctx(), self.settings.is_dark);
            self.is_dark_applied = Some(self.settings.is_dark);
        }
        if let Some((_, since)) = &self.status {
            if since.elapsed() > STATUS_DURATION {
                self.status = None;
            }
        }
        if let Some((message, _)) = self.status.clone() {
            egui::Panel::bottom("status").show(ui, |ui| {
                ui.label(message);
            });
            ui.ctx().request_repaint_after(STATUS_DURATION);
        }
        if self.screen == Screen::Song && self.settings.pin_chord_diagrams {
            if let Some(open) = &self.open {
                let colors = SongColors::of(self.settings.is_dark);
                let chords = chordpro::chords_in(&open.shown);
                let settings = self.settings.clone();
                // A column of diagrams, two to a row, that never takes more room than they need.
                egui::Panel::right("pinned_chords").resizable(false).exact_size(236.0).show(ui, |ui| {
                    ui.heading("Acordes");
                    egui::ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| {
                        egui::Grid::new("pinned_chords_grid").num_columns(2).spacing([6.0, 6.0]).show(ui, |ui| {
                            for (index, name) in chords.iter().enumerate() {
                                tools::pinned_chord(ui, name, &settings, &colors);
                                if index % 2 == 1 {
                                    ui.end_row();
                                }
                            }
                        });
                    });
                });
            }
        }
        if self.screen == Screen::Editor {
            if let Some(editor) = &mut self.editor {
                if editor.show_reference {
                    egui::Panel::right("reference").resizable(true).default_size(360.0).size_range(260.0..=520.0).show(ui, |ui| editor.show_reference(ui));
                }
            }
        }
        if let Some(file_name) = self.song_to_delete.clone() {
            let title = self.songs.iter().find(|entry| entry.file_name == file_name).map(SongEntry::display_title).unwrap_or_else(|| file_name.clone());
            egui::Window::new("Eliminar canción").collapsible(false).resizable(false).anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0]).show(ui.ctx(), |ui| {
                ui.label(format!("¿Eliminar «{title}»?"));
                ui.label(RichText::new("Se borra el archivo y se quita de los repertorios que la nombran. No se puede deshacer.").small().weak());
                ui.add_space(6.0);
                ui.horizontal(|ui| {
                    if ui.button(RichText::new("Eliminar").color(egui::Color32::from_rgb(0xE5, 0x57, 0x57))).clicked() {
                        self.song_to_delete = None;
                        self.delete_song(&file_name);
                    }
                    if ui.button("Cancelar").clicked() {
                        self.song_to_delete = None;
                    }
                });
            });
        }
        if let Some(name) = self.chord_window.clone() {
            let colors = SongColors::of(self.settings.is_dark);
            let mut is_open = true;
            egui::Window::new("Acorde").open(&mut is_open).resizable(true).default_width(420.0).show(ui.ctx(), |ui| {
                tools::chord_sheet(ui, &name, &self.settings, &colors, 6, 1.0);
            });
            if !is_open {
                self.chord_window = None;
            }
        }
        if self.screen != Screen::Song {
            self.song_metronome = None;
        }
        egui::CentralPanel::default().show(ui, |ui| match self.screen {
            Screen::Songs => self.songs_screen(ui),
            Screen::Song => self.song_screen(ui),
            Screen::Editor => self.editor_screen(ui),
            Screen::Tools => self.tools_screen(ui),
            Screen::Setlist => self.setlist_screen(ui),
            Screen::Settings => self.settings_screen(ui),
        });
    }
}

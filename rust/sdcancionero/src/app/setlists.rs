// This file is part of Campfire.
// Copyright (c) Pandula Péter 2017-2026.
// https://github.com/pandulapeter/campfire
//
// This Source Code Form is subject to the terms of the Mozilla Public License, v. 2.0.
// If a copy of the MPL was not distributed with this file, You can obtain one at
// https://mozilla.org/MPL/2.0/.

//! The repertoires: ordered lists of songs, each with the transposition it is played at, kept as `*.setlist.json` files the
//! Kotlin app reads and writes too.

use super::*;
use library::{Setlist, SetlistSong};

/// Which list the first screen shows.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum SongsTab {
    Songs,
    Setlists,
}

/// The repertoire on the screen.
pub(super) struct SetlistState {
    pub(super) file_name: String,
    pub(super) setlist: Setlist,
    is_adding: bool,
    add_search: String,
    is_confirming_delete: bool,
}

/// The transposition of a setlist entry as the song screen holds it, a tone and a half either way.
fn within_range(transposition: i64) -> i32 {
    ((transposition as i32 + 5).rem_euclid(12)) - 5
}

impl App {
    pub(super) fn reload_setlists(&mut self) {
        let names = match self.library.setlist_names() {
            Ok(names) => names,
            Err(error) => {
                self.say(format!("No se pudieron leer los repertorios: {error}"));
                return;
            }
        };
        let mut setlists: Vec<(String, Setlist)> = names.into_iter().filter_map(|name| self.library.read_setlist(&name).ok().flatten().map(|setlist| (name, setlist))).collect();
        setlists.sort_by_key(|(_, setlist)| setlist.title.to_lowercase());
        self.setlists = setlists;
    }

    fn song_title(&self, file_name: &str) -> Option<String> {
        self.songs.iter().find(|entry| entry.file_name == file_name).map(SongEntry::display_title)
    }

    fn save_current_setlist(&mut self) {
        let Some(state) = &self.current_setlist else {
            return;
        };
        if let Err(error) = self.library.write_setlist(&state.file_name, &state.setlist) {
            self.say(format!("No se pudo guardar el repertorio: {error}"));
        }
    }

    pub(super) fn open_setlist(&mut self, file_name: &str) {
        match self.library.read_setlist(file_name) {
            Ok(Some(setlist)) => {
                self.current_setlist = Some(SetlistState { file_name: file_name.to_string(), setlist, is_adding: false, add_search: String::new(), is_confirming_delete: false });
                self.screen = Screen::Setlist;
            }
            Ok(None) => self.say("El repertorio ya no existe."),
            Err(error) => self.say(format!("No se pudo abrir el repertorio: {error}")),
        }
    }

    /// Opens the song at `index` of the repertoire on the screen, at the transposition the repertoire gives it.
    pub(super) fn open_from_setlist(&mut self, index: usize) {
        let Some(state) = &self.current_setlist else {
            return;
        };
        let Some(entry) = state.setlist.songs.get(index).cloned() else {
            return;
        };
        let setlist_file = state.file_name.clone();
        self.open_song(&entry.file);
        let flats = self.settings.prefer_flats;
        if let Some(open) = &mut self.open {
            open.transposition = within_range(entry.transposition);
            open.move_song(flats);
        }
        if self.screen == Screen::Song {
            self.from_setlist = Some((setlist_file, index));
        }
    }

    /// Remembers in the repertoire the transposition the player gave the song they came to from it.
    pub(super) fn remember_transposition(&mut self) {
        let (Some((file, index)), Some(open)) = (&self.from_setlist, &self.open) else {
            return;
        };
        let transposition = open.transposition as i64;
        if let Some(state) = &mut self.current_setlist {
            if state.file_name == *file {
                if let Some(song) = state.setlist.songs.get_mut(*index) {
                    song.transposition = transposition;
                }
            }
        }
        self.save_current_setlist();
    }

    pub(super) fn setlists_tab(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.add(egui::TextEdit::singleline(&mut self.new_setlist_title).hint_text("Nombre del nuevo repertorio").desired_width(280.0));
            let can_create = !self.new_setlist_title.trim().is_empty();
            if ui.add_enabled(can_create, egui::Button::new("Crear repertorio")).clicked() {
                let title = self.new_setlist_title.trim().to_string();
                match self.library.free_setlist_name(&title) {
                    Ok(name) => {
                        let setlist = Setlist { title, ..Setlist::default() };
                        match self.library.write_setlist(&name, &setlist) {
                            Ok(()) => {
                                self.new_setlist_title.clear();
                                self.reload_setlists();
                                self.open_setlist(&name);
                            }
                            Err(error) => self.say(format!("No se pudo crear el repertorio: {error}")),
                        }
                    }
                    Err(error) => self.say(format!("No se pudo elegir un nombre: {error}")),
                }
            }
        });
        ui.add_space(8.0);
        if self.setlists.is_empty() {
            ui.label("Todavía no hay repertorios. Un repertorio es una lista ordenada de canciones, cada una con el tono en que se toca.");
            return;
        }
        let colors = SongColors::of(self.settings.is_dark);
        let mut chosen: Option<String> = None;
        ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| {
            for (file_name, setlist) in &self.setlists {
                let response = ui.allocate_response(Vec2::new(ui.available_width(), 46.0), egui::Sense::click());
                if response.hovered() {
                    ui.painter().rect_filled(response.rect, 6.0, ui.visuals().widgets.hovered.bg_fill);
                }
                let title = if setlist.title.is_empty() { file_name.trim_end_matches(".setlist.json").to_string() } else { setlist.title.clone() };
                ui.painter().text(response.rect.left_top() + Vec2::new(10.0, 5.0), egui::Align2::LEFT_TOP, title, egui::FontId::new(16.0, theme::bold()), colors.text);
                let count = setlist.songs.len();
                let detail = format!("{count} {}{}", if count == 1 { "canción" } else { "canciones" }, setlist.date.as_deref().map(|date| format!(" · {date}")).unwrap_or_default());
                ui.painter().text(response.rect.left_top() + Vec2::new(10.0, 26.0), egui::Align2::LEFT_TOP, detail, egui::FontId::proportional(12.5), colors.muted);
                if response.clicked() {
                    chosen = Some(file_name.clone());
                }
            }
        });
        if let Some(name) = chosen {
            self.open_setlist(&name);
        }
    }

    pub(super) fn setlist_screen(&mut self, ui: &mut egui::Ui) {
        if self.current_setlist.is_none() {
            self.screen = Screen::Songs;
            return;
        }
        let colors = SongColors::of(self.settings.is_dark);
        let mut leave = false;
        let mut changed = false;
        let mut open_index: Option<usize> = None;
        let mut delete_setlist = false;
        let titles: Vec<Option<String>> = self.current_setlist.as_ref().map(|state| state.setlist.songs.iter().map(|song| self.song_title(&song.file)).collect()).unwrap_or_default();
        let candidates: Vec<(String, String)> = self.songs.iter().map(|entry| (entry.file_name.clone(), entry.display_title())).collect();
        let state = self.current_setlist.as_mut().expect("checked above");
        ui.horizontal(|ui| {
            if ui.button("← Repertorios").clicked() {
                leave = true;
            }
            ui.heading("Repertorio");
        });
        ui.separator();
        ui.horizontal(|ui| {
            ui.label("Nombre");
            changed |= ui.add(egui::TextEdit::singleline(&mut state.setlist.title).desired_width(320.0)).changed();
            ui.label("Fecha");
            let mut date = state.setlist.date.clone().unwrap_or_default();
            if ui.add(egui::TextEdit::singleline(&mut date).hint_text("2026-10-25").desired_width(110.0)).changed() {
                state.setlist.date = if date.trim().is_empty() { None } else { Some(date) };
                changed = true;
            }
        });
        ui.horizontal(|ui| {
            ui.label("Notas");
            changed |= ui.add(egui::TextEdit::singleline(&mut state.setlist.description).desired_width(f32::INFINITY)).changed();
        });
        ui.add_space(6.0);
        ui.horizontal(|ui| {
            ui.toggle_value(&mut state.is_adding, "Agregar canciones");
            if state.is_confirming_delete {
                ui.label("¿Eliminar este repertorio? Las canciones no se borran.");
                if ui.button("Sí, eliminar").clicked() {
                    delete_setlist = true;
                }
                if ui.button("No").clicked() {
                    state.is_confirming_delete = false;
                }
            } else if ui.button("Eliminar repertorio").clicked() {
                state.is_confirming_delete = true;
            }
        });
        ui.separator();
        if state.is_adding {
            ui.add(egui::TextEdit::singleline(&mut state.add_search).hint_text("Buscar canción para agregar").desired_width(f32::INFINITY));
            let query = state.add_search.trim().to_lowercase();
            let mut to_add: Option<String> = None;
            ScrollArea::vertical().id_salt("candidates").max_height(220.0).show(ui, |ui| {
                for (file_name, title) in &candidates {
                    if !query.is_empty() && !title.to_lowercase().contains(&query) {
                        continue;
                    }
                    ui.horizontal(|ui| {
                        if ui.small_button("+").on_hover_text("Agregar al repertorio").clicked() {
                            to_add = Some(file_name.clone());
                        }
                        ui.label(title);
                    });
                }
            });
            if let Some(file) = to_add {
                state.setlist.songs.push(SetlistSong { file, ..SetlistSong::default() });
                changed = true;
            }
            ui.separator();
        }
        if state.setlist.songs.is_empty() {
            ui.label("El repertorio está vacío. Con «Agregar canciones» elegís las que tocás.");
        }
        let count = state.setlist.songs.len();
        let mut swap: Option<(usize, usize)> = None;
        let mut remove: Option<usize> = None;
        ScrollArea::vertical().id_salt("songs").auto_shrink([false, false]).show(ui, |ui| {
            for (index, song) in state.setlist.songs.iter_mut().enumerate() {
                ui.horizontal(|ui| {
                    ui.label(RichText::new(format!("{}.", index + 1)).color(colors.muted));
                    let title = titles.get(index).cloned().flatten();
                    match title {
                        Some(title) => {
                            if ui.link(RichText::new(title).size(16.0)).on_hover_text("Abrir").clicked() {
                                open_index = Some(index);
                            }
                        }
                        None => {
                            ui.label(RichText::new(format!("{} (no está en la biblioteca)", song.file)).color(colors.muted));
                        }
                    }
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.small_button("×").on_hover_text("Quitar del repertorio").clicked() {
                            remove = Some(index);
                        }
                        if ui.add_enabled(index + 1 < count, egui::Button::new("↓").small()).on_hover_text("Bajar").clicked() {
                            swap = Some((index, index + 1));
                        }
                        if ui.add_enabled(index > 0, egui::Button::new("↑").small()).on_hover_text("Subir").clicked() {
                            swap = Some((index, index - 1));
                        }
                        let current = within_range(song.transposition);
                        if ui.small_button("+").on_hover_text("Subir un semitono").clicked() {
                            song.transposition = within_range(song.transposition + 1) as i64;
                            changed = true;
                        }
                        ui.label(if current == 0 { "tono original".to_string() } else { format!("{current:+}") });
                        if ui.small_button("−").on_hover_text("Bajar un semitono").clicked() {
                            song.transposition = within_range(song.transposition - 1) as i64;
                            changed = true;
                        }
                    });
                });
            }
        });
        if let Some((a, b)) = swap {
            state.setlist.songs.swap(a, b);
            changed = true;
        }
        if let Some(index) = remove {
            state.setlist.songs.remove(index);
            changed = true;
        }
        if changed {
            self.save_current_setlist();
            self.reload_setlists();
        }
        if let Some(index) = open_index {
            self.open_from_setlist(index);
        }
        if delete_setlist {
            if let Some(state) = self.current_setlist.take() {
                match self.library.delete_setlist(&state.file_name) {
                    Ok(()) => self.say("Repertorio eliminado."),
                    Err(error) => self.say(format!("No se pudo eliminar el repertorio: {error}")),
                }
            }
            self.reload_setlists();
            self.songs_tab = SongsTab::Setlists;
            self.screen = Screen::Songs;
        }
        if leave {
            self.songs_tab = SongsTab::Setlists;
            self.screen = Screen::Songs;
        }
    }
}

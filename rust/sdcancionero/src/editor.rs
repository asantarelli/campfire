// This file is part of Campfire.
// Copyright (c) Pandula Péter 2017-2026.
// https://github.com/pandulapeter/campfire
//
// This Source Code Form is subject to the terms of the Mozilla Public License, v. 2.0.
// If a copy of the MPL was not distributed with this file, You can obtain one at
// https://mozilla.org/MPL/2.0/.

//! The editor: the text of a song with the buttons that write ChordPro for you, and a reference of everything it can say.

use std::sync::Arc;

use chordpro::edit::{self, Edited};
use chordpro::transposer;
use eframe::egui::text::{CCursor, CCursorRange, LayoutJob, TextFormat};
use eframe::egui::{self, Color32, FontId, Id, RichText, ScrollArea, Ui};

use crate::theme::SongColors;

/// What a button of the editor writes into the text.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Insertion {
    /// Around the selection, inside its line: a chord is `[` and `]`.
    Wrap { prefix: &'static str, suffix: &'static str },
    /// `{name: selection}` on a line of its own.
    Directive { name: &'static str, has_value: bool },
    /// `{start_of_name}` and `{end_of_name}` around the selected lines.
    Section { name: &'static str },
    /// A directive of the header, which a song can say `repeatable` times.
    Metadata { name: &'static str, repeatable: bool },
    /// `{meta: key value}` in the header.
    Meta { key: &'static str, repeatable: bool },
    /// Characters a keyboard has no key for, written at the caret.
    Symbol(&'static str),
    /// A tablature, around the selected lines or around an empty staff, for one instrument or for any.
    Tab { selector: Option<&'static str>, staff: &'static [&'static str] },
    /// A blank `{define}` of a shape of this many strings, in the header.
    Define { strings: usize },
    /// A directive said in the header and then again as a change from a line on (`tempo`, `time`).
    Changeable { name: &'static str },
}

/// The name a blank shape is given, which is typed over.
const SHAPE_NAME: &str = "Nombre";

/// The empty staff of one bar a new tablature starts as, its strings named from the highest.
pub const GUITAR_STAFF: [&str; 6] = ["e|----------------|", "B|----------------|", "G|----------------|", "D|----------------|", "A|----------------|", "E|----------------|"];
pub const UKULELE_STAFF: [&str; 4] = ["A|----------------|", "E|----------------|", "C|----------------|", "G|----------------|"];

impl Insertion {
    fn apply(self, text: &str, start: usize, end: usize) -> Edited {
        match self {
            Insertion::Wrap { prefix, suffix } => edit::wrap_selection(text, start, end, prefix, suffix),
            Insertion::Directive { name, has_value } => edit::insert_directive(text, start, end, name, has_value),
            Insertion::Section { name } => edit::insert_section(text, start, end, name),
            Insertion::Metadata { name, repeatable } => edit::insert_metadata(text, name, repeatable),
            Insertion::Meta { key, repeatable } => edit::insert_meta(text, key, repeatable),
            Insertion::Symbol(symbol) => edit::insert_at_caret(text, start, end, symbol),
            Insertion::Tab { selector, staff } => edit::insert_tab(text, start, end, selector, &staff.iter().map(|line| line.to_string()).collect::<Vec<_>>()),
            Insertion::Define { strings } => edit::insert_define(text, SHAPE_NAME, strings),
            Insertion::Changeable { name } => edit::insert_changeable(text, start, end, name),
        }
    }
}

/// One line of the reference: what it is called, how it is written, what it does, and what its button writes.
pub struct ReferenceEntry {
    pub title: &'static str,
    pub syntax: &'static str,
    pub description: &'static str,
    pub insertion: Insertion,
}

pub struct ReferenceGroup {
    pub title: &'static str,
    pub entries: Vec<ReferenceEntry>,
}

fn entry(title: &'static str, syntax: &'static str, description: &'static str, insertion: Insertion) -> ReferenceEntry {
    ReferenceEntry { title, syntax, description, insertion }
}

fn metadata(title: &'static str, name: &'static str, example: &'static str, description: &'static str, repeatable: bool) -> ReferenceEntry {
    let syntax: &'static str = Box::leak(format!("{{{name}: {example}}}").into_boxed_str());
    entry(title, syntax, description, Insertion::Metadata { name, repeatable })
}

/// A header directive that can be said again further down as a change, which the first time goes into the header.
fn changeable(title: &'static str, name: &'static str, example: &'static str, description: &'static str) -> ReferenceEntry {
    let syntax: &'static str = Box::leak(format!("{{{name}: {example}}}").into_boxed_str());
    entry(title, syntax, description, Insertion::Changeable { name })
}

fn meta(title: &'static str, key: &'static str, example: &'static str, description: &'static str, repeatable: bool) -> ReferenceEntry {
    let syntax: &'static str = Box::leak(format!("{{meta: {key} {example}}}").into_boxed_str());
    entry(title, syntax, description, Insertion::Meta { key, repeatable })
}

fn directive(title: &'static str, name: &'static str, example: &'static str, description: &'static str) -> ReferenceEntry {
    let syntax: &'static str = Box::leak(format!("{{{name}: {example}}}").into_boxed_str());
    entry(title, syntax, description, Insertion::Directive { name, has_value: true })
}

fn section(title: &'static str, name: &'static str, description: &'static str) -> ReferenceEntry {
    let syntax: &'static str = Box::leak(format!("{{start_of_{name}}} … {{end_of_{name}}}").into_boxed_str());
    entry(title, syntax, description, Insertion::Section { name })
}

fn tab(title: &'static str, selector: Option<&'static str>, staff: &'static [&'static str], description: &'static str) -> ReferenceEntry {
    let syntax: &'static str = Box::leak(format!("{{start_of_tab{}}} … {{end_of_tab}}", selector.map(|selector| format!("-{selector}")).unwrap_or_default()).into_boxed_str());
    entry(title, syntax, description, Insertion::Tab { selector, staff })
}

/// A blank shape of a chord of the song's own: the frets and the fingers, from the lowest string, all at 0 to be changed.
fn shape(title: &'static str, strings: usize) -> ReferenceEntry {
    let zeros = vec!["0"; strings].join(" ");
    let syntax: &'static str = Box::leak(format!("{{define: {SHAPE_NAME} base-fret 1 frets {zeros} fingers {zeros}}}").into_boxed_str());
    entry(
        title,
        syntax,
        "Dibuja el diagrama de un acorde con la forma que le das: los trastes de cada cuerda desde la más grave (x para una cuerda muda, 0 al aire, 1 para el primer traste desde base-fret) y los dedos (1 índice, 4 meñique, 0 ninguno).",
        Insertion::Define { strings },
    )
}

const SECTION_DESCRIPTION: &str = "Abre y cierra un bloque de este tipo. Todo lo que queda entre las dos líneas pertenece a la sección.";

/// Everything the toolbar writes, explained: what describes the song, what goes inside a line, the sections, the comments and the
/// changes a song makes as it goes.
pub fn reference_groups() -> Vec<ReferenceGroup> {
    vec![
        ReferenceGroup {
            title: "Datos de la canción",
            entries: vec![
                metadata("Título", "title", "Home on the Range", "El título de la canción.", false),
                metadata("Subtítulo", "subtitle", "My Western Home", "Una segunda línea que se muestra bajo el título.", false),
                metadata("Artista", "artist", "Traditional American", "Quién interpreta la canción.", false),
                metadata("Compositor", "composer", "Daniel E. Kelley", "Quién compuso la música.", false),
                metadata("Letrista", "lyricist", "Brewster M. Higley", "Quién escribió la letra.", false),
                metadata("Álbum", "album", "Campfire Songs", "El álbum del que viene la canción.", false),
                metadata("Año", "year", "1872", "El año en que salió.", false),
                metadata("Tono", "key", "G", "El tono en el que están escritos los acordes.", false),
                metadata("Capo", "capo", "2", "El traste donde va el capo.", false),
                changeable("Tempo", "tempo", "100", "Pulsos por minuto, que sigue el metrónomo. La primera vez va en el encabezado; las siguientes, donde está el cursor, como un cambio desde esa línea."),
                changeable("Compás", "time", "3/4", "El compás, como 3/4 o 6/8. La primera vez va en el encabezado; las siguientes, donde está el cursor, como un cambio desde esa línea."),
                metadata("Duración", "duration", "3:45", "Cuánto dura la canción, en minutos y segundos.", false),
                metadata("Etiqueta", "tag", "Fácil", "Una etiqueta para filtrar la biblioteca. Escribe una por línea.", true),
                meta("Idioma", "language", "es", "El idioma de la letra, con un código de dos letras (en, es, hu).", true),
                meta("Portada", "cover", "https://…", "La dirección de la imagen de portada del álbum.", false),
                meta("Enlace", "link", "https://… Nombre", "Un enlace a un video, una grabación o una tablatura, con un nombre opcional después de la dirección.", true),
            ],
        },
        ReferenceGroup {
            title: "Dentro de una línea",
            entries: vec![
                entry("Acorde", "[Am]", "Un acorde, entre corchetes y justo antes de la sílaba en la que cae.", Insertion::Wrap { prefix: "[", suffix: "]" }),
                entry("Anotación", "[*Suave]", "Una nota sobre la letra en lugar de un acorde, como una indicación.", Insertion::Wrap { prefix: "[*", suffix: "]" }),
            ],
        },
        ReferenceGroup {
            title: "Secciones",
            entries: vec![
                section("Intro", "intro", SECTION_DESCRIPTION),
                section("Estrofa", "verse", SECTION_DESCRIPTION),
                section("Pre-estribillo", "pre-chorus", SECTION_DESCRIPTION),
                section("Estribillo", "chorus", SECTION_DESCRIPTION),
                section("Puente", "bridge", SECTION_DESCRIPTION),
                section("Solo", "solo", SECTION_DESCRIPTION),
                section("Final", "outro", SECTION_DESCRIPTION),
                entry(
                    "Repetir el estribillo",
                    "{chorus}",
                    "Repite el último estribillo sin volver a escribirlo.",
                    Insertion::Directive { name: "chorus", has_value: false },
                ),
                section("Cuadrícula", "grid", "Una cuadrícula de acordes, con los compases separados por barras verticales."),
            ],
        },
        ReferenceGroup {
            title: "Tablatura",
            entries: vec![
                tab("Tablatura (cualquier instrumento)", None, &GUITAR_STAFF, "Un pentagrama vacío de una barra, con las cuerdas nombradas. Si hay líneas seleccionadas, las rodea."),
                tab("Tablatura de guitarra", Some("guitar"), &GUITAR_STAFF, "Solo la ve quien toca guitarra; con varias tablatura una detrás de otra, cada uno ve la suya."),
                tab("Tablatura de ukelele", Some("ukulele"), &UKULELE_STAFF, "Solo la ve quien toca ukelele."),
            ],
        },
        ReferenceGroup {
            title: "Formas de acordes",
            entries: vec![
                shape("Forma de acorde (guitarra)", 6),
                shape("Forma de acorde (ukelele)", 4),
            ],
        },
        ReferenceGroup {
            title: "Comentarios",
            entries: vec![
                directive("Comentario", "comment", "Repetir dos veces", "Una línea de texto que no se canta, como «Repetir dos veces»."),
                directive("Comentario en cursiva", "comment_italic", "Suave", "Un comentario que se muestra en cursiva."),
                directive("Comentario enmarcado", "comment_box", "Última vez", "Un comentario que se muestra dentro de un recuadro."),
            ],
        },
        ReferenceGroup {
            title: "Cambios a lo largo de la canción",
            entries: vec![directive(
                "Cambio de tono",
                "transpose",
                "2",
                "Desde esta línea, sube los acordes (o los baja, con un signo menos) la cantidad de semitonos escrita.",
            )],
        },
    ]
}

/// The strokes of a strumming pattern, written as annotations over the words they fall on, and the symbols of music and of a grid.
pub const STRUMS: [(&str, &str); 5] = [
    ("↓  Rasgueo hacia abajo", "[*↓]"),
    ("↑  Rasgueo hacia arriba", "[*↑]"),
    ("↓↑  Abajo y arriba", "[*↓↑]"),
    ("×  Rasgueo apagado", "[*×]"),
    (">  Acento", "[*>]"),
];

pub const SYMBOLS: [(&str, &str); 8] = [
    ("♯  Sostenido", "♯"),
    ("♭  Bemol", "♭"),
    ("♮  Becuadro", "♮"),
    ("|  Barra de compás", "|"),
    ("||  Barra doble", "||"),
    ("|:  Inicio de repetición", "|:"),
    (":|  Fin de repetición", ":|"),
    ("%  Repetir el compás anterior", "%"),
];

const TEXT_ID: &str = "editor_text";

/// The song being edited.
pub struct Editor {
    /// The file it is saved to, `None` for a song that has not been saved yet.
    pub file_name: Option<String>,
    pub text: String,
    /// What the file held when it was opened, to know whether anything changed.
    pub saved_text: String,
    pub show_reference: bool,
    reference: Vec<ReferenceGroup>,
    pub is_asking_to_discard: bool,
}

/// What the editor asks of the window around it.
#[derive(Debug, PartialEq, Eq)]
pub enum EditorAction {
    None,
    Save,
    Close,
    DiscardAndClose,
    /// Rename the file to what the title and the artist say.
    UpdateFileName,
}

impl Editor {
    pub const NEW_SONG: &'static str = "{title: }\n{artist: }\n{key: }\n\n";

    pub fn new(file_name: Option<String>, text: String) -> Editor {
        Editor { file_name, saved_text: text.clone(), text, show_reference: false, reference: reference_groups(), is_asking_to_discard: false }
    }

    pub fn is_dirty(&self) -> bool {
        self.text != self.saved_text
    }

    fn text_id() -> Id {
        Id::unique(TEXT_ID)
    }

    /// The selection of the text field, as character offsets.
    fn selection(&self, ctx: &egui::Context) -> (usize, usize) {
        let length = self.text.chars().count();
        egui::TextEdit::load_state(ctx, Self::text_id())
            .and_then(|state| state.cursor.char_range())
            .map(|range| (range.primary.index.0.min(length), range.secondary.index.0.min(length)))
            .unwrap_or((length, length))
    }

    fn put(&mut self, ctx: &egui::Context, edited: Edited) {
        self.text = edited.text;
        let mut state = egui::TextEdit::load_state(ctx, Self::text_id()).unwrap_or_default();
        state.cursor.set_char_range(Some(CCursorRange::two(CCursor::new(edited.selection_start), CCursor::new(edited.selection_end))));
        state.store(ctx, Self::text_id());
        ctx.memory_mut(|memory| memory.request_focus(Self::text_id()));
    }

    fn insert(&mut self, ctx: &egui::Context, insertion: Insertion) {
        let (start, end) = self.selection(ctx);
        let edited = insertion.apply(&self.text, start, end);
        self.put(ctx, edited);
    }

    /// Moves the chords of the text itself by `semitones`, which is how a song is written in another key.
    fn transpose(&mut self, ctx: &egui::Context, semitones: i32, prefer_flats: Option<bool>) {
        let selection = self.selection(ctx);
        self.text = transposer::transpose_text(&self.text, semitones, prefer_flats);
        let mut state = egui::TextEdit::load_state(ctx, Self::text_id()).unwrap_or_default();
        let length = self.text.chars().count();
        state.cursor.set_char_range(Some(CCursorRange::two(CCursor::new(selection.0.min(length)), CCursor::new(selection.1.min(length)))));
        state.store(ctx, Self::text_id());
    }

    /// The side panel with the reference: what every directive means, with a button that writes it.
    pub fn show_reference(&mut self, ui: &mut Ui) {
        let mut chosen: Option<Insertion> = None;
        ui.horizontal(|ui| {
            ui.heading("Referencia");
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.button("×").clicked() {
                    self.show_reference = false;
                }
            });
        });
        ui.separator();
        ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| {
            for group in &self.reference {
                ui.add_space(6.0);
                ui.label(RichText::new(group.title).strong().size(15.0));
                for entry in &group.entries {
                    ui.add_space(4.0);
                    ui.horizontal(|ui| {
                        ui.label(RichText::new(entry.title).strong());
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if ui.small_button("Insertar").on_hover_text(format!("Insertar {}", entry.title.to_lowercase())).clicked() {
                                chosen = Some(entry.insertion);
                            }
                        });
                    });
                    ui.label(RichText::new(entry.syntax).monospace().size(12.5));
                    ui.label(RichText::new(entry.description).small().weak());
                }
            }
            ui.add_space(10.0);
            ui.hyperlink_to("Todas las directivas, en chordpro.org", "https://www.chordpro.org/chordpro/chordpro-directives/");
        });
        if let Some(insertion) = chosen {
            self.insert(ui.ctx(), insertion);
        }
    }

    /// The toolbar and the text. Answers what the window should do next.
    pub fn show(&mut self, ui: &mut Ui, colors: &SongColors, text_size: f32) -> EditorAction {
        let mut action = EditorAction::None;
        let ctx = ui.ctx().clone();
        let sections = self.entries_of(&["Secciones"]);
        let tabs = self.entries_of(&["Tablatura"]);
        let shapes = self.entries_of(&["Formas de acordes"]);
        let comments = self.entries_of(&["Comentarios", "Cambios a lo largo de la canción"]);
        let data = self.entries_of(&["Datos de la canción"]);
        let mut pending: Option<Insertion> = None;
        let mut transposition: Option<(i32, Option<bool>)> = None;
        ui.horizontal_wrapped(|ui| {
            if ui.button("← Cerrar").clicked() {
                if self.is_dirty() {
                    self.is_asking_to_discard = true;
                } else {
                    action = EditorAction::Close;
                }
            }
            let save = ui.add_enabled(self.is_dirty() || self.file_name.is_none(), egui::Button::new("Guardar")).on_hover_text("Ctrl + S");
            if save.clicked() {
                action = EditorAction::Save;
            }
            if self.file_name.is_some() && ui.button("Actualizar nombre de archivo").on_hover_text("Lo renombra según el título y el artista").clicked() {
                action = EditorAction::UpdateFileName;
            }
            ui.separator();
            if ui.button("[ ] Acorde").on_hover_text("Rodea la selección con corchetes").clicked() {
                self.insert(&ctx, Insertion::Wrap { prefix: "[", suffix: "]" });
            }
            if ui.button("[*] Anotación").clicked() {
                self.insert(&ctx, Insertion::Wrap { prefix: "[*", suffix: "]" });
            }
            insertion_menu(ui, "Sección ▾", &sections, &mut pending);
            insertion_menu(ui, "Tablatura ▾", &tabs, &mut pending);
            insertion_menu(ui, "Acorde propio ▾", &shapes, &mut pending);
            insertion_menu(ui, "Comentario ▾", &comments, &mut pending);
            insertion_menu(ui, "Datos ▾", &data, &mut pending);
            let strums: Vec<(&'static str, Insertion)> = STRUMS.iter().map(|(label, symbol)| (*label, Insertion::Symbol(symbol))).collect();
            insertion_menu(ui, "Rasgueo ▾", &strums, &mut pending);
            let symbols: Vec<(&'static str, Insertion)> = SYMBOLS.iter().map(|(label, symbol)| (*label, Insertion::Symbol(symbol))).collect();
            insertion_menu(ui, "Símbolos ▾", &symbols, &mut pending);
            ui.menu_button("Transponer ▾", |ui| {
                for (label, semitones, flats) in [
                    ("Subir un semitono (♯)", 1, Some(false)),
                    ("Bajar un semitono (♭)", -1, Some(true)),
                    ("Subir un tono", 2, None),
                    ("Bajar un tono", -2, None),
                ] {
                    if ui.button(label).clicked() {
                        transposition = Some((semitones, flats));
                        ui.close();
                    }
                }
            });
            ui.separator();
            ui.toggle_value(&mut self.show_reference, "Referencia");
        });
        if let Some(insertion) = pending {
            self.insert(&ctx, insertion);
        }
        if let Some((semitones, flats)) = transposition {
            self.transpose(&ctx, semitones, flats);
        }
        if self.is_asking_to_discard {
            egui::Window::new("Cambios sin guardar").collapsible(false).resizable(false).anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0]).show(&ctx, |ui| {
                ui.label("Esta canción tiene cambios que no se guardaron.");
                ui.horizontal(|ui| {
                    if ui.button("Guardar y cerrar").clicked() {
                        self.is_asking_to_discard = false;
                        action = EditorAction::Save;
                    }
                    if ui.button("Descartar").clicked() {
                        self.is_asking_to_discard = false;
                        action = EditorAction::DiscardAndClose;
                    }
                    if ui.button("Seguir editando").clicked() {
                        self.is_asking_to_discard = false;
                    }
                });
            });
        }
        ui.separator();
        if ui.input_mut(|input| input.consume_key(egui::Modifiers::COMMAND, egui::Key::S)) {
            action = EditorAction::Save;
        }
        // What is pasted into the text is converted where it is a chord sheet (the chords over the words they fall on) and
        // written as it is everywhere else.
        if ctx.memory(|memory| memory.has_focus(Self::text_id())) {
            let pasted = ui.input_mut(|input| {
                let index = input.events.iter().position(|event| matches!(event, egui::Event::Paste(_)))?;
                match input.events.remove(index) {
                    egui::Event::Paste(text) => Some(text),
                    _ => None,
                }
            });
            if let Some(text) = pasted {
                let converted = chordpro::convert::prepare_pasted_fragment(&text);
                crate::diagnostics::record_paste(&text, &converted);
                let (start, end) = self.selection(&ctx);
                let edited = edit::insert_at_caret(&self.text, start, end, &converted);
                self.put(&ctx, edited);
            }
        }
        let font = FontId::new(text_size, crate::theme::editor_family());
        let colors_for_layout = (colors.text, colors.heading, colors.chord, colors.muted);
        let mut layouter = |ui: &Ui, text: &dyn egui::TextBuffer, wrap_width: f32| -> Arc<egui::Galley> {
            let job = highlighted(text.as_str(), &font, colors_for_layout, wrap_width);
            ui.fonts_mut(|fonts| fonts.layout_job(job))
        };
        ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| {
            ui.add(
                egui::TextEdit::multiline(&mut self.text)
                    .id(Self::text_id())
                    .layouter(&mut layouter)
                    .desired_width(f32::INFINITY)
                    .desired_rows(32)
                    .lock_focus(true),
            );
        });
        action
    }

    /// The entries of the reference groups called `titles`, as what a menu offers.
    fn entries_of(&self, titles: &[&str]) -> Vec<(&'static str, Insertion)> {
        self.reference.iter().filter(|group| titles.contains(&group.title)).flat_map(|group| group.entries.iter().map(|entry| (entry.title, entry.insertion))).collect()
    }

    /// Records that what is on the screen is what the file holds.
    pub fn mark_saved(&mut self, file_name: String) {
        self.file_name = Some(file_name);
        self.saved_text = self.text.clone();
    }
}

/// A drop-down menu of things the editor can write: the one picked is handed back through `pending`.
fn insertion_menu(ui: &mut Ui, title: &str, entries: &[(&'static str, Insertion)], pending: &mut Option<Insertion>) {
    ui.menu_button(title, |ui| {
        for (label, insertion) in entries {
            if ui.button(*label).clicked() {
                *pending = Some(*insertion);
                ui.close();
            }
        }
    });
}

/// The text with the parts of ChordPro told apart by color: the directives, the chords in brackets and the `#` comments.
fn highlighted(text: &str, font: &FontId, colors: (Color32, Color32, Color32, Color32), wrap_width: f32) -> LayoutJob {
    let (plain, directive, chord, comment) = colors;
    let mut job = LayoutJob::default();
    job.wrap.max_width = wrap_width;
    let format = |color: Color32| TextFormat { font_id: font.clone(), color, ..Default::default() };
    let mut lines = text.split('\n').peekable();
    while let Some(line) = lines.next() {
        let trimmed = line.trim_start();
        if trimmed.starts_with('#') {
            job.append(line, 0.0, format(comment));
        } else if trimmed.starts_with('{') {
            job.append(line, 0.0, format(directive));
        } else {
            let mut rest = line;
            while let Some(open) = rest.find('[') {
                let Some(close) = rest[open..].find(']') else {
                    break;
                };
                job.append(&rest[..open], 0.0, format(plain));
                job.append(&rest[open..open + close + 1], 0.0, format(chord));
                rest = &rest[open + close + 1..];
            }
            job.append(rest, 0.0, format(plain));
        }
        if lines.peek().is_some() {
            job.append("\n", 0.0, format(plain));
        }
    }
    job
}

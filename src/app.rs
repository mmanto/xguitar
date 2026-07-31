use crate::musicxml::parse_musicxml;
use std::collections::HashSet;
use crate::{
    Accidental, BASE_SCALE, Barline, Clef, I18n, KeySignature, Lang, Measure, MeasureElement, Note,
    NoteFigure, PAGE_GAP, Pitch, RenderStyle, STAFF_LINE_SPACING, Score, ScoreStylesheet, Staff,
    StaffKind, StemDirection, Step, System, TimeSignature, TimeSignatureStyle, compute_pages,
    configure_fonts, render_pages,
};
use eframe::egui;
use egui::ViewportCommand;

const DEFAULT_OCTAVE: i8 = 4;

/// Per-tab document state.
pub struct Document {
    pub(crate) score: Score,
    pub(crate) zoom: f32,
    /// Display name for the tab (file name or "Sin título").
    pub(crate) label: String,
    /// Source file path for session persistence.
    pub(crate) file_path: Option<String>,
    pub(crate) pending_step: Option<Step>,
    pub(crate) pending_figure_digits: String,
    pub(crate) dirty: bool,
}

impl Document {
    fn empty() -> Self {
        Self {
            score: Score {
                title: String::new(),
                composer: String::new(),
                systems: vec![System {
                    staves: vec![Staff {
                        clef: Clef::Treble,
                        line: Clef::Treble.default_line(),
                        measures: vec![Measure {
                            number: "1".into(),
                            time_signature: TimeSignature {
                                numerator: 4,
                                denominator: 4,
                                style: TimeSignatureStyle::Numeric,
                            },
                            key_signature: KeySignature::default(),
                            elements: vec![],
                            barline: Barline::default(),
                            ending: None,
                            directions: vec![],
                            divisions: 1,
                            system_break: false,
                        }],
                        name: String::new(),
                        abbreviation: String::new(),
                        kind: StaffKind::Standard,
                    }],
                    left_margin: 0.0,
                    bracket: None,
                }],
                credits: vec![],
                scaling: None,
                part_list: crate::PartList::default(),
            },
            file_path: None,
            zoom: 1.0,
            label: "Sin título".into(),
            pending_step: None,
            pending_figure_digits: String::new(),
            dirty: false,
        }
    }

    fn from_score(score: Score, label: String, file_path: Option<String>) -> Self {
        Self {
            score,
            zoom: 1.0,
            label,
            file_path,
            pending_step: None,
            pending_figure_digits: String::new(),
            dirty: false,
        }
    }
}

pub struct MGuitarApp {
    pub window_open: bool,
    pub dark_mode: bool,
    pub i18n: I18n,
    pub first_frame: bool,
    pub documents: Vec<Document>,
    pub active_doc: usize,
    pub stylesheets: Vec<ScoreStylesheet>,
    pub selected_sheet: usize,
    pub status_message: Option<String>,
    pub pending_open: bool,
    pub pending_xml: std::rc::Rc<std::cell::RefCell<Option<(String, String)>>>,
    session_path: std::path::PathBuf,
    /// Reproducción de partituras (sfizz + cpal) — solo nativo, ver ADR-008.
    #[cfg(not(target_arch = "wasm32"))]
    pub audio: crate::audio::player::AudioService,
}

impl MGuitarApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        configure_fonts(&cc.egui_ctx);
        cc.egui_ctx.set_visuals(egui::Visuals::dark());
        // Increase UI font sizes (1.5x default)
        let mut style = (*cc.egui_ctx.style_of(egui::Theme::Dark)).clone();
        for font_id in style.text_styles.values_mut() {
            font_id.size *= 1.5;
        }
        cc.egui_ctx.set_style_of(egui::Theme::Dark, style);

        #[cfg(not(target_arch = "wasm32"))]
        let stylesheets = {
            let dir = dirs::config_dir()
                .unwrap_or_else(|| std::path::PathBuf::from("."))
                .join("m-guitar")
                .join("stylesheets");
            std::fs::create_dir_all(&dir).ok();
            if let Ok(entries) = std::fs::read_dir("assets/stylesheets") {
                for entry in entries.flatten() {
                    let dest = dir.join(entry.file_name());
                    std::fs::copy(entry.path(), &dest).ok();
                }
            }
            ScoreStylesheet::load_all(&dir)
        };
        #[cfg(target_arch = "wasm32")]
        let stylesheets = { vec![ScoreStylesheet::builtin_default()] };

        let pending_xml: std::rc::Rc<std::cell::RefCell<Option<(String, String)>>> = Default::default();

        let session_path = dirs::config_dir()
            .unwrap_or_else(|| std::path::PathBuf::from("."))
            .join("m-guitar")
            .join("session.json");

        #[cfg(not(target_arch = "wasm32"))]
        let audio = {
            let mut audio = crate::audio::player::AudioService::new();
            let default_instrument = dirs::config_dir()
                .unwrap_or_else(|| std::path::PathBuf::from("."))
                .join("m-guitar")
                .join("instruments")
                .join("Strumstick.sfz");
            audio.set_instrument_path(Some(default_instrument));
            audio
        };

        let mut slf = Self {
            window_open: true,
            pending_xml: pending_xml.clone(),
            dark_mode: true,
            i18n: I18n::new(Lang::Es),
            first_frame: true,
            documents: vec![],
            active_doc: 0,
            stylesheets,
            selected_sheet: 0,
            status_message: None,
            pending_open: false,
            session_path,
            #[cfg(not(target_arch = "wasm32"))]
            audio,
        };

        // Restore session: load files from previous run
        slf.restore_session();
        #[cfg(not(target_arch = "wasm32"))]
        slf.audio.ensure_stream();

        slf

    }

    fn save_session(&self) {
        let paths: Vec<&str> = self
            .documents
            .iter()
            .filter_map(|d| d.file_path.as_deref())
            .collect();
        if let Ok(json) = serde_json::to_string(&paths) {
            let _ = std::fs::write(&self.session_path, json);
        }
    }

    fn restore_session(&mut self) {
        let paths: Vec<String> = match std::fs::read_to_string(&self.session_path) {
            Ok(content) => serde_json::from_str(&content).unwrap_or_default(),
            Err(_) => return,
        };
        for path in &paths {
            match std::fs::read_to_string(path) {
                Ok(xml) => {
                    if let Ok(score) = parse_musicxml(&xml) {
                        let label = std::path::Path::new(path)
                            .file_stem()
                            .and_then(|s| s.to_str())
                            .unwrap_or("Sin título")
                            .to_string();
                        self.documents.push(Document::from_score(
                            score,
                            label,
                            Some(path.clone()),
                        ));
                    }
                }
                Err(_) => {
                    // File gone — keep the tab entry with an error status?
                    // For now, just skip missing files silently.
                }
            }
        }
        if !self.documents.is_empty() {
            self.active_doc = 0;
        }
        // Sync: if session had files but none loaded, remove stale session
        if self.documents.is_empty() && !paths.is_empty() {
            let _ = std::fs::remove_file(&self.session_path);
        }
    }
}

impl MGuitarApp {

    fn doc(&self) -> &Document {
        &self.documents[self.active_doc]
    }

    fn doc_mut(&mut self) -> &mut Document {
        &mut self.documents[self.active_doc]
    }

    fn insert_note(&mut self) {
        let figure = {
            let digits = &self.doc().pending_figure_digits;
            match digits.as_str() {
                "1" => Some(NoteFigure::Whole),
                "2" => Some(NoteFigure::Half),
                "4" => Some(NoteFigure::Quarter),
                "6" => Some(NoteFigure::Sixteenth),
                "8" => Some(NoteFigure::Eighth),
                "32" => Some(NoteFigure::ThirtySecond),
                "33" => Some(NoteFigure::SixtyFourth),
                _ => None,
            }
        };
        let figure = match figure {
            Some(f) => f,
            None => {
                self.doc_mut().pending_figure_digits.clear();
                return;
            }
        };

        let doc = self.doc_mut();
        let step = match doc.pending_step.take() {
            Some(s) => s,
            None => return,
        };
        let pitch = Pitch {
            step,
            accidental: Accidental::Natural,
            octave: DEFAULT_OCTAVE,
        };
        let note = Note {
            pitch,
            figure,
            dotted: 0,
            time_modification: None,
            accidental_override: None,
            stem_direction: StemDirection::Up,
            grace: false,
            chord: false,
            attachments: None,
            lyrics: vec![],
        };
        if doc.score.systems.is_empty() {
            doc.score.systems.push(System {
                staves: vec![Staff {
                    clef: Clef::Treble,
                    line: Clef::Treble.default_line(),
                    measures: vec![],
                    name: String::new(),
                    abbreviation: String::new(),
                    kind: StaffKind::Standard,
                }],
                left_margin: 0.0,
                bracket: None,
            });
        }
        let staff = &mut doc.score.systems[0].staves[0];
        if staff.measures.is_empty() {
            staff.measures.push(Measure {
                number: "1".into(),
                time_signature: TimeSignature {
                    numerator: 4,
                    denominator: 4,
                    style: TimeSignatureStyle::Numeric,
                },
                key_signature: KeySignature::default(),
                elements: vec![],
                barline: Barline::default(),
                ending: None,
                directions: vec![],
                divisions: 1,
                system_break: false,
            });
        }
        let last = staff.measures.last_mut().unwrap();
        last.elements.push(MeasureElement::Note(note));
        let msg = format!("{} {} agregada", pitch.name_es(), figure.name_es());
        doc.pending_figure_digits.clear();
        doc.dirty = true;
        self.status_message = Some(msg);
    }
}

fn handle_zoom(ui: &egui::Ui, zoom: &mut f32) {
    ui.ctx().input(|i| {
        if i.modifiers.ctrl {
            if i.key_pressed(egui::Key::Plus) || i.key_pressed(egui::Key::Equals) {
                *zoom = (*zoom + 0.15).min(4.0);
            }
            if i.key_pressed(egui::Key::Minus) {
                *zoom = (*zoom - 0.15).max(0.25);
            }
            if i.key_pressed(egui::Key::Num0) {
                *zoom = 1.0;
            }
        }
    });
}

fn handle_note_input(app: &mut MGuitarApp, ui: &egui::Ui) {
    let active = app.active_doc;
    ui.ctx().input(|i| {
        for event in &i.events {
            let key = match event {
                egui::Event::Key {
                    key, pressed: true, ..
                } => *key,
                _ => continue,
            };
            let step = match key {
                egui::Key::C => Some(Step::C),
                egui::Key::D => Some(Step::D),
                egui::Key::E => Some(Step::E),
                egui::Key::F => Some(Step::F),
                egui::Key::G => Some(Step::G),
                egui::Key::A => Some(Step::A),
                egui::Key::B => Some(Step::B),
                _ => None,
            };
            if let Some(s) = step {
                app.documents[active].pending_step = Some(s);
                app.documents[active].pending_figure_digits.clear();
            }
        }
        if app.documents[active].pending_step.is_some() {
            for key in &i.keys_down {
                let ch = match key {
                    egui::Key::Num1 => '1',
                    egui::Key::Num2 => '2',
                    egui::Key::Num3 => '3',
                    egui::Key::Num4 => '4',
                    egui::Key::Num5 => '5',
                    egui::Key::Num6 => '6',
                    egui::Key::Num7 => '7',
                    egui::Key::Num8 => '8',
                    egui::Key::Num9 => '9',
                    egui::Key::Num0 => '0',
                    _ => continue,
                };
                app.documents[active].pending_figure_digits.push(ch);
                if app.documents[active].pending_figure_digits.len() >= 2 {
                    app.insert_note();
                    return;
                }
            }
        }
    });
    if app.documents[active].dirty {
        ui.ctx().request_repaint();
        app.documents[active].dirty = false;
    }
}

impl eframe::App for MGuitarApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        if self.first_frame {
            self.first_frame = false;
            ui.ctx().send_viewport_cmd(ViewportCommand::Maximized(true));
        }
        ui.ctx().set_zoom_factor(1.0);
        let font_h = ui.style().text_styles[&egui::TextStyle::Body].size;
        let margin = font_h * 0.5;

        // ── Top panel: menu + toolbar + tabs ──
        egui::Panel::top("top_bar")
            .resizable(false)
            .frame(
                egui::Frame::default()
                    .fill(egui::Color32::from_rgb(0x44, 0x47, 0x51))
                    .stroke(egui::Stroke::NONE)
                    .inner_margin(egui::Margin::same(margin as i8)),
            )
            .show(ui, |ui| {
                ui.vertical(|ui| {
                    // ── Row 1: Menu ──
                    // ── Menu row ──
                    ui.horizontal(|ui| {
                        egui::MenuBar::new().ui(ui, |ui| {
                            ui.menu_button(format!("📄  {}", self.i18n.t("file")), |ui| {
                                if ui.button(format!("✨  {}", self.i18n.t("new"))).clicked() {
                                    self.documents.push(Document::empty());
                                    self.active_doc = self.documents.len() - 1;
                                    self.status_message =
                                        Some(self.i18n.t("new_score").to_string());
                                    ui.close();
                                }
                                if ui.button(format!("📂  {}", self.i18n.t("open"))).clicked() {
                                    self.pending_open = true;
                                    ui.close();
                                }
                                ui.separator();
                                if ui.button(format!("✕  {}", self.i18n.t("close"))).clicked() {
                                    ui.close();
                                    if !self.documents.is_empty() {
                                        self.documents.remove(self.active_doc);
                                        if self.active_doc >= self.documents.len() {
                                            self.active_doc = self.active_doc.saturating_sub(1);
                                        }
                                        self.save_session();
                                    }
                                }
                                if ui.button(format!("🚪  {}", self.i18n.t("exit"))).clicked() {
                                    ui.close();
                                    ui.ctx().send_viewport_cmd(ViewportCommand::Close);
                                }
                            });
                        });
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if ui
                                .selectable_label(
                                    matches!(self.i18n.lang, Lang::En),
                                    self.i18n.t("lang_toggle"),
                                )
                                .clicked()
                            {
                                self.i18n.lang = match self.i18n.lang {
                                    Lang::Es => Lang::En,
                                    Lang::En => Lang::Es,
                                };
                            }
                            if ui.selectable_label(self.dark_mode, "🌙").clicked() {
                                self.dark_mode = !self.dark_mode;
                                ui.ctx().set_visuals(if self.dark_mode {
                                    egui::Visuals::dark()
                                } else {
                                    egui::Visuals::light()
                                });
                            }
                            if !self.documents.is_empty() {
                                let mut zoom_pct =
                                    (self.documents[self.active_doc].zoom * 100.0) as i32;
                                if ui
                                    .add(
                                        egui::DragValue::new(&mut zoom_pct)
                                            .range(25..=400)
                                            .suffix("%")
                                            .speed(5),
                                    )
                                    .changed()
                                {
                                    self.documents[self.active_doc].zoom =
                                        (zoom_pct as f32 / 100.0).clamp(0.25, 4.0);
                                }
                            }
                            if !self.stylesheets.is_empty() {
                                egui::ComboBox::from_id_salt("stylesheet")
                                    .selected_text(&self.stylesheets[self.selected_sheet].name)
                                    .show_ui(ui, |ui| {
                                        for (i, s) in self.stylesheets.iter().enumerate() {
                                            ui.selectable_value(
                                                &mut self.selected_sheet,
                                                i,
                                                &s.name,
                                            );
                                        }
                                    });
                            }
                        });
                    });

                    // ── Row 2: Toolbar ──
                    ui.columns(2, |cols| {
                        cols[1].horizontal(|ui| {
                            #[cfg(not(target_arch = "wasm32"))]
                            {
                                if let Some(err) = self.audio.take_last_error() {
                                    self.status_message = Some(err);
                                }
                                let has_doc = !self.documents.is_empty();
                                let playing = self.audio.is_playing();
                                let paused = self.audio.is_paused();

                                let seek_btn = ui.add_enabled(has_doc, egui::Button::new(self.i18n.t("seek_start")));
                                if seek_btn.clicked() {
                                    self.audio.seek_start();
                                }

                                if playing {
                                    if ui.button(self.i18n.t("pause")).clicked() {
                                        self.audio.pause();
                                    }
                                    ui.ctx().request_repaint();
                                } else {
                                    let play_btn = ui.add_enabled(has_doc, egui::Button::new(self.i18n.t("play")));
                                    if play_btn.clicked() {
                                        if paused {
                                            self.audio.resume();
                                        } else {
                                            self.audio.play(&self.documents[self.active_doc].score);
                                        }
                                    }
                                    if paused {
                                        ui.ctx().request_repaint();
                                    }
                                }

                                let stop_enabled = has_doc && (playing || paused);
                                let stop_btn = ui.add_enabled(stop_enabled, egui::Button::new(self.i18n.t("stop")));
                                if stop_btn.clicked() {
                                    self.audio.stop();
                                }
                            }
                            #[cfg(target_arch = "wasm32")]
                            {
                                ui.add_enabled(false, egui::Button::new(self.i18n.t("play")))
                                    .on_disabled_hover_text(self.i18n.t("play_wasm_unavailable"));
                            }
                        });
                    });
                    ui.add_space(2.0);
                });
            });

        // ── File dialog ──
        if self.pending_open {
            self.pending_open = false;
            #[cfg(target_arch = "wasm32")]
            {
                let pending = self.pending_xml.clone();
                let ctx2 = ui.ctx().clone();
                wasm_bindgen_futures::spawn_local(async move {
                    if let Some(file) = rfd::AsyncFileDialog::new()
                        .add_filter("MusicXML", &["xml", "musicxml", "mxl"])
                        .pick_file()
                        .await
                    {
                        let name = file.file_name();
                        if let Ok(xml) = String::from_utf8(file.read().await) {
                            *pending.borrow_mut() = Some((xml, name));
                            ctx2.request_repaint();
                        }
                    }
                });
            }
            #[cfg(not(target_arch = "wasm32"))]
            {
                if let Some(path) = rfd::FileDialog::new()
                    .add_filter("MusicXML", &["xml", "musicxml", "mxl"])
                    .pick_file()
                {
                    match std::fs::read_to_string(&path) {
                        Ok(xml) => {
                            let path_str = path.to_string_lossy().to_string();
                            *self.pending_xml.borrow_mut() = Some((xml, path_str));
                        }
                        Err(e) => {
                            self.status_message = Some(
                                self.i18n.t("open_error").replace("{}", &e.to_string()),
                            );
                        }
                    }
                }
            }
        }

        if let Some((xml, path)) = self.pending_xml.borrow_mut().take() {
            match parse_musicxml(&xml) {
                Ok(score) => {
                    let title = if score.title.is_empty() {
                        "Sin título".to_string()
                    } else {
                        score.title.clone()
                    };
                    self.documents.push(Document::from_score(
                        score,
                        title,
                        Some(path),
                    ));
                    self.active_doc = self.documents.len() - 1;
                    self.status_message =
                        Some(self.i18n.t("open_success").replace("{}", ""));
                    self.save_session();
                }
                Err(e) => {
                    self.status_message = Some(
                        self.i18n.t("open_error").replace("{}", &e.to_string()),
                    );
                }
            }
        }
        if self.documents.is_empty() {
            egui::CentralPanel::default()
                .frame(
                    egui::Frame::default()
                        .fill(egui::Color32::from_rgb(0x44, 0x47, 0x51))
                        .stroke(egui::Stroke::NONE),
                )
                .show(ui, |ui| {
                    ui.centered_and_justified(|ui| {
                        ui.label(self.i18n.t("input_hint"));
                    });
                });
        } else {
        let zoom = self.documents[self.active_doc].zoom;
        let active = self.active_doc;

        egui::CentralPanel::default()
            .frame(
                egui::Frame::default()
                    .fill(egui::Color32::from_rgb(0x44, 0x47, 0x51))
                    .stroke(egui::Stroke::NONE),
            )
            .show(ui, |ui| {
                let sheet = &self.stylesheets[self.selected_sheet];
                // `zoom` es el 100% "lógico" que el usuario controla; BASE_SCALE
                // calibra ese 100% para que la partitura se vea como una edición
                // bien grabada (ver comentario en constants.rs / ADR-007), y se
                // aplica acá — el único punto donde se lee el zoom para renderizar —
                // para cubrir por igual la geometría de notación y el encabezado.
                let render_zoom = zoom * BASE_SCALE;
                let line_spacing =
                    STAFF_LINE_SPACING * render_zoom * sheet.notation.staff_scale;
                #[cfg(not(target_arch = "wasm32"))]
                let active_notes = if self.audio.is_playing() || self.audio.is_paused() {
                    self.audio.active_note_refs()
                } else {
                    HashSet::new()
                };
                #[cfg(target_arch = "wasm32")]
                let active_notes = HashSet::new();

                let style = RenderStyle {
                    line_spacing,
                    dark_mode: false,
                    sheet: sheet.clone(),
                    active_notes,
                };
                let header_height = (sheet.header.title_top_offset
                    + sheet.header.title_size
                    + sheet.header.row_gap
                    + sheet.header.composer_size
                    + sheet.header.row_gap
                    + sheet.header.tempo_size
                    + sheet.header.header_staff_gap)
                    * render_zoom;
                let layout =
                    compute_pages(&self.documents[active].score, render_zoom, header_height);

                // Tabs bar — alineada con el borde izquierdo de la partitura
                if self.documents.len() > 1 {
                    let available = ui.available_width();
                    let pages_per_row =
                        if available >= layout.page_width * 2.0 + PAGE_GAP * render_zoom {
                            2
                        } else {
                            1
                        };
                    let row_width = pages_per_row as f32 * layout.page_width
                        + (pages_per_row as f32 - 1.0) * PAGE_GAP * render_zoom;
                    let left_pad = ((available - row_width) / 2.0).max(20.0);

                    ui.horizontal(|ui| {
                        ui.add_space(left_pad);
                        let active_color = egui::Color32::from_rgb(0x44, 0x99, 0xFF);
                        for (i, doc) in self.documents.iter().enumerate() {
                            if i > 0 {
                                ui.label(
                                    egui::RichText::new("|")
                                        .color(egui::Color32::from_rgb(0x66, 0x66, 0x66))
                                );
                            }
                            let label = if doc.dirty {
                                format!("● {}", doc.label)
                            } else {
                                doc.label.clone()
                            };
                            let response = ui
                                .add(
                                    egui::Label::new(egui::RichText::new(label))
                                        .sense(egui::Sense::click()),
                                )
                                .on_hover_cursor(egui::CursorIcon::Default);
                            if i == self.active_doc {
                                let rect = response.rect;
                                let underline = egui::Rect::from_min_max(
                                    egui::Pos2::new(rect.left(), rect.bottom()),
                                    egui::Pos2::new(rect.right(), rect.bottom() + 1.0),
                                );
                                ui.painter().rect_filled(underline, 0.0, active_color);
                            }
                            if response.clicked() {
                                self.active_doc = i;
                            }
                        }
                    });
                    ui.add_space(4.0);
                }

                egui::ScrollArea::vertical()
                    .auto_shrink([false; 2])
                    .show(ui, |ui| {
                        let available = ui.available_width();
                        let pages_per_row =
                            if available >= layout.page_width * 2.0 + PAGE_GAP * render_zoom {
                                2
                            } else {
                                1
                            };
                        let row_width = pages_per_row as f32 * layout.page_width
                            + (pages_per_row as f32 - 1.0) * PAGE_GAP * render_zoom;
                        let left_pad = ((available - row_width) / 2.0).max(20.0);
                        let total_h = layout.total_height + 40.0;
                        let size = egui::Vec2::new(available, total_h);
                        let (response, painter) =
                            ui.allocate_painter(size, egui::Sense::hover());
                        let top_left =
                            response.rect.min + egui::Vec2::new(left_pad, 20.0);
                        render_pages(
                            &painter,
                            top_left,
                            &layout,
                            &self.documents[active].score,
                            &style,
                            pages_per_row,
                        );
                        self.documents[active].zoom = zoom;
                        handle_zoom(ui, &mut self.documents[active].zoom);
                        handle_note_input(self, ui);
                    });
            });
        }
        // ── Status bar ──
        let (pending_step, pending_digits) = if !self.documents.is_empty() {
            let doc = &self.documents[self.active_doc];
            (doc.pending_step, doc.pending_figure_digits.clone())
        } else {
            (None, String::new())
        };
        egui::Panel::bottom("status_bar")
            .exact_size(font_h * 1.8)
            .frame(
                egui::Frame::default()
                    .fill(ui.style().visuals.panel_fill)
                    .inner_margin(egui::Margin::same(margin as i8)),
            )
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    if let Some(msg) = &self.status_message {
                        ui.label(msg);
                    } else if let Some(step) = pending_step {
                        let pitch = Pitch {
                            step,
                            accidental: Accidental::Natural,
                            octave: 4,
                        };
                        if pending_digits.is_empty() {
                            ui.label(format!(
                                "Nota: {} — figura (1,2,4,8,6,32,33) + Enter",
                                pitch.name_es()
                            ));
                        } else {
                            ui.label(format!(
                                "Nota: {} — figura: {}… (Enter para confirmar)",
                                pitch.name_es(),
                                pending_digits
                            ));
                        }
                    } else {
                        ui.label(self.i18n.t("input_hint"));
                    }
                });
            });
    }
}

use crate::musicxml::parse_musicxml;
use crate::{
    Accidental, Barline, Clef, I18n, KeySignature, Lang, Measure, MeasureElement, Note, NoteFigure,
    PAGE_GAP, Pitch, RenderStyle, STAFF_LINE_SPACING, Score, ScoreStylesheet, Staff, StaffKind,
    StemDirection, Step, System, TimeSignature, TimeSignatureStyle, compute_pages, configure_fonts,
    render_pages,
};
use eframe::egui;
use egui::ViewportCommand;

const DEFAULT_OCTAVE: i8 = 4;

pub struct MGuitarApp {
    pub window_open: bool,
    pub dark_mode: bool,
    pub i18n: I18n,
    pub first_frame: bool,
    pub score: Score,
    pub zoom: f32,
    pub stylesheets: Vec<ScoreStylesheet>,
    pub selected_sheet: usize,
    pub pending_step: Option<Step>,
    pub pending_figure_digits: String,
    pub status_message: Option<String>,
    pub pending_open: bool,
    pub pending_xml: std::rc::Rc<std::cell::RefCell<Option<String>>>,
    pub dirty: bool,
}

impl MGuitarApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        configure_fonts(&cc.egui_ctx);
        cc.egui_ctx.set_visuals(egui::Visuals::light());

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

        let pending_xml: std::rc::Rc<std::cell::RefCell<Option<String>>> = Default::default();

        Self {
            window_open: true,
            pending_xml: pending_xml.clone(),
            dark_mode: false,
            i18n: I18n::new(Lang::Es),
            first_frame: true,
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
            zoom: 1.0,
            stylesheets,
            selected_sheet: 0,
            pending_step: None,
            pending_figure_digits: String::new(),
            status_message: None,
            pending_open: false,
            dirty: false,
        }
    }

    fn resolve_figure(&self) -> Option<NoteFigure> {
        match self.pending_figure_digits.as_str() {
            "1" => Some(NoteFigure::Whole),
            "2" => Some(NoteFigure::Half),
            "4" => Some(NoteFigure::Quarter),
            "6" => Some(NoteFigure::Sixteenth),
            "8" => Some(NoteFigure::Eighth),
            "32" => Some(NoteFigure::ThirtySecond),
            "33" => Some(NoteFigure::SixtyFourth),
            _ => None,
        }
    }

    fn insert_note(&mut self) {
        let step = match self.pending_step.take() {
            Some(s) => s,
            None => return,
        };
        let figure = match self.resolve_figure() {
            Some(f) => f,
            None => {
                self.pending_figure_digits.clear();
                return;
            }
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
            accidental_override: None,
            stem_direction: StemDirection::Up,
            grace: false,
            chord: false,
            attachments: None,
            lyrics: vec![],
        };
        if self.score.systems.is_empty() {
            self.score.systems.push(System {
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
        let staff = &mut self.score.systems[0].staves[0];
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
            });
        }
        let last = staff.measures.last_mut().unwrap();
        last.elements.push(MeasureElement::Note(note));
        self.status_message = Some(format!("{} {} agregada", pitch.name_es(), figure.name_es()));
        self.pending_figure_digits.clear();
        self.dirty = true;
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
                app.pending_step = Some(s);
                app.pending_figure_digits.clear();
            }
        }
        if app.pending_step.is_some() {
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
                app.pending_figure_digits.push(ch);
                if app.pending_figure_digits.len() >= 2 {
                    app.insert_note();
                }
            }
        }
    });
    if app.dirty {
        ui.ctx().request_repaint();
        app.dirty = false;
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
        let screen_h = ui.ctx().viewport_rect().height();

        egui::Panel::top("top_bar")
            .exact_size(screen_h * 0.05)
            .frame(
                egui::Frame::default()
                    .fill(ui.style().visuals.panel_fill)
                    .stroke(egui::Stroke::new(
                        1.0,
                        ui.style().visuals.widgets.noninteractive.fg_stroke.color,
                    ))
                    .inner_margin(egui::Margin::same(margin as i8)),
            )
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    egui::MenuBar::new().ui(ui, |ui| {
                        ui.menu_button(format!("📄  {}", self.i18n.t("file")), |ui| {
                            if ui.button(format!("✨  {}", self.i18n.t("new"))).clicked() {
                                self.score = Score {
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
                                };
                                self.pending_step = None;
                                self.pending_figure_digits.clear();
                                self.status_message = Some(self.i18n.t("new_score").to_string());
                                ui.close();
                            }
                            if ui.button(format!("📂  {}", self.i18n.t("open"))).clicked() {
                                self.pending_open = true;
                                ui.close();
                            }
                            ui.separator();
                            if ui.button(format!("✕  {}", self.i18n.t("close"))).clicked() {
                                ui.close();
                                self.window_open = false;
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
                        if !self.stylesheets.is_empty() {
                            egui::ComboBox::from_id_salt("stylesheet")
                                .selected_text(&self.stylesheets[self.selected_sheet].name)
                                .show_ui(ui, |ui| {
                                    for (i, s) in self.stylesheets.iter().enumerate() {
                                        ui.selectable_value(&mut self.selected_sheet, i, &s.name);
                                    }
                                });
                        }
                    });
                });
            });

        // File dialog
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
                        if let Ok(xml) = String::from_utf8(file.read().await) {
                            *pending.borrow_mut() = Some(xml);
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
                            *self.pending_xml.borrow_mut() = Some(xml);
                        }
                        Err(e) => {
                            self.status_message =
                                Some(self.i18n.t("open_error").replace("{}", &e.to_string()));
                        }
                    }
                }
            }
        }

        if let Some(xml) = self.pending_xml.borrow_mut().take() {
            match parse_musicxml(&xml) {
                Ok(score) => {
                    self.score = score;
                    self.status_message = Some(self.i18n.t("open_success").replace("{}", ""));
                }
                Err(e) => {
                    self.status_message =
                        Some(self.i18n.t("open_error").replace("{}", &e.to_string()));
                }
            }
        }

        egui::CentralPanel::default().show(ui, |ui| {
            let sheet = &self.stylesheets[self.selected_sheet];
            let line_spacing = STAFF_LINE_SPACING * self.zoom * sheet.notation.staff_scale;
            let style = RenderStyle {
                line_spacing,
                dark_mode: self.dark_mode,
                sheet: sheet.clone(),
            };
            let header_height = (sheet.header.title_top_offset
                + sheet.header.title_size
                + sheet.header.row_gap
                + sheet.header.composer_size
                + sheet.header.row_gap
                + sheet.header.tempo_size
                + sheet.header.header_staff_gap)
                * self.zoom;
            let layout = compute_pages(&self.score, self.zoom, 4, header_height);
            egui::ScrollArea::vertical()
                .auto_shrink([false; 2])
                .show(ui, |ui| {
                    let available = ui.available_width();
                    let pages_per_row =
                        if available >= layout.page_width * 2.0 + PAGE_GAP * self.zoom {
                            2
                        } else {
                            1
                        };
                    let row_width = pages_per_row as f32 * layout.page_width
                        + (pages_per_row as f32 - 1.0) * PAGE_GAP * self.zoom;
                    let left_pad = ((available - row_width) / 2.0).max(20.0);
                    let total_h = layout.total_height + 40.0;
                    let size = egui::Vec2::new(available, total_h);
                    let (response, painter) = ui.allocate_painter(size, egui::Sense::hover());
                    let top_left = response.rect.min + egui::Vec2::new(left_pad, 20.0);
                    render_pages(
                        &painter,
                        top_left,
                        &layout,
                        &self.score,
                        &style,
                        pages_per_row,
                    );
                    handle_zoom(ui, &mut self.zoom);
                    handle_note_input(self, ui);
                });
        });

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
                    } else if let Some(step) = self.pending_step {
                        let pitch = Pitch {
                            step,
                            accidental: Accidental::Natural,
                            octave: 4,
                        };
                        if self.pending_figure_digits.is_empty() {
                            ui.label(format!(
                                "Nota: {} — figura (1,2,4,8,6,32,33) + Enter",
                                pitch.name_es()
                            ));
                        } else {
                            ui.label(format!(
                                "Nota: {} — figura: {}… (Enter para confirmar)",
                                pitch.name_es(),
                                self.pending_figure_digits
                            ));
                        }
                    } else {
                        ui.label(self.i18n.t("input_hint"));
                    }
                });
            });
    }
}

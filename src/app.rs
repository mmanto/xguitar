use crate::musicxml::parse_musicxml;
use crate::{
    Accidental, BASE_SCALE, Barline, ChordProgression, ChordStep, ChordSymbol, Clef, DirectionKind,
    I18n, KeySignature, Lang, Measure, MeasureElement, Note, NoteFigure, PAGE_GAP, Pitch,
    RenderStyle, STAFF_LINE_SPACING, Score, ScoreStylesheet, Section, SectionKind, Staff,
    StaffKind, StemDirection, Step, System, Theme, TimeSignature, TimeSignatureStyle,
    compute_pages, configure_fonts, render_pages,
};
use eframe::egui;
use egui::ViewportCommand;
use std::collections::HashSet;

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
    /// Temas musicales asociados a esta partitura (metadatos).
    pub(crate) themes: Vec<crate::Theme>,
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
                            chord_symbol: None,
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
            themes: Vec::new(),
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
            themes: Vec::new(),
            dirty: false,
        }
    }
}

/// Vista activa: partitura (default) o mapa de temas.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ViewMode {
    Score,
    ThemeMap,
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
    /// Cierre de solapa diferido al próximo frame.
    pending_close: bool,
    /// Vista activa: partitura (default) o mapa de temas.
    pub(crate) view_mode: ViewMode,
    /// Si está presente, scrollea la partitura a este compás al renderizar.
    pub(crate) scroll_to_measure: Option<usize>,
    /// Fingerboard (diapasón) abierto.
    pub fingerboard_open: bool,
    /// Configuración del diapasón.
    pub fingerboard_config: crate::fingerboard::FingerboardConfig,
    /// Estado interactivo del diapasón.
    pub fingerboard_state: crate::fingerboard::FingerboardState,
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

        let pending_xml: std::rc::Rc<std::cell::RefCell<Option<(String, String)>>> =
            Default::default();

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
            pending_close: false,
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
            view_mode: ViewMode::Score,
            scroll_to_measure: None,
            fingerboard_open: false,
            fingerboard_config: crate::fingerboard::FingerboardConfig::guitar(),
            fingerboard_state: crate::fingerboard::FingerboardState::default(),
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
                            .file_name()
                            .and_then(|s| s.to_str())
                            .unwrap_or("Sin título")
                            .to_string();
                        self.documents
                            .push(Document::from_score(score, label, Some(path.clone())));
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
                chord_symbol: None,
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

/// Renderiza la vista de mapa de temas: timeline horizontal con bloques coloreados por sección.

/// Escanea la partitura en busca de marcas de ensayo (rehearsal marks) y las convierte
/// en secciones de un tema. Cada marca define el inicio de una sección; la sección
/// termina donde empieza la siguiente (o al final de la partitura).
fn detect_sections_from_score(score: &Score) -> Vec<Section> {
    // Collect (measure_index, rehearsal_text) pairs in flat measure order
    let mut marks: Vec<(usize, String)> = Vec::new();
    let chord_symbols = collect_chord_symbols(score);
    let mut flat_idx: usize = 0;

    for system in &score.systems {
        for staff in &system.staves {
            for measure in &staff.measures {
                for dir in &measure.directions {
                    if let DirectionKind::Rehearsal(text) = &dir.kind {
                        marks.push((flat_idx, text.clone()));
                    }
                }
                flat_idx += 1;
            }
        }
    }

    let total = flat_idx;
    if marks.is_empty() || total == 0 {
        return vec![];
    }

    // Sort by measure index (should already be in order, but be safe)
    marks.sort_by_key(|(idx, _)| *idx);

    let mut sections: Vec<Section> = Vec::new();
    for i in 0..marks.len() {
        let (start, ref text) = marks[i];
        let end = if i + 1 < marks.len() {
            marks[i + 1].0.saturating_sub(1)
        } else {
            total.saturating_sub(1)
        };

        if end < start {
            continue;
        }

        let kind = classify_section_kind(text);
        let progression = build_chord_progression(&chord_symbols, start, end);
        sections.push(Section::new(
            kind,
            text.clone(),
            start,
            end,
            progression,
            kind.default_color().to_string(),
        ));
    }

    sections
}

/// Colecta los símbolos de acorde de todos los compases en orden plano.
fn collect_chord_symbols(score: &Score) -> Vec<Option<String>> {
    let mut symbols: Vec<Option<String>> = Vec::new();
    for system in &score.systems {
        for staff in &system.staves {
            for measure in &staff.measures {
                symbols.push(measure.chord_symbol.clone());
            }
        }
    }
    symbols
}

/// Compases consecutivos con el mismo acorde se fusionan en un solo `ChordStep`.
fn build_chord_progression(
    chords: &[Option<String>],
    start: usize,
    end: usize,
) -> ChordProgression {
    let mut steps: Vec<ChordStep> = Vec::new();
    let mut current: Option<&str> = None;
    let mut count: u32 = 0;

    for i in start..=end {
        let sym = chords.get(i).and_then(|c| c.as_deref());
        if sym == current {
            count += 1;
        } else {
            if let Some(s) = current {
                steps.push(ChordStep {
                    symbol: ChordSymbol(s.to_string()),
                    measures: count,
                });
            }
            current = sym;
            count = 1;
        }
    }

    if let Some(s) = current {
        steps.push(ChordStep {
            symbol: ChordSymbol(s.to_string()),
            measures: count,
        });
    }

    ChordProgression { chords: steps }
}

/// Clasifica el texto de una marca de ensayo en un `SectionKind`.
fn classify_section_kind(text: &str) -> SectionKind {
    let lower = text.to_lowercase();
    if lower.contains("intro") || lower.contains("introducción") || lower.contains("introduccion")
    {
        return SectionKind::Intro;
    }
    if lower.contains("verso") || lower.contains("verse") || lower.contains("estrofa") {
        return SectionKind::Verse;
    }
    if lower.contains("estribillo")
        || lower.contains("chorus")
        || lower.contains("coro")
        || lower.contains("refrain")
    {
        return SectionKind::Chorus;
    }
    if lower.contains("puente") || lower.contains("bridge") {
        return SectionKind::Bridge;
    }
    if lower.contains("solo") || lower.contains("interlude") || lower.contains("interludio") {
        return SectionKind::Solo;
    }
    if lower.contains("outro")
        || lower.contains("final")
        || lower.contains("coda")
        || lower.contains("fin")
    {
        return SectionKind::Outro;
    }
    SectionKind::Custom
}

/// Renderiza la vista de mapa: grilla de bloques por compás con aspecto de
/// hoja de partitura, agrupados por sección, más un minimapa horizontal de
/// secciones para navegar rápido. El minimapa asume un único tema dominante
/// (`themes[0]`) — hoy la UI nunca crea más de uno (ver `detect_sections_from_score`).
fn render_theme_map(
    ui: &mut egui::Ui,
    themes: &[Theme],
    sheet: &ScoreStylesheet,
    dark_mode: bool,
    view_mode: &mut ViewMode,
    scroll_to_measure: &mut Option<usize>,
) {
    if themes.is_empty() || themes[0].sections.is_empty() {
        ui.centered_and_justified(|ui| {
            ui.label("Sin temas definidos");
        });
        return;
    }

    let sections: Vec<&Section> = themes[0].sections.iter().collect();

    let font_h = ui.style().text_styles[&egui::TextStyle::Body].size;
    let block_w = font_h * 3.6;
    let block_h = font_h * 2.4;
    let gap = 6.0_f32;
    let header_h = font_h * 1.6;
    let section_gap = font_h * 0.8;
    let card_pad = font_h;
    let minimap_h = font_h * 2.2;

    let header_font = egui::FontId::new(font_h * 1.05, egui::FontFamily::Proportional);
    let num_font = egui::FontId::new(font_h * 0.7, egui::FontFamily::Proportional);
    let chord_font = egui::FontId::new(font_h * 0.85, egui::FontFamily::Proportional);

    let total_measures: usize = sections
        .iter()
        .map(|s| s.end_measure - s.start_measure + 1)
        .sum();

    let avail_w = ui.available_width();
    let card_w = avail_w - card_pad * 2.0;
    let grid_w = (card_w - card_pad * 2.0).max(block_w);
    let blocks_per_row = ((grid_w + gap) / (block_w + gap)).floor().max(1.0) as usize;

    // Layout vertical: posición (relativa al contenido de la tarjeta) de
    // cada sección, según cuántas filas de bloques necesita.
    struct SecLayout {
        y: f32,
    }
    let mut sec_layouts: Vec<SecLayout> = Vec::with_capacity(sections.len());
    let mut y = 0.0f32;
    for section in &sections {
        let measures = section.end_measure - section.start_measure + 1;
        let rows = measures.div_ceil(blocks_per_row).max(1);
        let height = header_h + rows as f32 * block_h + (rows as f32 - 1.0).max(0.0) * gap;
        sec_layouts.push(SecLayout { y });
        y += height + section_gap;
    }
    let content_h = (y - section_gap).max(0.0);
    let card_h = content_h + card_pad * 2.0;

    // ── Minimapa: un segmento por sección, ancho proporcional a sus compases ──
    let (minimap_rect, minimap_resp) =
        ui.allocate_exact_size(egui::Vec2::new(avail_w, minimap_h), egui::Sense::click());
    let mm_painter = ui.painter_at(minimap_rect);
    mm_painter.rect_filled(minimap_rect, 3.0, egui::Color32::BLACK.gamma_multiply(0.15));

    let mut seg_x = minimap_rect.left();
    let mut scroll_target: Option<f32> = None;
    for (i, section) in sections.iter().enumerate() {
        let measures = section.end_measure - section.start_measure + 1;
        let seg_w = minimap_rect.width() * measures as f32 / total_measures.max(1) as f32;
        let seg_rect = egui::Rect::from_min_size(
            egui::Pos2::new(seg_x, minimap_rect.top()),
            egui::Vec2::new(seg_w, minimap_rect.height()),
        );
        let color =
            parse_hex_color(&section.color).unwrap_or_else(|| section.kind.default_color_as_egui());
        mm_painter.rect_filled(seg_rect, 2.0, color.gamma_multiply(0.85));
        mm_painter.rect_stroke(
            seg_rect,
            2.0,
            egui::Stroke::new(1.0, egui::Color32::BLACK.gamma_multiply(0.3)),
            egui::StrokeKind::Inside,
        );
        if seg_w > font_h * 2.5 {
            mm_painter.text(
                seg_rect.center(),
                egui::Align2::CENTER_CENTER,
                &section.label,
                num_font.clone(),
                egui::Color32::WHITE,
            );
        }
        if minimap_resp.clicked()
            && let Some(pos) = minimap_resp.interact_pointer_pos()
            && seg_rect.contains(pos)
        {
            scroll_target = Some(card_pad + sec_layouts[i].y);
        }
        seg_x += seg_w;
    }
    minimap_resp.on_hover_cursor(egui::CursorIcon::PointingHand);

    ui.add_space(font_h * 0.5);

    // ── Grilla de compases estilo hoja de partitura ──
    let mut scroll_area = egui::ScrollArea::vertical()
        .id_salt("theme_map_grid")
        .auto_shrink([false; 2]);
    if let Some(target) = scroll_target {
        scroll_area = scroll_area.vertical_scroll_offset(target);
    }
    scroll_area.show(ui, |ui| {
        let (card_rect, _resp) =
            ui.allocate_exact_size(egui::Vec2::new(card_w, card_h), egui::Sense::hover());
        let painter = ui.painter_at(card_rect);

        painter.rect_filled(card_rect, 4.0, sheet.page_bg(dark_mode));
        painter.rect_stroke(
            card_rect,
            4.0,
            egui::Stroke::new(
                sheet.page.border_width.max(1.0),
                sheet.page_border(dark_mode),
            ),
            egui::StrokeKind::Inside,
        );

        let title_color = sheet.title_color(dark_mode);
        let text_color = sheet.staff_color(dark_mode);

        for (i, section) in sections.iter().enumerate() {
            let sec = &sec_layouts[i];
            let sec_top = card_rect.top() + card_pad + sec.y;

            let color = parse_hex_color(&section.color)
                .unwrap_or_else(|| section.kind.default_color_as_egui());
            let swatch_rect = egui::Rect::from_min_size(
                egui::Pos2::new(card_rect.left() + card_pad, sec_top + header_h * 0.2),
                egui::Vec2::new(header_h * 0.6, header_h * 0.6),
            );
            painter.rect_filled(swatch_rect, 2.0, color);
            painter.text(
                egui::Pos2::new(swatch_rect.right() + 6.0, sec_top + header_h * 0.5),
                egui::Align2::LEFT_CENTER,
                &section.label,
                header_font.clone(),
                title_color,
            );

            let section_measures = section.end_measure - section.start_measure + 1;
            let mut chords_flat: Vec<Option<&str>> = Vec::with_capacity(section_measures);
            for step in &section.progression.chords {
                for _ in 0..step.measures {
                    chords_flat.push(Some(step.symbol.0.as_str()));
                }
            }
            while chords_flat.len() < section_measures {
                chords_flat.push(None);
            }

            let grid_top = sec_top + header_h;
            let mut last_chord: Option<&str> = None;
            for m in 0..section_measures {
                let row = m / blocks_per_row;
                let col = m % blocks_per_row;
                let bx = card_rect.left() + card_pad + col as f32 * (block_w + gap);
                let by = grid_top + row as f32 * (block_h + gap);
                let block_rect = egui::Rect::from_min_size(
                    egui::Pos2::new(bx, by),
                    egui::Vec2::new(block_w, block_h),
                );

                let chord = chords_flat.get(m).copied().flatten();
                let show_chord = chord.is_some() && chord != last_chord;
                last_chord = chord;

                painter.rect_filled(block_rect, 3.0, color.gamma_multiply(0.22));
                painter.rect_stroke(
                    block_rect,
                    3.0,
                    egui::Stroke::new(1.0, color.gamma_multiply(0.7)),
                    egui::StrokeKind::Inside,
                );

                let abs_measure = section.start_measure + m;
                painter.text(
                    egui::Pos2::new(block_rect.left() + 4.0, block_rect.top() + 2.0),
                    egui::Align2::LEFT_TOP,
                    format!("{}", abs_measure + 1),
                    num_font.clone(),
                    text_color,
                );
                if show_chord && let Some(c) = chord {
                    painter.text(
                        block_rect.center(),
                        egui::Align2::CENTER_CENTER,
                        c,
                        chord_font.clone(),
                        title_color,
                    );
                }

                let click_resp = ui.interact(
                    block_rect,
                    ui.id().with(("theme_map_block", abs_measure)),
                    egui::Sense::click(),
                );
                if click_resp.clicked() {
                    *scroll_to_measure = Some(abs_measure);
                    *view_mode = ViewMode::Score;
                }
                click_resp.on_hover_cursor(egui::CursorIcon::PointingHand);
            }
        }
    });
}

/// Parsea un color hex (ej. "#4A90D9") a `egui::Color32`.
fn parse_hex_color(hex: &str) -> Option<egui::Color32> {
    let hex = hex.trim_start_matches('#');
    if hex.len() != 6 {
        return None;
    }
    let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
    let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
    let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
    Some(egui::Color32::from_rgb(r, g, b))
}

impl SectionKind {
    /// Returns the default color as an `egui::Color32`.
    fn default_color_as_egui(&self) -> egui::Color32 {
        parse_hex_color(self.default_color()).unwrap_or(egui::Color32::GRAY)
    }
}

impl eframe::App for MGuitarApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        if self.first_frame {
            self.first_frame = false;
            ui.ctx().send_viewport_cmd(ViewportCommand::Maximized(true));
        }
        if self.pending_close {
            self.pending_close = false;
            if !self.documents.is_empty() {
                self.documents.remove(self.active_doc);
                if self.active_doc >= self.documents.len() {
                    self.active_doc = self.active_doc.saturating_sub(1);
                }
                self.save_session();
            }
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

                                // Theme map toggle
                                let map_label = if self.view_mode == ViewMode::ThemeMap {
                                    "🎼"
                                } else {
                                    "🗺️"
                                };
                                let mut map_clicked = false;
                                ui.add_enabled_ui(has_doc, |ui| {
                                    if ui
                                        .selectable_label(
                                            self.view_mode == ViewMode::ThemeMap,
                                            map_label,
                                        )
                                        .clicked()
                                    {
                                        map_clicked = true;
                                    }
                                });
                                if map_clicked {
                                    self.view_mode = match self.view_mode {
                                        ViewMode::Score => ViewMode::ThemeMap,
                                        ViewMode::ThemeMap => ViewMode::Score,
                                    };
                                }
                                let seek_btn = ui.add_enabled(
                                    has_doc,
                                    egui::Button::new(self.i18n.t("seek_start")),
                                );
                                if seek_btn.clicked() {
                                    self.audio.seek_start();
                                }

                                if playing {
                                    if ui.button(self.i18n.t("pause")).clicked() {
                                        self.audio.pause();
                                    }
                                    ui.ctx().request_repaint();
                                } else {
                                    let play_btn = ui.add_enabled(
                                        has_doc,
                                        egui::Button::new(self.i18n.t("play")),
                                    );
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
                                let stop_btn = ui.add_enabled(
                                    stop_enabled,
                                    egui::Button::new(self.i18n.t("stop")),
                                );
                                if stop_btn.clicked() {
                                    self.audio.stop();
                                }
                            }
                            #[cfg(target_arch = "wasm32")]
                            {
                                ui.add_enabled(false, egui::Button::new(self.i18n.t("play")))
                                    .on_disabled_hover_text(self.i18n.t("play_wasm_unavailable"));
                            }

                            // ── Fingerboard toggle ──
                            if ui
                                .selectable_label(self.fingerboard_open, "🎸")
                                .on_hover_text("Diapasón")
                                .clicked()
                            {
                                self.fingerboard_open = !self.fingerboard_open;
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
                            self.status_message =
                                Some(self.i18n.t("open_error").replace("{}", &e.to_string()));
                        }
                    }
                }
            }
        }

        if let Some((xml, path)) = self.pending_xml.borrow_mut().take() {
            match parse_musicxml(&xml) {
                Ok(score) => {
                    let label = std::path::Path::new(&path)
                        .file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or("Sin título")
                        .to_string();
                    self.documents
                        .push(Document::from_score(score, label, Some(path)));
                    self.active_doc = self.documents.len() - 1;
                    self.status_message = Some(self.i18n.t("open_success").replace("{}", ""));
                    self.save_session();
                }
                Err(e) => {
                    self.status_message =
                        Some(self.i18n.t("open_error").replace("{}", &e.to_string()));
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

            if self.view_mode == ViewMode::ThemeMap {
                // Auto-detect sections from the score if no themes are defined
                if self.documents[active].themes.is_empty() {
                    let sections = detect_sections_from_score(&self.documents[active].score);
                    let title = self.documents[active].score.title.clone();
                    let theme_name = if title.is_empty() {
                        self.i18n.t("theme_map_default_name").to_string()
                    } else {
                        title.clone()
                    };
                    if sections.is_empty() {
                        // Fallback: single section spanning all measures
                        let total: usize = self.documents[active]
                            .score
                            .systems
                            .iter()
                            .flat_map(|s| s.staves.iter())
                            .map(|st| st.measures.len())
                            .sum();
                        if total > 0 {
                            let fallback_chords =
                                collect_chord_symbols(&self.documents[active].score);
                            let progression = if total > 0 {
                                build_chord_progression(&fallback_chords, 0, total - 1)
                            } else {
                                ChordProgression { chords: vec![] }
                            };
                            self.documents[active].themes.push(Theme {
                                name: theme_name,
                                sections: vec![Section::new(
                                    SectionKind::Custom,
                                    title,
                                    0,
                                    total.saturating_sub(1),
                                    progression,
                                    SectionKind::Custom.default_color().to_string(),
                                )],
                            });
                        }
                    } else {
                        self.documents[active].themes.push(Theme {
                            name: theme_name,
                            sections,
                        });
                    }
                }

                egui::CentralPanel::default()
                    .frame(
                        egui::Frame::default()
                            .fill(egui::Color32::from_rgb(0x44, 0x47, 0x51))
                            .stroke(egui::Stroke::NONE),
                    )
                    .show(ui, |ui| {
                        let sheet = &self.stylesheets[self.selected_sheet];
                        render_theme_map(
                            ui,
                            &self.documents[active].themes,
                            sheet,
                            self.dark_mode,
                            &mut self.view_mode,
                            &mut self.scroll_to_measure,
                        );
                    });
            } else {
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
                        let layout = compute_pages(
                            &self.documents[active].score,
                            render_zoom,
                            header_height,
                        );

                        // Tabs bar — alineada con el borde izquierdo de la partitura
                        if !self.documents.is_empty() {
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
                                                .color(egui::Color32::from_rgb(0x66, 0x66, 0x66)),
                                        );
                                    }
                                    let label = if doc.dirty {
                                        format!("● {}", doc.label)
                                    } else {
                                        doc.label.clone()
                                    };
                                    if i == self.active_doc {
                                        let group = ui.horizontal(|ui| {
                                            let r = ui
                                                .add(
                                                    egui::Label::new(egui::RichText::new(label))
                                                        .sense(egui::Sense::click()),
                                                )
                                                .on_hover_cursor(egui::CursorIcon::Default);
                                            let close = if self.documents.len() > 1 {
                                                ui.add(
                                                    egui::Label::new(
                                                        egui::RichText::new("×")
                                                            .color(egui::Color32::from_rgb(
                                                                0x99, 0x99, 0x99,
                                                            ))
                                                            .size(12.0),
                                                    )
                                                    .sense(egui::Sense::click()),
                                                )
                                                .on_hover_cursor(egui::CursorIcon::Default)
                                            } else {
                                                ui.add(egui::Label::new(
                                                    egui::RichText::new("×")
                                                        .color(egui::Color32::from_rgb(
                                                            0x55, 0x55, 0x55,
                                                        ))
                                                        .size(12.0),
                                                ))
                                                .on_hover_cursor(egui::CursorIcon::Default)
                                            };
                                            (r, close)
                                        });
                                        let rect = group.response.rect;
                                        let underline = egui::Rect::from_min_max(
                                            egui::Pos2::new(rect.left(), rect.bottom()),
                                            egui::Pos2::new(rect.right(), rect.bottom() + 1.0),
                                        );
                                        ui.painter().rect_filled(underline, 0.0, active_color);
                                        if group.inner.0.clicked() {
                                            self.active_doc = i;
                                        }
                                        if group.inner.1.clicked() {
                                            self.pending_close = true;
                                            ui.ctx().request_repaint();
                                        }
                                    } else {
                                        let response = ui
                                            .add(
                                                egui::Label::new(egui::RichText::new(label))
                                                    .sense(egui::Sense::click()),
                                            )
                                            .on_hover_cursor(egui::CursorIcon::Default);
                                        if response.clicked() {
                                            self.active_doc = i;
                                        }
                                    }
                                }
                            });

                            ui.add_space(4.0);
                        }

                        let maybe_scroll = self.scroll_to_measure.take().and_then(|measure_idx| {
                            layout
                                .measure_position(measure_idx, &self.documents[active].score)
                                .map(|(page_idx, y_in_page)| {
                                    let pages_per_row = 1usize;
                                    let row = page_idx / pages_per_row;
                                    (row as f32) * (layout.page_height + PAGE_GAP * render_zoom)
                                        + y_in_page
                                })
                        });

                        let mut scroll_area = egui::ScrollArea::vertical().auto_shrink([false; 2]);
                        if let Some(offset) = maybe_scroll {
                            scroll_area = scroll_area.vertical_scroll_offset(offset);
                        }
                        scroll_area.show(ui, |ui| {
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
                            let top_left = response.rect.min + egui::Vec2::new(left_pad, 20.0);
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
        }

        // ── Fingerboard window ──
        if self.fingerboard_open {
            use crate::fingerboard::{fingerboard_draw_size, render_fingerboard, FingerboardConfig};
            // Al abrir, dimensiona la ventana para que el diapasón completo (escala
            // actual, por defecto 3.0x) entre sin scroll. Margen para la toolbar
            // interna y el separador.
            let draw = fingerboard_draw_size(&self.fingerboard_config, ui);
            let win_w = draw.x + 16.0;
            let win_h = draw.y + font_h * 3.5 + 16.0;
            let win_size = egui::vec2(win_w.max(350.0), win_h.max(250.0));
            egui::Window::new("🎸 Diapasón")
                .id(egui::Id::new("fingerboard_window"))
                .default_size(win_size)
                .min_size(egui::vec2(350.0, 220.0))
                .resizable(true)
                .collapsible(true)
                .open(&mut self.fingerboard_open)
                .show(ui, |ui| {
                    // ── Toolbar dentro del frame ──
                    ui.horizontal(|ui| {
                        ui.label("Afinación:");
                        let is_guitar = self.fingerboard_config.tuning.len() == 6;
                        if ui.selectable_label(is_guitar, "🎸 Guitarra").clicked() {
                            if !is_guitar {
                                self.fingerboard_config = FingerboardConfig::guitar();
                                self.fingerboard_state.selected.clear();
                            }
                        }
                        if ui.selectable_label(!is_guitar, "🎸 Bajo").clicked() {
                            if is_guitar {
                                self.fingerboard_config = FingerboardConfig::bass();
                                self.fingerboard_state.selected.clear();
                            }
                        }

                        ui.separator();
                        ui.label("Trastes:");
                        let mut frets = self.fingerboard_config.frets;
                        if ui
                            .add(egui::Slider::new(&mut frets, 5..=24).text(""))
                            .changed()
                        {
                            self.fingerboard_config.frets = frets;
                        }

                        ui.separator();
                        ui.label("Tamaño:");
                        ui.add(
                            egui::Slider::new(&mut self.fingerboard_config.scale, 0.5..=3.0)
                                .text("x"),
                        );
                        if ui
                            .selectable_label(self.fingerboard_config.show_intervals, "Intervalos")
                            .clicked()
                        {
                            self.fingerboard_config.show_intervals =
                                !self.fingerboard_config.show_intervals;
                            if self.fingerboard_config.show_intervals
                                && self.fingerboard_config.tonic.is_none()
                            {
                                // Default tonic: E2 (primer cuerda al aire de guitarra)
                                self.fingerboard_config.tonic =
                                    Some(40); // E2 = 40 semitonos desde C0
                            }
                        }
                        if ui.button("Limpiar").clicked() {
                            self.fingerboard_state.selected.clear();
                        }
                    });
                    ui.separator();

                    // ── El diapasón ──
                    egui::ScrollArea::horizontal()
                        .id_salt("fingerboard_scroll")
                        .show(ui, |ui| {
                            render_fingerboard(
                                ui,
                                &self.fingerboard_config,
                                &mut self.fingerboard_state,
                            );
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

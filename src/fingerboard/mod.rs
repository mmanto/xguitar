//! Módulo Fingerboard: diapasón interactivo de guitarra y bajo.
//!
//! Renderiza un diapasón con trastes, cuerdas y notas.
//! Soporta guitarra (6 cuerdas) y bajo (4 cuerdas) con afinación estándar.

use eframe::egui;
use egui::Color32;
use std::collections::HashSet;

/// Afinación estándar para guitarra (6 cuerdas): E2 A2 D3 G3 B3 E4 (de grave a agudo).
pub const GUITAR_TUNING: [&str; 6] = ["E2", "A2", "D3", "G3", "B3", "E4"];
/// Afinación estándar para bajo (4 cuerdas): E1 A1 D2 G2.
pub const BASS_TUNING: [&str; 4] = ["E1", "A1", "D2", "G2"];

/// Número de trastes visibles por defecto.
pub const DEFAULT_FRETS: usize = 12;

/// Intervalo de semitonos entre trastes consecutivos.
const SEMITONES_PER_FRET: i32 = 1;

/// Convierte un nombre de nota (ej. "C4", "F#3", "Bb2") a semitonos desde C0.
fn note_name_to_semitones(name: &str) -> Option<i32> {
    let name = name.trim();
    if name.is_empty() {
        return None;
    }

    let mut chars = name.chars();
    let base = chars.next()?;
    let step = match base {
        'C' => 0,
        'D' => 2,
        'E' => 4,
        'F' => 5,
        'G' => 7,
        'A' => 9,
        'B' => 11,
        _ => return None,
    };

    let mut accidental: i32 = 0;
    for c in chars {
        match c {
            '#' | '♯' => accidental += 1,
            'b' | '♭' => accidental -= 1,
            '0'..='9' => {
                let octave = c.to_digit(10)? as i32;
                return Some(octave * 12 + step + accidental);
            }
            _ => {}
        }
    }
    None
}

/// Convierte semitonos desde C0 a nombre de nota con octava (ej. 40 → "E2").
pub fn semitones_to_note_name(semitones: i32) -> String {
    let step_names = ["C", "C#", "D", "D#", "E", "F", "F#", "G", "G#", "A", "A#", "B"];
    let octave = semitones.div_euclid(12);
    let step = semitones.rem_euclid(12) as usize;
    format!("{}{}", step_names[step], octave)
}

/// Configuración del diapasón.
#[derive(Clone)]
pub struct FingerboardConfig {
    /// Nombres de las cuerdas de grave a agudo (ej. ["E2", "A2", ...]).
    pub tuning: Vec<String>,
    /// Número de trastes visibles.
    pub frets: usize,
    /// Escala de dibujo (1.0 = tamaño base; >1 amplía el diapasón para lectura).
    pub scale: f32,
    /// Mostrar intervalos en lugar de nombres de nota.
    pub show_intervals: bool,
    /// Nota tónica para los intervalos (en semitonos desde C0).
    pub tonic: Option<i32>,
}

impl Default for FingerboardConfig {
    fn default() -> Self {
        Self {
            tuning: GUITAR_TUNING.iter().map(|s| s.to_string()).collect(),
            frets: DEFAULT_FRETS,
            scale: 3.0,
            show_intervals: false,
            tonic: None,
        }
    }
}

impl FingerboardConfig {
    pub fn guitar() -> Self {
        Self::default()
    }

    pub fn bass() -> Self {
        Self {
            tuning: BASS_TUNING.iter().map(|s| s.to_string()).collect(),
            frets: DEFAULT_FRETS,
            scale: 3.0,
            show_intervals: false,
            tonic: None,
        }
    }

    /// Devuelve los semitonos (desde C0) de la cuerda `string_idx` en el traste `fret`.
    pub fn note_semitones(&self, string_idx: usize, fret: usize) -> Option<i32> {
        let open = note_name_to_semitones(&self.tuning.get(string_idx)?)?;
        Some(open + fret as i32 * SEMITONES_PER_FRET)
    }
}

/// Estado interactivo del diapasón.
#[derive(Default)]
pub struct FingerboardState {
    /// Posiciones seleccionadas: (string_idx, fret).
    pub selected: HashSet<(usize, usize)>,
    /// Cuerda hover (para tooltip).
    pub hover_string: Option<usize>,
    /// Traste hover.
    pub hover_fret: Option<usize>,
}

/// Computa el tamaño de dibujo en píxeles del diapasón según la configuración y el
/// estilo de UI actual (incluye la escala). Usado para dimensionar la ventana.
pub fn fingerboard_draw_size(config: &FingerboardConfig, ui: &egui::Ui) -> egui::Vec2 {
    if config.tuning.is_empty() || config.frets == 0 {
        return egui::Vec2::ZERO;
    }
    let scale = config.scale.max(0.25);
    let font_h = ui.style().text_styles[&egui::TextStyle::Body].size * scale;
    let string_spacing = font_h * 0.9;
    let fret_spacing = font_h * 1.5;
    let label_w = font_h * 2.5;
    let top_pad = font_h * 1.2;
    let side_pad = font_h * 0.5;
    let width = label_w + side_pad * 2.0 + config.frets as f32 * fret_spacing;
    let height =
        top_pad + (config.tuning.len() - 1) as f32 * string_spacing + font_h * 0.8;
    egui::vec2(width, height)
}

/// Widget que renderiza el diapasón.
pub fn render_fingerboard(
    ui: &mut egui::Ui,
    config: &FingerboardConfig,
    state: &mut FingerboardState,
) {
    let size = fingerboard_draw_size(config, ui);
    let num_strings = config.tuning.len();
    let num_frets = config.frets;
    if config.tuning.is_empty() || config.frets == 0 {
        ui.label("Sin cuerdas o trastes configurados");
        return;
    }

    // Escala de dibujo: amplía todo el diapasón (espaciado, textos, notas) de forma
    // proporcional para facilitar la lectura. Se aplica sobre `font_h`, del cual se
    // derivan todas las demás dimensiones.
    let scale = config.scale.max(0.25);
    let font_h = ui.style().text_styles[&egui::TextStyle::Body].size * scale;

    // Dimensiones
    let string_spacing = font_h * 0.9;
    let fret_spacing = font_h * 1.5;
    let label_w = font_h * 2.5;
    let top_pad = font_h * 1.2;
    let side_pad = font_h * 0.5;

    let (response, painter) = ui.allocate_painter(size, egui::Sense::click());

    let rect = response.rect;
    let origin = rect.min + egui::Vec2::new(label_w + side_pad, top_pad);

    let bg_color = Color32::from_rgb(0x1E, 0x1E, 0x24);
    let string_color = Color32::from_rgb(0xCC, 0xCC, 0xCC);
    let fret_color = Color32::from_rgb(0x55, 0x55, 0x55);
    let nut_color = Color32::from_rgb(0x88, 0x66, 0x22);
    let note_color = Color32::from_rgb(0xFF, 0xD7, 0x00);
    let note_bg = Color32::from_rgb(0x33, 0x33, 0x44);
    let selected_bg = Color32::from_rgb(0x44, 0x77, 0xBB);
    let selected_color = Color32::WHITE;
    let hover_bg = Color32::from_rgb(0x3A, 0x3A, 0x4A);
    let tuning_color = Color32::from_rgb(0x99, 0x99, 0x99);
    let fret_text_color = Color32::from_rgb(0x88, 0x88, 0x88);

    // Fondo
    painter.rect_filled(response.rect, 0.0, bg_color);

    // ── Marcas de trastes ──
    // Cejilla (nut) en el traste 0
    painter.line_segment(
        [
            egui::pos2(origin.x, origin.y - font_h * 0.3),
            egui::pos2(origin.x, origin.y + (num_strings - 1) as f32 * string_spacing + font_h * 0.3),
        ],
        egui::Stroke::new(font_h * 0.15, nut_color),
    );

    for fret in 0..=num_frets {
        let x = origin.x + fret as f32 * fret_spacing;
        if fret == 0 {
            continue;
        }
        let alpha = if fret % 12 == 0 {
            0.8
        } else if fret % 5 == 0 {
            0.5
        } else {
            0.3
        };
        let color = Color32::from_rgba_premultiplied(
            fret_color.r(),
            fret_color.g(),
            fret_color.b(),
            (alpha * 255.0) as u8,
        );
        painter.line_segment(
            [
                egui::pos2(x, origin.y),
                egui::pos2(x, origin.y + (num_strings - 1) as f32 * string_spacing),
            ],
            egui::Stroke::new(1.0, color),
        );

        // Número de traste en la parte superior
        if fret <= 12 || fret % 12 == 0 {
            painter.text(
                egui::pos2(x, origin.y - font_h * 0.1),
                egui::Align2::CENTER_BOTTOM,
                &fret.to_string(),
                egui::FontId::new(font_h * 0.5, egui::FontFamily::Proportional),
                fret_text_color,
            );
        }
    }

    // ── Puntos de referencia en el diapasón (trastes 3, 5, 7, 9, 12, 15) ──
    for &fret in &[3usize, 5, 7, 9, 12, 15] {
        if fret > num_frets {
            continue;
        }
        let x = origin.x + fret as f32 * fret_spacing - fret_spacing / 2.0;
        let dot_radius = font_h * 0.12;
        if fret == 12 {
            // Dos puntos
            for (y_off, _) in [(0.35, 0), (0.65, 1)] {
                let cy = origin.y + (num_strings - 1) as f32 * string_spacing * y_off;
                painter.circle_filled(
                    egui::pos2(x, cy),
                    dot_radius,
                    Color32::from_rgb(0x88, 0x88, 0x88),
                );
            }
        } else {
            let cy = origin.y + (num_strings - 1) as f32 * string_spacing * 0.5;
            painter.circle_filled(
                egui::pos2(x, cy),
                dot_radius,
                Color32::from_rgb(0x88, 0x88, 0x88),
            );
        }
    }

    // ── Cuerdas ──
    for s in 0..num_strings {
        let y = origin.y + s as f32 * string_spacing;
        // Grosor según cuerda (más grave = más gruesa), escalado con el diapasón
        let thickness = (1.0 + (num_strings - s) as f32 * 0.4) * scale.max(1.0);
        painter.line_segment(
            [
                egui::pos2(origin.x, y),
                egui::pos2(origin.x + num_frets as f32 * fret_spacing, y),
            ],
            egui::Stroke::new(thickness, string_color),
        );

        // Nombre de la cuerda (afinación)
        let label_x = origin.x - font_h * 0.15;
        painter.text(
            egui::pos2(label_x, y),
            egui::Align2::RIGHT_CENTER,
            &config.tuning[s],
            egui::FontId::new(font_h * 0.55, egui::FontFamily::Proportional),
            tuning_color,
        );
    }

    // ── Notas en el diapasón ──
    let note_font = egui::FontId::new(font_h * 0.45, egui::FontFamily::Proportional);

    for s in 0..num_strings {
        let y = origin.y + s as f32 * string_spacing;
        for f in 0..=num_frets {
            if f == 0 {
                continue; // No mostrar notas al aire aquí
            }
            let x = origin.x + f as f32 * fret_spacing - fret_spacing / 2.0;

            let selected = state.selected.contains(&(s, f));
            let hovered = state.hover_string == Some(s) && state.hover_fret == Some(f);

            let pos = egui::pos2(x, y);
            let note_size = font_h * 0.35;

            // Fondo de nota (círculo)
            let fill = if selected {
                selected_bg
            } else if hovered {
                hover_bg
            } else {
                note_bg
            };
            painter.circle_filled(pos, note_size + 2.0, fill);

            if selected {
                painter.circle_stroke(pos, note_size + 3.0, egui::Stroke::new(1.0, selected_color));
            }

            // Texto de la nota
            if let Some(semitones) = config.note_semitones(s, f) {
                let note_name = if config.show_intervals {
                    if let Some(tonic) = config.tonic {
                        let interval = semitones - tonic;
                        let interval_mod = interval.rem_euclid(12);
                        let interval_names = [
                            "1", "b2", "2", "b3", "3", "4", "#4", "5", "b6", "6", "b7", "7",
                        ];
                        interval_names[interval_mod as usize].to_string()
                    } else {
                        semitones_to_note_name(semitones)
                    }
                } else {
                    semitones_to_note_name(semitones)
                };
                let color = if selected { selected_color } else { note_color };
                painter.text(pos, egui::Align2::CENTER_CENTER, &note_name, note_font.clone(), color);
            }
        }
    }

    // ── Interacción ──
    let mut hover_text = String::new();
    if let Some(pos) = response.hover_pos() {
        let rel = pos - origin;
        let fret_f = rel.x / fret_spacing;
        let string_f = rel.y / string_spacing;

        let f = (fret_f + 0.5).floor() as usize;
        let s = (string_f + 0.5).floor() as usize;

        if f >= 1 && f <= num_frets && s < num_strings {
            state.hover_string = Some(s);
            state.hover_fret = Some(f);

            if let Some(semitones) = config.note_semitones(s, f) {
                let note_name = semitones_to_note_name(semitones);
                hover_text = format!(
                    "Cuerda {} ({}), Traste {}: {}",
                    s + 1,
                    config.tuning[s],
                    f,
                    note_name
                );
            }
            ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
        } else {
            state.hover_string = None;
            state.hover_fret = None;
        }
    } else {
        state.hover_string = None;
        state.hover_fret = None;
    }
    // Tooltip texto via response
    if !hover_text.is_empty() {
        let _ = response.clone().on_hover_text(hover_text);
    }

    // Click para seleccionar/deseleccionar
    if response.clicked() {
        if let Some(pos) = response.interact_pointer_pos() {
            let rel = pos - origin;
            let fret_f = rel.x / fret_spacing;
            let string_f = rel.y / string_spacing;

            let f = (fret_f + 0.5).floor() as usize;
            let s = (string_f + 0.5).floor() as usize;

            if f >= 1 && f <= num_frets && s < num_strings {
                if state.selected.contains(&(s, f)) {
                    state.selected.remove(&(s, f));
                } else {
                    state.selected.insert((s, f));
                }
            }
        }
    }
}
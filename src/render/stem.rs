use crate::notation::{NoteFigure, StemDirection};
use eframe::egui;

/// Dibuja la plica de una nota.
///
/// `note_x`, `note_y` — posición del centro de la cabeza de nota.
/// `staff_position` — posición en el staff (0 = línea central).
/// `figure` — figura rítmica (solo Half o más cortas llevan plica).
/// `direction` — dirección de la plica.
/// `is_beamed` — si es true, la plica se extiende hasta el beam (sin bandera).
pub fn render_stem(
    painter: &egui::Painter,
    note_x: f32,
    note_y: f32,
    _staff_position: i8,
    figure: NoteFigure,
    direction: StemDirection,
    is_beamed: bool,
    beam_y: Option<f32>,
    line_spacing: f32,
    color: egui::Color32,
) {
    // Whole and Breve have no stem
    if matches!(figure, NoteFigure::Whole | NoteFigure::Breve) {
        return;
    }
    let half_spacing = line_spacing / 2.0;

    let (stem_top, stem_bottom) = match direction {
        StemDirection::Up => {
            let start_y = note_y - half_spacing * 0.5;
            let end_y = beam_y.unwrap_or(start_y - line_spacing * 3.5);
            (end_y, start_y)
        }
        StemDirection::Down => {
            let start_y = note_y + half_spacing * 0.5;
            let end_y = beam_y.unwrap_or(start_y + line_spacing * 3.5);
            (start_y, end_y)
        }
    };

    let stem_x = match direction {
        StemDirection::Up => note_x + line_spacing * 0.40,
        StemDirection::Down => note_x - line_spacing * 0.40,
    };

    let stem_width = line_spacing * 0.08;
    let stroke = egui::Stroke::new(stem_width, color);
    painter.line_segment(
        [
            egui::Pos2::new(stem_x, stem_top),
            egui::Pos2::new(stem_x, stem_bottom),
        ],
        stroke,
    );

    // Draw flag if not beamed and figure needs one
    if !is_beamed && figure.flag_count() > 0 {
        let tip_y = match direction {
            StemDirection::Up => stem_top,
            StemDirection::Down => stem_bottom,
        };
        render_flag(
            painter,
            stem_x,
            tip_y,
            figure,
            direction,
            line_spacing,
            color,
        );
    }
}

/// Dibuja las banderas/barritas en el extremo de la plica.
pub fn render_flag(
    painter: &egui::Painter,
    tip_x: f32,
    tip_y: f32,
    figure: NoteFigure,
    direction: StemDirection,
    line_spacing: f32,
    color: egui::Color32,
) {
    let flag_count = figure.flag_count();
    if flag_count == 0 {
        return;
    }

    // SMuFL flag glyphs: U+E240 (eighth up), U+E241 (eighth down),
    // U+E242 (16th up), U+E243 (16th down), etc.
    let base_codepoint: u32 = match direction {
        StemDirection::Up => 0xE240,
        StemDirection::Down => 0xE241,
    };

    let font_size = line_spacing * 2.8;
    let font_id = egui::FontId::new(font_size, egui::FontFamily::Name("Leland".into()));

    // Flags stack: each subsequent flag is 2 SMuFL codepoints higher
    for i in 0..flag_count {
        let glyph = char::from_u32(base_codepoint + i as u32 * 2).unwrap_or('\u{E240}');
        let offset_y = match direction {
            StemDirection::Up => -line_spacing * 0.5 * i as f32,
            StemDirection::Down => line_spacing * 0.5 * i as f32,
        };
        painter.text(
            egui::Pos2::new(tip_x, tip_y + offset_y),
            egui::Align2::LEFT_CENTER,
            glyph.to_string(),
            font_id.clone(),
            color,
        );
    }
}

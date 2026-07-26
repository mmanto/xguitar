use super::constants::LEDGER_LINE_WIDTH;
use crate::notation::{Accidental, NoteFigure};
use eframe::egui;

/// Dibuja una cabeza de nota con alteración y puntillos.
///
/// `staff_origin` = Pos2 de la línea central del pentagrama (tercera línea).
/// `staff_position` = desplazamiento en posiciones staff (0 = línea central).
///
/// Cada paso de staff_position equivale a `line_spacing / 2.0` píxeles.
/// Valores positivos = más agudo (hacia arriba en pantalla).
pub fn render_notehead(
    painter: &egui::Painter,
    note_x: f32,
    staff_origin: egui::Pos2,
    staff_position: i8,
    figure: NoteFigure,
    color: egui::Color32,
    line_spacing: f32,
) {
    let half_spacing = line_spacing / 2.0;
    let note_y = staff_origin.y - staff_position as f32 * half_spacing;

    render_ledger_lines(
        painter,
        note_x,
        staff_origin,
        staff_position,
        color,
        line_spacing,
    );

    // Render SMuFL glyph via Leland font.
    // Font size = 4 staff spaces gives proper notehead-to-staff ratio.
    let font_size = line_spacing * 4.0;
    let glyph_char = figure.notehead_glyph();
    // Center the notehead glyph on the staff position
    let glyph_center = egui::Pos2::new(note_x, note_y);
    painter.text(
        glyph_center,
        egui::Align2::CENTER_CENTER,
        glyph_char.to_string(),
        egui::FontId::new(font_size, egui::FontFamily::Name("Leland".into())),
        color,
    );
}

/// Dibuja una alteración a la izquierda de la nota.
pub fn render_accidental(
    painter: &egui::Painter,
    note_x: f32,
    note_y: f32,
    accidental: Accidental,
    line_spacing: f32,
    color: egui::Color32,
) {
    let font_size = line_spacing * 3.6;
    let font_id = egui::FontId::new(font_size, egui::FontFamily::Name("Leland".into()));
    let acc_x = note_x - line_spacing * 1.2; // ~12pt left of notehead

    painter.text(
        egui::Pos2::new(acc_x, note_y + line_spacing * 0.1),
        egui::Align2::CENTER_CENTER,
        accidental.glyph().to_string(),
        font_id,
        color,
    );
}

/// Dibuja los puntillos a la derecha de la nota.
pub fn render_dots(
    painter: &egui::Painter,
    note_x: f32,
    note_y: f32,
    count: u8,
    staff_position: i8,
    line_spacing: f32,
    color: egui::Color32,
) {
    if count == 0 {
        return;
    }

    let half_spacing = line_spacing / 2.0;
    let dot_radius = line_spacing * 0.21; // ~2.5pt
    let dot_spacing = line_spacing * 0.35; // ~4pt between dots

    // Dot vertical position: if note is on a line, dot goes in space above;
    // if note is on a space, dot goes in same space.
    let dot_y = if staff_position % 2 == 0 {
        // On a line — put dot in space above
        note_y - half_spacing
    } else {
        // In a space
        note_y
    };

    let base_x = note_x + line_spacing * 1.0;
    for i in 0..count {
        let dot_x = base_x + i as f32 * dot_spacing;
        painter.circle_filled(egui::Pos2::new(dot_x, dot_y), dot_radius, color);
    }
}

/// Dibuja líneas adicionales para notas fuera del pentagrama.
///
/// Las líneas de ledger se dibujan en posiciones de línea (staff_position par)
/// entre el borde del pentagrama y la nota.
fn render_ledger_lines(
    painter: &egui::Painter,
    note_x: f32,
    staff_origin: egui::Pos2,
    staff_position: i8,
    color: egui::Color32,
    line_spacing: f32,
) {
    let stroke = egui::Stroke::new(1.0, color);
    let half_spacing = line_spacing / 2.0;
    let half_width = LEDGER_LINE_WIDTH / 2.0;
    let center_x = note_x;

    let line_positions: Vec<i8> = if staff_position > 5 {
        // Notes above the staff — first ledger line at position 6
        let last_line = if staff_position % 2 == 0 {
            staff_position
        } else {
            staff_position - 1 // odd = space; ledger stops at previous line
        };
        (6..=last_line).step_by(2).collect()
    } else if staff_position < -5 {
        // Notes below the staff — first ledger line at position -6
        let first_line = if staff_position % 2 == 0 {
            staff_position
        } else {
            staff_position + 1 // odd = space; ledger starts at next line
        };
        (first_line..=-6).step_by(2).collect()
    } else {
        // Inside or immediately adjacent to the staff — no ledger lines
        return;
    };

    for line_pos in line_positions {
        let y = staff_origin.y - line_pos as f32 * half_spacing;
        painter.line_segment(
            [
                egui::Pos2::new(center_x - half_width, y),
                egui::Pos2::new(center_x + half_width, y),
            ],
            stroke,
        );
    }
}

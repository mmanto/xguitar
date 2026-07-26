use crate::notation::Clef;
use eframe::egui;

/// Dibuja una clave al inicio del staff.
///
/// `staff_top` es la esquina superior izquierda del pentagrama (primera línea = top).
/// `glyph_scale` y `fine_offset` vienen del stylesheet.
pub fn render_clef(
    painter: &egui::Painter,
    staff_top: egui::Pos2,
    clef: Clef,
    line_spacing: f32,
    color: egui::Color32,
    glyph_scale: f32,
    fine_offset: f32,
) {
    let font_size = line_spacing * glyph_scale;
    let font_id = egui::FontId::new(font_size, egui::FontFamily::Name("Leland".into()));

    // Reference staff line (measured from staff_top in line_spacing units):
    // Treble (G clef, line 2): staff_top + 3 * line_spacing (2nd from bottom)
    // Bass (F clef, line 4): staff_top + 1 * line_spacing (2nd from top)
    // Alto (C clef, line 3): staff_top + 2 * line_spacing (middle)
    // Tenor (C clef, line 4): staff_top + 1 * line_spacing (2nd from top)
    // Percussion (line 3): staff_top + 2 * line_spacing (middle)
    // Tab (line 5): staff_top + 0 * line_spacing (top line)
    let reference_y = match clef {
        Clef::Treble => staff_top.y + line_spacing * 3.0,
        Clef::Bass => staff_top.y + line_spacing * 1.0,
        Clef::Alto => staff_top.y + line_spacing * 2.0,
        Clef::Tenor => staff_top.y + line_spacing * 1.0,
        Clef::Percussion => staff_top.y + line_spacing * 2.0,
        Clef::Tab => staff_top.y,
    };

    // SMuFL clef glyphs need a slight empirical offset for Leland.
    // fine_offset is a multiplier applied to line_spacing.
    let fine_offset_val = line_spacing * fine_offset;

    painter.text(
        egui::Pos2::new(staff_top.x, reference_y + fine_offset_val),
        egui::Align2::LEFT_CENTER,
        clef.glyph().to_string(),
        font_id,
        color,
    );
}

use crate::notation::{NoteFigure, Rest};
use eframe::egui;

/// Dibuja un silencio en la posición staff indicada.
///
/// `x` — posición horizontal (centro del glifo).
/// `staff_origin_y` — coordenada y de la línea central del staff.
/// `rest` — datos del silencio.
pub fn render_rest(
    painter: &egui::Painter,
    x: f32,
    staff_origin_y: f32,
    rest: &Rest,
    line_spacing: f32,
    color: egui::Color32,
) {
    let glyph = rest_glyph(rest.figure);
    let font_size = line_spacing * 2.8;
    let font_id = egui::FontId::new(font_size, egui::FontFamily::Name("Leland".into()));

    // Position: rests sit on the middle line (or as specified)
    let y = if rest.measure {
        // Whole-measure rest: centered on middle line
        staff_origin_y
    } else {
        // Positioned rest: middle line offset by step
        staff_origin_y
    };

    painter.text(
        egui::Pos2::new(x, y),
        egui::Align2::CENTER_CENTER,
        glyph.to_string(),
        font_id,
        color,
    );

    // Dots for dotted rests
    if rest.dotted > 0 {
        let half_spacing = line_spacing / 2.0;
        let dot_radius = line_spacing * 0.21; // ~2.5pt
        let dot_spacing = line_spacing * 0.35;
        for d in 0..rest.dotted {
            let dot_x = x + font_size * 0.35 + d as f32 * dot_spacing;
            // Dots go in the space above the rest
            let dot_y = y - half_spacing;
            painter.circle_filled(egui::Pos2::new(dot_x, dot_y), dot_radius, color);
        }
    }
}

/// Glifo SMuFL para un silencio según la figura rítmica.
fn rest_glyph(figure: NoteFigure) -> char {
    match figure {
        NoteFigure::Breve => '\u{E4E2}',               // breve rest
        NoteFigure::Whole => '\u{E4E3}',               // whole rest
        NoteFigure::Half => '\u{E4E4}',                // half rest
        NoteFigure::Quarter => '\u{E4E5}',             // quarter rest
        NoteFigure::Eighth => '\u{E4E6}',              // eighth rest
        NoteFigure::Sixteenth => '\u{E4E7}',           // 16th rest
        NoteFigure::ThirtySecond => '\u{E4E8}',        // 32nd rest
        NoteFigure::SixtyFourth => '\u{E4E9}',         // 64th rest
        NoteFigure::HundredTwentyEighth => '\u{E4EA}', // 128th rest
    }
}

use super::constants::STAFF_LINE_COUNT;
use crate::notation::{BarStyle, Ending};
use eframe::egui;

/// Dibuja las 5 líneas de un pentagrama en el painter.
///
/// `top_left` es la esquina superior izquierda de la primera línea.
/// `width` es el ancho total (incluye todas las medidas).
pub fn render_staff_lines(
    painter: &egui::Painter,
    top_left: egui::Pos2,
    width: f32,
    line_spacing: f32,
    color: egui::Color32,
) {
    let stroke = egui::Stroke::new(1.0, color);
    for i in 0..STAFF_LINE_COUNT {
        let y = top_left.y + i as f32 * line_spacing;
        painter.line_segment(
            [
                egui::Pos2::new(top_left.x, y),
                egui::Pos2::new(top_left.x + width, y),
            ],
            stroke,
        );
    }
}

/// Retorna la altura total del pentagrama (distancia entre la primera y última línea).
pub fn staff_total_height(line_spacing: f32) -> f32 {
    line_spacing * (STAFF_LINE_COUNT as f32 - 1.0)
}

/// Dibuja una barra de compás con el estilo especificado.
pub fn render_bar_line(
    painter: &egui::Painter,
    x: f32,
    staff_top_y: f32,
    line_spacing: f32,
    style: BarStyle,
    color: egui::Color32,
) {
    let staff_h = line_spacing * (STAFF_LINE_COUNT as f32 - 1.0);
    let thin = 1.0;
    let thick = 3.0;
    let gap = 2.0;

    match style {
        BarStyle::Regular | BarStyle::Short => {
            let stroke = egui::Stroke::new(thin, color);
            painter.line_segment(
                [
                    egui::Pos2::new(x, staff_top_y),
                    egui::Pos2::new(x, staff_top_y + staff_h),
                ],
                stroke,
            );
        }
        BarStyle::Heavy => {
            let stroke = egui::Stroke::new(thick, color);
            painter.line_segment(
                [
                    egui::Pos2::new(x, staff_top_y),
                    egui::Pos2::new(x, staff_top_y + staff_h),
                ],
                stroke,
            );
        }
        BarStyle::LightLight => {
            // Two thin lines
            let stroke = egui::Stroke::new(thin, color);
            painter.line_segment(
                [
                    egui::Pos2::new(x, staff_top_y),
                    egui::Pos2::new(x, staff_top_y + staff_h),
                ],
                stroke,
            );
            painter.line_segment(
                [
                    egui::Pos2::new(x + gap + thin, staff_top_y),
                    egui::Pos2::new(x + gap + thin, staff_top_y + staff_h),
                ],
                stroke,
            );
        }
        BarStyle::LightHeavy => {
            // Thin then thick (final barline)
            let thin_stroke = egui::Stroke::new(thin, color);
            painter.line_segment(
                [
                    egui::Pos2::new(x, staff_top_y),
                    egui::Pos2::new(x, staff_top_y + staff_h),
                ],
                thin_stroke,
            );
            let thick_stroke = egui::Stroke::new(thick, color);
            painter.line_segment(
                [
                    egui::Pos2::new(x + gap + thin, staff_top_y),
                    egui::Pos2::new(x + gap + thin, staff_top_y + staff_h),
                ],
                thick_stroke,
            );
        }
        BarStyle::HeavyLight => {
            // Thick then thin (reverse final)
            let thick_stroke = egui::Stroke::new(thick, color);
            painter.line_segment(
                [
                    egui::Pos2::new(x, staff_top_y),
                    egui::Pos2::new(x, staff_top_y + staff_h),
                ],
                thick_stroke,
            );
            let thin_stroke = egui::Stroke::new(thin, color);
            painter.line_segment(
                [
                    egui::Pos2::new(x + gap + thick, staff_top_y),
                    egui::Pos2::new(x + gap + thick, staff_top_y + staff_h),
                ],
                thin_stroke,
            );
        }
        BarStyle::HeavyHeavy => {
            // Two thick lines
            let stroke = egui::Stroke::new(thick, color);
            painter.line_segment(
                [
                    egui::Pos2::new(x, staff_top_y),
                    egui::Pos2::new(x, staff_top_y + staff_h),
                ],
                stroke,
            );
            painter.line_segment(
                [
                    egui::Pos2::new(x + gap + thick, staff_top_y),
                    egui::Pos2::new(x + gap + thick, staff_top_y + staff_h),
                ],
                stroke,
            );
        }
        BarStyle::Dotted | BarStyle::Dashed | BarStyle::Tick => {
            // Draw as thin dashed/dotted line
            let stroke = egui::Stroke::new(thin, color);
            let dash_len = if matches!(style, BarStyle::Dotted) {
                line_spacing * 0.15
            } else if matches!(style, BarStyle::Tick) {
                line_spacing * 0.5
            } else {
                line_spacing * 0.3
            };
            let mut y = staff_top_y;
            while y < staff_top_y + staff_h {
                let end_y = (y + dash_len).min(staff_top_y + staff_h);
                painter.line_segment([egui::Pos2::new(x, y), egui::Pos2::new(x, end_y)], stroke);
                y += dash_len * 2.0;
            }
        }
        BarStyle::None => {
            // No barline (invisible)
        }
    }
}

/// Dibuja los puntos de repetición (dos puntos en espacios 2 y 3 del staff).
pub fn render_repeat_dots(
    painter: &egui::Painter,
    x: f32,
    staff_top_y: f32,
    line_spacing: f32,
    color: egui::Color32,
) {
    let dot_radius = line_spacing * 0.18;
    let half_spacing = line_spacing / 2.0;
    // Space 2: between line 2 and 3 = staff_top_y + 1.5 * line_spacing
    let space2_y = staff_top_y + line_spacing * 1.5;
    // Space 3: between line 3 and 4 = staff_top_y + 2.5 * line_spacing
    let space3_y = staff_top_y + line_spacing * 2.5;

    // Dots are slightly offset from barline
    let dot_x = x - half_spacing;
    painter.circle_filled(egui::Pos2::new(dot_x, space2_y), dot_radius, color);
    painter.circle_filled(egui::Pos2::new(dot_x, space3_y), dot_radius, color);
}

/// Dibuja el corchete de casilla de repetición (1ra/2da vez).
pub fn render_ending_bracket(
    painter: &egui::Painter,
    start_x: f32,
    end_x: f32,
    staff_top_y: f32,
    line_spacing: f32,
    ending: &Ending,
    color: egui::Color32,
) {
    let bracket_y = staff_top_y - line_spacing * 1.2;
    let stroke = egui::Stroke::new(1.5, color);

    // Horizontal line
    painter.line_segment(
        [
            egui::Pos2::new(start_x, bracket_y),
            egui::Pos2::new(end_x, bracket_y),
        ],
        stroke,
    );

    // Left descender
    painter.line_segment(
        [
            egui::Pos2::new(start_x, bracket_y),
            egui::Pos2::new(start_x, bracket_y + line_spacing * 0.8),
        ],
        stroke,
    );

    // Right descender (only if last ending in group)
    painter.line_segment(
        [
            egui::Pos2::new(end_x, bracket_y),
            egui::Pos2::new(end_x, bracket_y + line_spacing * 0.8),
        ],
        stroke,
    );

    // Number above
    let font_size = line_spacing * 1.0;
    painter.text(
        egui::Pos2::new(start_x + line_spacing * 0.5, bracket_y - line_spacing * 0.3),
        egui::Align2::LEFT_BOTTOM,
        &ending.number,
        egui::FontId::new(font_size, egui::FontFamily::Proportional),
        color,
    );
}

use crate::notation::{
    HarmonicKind, NoteFigure, SlideKind, TabElement, TabMeasure, TabNote, TabTechnique,
};
use eframe::egui;

// ────────────────────────────────────────────────────────────
//  Tablature staff rendering
// ────────────────────────────────────────────────────────────

/// Dibuja las líneas de un pentagrama de tablatura (N cuerdas).
pub fn render_tab_staff_lines(
    painter: &egui::Painter,
    top_left: egui::Pos2,
    width: f32,
    line_spacing: f32,
    string_count: u8,
    color: egui::Color32,
) {
    let stroke = egui::Stroke::new(1.0, color);
    for i in 0..string_count {
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

/// Altura total del pentagrama de tablatura.
pub fn tab_staff_height(line_spacing: f32, string_count: u8) -> f32 {
    line_spacing * (string_count as f32 - 1.0)
}

/// Renderiza los elementos de un compás de tablatura.
pub fn render_tab_measure(
    painter: &egui::Painter,
    measure: &TabMeasure,
    staff_top_y: f32,
    line_spacing: f32,
    string_count: u8,
    x: f32,
    width: f32,
    color: egui::Color32,
) {
    let elements = &measure.elements;
    if elements.is_empty() {
        return;
    }

    let count = elements.len() as f32;
    let spacing = width / count;

    for (i, elem) in elements.iter().enumerate() {
        let elem_x = x + spacing * (i as f32 + 0.5);

        match elem {
            TabElement::TabNote(note) => {
                render_fret_number(
                    painter,
                    elem_x,
                    staff_top_y,
                    note,
                    line_spacing,
                    string_count,
                    color,
                );
                // Render techniques
                for tech in &note.technique {
                    render_technique(
                        painter,
                        elem_x,
                        staff_top_y,
                        note,
                        tech,
                        line_spacing,
                        string_count,
                        color,
                    );
                }
            }
            TabElement::TabRest(rest) => {
                // Draw rest symbol centered on middle strings
                let rest_y = staff_top_y + tab_staff_height(line_spacing, string_count) / 2.0;
                let glyph = rest_glyph(rest.figure);
                let font_size = line_spacing * 3.0;
                painter.text(
                    egui::Pos2::new(elem_x, rest_y),
                    egui::Align2::CENTER_CENTER,
                    glyph.to_string(),
                    egui::FontId::new(font_size, egui::FontFamily::Name("Leland".into())),
                    color,
                );
            }
            TabElement::TabChord { notes } => {
                for note in notes {
                    render_fret_number(
                        painter,
                        elem_x,
                        staff_top_y,
                        note,
                        line_spacing,
                        string_count,
                        color,
                    );
                    for tech in &note.technique {
                        render_technique(
                            painter,
                            elem_x,
                            staff_top_y,
                            note,
                            tech,
                            line_spacing,
                            string_count,
                            color,
                        );
                    }
                }
            }
        }
    }
}

/// Dibuja el número de traste en la cuerda correspondiente.
fn render_fret_number(
    painter: &egui::Painter,
    x: f32,
    staff_top_y: f32,
    note: &TabNote,
    line_spacing: f32,
    string_count: u8,
    color: egui::Color32,
) {
    let string_y = staff_top_y + (string_count as f32 - note.string as f32) * line_spacing;
    let font_size = line_spacing * 1.2;
    let font_id = egui::FontId::new(font_size, egui::FontFamily::Proportional);

    // Check for ghost note (parenthesized)
    let has_ghost = note
        .technique
        .iter()
        .any(|t| matches!(t, TabTechnique::GhostNote));
    let has_dead = note
        .technique
        .iter()
        .any(|t| matches!(t, TabTechnique::DeadNote));

    if has_dead {
        painter.text(
            egui::Pos2::new(x, string_y),
            egui::Align2::CENTER_CENTER,
            "X",
            font_id,
            color,
        );
    } else if has_ghost {
        painter.text(
            egui::Pos2::new(x, string_y),
            egui::Align2::CENTER_CENTER,
            format!("({})", note.fret),
            font_id,
            color,
        );
    } else {
        // Try SMuFL circled-fret glyphs for 0-14
        if note.fret <= 14 {
            let circled = char::from_u32(0xEBD0 + note.fret as u32).unwrap_or('0');
            let smufl_font =
                egui::FontId::new(line_spacing * 2.0, egui::FontFamily::Name("Leland".into()));
            painter.text(
                egui::Pos2::new(x, string_y),
                egui::Align2::CENTER_CENTER,
                circled.to_string(),
                smufl_font,
                color,
            );
        } else {
            painter.text(
                egui::Pos2::new(x, string_y),
                egui::Align2::CENTER_CENTER,
                note.fret.to_string(),
                font_id,
                color,
            );
        }
    }
}

/// Renderiza técnicas de tablatura.
fn render_technique(
    painter: &egui::Painter,
    x: f32,
    staff_top_y: f32,
    note: &TabNote,
    tech: &TabTechnique,
    line_spacing: f32,
    string_count: u8,
    color: egui::Color32,
) {
    let string_y = staff_top_y + (string_count as f32 - note.string as f32) * line_spacing;
    let small_font = egui::FontId::new(line_spacing * 0.8, egui::FontFamily::Proportional);

    match tech {
        TabTechnique::HammerOn => {
            painter.text(
                egui::Pos2::new(x + line_spacing * 0.6, string_y - line_spacing * 0.6),
                egui::Align2::LEFT_BOTTOM,
                "H",
                small_font,
                color,
            );
        }
        TabTechnique::PullOff => {
            painter.text(
                egui::Pos2::new(x + line_spacing * 0.6, string_y - line_spacing * 0.6),
                egui::Align2::LEFT_BOTTOM,
                "P",
                small_font,
                color,
            );
        }
        TabTechnique::Bend {
            alter,
            pre_bend: _,
            release: _,
        } => {
            let amount = if *alter >= 2.0 {
                "full"
            } else if *alter >= 1.0 {
                "1/2"
            } else {
                "1/4"
            };
            painter.text(
                egui::Pos2::new(x + line_spacing * 0.3, string_y - line_spacing * 1.2),
                egui::Align2::LEFT_BOTTOM,
                amount,
                small_font.clone(),
                color,
            );
            // Curved arrow
            let stroke = egui::Stroke::new(line_spacing * 0.06, color);
            let arrow_y = string_y - line_spacing * 0.8;
            let _arrow_width = line_spacing * 1.0;
            // Simple arc: three segments
            painter.line_segment(
                [
                    egui::Pos2::new(x + line_spacing * 0.2, arrow_y),
                    egui::Pos2::new(x + line_spacing * 0.5, arrow_y - line_spacing * 0.4),
                ],
                stroke,
            );
            painter.line_segment(
                [
                    egui::Pos2::new(x + line_spacing * 0.5, arrow_y - line_spacing * 0.4),
                    egui::Pos2::new(x + line_spacing * 0.8, arrow_y),
                ],
                stroke,
            );
            // Arrow tip
            painter.line_segment(
                [
                    egui::Pos2::new(x + line_spacing * 0.8, arrow_y),
                    egui::Pos2::new(x + line_spacing * 0.65, arrow_y - line_spacing * 0.15),
                ],
                stroke,
            );
        }
        TabTechnique::Slide { kind } => {
            let stroke = egui::Stroke::new(line_spacing * 0.08, color);
            match kind {
                SlideKind::Into => {
                    // Short line from left approaching the note
                    painter.line_segment(
                        [
                            egui::Pos2::new(x - line_spacing * 1.2, string_y),
                            egui::Pos2::new(x - line_spacing * 0.2, string_y),
                        ],
                        stroke,
                    );
                }
                SlideKind::OutOf => {
                    painter.line_segment(
                        [
                            egui::Pos2::new(x + line_spacing * 0.2, string_y),
                            egui::Pos2::new(x + line_spacing * 1.2, string_y),
                        ],
                        stroke,
                    );
                }
                SlideKind::Shift => {
                    // Rendered as connection between two notes (handled by cross-note rendering)
                    painter.line_segment(
                        [
                            egui::Pos2::new(x + line_spacing * 0.2, string_y),
                            egui::Pos2::new(x + line_spacing * 1.2, string_y),
                        ],
                        stroke,
                    );
                }
            }
        }
        TabTechnique::Vibrato | TabTechnique::WideVibrato => {
            // Wavy line above
            let vib_y = string_y - line_spacing * 0.8;
            let stroke = egui::Stroke::new(line_spacing * 0.06, color);
            let wave_amp = if matches!(tech, TabTechnique::WideVibrato) {
                line_spacing * 0.25
            } else {
                line_spacing * 0.15
            };
            let mut cx = x;
            while cx < x + line_spacing * 1.5 {
                painter.line_segment(
                    [
                        egui::Pos2::new(cx, vib_y - wave_amp),
                        egui::Pos2::new(cx + line_spacing * 0.15, vib_y + wave_amp),
                    ],
                    stroke,
                );
                painter.line_segment(
                    [
                        egui::Pos2::new(cx + line_spacing * 0.15, vib_y + wave_amp),
                        egui::Pos2::new(cx + line_spacing * 0.3, vib_y - wave_amp),
                    ],
                    stroke,
                );
                cx += line_spacing * 0.3;
            }
        }
        TabTechnique::Tap => {
            painter.text(
                egui::Pos2::new(x + line_spacing * 0.6, string_y - line_spacing * 0.6),
                egui::Align2::LEFT_BOTTOM,
                "T",
                small_font,
                color,
            );
        }
        TabTechnique::Harmonic { kind } => {
            let harm_text = match kind {
                HarmonicKind::Natural => "N.H.",
                HarmonicKind::Artificial => "A.H.",
                HarmonicKind::Pinch => "P.H.",
                HarmonicKind::Tap => "T.H.",
                HarmonicKind::Semi => "S.H.",
            };
            painter.text(
                egui::Pos2::new(x + line_spacing * 0.3, string_y - line_spacing * 1.0),
                egui::Align2::LEFT_BOTTOM,
                harm_text,
                small_font,
                color,
            );
            // Diamond brackets <> for natural harmonics
            if matches!(kind, HarmonicKind::Natural) {
                let bracket_font =
                    egui::FontId::new(line_spacing * 2.0, egui::FontFamily::Name("Leland".into()));
                painter.text(
                    egui::Pos2::new(x - line_spacing * 0.6, string_y),
                    egui::Align2::RIGHT_CENTER,
                    '\u{EAB0}'.to_string(),
                    bracket_font.clone(),
                    color,
                );
                painter.text(
                    egui::Pos2::new(x + line_spacing * 0.6, string_y),
                    egui::Align2::LEFT_CENTER,
                    '\u{EAB1}'.to_string(),
                    bracket_font,
                    color,
                );
            }
        }
        TabTechnique::PalmMute => {
            painter.text(
                egui::Pos2::new(x, string_y - line_spacing * 1.0),
                egui::Align2::CENTER_BOTTOM,
                "P.M.",
                small_font,
                color,
            );
        }
        TabTechnique::LetRing => {
            let stroke = egui::Stroke::new(line_spacing * 0.04, color);
            // Small dashed line extending right
            let mut cx = x + line_spacing * 0.5;
            while cx < x + line_spacing * 1.2 {
                painter.line_segment(
                    [
                        egui::Pos2::new(cx, string_y),
                        egui::Pos2::new(
                            (cx + line_spacing * 0.15).min(x + line_spacing * 1.2),
                            string_y,
                        ),
                    ],
                    stroke,
                );
                cx += line_spacing * 0.3;
            }
        }
        TabTechnique::Trill { fret } => {
            painter.text(
                egui::Pos2::new(x + line_spacing * 0.5, string_y - line_spacing * 0.6),
                egui::Align2::LEFT_BOTTOM,
                format!("tr{}", fret),
                small_font,
                color,
            );
        }
        TabTechnique::GhostNote | TabTechnique::DeadNote => {
            // Already rendered in fret number
        }
    }
}

fn rest_glyph(figure: NoteFigure) -> char {
    match figure {
        NoteFigure::Breve => '\u{E4E2}',
        NoteFigure::Whole => '\u{E4E3}',
        NoteFigure::Half => '\u{E4E4}',
        NoteFigure::Quarter => '\u{E4E5}',
        NoteFigure::Eighth => '\u{E4E6}',
        NoteFigure::Sixteenth => '\u{E4E7}',
        NoteFigure::ThirtySecond => '\u{E4E8}',
        NoteFigure::SixtyFourth => '\u{E4E9}',
        NoteFigure::HundredTwentyEighth => '\u{E4EA}',
    }
}

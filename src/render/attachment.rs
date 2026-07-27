use crate::notation::{
    Arpeggiate, Articulation, DynamicMark, Fermata, NoteAttachment, Ornament, Placement, Tremolo,
};
use eframe::egui;
// ────────────────────────────────────────────────────────────
//  Main dispatch
// ────────────────────────────────────────────────────────────

/// Renderiza todas las marcas adjuntas a una nota.
pub fn render_attachments(
    painter: &egui::Painter,
    note_x: f32,
    note_y: f32,
    _staff_origin_y: f32,
    stem_tip_y: Option<f32>,
    stem_direction_up: bool,
    attachments: &NoteAttachment,
    line_spacing: f32,
    color: egui::Color32,
) {
    let placement_y = |placement: Placement| -> f32 {
        match placement {
            Placement::Above => {
                if let Some(tip_y) = stem_tip_y {
                    if stem_direction_up {
                        tip_y - line_spacing * 0.5
                    } else {
                        note_y - line_spacing * 2.5
                    }
                } else {
                    note_y - line_spacing * 2.5
                }
            }
            Placement::Below => {
                if let Some(tip_y) = stem_tip_y {
                    if !stem_direction_up {
                        tip_y + line_spacing * 0.5
                    } else {
                        note_y + line_spacing * 2.5
                    }
                } else {
                    note_y + line_spacing * 2.5
                }
            }
        }
    };

    // Articulations
    if !attachments.articulations.is_empty() {
        render_articulations(
            painter,
            note_x,
            note_y,
            &attachments.articulations,
            line_spacing,
            color,
        );
    }

    // Ornaments
    if !attachments.ornaments.is_empty() {
        render_ornaments(
            painter,
            note_x,
            note_y,
            &attachments.ornaments,
            line_spacing,
            color,
        );
    }

    // Fermata
    if let Some(ref fermata) = attachments.fermata {
        render_fermata(
            painter,
            note_x,
            placement_y(fermata.placement),
            fermata,
            line_spacing,
            color,
        );
    }

    // Dynamics
    if let Some(ref dyn_mark) = attachments.dynamics {
        render_dynamics(painter, note_x, note_y, dyn_mark, line_spacing, color);
    }

    // Tremolo (on a single note)
    if let Some(ref tremolo) = attachments.tremolo {
        render_single_tremolo(painter, note_x, note_y, tremolo, line_spacing, color);
    }

    // Arpeggiate
    if let Some(ref arp) = attachments.arpeggiate {
        render_arpeggiate_before(painter, note_x, note_y, arp, line_spacing, color);
    }
}

// ────────────────────────────────────────────────────────────
//  Articulations
// ────────────────────────────────────────────────────────────

fn render_articulations(
    painter: &egui::Painter,
    note_x: f32,
    note_y: f32,
    articulations: &[Articulation],
    line_spacing: f32,
    color: egui::Color32,
) {
    let font_size = line_spacing * 2.5;
    let font_id = egui::FontId::new(font_size, egui::FontFamily::Name("Leland".into()));

    // Stack articulations vertically above the notehead
    let mut y_offset = note_y - line_spacing * 1.8;

    for art in articulations {
        match art {
            Articulation::DetachedLegato => {
                // Composite: draw staccato dot + tenuto line
                let dot = '\u{E4A2}';
                let tenuto = '\u{E4A4}';
                // Staccato dot closer to note
                painter.text(
                    egui::Pos2::new(note_x, y_offset),
                    egui::Align2::CENTER_CENTER,
                    dot.to_string(),
                    font_id.clone(),
                    color,
                );
                y_offset -= line_spacing * 0.6;
                painter.text(
                    egui::Pos2::new(note_x, y_offset),
                    egui::Align2::CENTER_CENTER,
                    tenuto.to_string(),
                    font_id.clone(),
                    color,
                );
                y_offset -= line_spacing * 0.8;
            }
            Articulation::SoftAccent => {
                // Composite: two staccato-like marks forming <>
                let s1 = '\u{E4A2}';
                let s2 = '\u{E4A3}';
                painter.text(
                    egui::Pos2::new(note_x - line_spacing * 0.3, y_offset),
                    egui::Align2::CENTER_CENTER,
                    s1.to_string(),
                    font_id.clone(),
                    color,
                );
                painter.text(
                    egui::Pos2::new(note_x + line_spacing * 0.3, y_offset),
                    egui::Align2::CENTER_CENTER,
                    s2.to_string(),
                    font_id.clone(),
                    color,
                );
                y_offset -= line_spacing * 0.8;
            }
            _ => {
                if let Some(glyph) = art.glyph() {
                    painter.text(
                        egui::Pos2::new(note_x, y_offset),
                        egui::Align2::CENTER_CENTER,
                        glyph.to_string(),
                        font_id.clone(),
                        color,
                    );
                    y_offset -= line_spacing * 0.7;
                }
            }
        }
    }
}

// ────────────────────────────────────────────────────────────
//  Ornaments
// ────────────────────────────────────────────────────────────

fn render_ornaments(
    painter: &egui::Painter,
    note_x: f32,
    note_y: f32,
    ornaments: &[Ornament],
    line_spacing: f32,
    color: egui::Color32,
) {
    let font_size = line_spacing * 2.0;
    let font_id = egui::FontId::new(font_size, egui::FontFamily::Name("Leland".into()));

    let mut y_offset = note_y - line_spacing * 3.0;

    for orn in ornaments {
        match orn {
            Ornament::Tremolo { marks } => {
                // Tremolo bars: draw diagonal strokes on the stem (handled in stem rendering)
                // Here we draw them above the note for single-note tremolo
                for i in 0..*marks {
                    let x = note_x + i as f32 * line_spacing * 0.15;
                    painter.line_segment(
                        [
                            egui::Pos2::new(x, y_offset),
                            egui::Pos2::new(x + line_spacing * 0.5, y_offset + line_spacing * 0.4),
                        ],
                        egui::Stroke::new(line_spacing * 0.12, color),
                    );
                }
                y_offset -= line_spacing * 0.8;
            }
            _ => {
                if let Some(glyph) = orn.glyph() {
                    painter.text(
                        egui::Pos2::new(note_x, y_offset),
                        egui::Align2::CENTER_CENTER,
                        glyph.to_string(),
                        font_id.clone(),
                        color,
                    );
                    y_offset -= line_spacing * 0.7;
                }
            }
        }
    }
}

// ────────────────────────────────────────────────────────────
//  Fermata
// ────────────────────────────────────────────────────────────

fn render_fermata(
    painter: &egui::Painter,
    x: f32,
    y: f32,
    fermata: &Fermata,
    line_spacing: f32,
    color: egui::Color32,
) {
    let glyph = fermata_glyph(fermata.shape, fermata.upright);
    let font_size = line_spacing * 3.0;
    let font_id = egui::FontId::new(font_size, egui::FontFamily::Name("Leland".into()));

    painter.text(
        egui::Pos2::new(x, y),
        egui::Align2::CENTER_CENTER,
        glyph.to_string(),
        font_id,
        color,
    );
}

fn fermata_glyph(shape: crate::notation::FermataShape, upright: bool) -> char {
    match (shape, upright) {
        (crate::notation::FermataShape::Normal, true) => '\u{E4C0}',
        (crate::notation::FermataShape::Normal, false) => '\u{E4C1}',
        (crate::notation::FermataShape::Angled, true) => '\u{E4C2}',
        (crate::notation::FermataShape::Angled, false) => '\u{E4C3}',
        (crate::notation::FermataShape::Square, true) => '\u{E4C4}',
        (crate::notation::FermataShape::Square, false) => '\u{E4C5}',
        _ => '\u{E4C0}', // fallback
    }
}

// ────────────────────────────────────────────────────────────
//  Dynamics
// ────────────────────────────────────────────────────────────

fn render_dynamics(
    painter: &egui::Painter,
    note_x: f32,
    note_y: f32,
    dynamic: &DynamicMark,
    line_spacing: f32,
    color: egui::Color32,
) {
    let text = dynamic_text(dynamic);
    let font_size = line_spacing * 1.6;
    let font_id = egui::FontId::new(font_size, egui::FontFamily::Proportional);

    painter.text(
        egui::Pos2::new(note_x, note_y + line_spacing * 2.5),
        egui::Align2::CENTER_TOP,
        text,
        font_id,
        color,
    );
}

fn dynamic_text(dm: &DynamicMark) -> String {
    use DynamicMark::*;
    match dm {
        PPPP => "pppp".into(),
        PPP => "ppp".into(),
        PP => "pp".into(),
        P => "p".into(),
        MP => "mp".into(),
        MF => "mf".into(),
        F => "f".into(),
        FF => "ff".into(),
        FFF => "fff".into(),
        FFFF => "ffff".into(),
        SF => "sf".into(),
        SFZ => "sfz".into(),
        SFFZ => "sffz".into(),
        SFP => "sfp".into(),
        SFPP => "sfpp".into(),
        FP => "fp".into(),
        RF => "rf".into(),
        RFZ => "rfz".into(),
        FZ => "fz".into(),
        N => "n".into(),
        PF => "pf".into(),
        Other(s) => s.clone(),
    }
}

// ────────────────────────────────────────────────────────────
//  Tremolo (single note)
// ────────────────────────────────────────────────────────────

fn render_single_tremolo(
    painter: &egui::Painter,
    x: f32,
    y: f32,
    tremolo: &Tremolo,
    line_spacing: f32,
    color: egui::Color32,
) {
    // For single-note tremolo, draw short diagonal strokes above the note
    for i in 0..tremolo.marks {
        let offset = i as f32 * line_spacing * 0.2;
        let top_y = y - line_spacing * 2.5 - offset;
        painter.line_segment(
            [
                egui::Pos2::new(x - line_spacing * 0.2, top_y),
                egui::Pos2::new(x + line_spacing * 0.3, top_y + line_spacing * 0.4),
            ],
            egui::Stroke::new(line_spacing * 0.1, color),
        );
    }
}

// ────────────────────────────────────────────────────────────
//  Arpeggiate
// ────────────────────────────────────────────────────────────

fn render_arpeggiate_before(
    painter: &egui::Painter,
    x: f32,
    y: f32,
    _arp: &Arpeggiate,
    line_spacing: f32,
    color: egui::Color32,
) {
    // Vertical wavy line to the left of the chord
    let start_y = y - line_spacing * 2.0;
    let end_y = y + line_spacing * 2.0;
    let arp_x = x - line_spacing * 1.5;

    let stroke = egui::Stroke::new(line_spacing * 0.08, color);
    let segments = 8;
    for i in 0..segments {
        let t0 = i as f32 / segments as f32;
        let t1 = (i + 1) as f32 / segments as f32;
        let y0 = start_y + (end_y - start_y) * t0;
        let y1 = start_y + (end_y - start_y) * t1;
        let wave = line_spacing * 0.3 * ((i % 2) as f32 * 2.0 - 1.0);
        painter.line_segment(
            [
                egui::Pos2::new(arp_x + wave * 0.3, y0),
                egui::Pos2::new(arp_x - wave * 0.3, y1),
            ],
            stroke,
        );
    }
}

// ────────────────────────────────────────────────────────────
//  Slurs and Ties (cross-note connectors)
// ────────────────────────────────────────────────────────────

/// Dibuja una ligadura de expresión entre dos puntos.
pub fn render_slur_curve(
    painter: &egui::Painter,
    start_x: f32,
    start_y: f32,
    end_x: f32,
    end_y: f32,
    placement: Placement,
    line_spacing: f32,
    color: egui::Color32,
) {
    let dx = end_x - start_x;
    let dy = end_y - start_y;
    let dist = (dx * dx + dy * dy).sqrt().max(line_spacing);

    // Arc height proportional to distance
    let arc_height = dist * 0.3;
    let arc_dir: f32 = match placement {
        Placement::Above => -1.0,
        Placement::Below => 1.0,
    };

    // Control points for cubic Bezier
    let cp1x = start_x + dist * 0.25;
    let cp1y = start_y + arc_dir * arc_height;
    let cp2x = start_x + dist * 0.75;
    let cp2y = end_y + arc_dir * arc_height;

    let stroke = egui::Stroke::new(line_spacing * 0.1, color);
    let steps = 20;

    let mut prev = egui::Pos2::new(start_x, start_y);
    for i in 1..=steps {
        let t = i as f32 / steps as f32;
        let t2 = t * t;
        let t3 = t2 * t;
        let mt = 1.0 - t;
        let mt2 = mt * mt;
        let mt3 = mt2 * mt;

        let px = mt3 * start_x + 3.0 * mt2 * t * cp1x + 3.0 * mt * t2 * cp2x + t3 * end_x;
        let py = mt3 * start_y + 3.0 * mt2 * t * cp1y + 3.0 * mt * t2 * cp2y + t3 * end_y;

        let curr = egui::Pos2::new(px, py);
        painter.line_segment([prev, curr], stroke);
        prev = curr;
    }
}

/// Dibuja una ligadura de prolongación (tie) entre dos puntos.
pub fn render_tie_curve(
    painter: &egui::Painter,
    start_x: f32,
    start_y: f32,
    end_x: f32,
    end_y: f32,
    placement: Placement,
    line_spacing: f32,
    color: egui::Color32,
) {
    // Ties are flatter than slurs, with SMuFL end caps
    let dx = end_x - start_x;
    let dist = dx.max(line_spacing);

    let arc_height = dist * 0.15;
    let arc_dir: f32 = match placement {
        Placement::Above => -1.0,
        Placement::Below => 1.0,
    };

    let avg_y = (start_y + end_y) / 2.0;
    // Control points spread wider and lower than the peak for a flatter top
    let cp1x = start_x + dist * 0.25;
    let cp1y = avg_y + arc_dir * arc_height * 0.65;
    let cp2x = start_x + dist * 0.75;
    let cp2y = avg_y + arc_dir * arc_height * 0.65;

    let stroke = egui::Stroke::new(line_spacing * 0.12, color);
    let steps = 24;

    let mut prev = egui::Pos2::new(start_x, start_y);
    for i in 1..=steps {
        let t = i as f32 / steps as f32;
        let t2 = t * t;
        let t3 = t2 * t;
        let mt = 1.0 - t;
        let mt2 = mt * mt;
        let mt3 = mt2 * mt;

        let px = mt3 * start_x + 3.0 * mt2 * t * cp1x + 3.0 * mt * t2 * cp2x + t3 * end_x;
        let py = mt3 * start_y + 3.0 * mt2 * t * cp1y + 3.0 * mt * t2 * cp2y + t3 * end_y;

        let curr = egui::Pos2::new(px, py);
        painter.line_segment([prev, curr], stroke);
        prev = curr;
    }
}

/// Dibuja una línea de glissando entre dos puntos.
pub fn render_glissando_line(
    painter: &egui::Painter,
    start_x: f32,
    start_y: f32,
    end_x: f32,
    end_y: f32,
    line_spacing: f32,
    color: egui::Color32,
) {
    let stroke = egui::Stroke::new(line_spacing * 0.08, color);
    painter.line_segment(
        [
            egui::Pos2::new(start_x, start_y),
            egui::Pos2::new(end_x, end_y),
        ],
        stroke,
    );

    // "gliss." text
    let mid_x = (start_x + end_x) / 2.0;
    let mid_y = (start_y + end_y) / 2.0 - line_spacing * 0.8;
    let font_id = egui::FontId::new(line_spacing * 0.7, egui::FontFamily::Proportional);
    painter.text(
        egui::Pos2::new(mid_x, mid_y),
        egui::Align2::CENTER_BOTTOM,
        "gliss.",
        font_id,
        color,
    );
}

/// Renderiza un corchete de tresillo/quintillo con su número.
pub fn render_tuplet(
    painter: &egui::Painter,
    start_x: f32,
    end_x: f32,
    y: f32, // staff position y for bracket
    tuplet: &crate::notation::Tuplet,
    line_spacing: f32,
    color: egui::Color32,
) {
    let placement_dir: f32 = match tuplet.placement {
        crate::notation::Placement::Above => -1.0,
        crate::notation::Placement::Below => 1.0,
    };
    let bracket_y = y + placement_dir * line_spacing * 1.2;
    let stroke = egui::Stroke::new(line_spacing * 0.06, color);
    let hook_len = line_spacing * 0.4;

    // Horizontal line
    painter.line_segment(
        [
            egui::Pos2::new(start_x, bracket_y),
            egui::Pos2::new(end_x, bracket_y),
        ],
        stroke,
    );

    // Left hook
    if tuplet.bracket {
        painter.line_segment(
            [
                egui::Pos2::new(start_x, bracket_y),
                egui::Pos2::new(start_x, bracket_y + placement_dir * hook_len),
            ],
            stroke,
        );
        // Right hook
        painter.line_segment(
            [
                egui::Pos2::new(end_x, bracket_y),
                egui::Pos2::new(end_x, bracket_y + placement_dir * hook_len),
            ],
            stroke,
        );
    }

    // Number
    if tuplet.show_number != Some(crate::notation::TupletShow::None) {
        let num_str = tuplet.number.to_string();
        let mid_x = (start_x + end_x) / 2.0;
        let num_y = bracket_y + placement_dir * line_spacing * 0.6;
        let font_id = egui::FontId::new(line_spacing * 0.9, egui::FontFamily::Proportional);
        painter.text(
            egui::Pos2::new(mid_x, num_y),
            egui::Align2::CENTER_BOTTOM,
            num_str,
            font_id,
            color,
        );
    }
}

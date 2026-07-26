use super::beam::compute_beams;
use super::constants::{
    CLEF_WIDTH, DEFAULT_MEASURE_WIDTH, INITIAL_BAR_GAP, STAFF_HEIGHT, STAFF_LINE_COUNT,
    STAFF_LINE_SPACING, TIME_SIG_WIDTH,
};
use super::key_render::render_key_signature;
use super::note::render_dots;
use super::rest_render::render_rest;
use super::stem::render_stem;
use super::stylesheet::ScoreStylesheet;
use super::{
    render_bar_line, render_clef, render_direction, render_lyrics, render_notehead,
    render_staff_lines,
};
use crate::notation::{
    BarStyle, Clef, KeySignature, Measure, MeasureElement, NoteFigure, Score, StemDirection,
    TimeSignatureStyle,
};
use eframe::egui;
/// Estilo visual para el renderizado de partituras.
#[derive(Clone)]
pub struct RenderStyle {
    pub line_spacing: f32,
    pub dark_mode: bool,
    pub sheet: ScoreStylesheet,
}

impl Default for RenderStyle {
    fn default() -> Self {
        Self {
            line_spacing: STAFF_LINE_SPACING,
            dark_mode: false,
            sheet: ScoreStylesheet::builtin_default(),
        }
    }
}

/// Padding vertical entre staves.
const STAFF_PADDING: f32 = STAFF_HEIGHT + STAFF_LINE_SPACING * 6.0;

/// Padding izquierdo desde donde arranca la clave.
const LEFT_MARGIN: f32 = 20.0;

/// Retorna la altura total necesaria para renderizar todos los sistemas.
pub fn score_total_height(score: &Score, line_spacing: f32) -> f32 {
    score
        .systems
        .iter()
        .map(|sys| sys.staves.len() as f32 * (staff_height(line_spacing) + STAFF_PADDING))
        .sum()
}

/// Altura de un solo staff incluyendo su propio padding.
fn staff_height(line_spacing: f32) -> f32 {
    line_spacing * (STAFF_LINE_COUNT as f32 - 1.0)
}

/// Obtiene las configuración de glifo para una clave del stylesheet.
pub(crate) fn get_clef_config(sheet: &ScoreStylesheet, clef: Clef) -> (f32, f32) {
    let v = sheet.clef_variant(clef);
    (v.glyph_scale, v.fine_offset)
}

/// Renderiza los elementos de un compás dentro de una medida.
pub fn render_measure_elements(
    painter: &egui::Painter,
    measure: &Measure,
    staff_origin: egui::Pos2,
    clef: Clef,
    measure_x: f32,
    measure_width: f32,
    style: &RenderStyle,
) {
    let elements = &measure.elements;
    if elements.is_empty() {
        return;
    }

    let line_spacing = style.line_spacing;
    let color = style.sheet.note_color(style.dark_mode);
    let divisions = measure.time_signature.denominator as u32; // simplified: 1/4 = 1 division

    // Compute beam groups
    let beams = compute_beams(elements, divisions);

    // Compute stem direction and beam Y per beam group.
    // The beam Y is the stem tip of the note furthest from the middle line.
    let beam_meta: Vec<(usize, usize, StemDirection, f32)> = beams
        .iter()
        .map(|b| {
            let half = line_spacing / 2.0;
            let mut extreme_sp: i8 = 0;
            let mut extreme_note_y: f32 = staff_origin.y;
            for (ei, elem) in elements.iter().enumerate() {
                if ei < b.start_idx || ei > b.end_idx {
                    continue;
                }
                if let MeasureElement::Note(note) = elem {
                    let sp = note.pitch.staff_position(clef);
                    let ny = staff_origin.y - sp as f32 * half;
                    if sp.abs() > extreme_sp.abs()
                        || (sp.abs() == extreme_sp.abs() && sp < 0)
                    {
                        extreme_sp = sp;
                        extreme_note_y = ny;
                    }
                }
            }
            let dir = if extreme_sp < 0 {
                StemDirection::Up
            } else {
                StemDirection::Down
            };
            let beam_y = match dir {
                StemDirection::Up => extreme_note_y - line_spacing * 3.5,
                StemDirection::Down => extreme_note_y + line_spacing * 3.5,
            };
            (b.start_idx, b.end_idx, dir, beam_y)
        })
        .collect();

    // Count actual rendered items for spacing
    let renderable_count = elements
        .iter()
        .filter(|e| !matches!(e, MeasureElement::Backup(_) | MeasureElement::Forward(_)))
        .count();

    if renderable_count == 0 {
        return;
    }

    let mut render_idx: usize = 0;

    for (ei, elem) in elements.iter().enumerate() {
        match elem {
            MeasureElement::Backup(_) | MeasureElement::Forward(_) => continue,

            MeasureElement::Note(note) => {
                let note_spacing = measure_width / renderable_count as f32;
                let note_x = measure_x + note_spacing * (render_idx as f32 + 0.5);
                let sp = note.pitch.staff_position(clef);
                let note_y = staff_origin.y - sp as f32 * line_spacing / 2.0;
                if let Some(acc) = note.accidental_override {
                    super::note::render_accidental(
                        painter,
                        note_x,
                        note_y,
                        acc,
                        line_spacing,
                        color,
                    );
                } else if note.pitch.accidental != crate::notation::Accidental::Natural {
                    super::note::render_accidental(
                        painter,
                        note_x,
                        note_y,
                        note.pitch.accidental,
                        line_spacing,
                        color,
                    );
                }

                // Notehead
                render_notehead(
                    painter,
                    note_x,
                    staff_origin,
                    sp,
                    note.figure,
                    color,
                    line_spacing,
                );
                let is_beamed = beams.iter().any(|b| ei >= b.start_idx && ei <= b.end_idx);
                // Use the beam group's uniform direction and beam Y when beamed,
                // otherwise auto-compute from staff position.
                let (direction, stem_beam_y) = if is_beamed {
                    let meta = beam_meta
                        .iter()
                        .find(|(s, e, _, _)| ei >= *s && ei <= *e);
                    let dir = meta.map(|(_, _, d, _)| *d).unwrap_or(StemDirection::Up);
                    let by = meta.map(|(_, _, _, y)| *y);
                    (dir, by)
                } else if sp < 0 {
                    (StemDirection::Up, None)
                } else {
                    (StemDirection::Down, None)
                };

                render_stem(
                    painter,
                    note_x,
                    note_y,
                    sp,
                    note.figure,
                    direction,
                    is_beamed,
                    stem_beam_y,
                    line_spacing,
                    color,
                );

                // Dots
                if note.dotted > 0 {
                    render_dots(
                        painter,
                        note_x,
                        note_y,
                        note.dotted,
                        sp,
                        line_spacing,
                        color,
                    );
                }

                // Attachments (articulations, ornaments, fermata, dynamics, etc.)
                if let Some(ref attachments) = note.attachments {
                    let stem_tip_y = if matches!(direction, StemDirection::Up) {
                        Some(note_y - line_spacing * 3.5)
                    } else {
                        Some(note_y + line_spacing * 3.5)
                    };
                    super::attachment_render::render_attachments(
                        painter,
                        note_x,
                        note_y,
                        staff_origin.y,
                        stem_tip_y,
                        matches!(direction, StemDirection::Up),
                        attachments,
                        line_spacing,
                        color,
                    );
                }

                render_idx += 1;
            }

            MeasureElement::Rest(rest) => {
                let note_spacing = measure_width / renderable_count as f32;
                let rest_x = measure_x + note_spacing * (render_idx as f32 + 0.5);
                render_rest(painter, rest_x, staff_origin.y, rest, line_spacing, color);
                render_idx += 1;
            }

            MeasureElement::Chord(notes) => {
                let note_spacing = measure_width / renderable_count as f32;
                let chord_x = measure_x + note_spacing * (render_idx as f32 + 0.5);

                // Find highest and lowest note
                let mut positions: Vec<(i8, &crate::notation::Note)> = notes
                    .iter()
                    .map(|n| (n.pitch.staff_position(clef), n))
                    .collect();
                positions.sort_by_key(|(sp, _)| -sp); // highest first

                // Stem direction for chord
                let avg_sp: f32 = positions.iter().map(|(sp, _)| *sp as f32).sum::<f32>()
                    / positions.len() as f32;
                let direction = if avg_sp < 0.0 {
                    StemDirection::Up
                } else {
                    StemDirection::Down
                };

                let half_spacing = line_spacing / 2.0;

                for (sp, note) in &positions {
                    let note_y = staff_origin.y - *sp as f32 * half_spacing;

                    // Offset seconds: if two notes are adjacent (diff=1), shift one horizontally
                    let offset_x = if positions
                        .iter()
                        .any(|(other_sp, _)| (*other_sp - *sp).abs() == 1)
                    {
                        if *sp % 2 == 0 {
                            line_spacing * 0.8
                        } else {
                            -line_spacing * 0.8
                        }
                    } else {
                        0.0
                    };

                    // Accidental
                    if let Some(acc) = note.accidental_override {
                        super::note::render_accidental(
                            painter,
                            chord_x + offset_x,
                            note_y,
                            acc,
                            line_spacing,
                            color,
                        );
                    }

                    render_notehead(
                        painter,
                        staff_origin.x + offset_x,
                        egui::Pos2::new(staff_origin.x + offset_x, staff_origin.y),
                        *sp,
                        note.figure,
                        color,
                        line_spacing,
                    );
                }

                // Chord stem
                let top_y = staff_origin.y
                    - positions.first().map(|(sp, _)| *sp).unwrap_or(0) as f32 * half_spacing;
                let bottom_y = staff_origin.y
                    - positions.last().map(|(sp, _)| *sp).unwrap_or(0) as f32 * half_spacing;
                let chord_center_y = (top_y + bottom_y) / 2.0;
                let chord_sp = ((staff_origin.y - chord_center_y) / half_spacing) as i8;

                render_stem(
                    painter,
                    chord_x,
                    chord_center_y,
                    chord_sp,
                    positions
                        .first()
                        .map(|(_, n)| n.figure)
                        .unwrap_or(NoteFigure::Quarter),
                    direction,
                    false,
                    None,
                    line_spacing,
                    color,
                );

                // Dots for chord
                if let Some(first_note) = notes.first() {
                    if first_note.dotted > 0 {
                        let first_sp = first_note.pitch.staff_position(clef);
                        let first_y = staff_origin.y - first_sp as f32 * half_spacing;
                        render_dots(
                            painter,
                            chord_x,
                            first_y,
                            first_note.dotted,
                            first_sp,
                            line_spacing,
                            color,
                        );
                    }
                }

                render_idx += 1;
            }

            MeasureElement::MultipleRest(mrest) => {
                let mid_y = staff_origin.y;
                let half_width = measure_width * 0.3;
                let thick_stroke = egui::Stroke::new(line_spacing * 0.15, color);
                let thin_stroke = egui::Stroke::new(line_spacing * 0.06, color);
                let rest_x = measure_x + measure_width * 0.5;
                let top_y = mid_y - line_spacing * 0.5;
                let bot_y = mid_y + line_spacing * 0.5;

                // Two thick horizontal lines
                painter.line_segment(
                    [
                        egui::Pos2::new(rest_x - half_width, top_y),
                        egui::Pos2::new(rest_x + half_width, top_y),
                    ],
                    thick_stroke,
                );
                painter.line_segment(
                    [
                        egui::Pos2::new(rest_x - half_width, bot_y),
                        egui::Pos2::new(rest_x + half_width, bot_y),
                    ],
                    thick_stroke,
                );

                // Vertical end caps
                painter.line_segment(
                    [
                        egui::Pos2::new(rest_x - half_width, bot_y),
                        egui::Pos2::new(rest_x - half_width, top_y),
                    ],
                    thin_stroke,
                );
                painter.line_segment(
                    [
                        egui::Pos2::new(rest_x + half_width, bot_y),
                        egui::Pos2::new(rest_x + half_width, top_y),
                    ],
                    thin_stroke,
                );

                // Count number above
                let count_str = mrest.count.to_string();
                let font_id = egui::FontId::new(line_spacing * 1.2, egui::FontFamily::Proportional);
                painter.text(
                    egui::Pos2::new(rest_x, top_y - line_spacing * 0.3),
                    egui::Align2::CENTER_BOTTOM,
                    count_str,
                    font_id,
                    color,
                );
            }
        }
    }

    // Render beams
    for (bi, beam) in beams.iter().enumerate() {
        let first_idx = beam.start_idx;
        let last_idx = beam.end_idx;

        let mut first_x: Option<f32> = None;
        let mut last_x: Option<f32> = None;
        let mut ri: usize = 0;

        for (ei, elem) in elements.iter().enumerate() {
            if matches!(elem, MeasureElement::Backup(_) | MeasureElement::Forward(_)) {
                continue;
            }
            let note_spacing = measure_width / renderable_count as f32;
            let x = measure_x + note_spacing * (ri as f32 + 0.5);

            if ei == first_idx {
                first_x = Some(x);
            }
            if ei == last_idx {
                last_x = Some(x);
            }
            ri += 1;
        }

        if let (Some(x1), Some(x2)) = (first_x, last_x) {
            // Use pre-computed beam Y and direction from beam_meta
            let (beam_y, stem_x_offset) = beam_meta
                .get(bi)
                .map(|(_, _, d, y)| {
                    let off = match d {
                        StemDirection::Up => line_spacing * 0.55,
                        StemDirection::Down => -line_spacing * 0.55,
                    };
                    (*y, off)
                })
                .unwrap_or((staff_origin.y - line_spacing * 3.5, 0.0));
            let half_thickness = line_spacing * 0.25;
            let beam_rect = egui::Rect::from_min_max(
                egui::Pos2::new(x1 + stem_x_offset, beam_y - half_thickness),
                egui::Pos2::new(x2 + stem_x_offset, beam_y + half_thickness),
            );
            painter.rect_filled(beam_rect, 0.0, color);
        }
    }
}

/// Renderiza una armadura de clave o la armadura por defecto.
fn render_key_or_default(
    painter: &egui::Painter,
    measure: &Measure,
    x: f32,
    staff_top_y: f32,
    line_spacing: f32,
    clef: Clef,
    color: egui::Color32,
    prev_key: Option<&KeySignature>,
) -> f32 {
    let key = &measure.key_signature;

    // Determine cancel: if this key is different from previous
    let effective_key = if let Some(pk) = prev_key {
        if pk.fifths != key.fifths {
            KeySignature {
                cancel: Some(pk.fifths),
                ..key.clone()
            }
        } else {
            key.clone()
        }
    } else {
        key.clone()
    };

    render_key_signature(
        painter,
        x,
        staff_top_y,
        &effective_key,
        line_spacing,
        clef,
        color,
    )
}

/// Renderiza la indicación de compás usando glifos SMuFL.
pub fn render_time_signature(
    painter: &egui::Painter,
    x: f32,
    staff_top_y: f32,
    numerator: u8,
    denominator: u8,
    style_ts: TimeSignatureStyle,
    line_spacing: f32,
    color: egui::Color32,
) {
    let font_size = line_spacing * 3.0;
    let font_id = egui::FontId::new(font_size, egui::FontFamily::Name("Leland".into()));

    match style_ts {
        TimeSignatureStyle::Common => {
            let glyph = '\u{E08A}'; // common time
            let y = staff_top_y + line_spacing * 2.0;
            painter.text(
                egui::Pos2::new(x, y),
                egui::Align2::CENTER_CENTER,
                glyph.to_string(),
                font_id,
                color,
            );
        }
        TimeSignatureStyle::Cut => {
            let glyph = '\u{E08B}'; // cut time
            let y = staff_top_y + line_spacing * 2.0;
            painter.text(
                egui::Pos2::new(x, y),
                egui::Align2::CENTER_CENTER,
                glyph.to_string(),
                font_id,
                color,
            );
        }
        TimeSignatureStyle::Numeric => {
            // Use SMuFL numbers U+E080–U+E089
            let num_glyph = |d: u8| -> char { char::from_u32(0xE080 + d as u32).unwrap_or('0') };

            let top_y = staff_top_y + line_spacing * 0.8;
            let bottom_y = staff_top_y + line_spacing * 3.2;

            // Numerator
            let num_str: String = numerator
                .to_string()
                .chars()
                .map(|c| num_glyph(c.to_digit(10).unwrap_or(0) as u8))
                .collect();
            painter.text(
                egui::Pos2::new(x, top_y),
                egui::Align2::CENTER_CENTER,
                num_str,
                font_id.clone(),
                color,
            );

            // Denominator
            let den_str: String = denominator
                .to_string()
                .chars()
                .map(|c| num_glyph(c.to_digit(10).unwrap_or(0) as u8))
                .collect();
            painter.text(
                egui::Pos2::new(x, bottom_y),
                egui::Align2::CENTER_CENTER,
                den_str,
                font_id,
                color,
            );
        }
    }
}

use crate::notation::attachment::{GlissandoKind, SlurKind, TieKind};

/// Render cross-note elements: ties, slurs, glissando.
/// Must be called after all measure elements in a staff are rendered.
pub fn render_cross_note_elements(
    painter: &egui::Painter,
    staff_origin: egui::Pos2,
    clef: Clef,
    measures: &[Measure],
    measure_xs: &[f32],
    measure_widths: &[f32],
    line_spacing: f32,
    color: egui::Color32,
) {
    use crate::render::attachment_render::{
        render_glissando_line, render_slur_curve, render_tie_curve,
    };

    // Collect all note positions with their attachments
    struct NotePos {
        x: f32,
        y: f32,
        attachments: crate::notation::NoteAttachment,
    }

    let half_spacing = line_spacing / 2.0;
    let mut notes: Vec<NotePos> = Vec::new();

    for (mi, measure) in measures.iter().enumerate() {
        let m_x = measure_xs[mi];
        for elem in &measure.elements {
            let (note, _render_idx) = match elem {
                MeasureElement::Note(n) => (n, 0),
                _ => continue,
            };
            if let Some(ref att) = note.attachments {
                let sp = note.pitch.staff_position(clef);
                let note_y = staff_origin.y - sp as f32 * half_spacing;
                // Approximate x: use measure center for now
                notes.push(NotePos {
                    x: m_x + measure_widths[mi] / 2.0,
                    y: note_y,
                    attachments: att.clone(),
                });
            }
        }
    }

    // Render ties
    for (i, np) in notes.iter().enumerate() {
        for tie in &np.attachments.ties {
            if tie.kind != TieKind::Start {
                continue;
            }
            // Find matching stop tie
            for (_j, np2) in notes.iter().enumerate().skip(i + 1) {
                if np2
                    .attachments
                    .ties
                    .iter()
                    .any(|t| t.kind == TieKind::Stop && t.number == tie.number)
                {
                    render_tie_curve(
                        painter,
                        np.x,
                        np.y,
                        np2.x,
                        np2.y,
                        tie.placement,
                        line_spacing,
                        color,
                    );
                    break;
                }
            }
        }
    }

    // Render slurs
    for (i, np) in notes.iter().enumerate() {
        for slur in &np.attachments.slurs {
            if slur.kind != SlurKind::Start {
                continue;
            }
            // Find matching stop slur
            for (_j, np2) in notes.iter().enumerate().skip(i + 1) {
                if np2
                    .attachments
                    .slurs
                    .iter()
                    .any(|s| s.kind == SlurKind::Stop && s.number == slur.number)
                {
                    render_slur_curve(
                        painter,
                        np.x,
                        np.y,
                        np2.x,
                        np2.y,
                        slur.placement,
                        line_spacing,
                        color,
                    );
                    break;
                }
            }
        }
    }

    // Render glissando
    for (i, np) in notes.iter().enumerate() {
        if let Some(gliss) = &np.attachments.glissando {
            if gliss.kind != GlissandoKind::Start {
                continue;
            }
            for (_j, np2) in notes.iter().enumerate().skip(i + 1) {
                if np2
                    .attachments
                    .glissando
                    .as_ref()
                    .is_some_and(|g| g.kind == GlissandoKind::Stop)
                {
                    render_glissando_line(painter, np.x, np.y, np2.x, np2.y, line_spacing, color);
                    break;
                }
            }
        }
    }

    // Render tuplets
    let mut i = 0;
    while i < notes.len() {
        if let Some(ref tuplet) = notes[i].attachments.tuplet {
            // Find all consecutive notes with the same tuplet
            let start_i = i;
            while i + 1 < notes.len() && notes[i + 1].attachments.tuplet.is_some() {
                i += 1;
            }
            let end_i = i;
            if end_i > start_i {
                use crate::render::attachment_render::render_tuplet;
                let mid_y = (notes[start_i].y + notes[end_i].y) / 2.0;
                render_tuplet(
                    painter,
                    notes[start_i].x,
                    notes[end_i].x,
                    mid_y,
                    tuplet,
                    line_spacing,
                    color,
                );
            }
        }
        i += 1;
    }
}

/// Renderiza una partitura completa en un painter.
///
/// Retorna la altura total usada.
pub fn render_score(
    painter: &egui::Painter,
    top_left: egui::Pos2,
    score: &Score,
    max_width: f32,
    style: &RenderStyle,
) -> f32 {
    let mut y_offset = 0.0f32;
    let line_spacing = style.line_spacing;

    for system in &score.systems {
        for staff in &system.staves {
            let staff_top_y = top_left.y + y_offset;
            let staff_color = style.sheet.staff_color(style.dark_mode);
            let bar_x = top_left.x + LEFT_MARGIN + system.left_margin;
            let clef_x = bar_x + INITIAL_BAR_GAP;
            let key_sig_width = if let Some(first_m) = staff.measures.first() {
                // Estimate key sig width
                let fifths_abs = first_m.key_signature.fifths.unsigned_abs() as f32;
                line_spacing * 1.1 * fifths_abs + line_spacing * 0.5
            } else {
                0.0
            };
            let time_x = clef_x + CLEF_WIDTH + key_sig_width;
            let measures_x = time_x + TIME_SIG_WIDTH;
            let staff_top = egui::Pos2::new(bar_x, staff_top_y);

            // Staff lines
            render_staff_lines(painter, staff_top, max_width, line_spacing, staff_color);

            // Initial bar line
            render_bar_line(
                painter,
                bar_x,
                staff_top_y,
                line_spacing,
                BarStyle::Regular,
                staff_color,
            );

            // Clef
            let clef_top = egui::Pos2::new(clef_x, staff_top_y);
            let (glyph_scale, fine_offset) = get_clef_config(&style.sheet, staff.clef);
            render_clef(
                painter,
                clef_top,
                staff.clef,
                line_spacing,
                style.sheet.clef_color(style.dark_mode),
                glyph_scale,
                fine_offset,
            );

            // Key signature
            if let Some(first_measure) = staff.measures.first() {
                let key_x = clef_x + CLEF_WIDTH;
                let _key_end_x = render_key_or_default(
                    painter,
                    first_measure,
                    key_x,
                    staff_top_y,
                    line_spacing,
                    staff.clef,
                    staff_color,
                    None,
                );
            }

            // Time signature
            if let Some(first_measure) = staff.measures.first() {
                let ts = &first_measure.time_signature;
                let ts_x = time_x + TIME_SIG_WIDTH * 0.5;
                render_time_signature(
                    painter,
                    ts_x,
                    staff_top_y,
                    ts.numerator,
                    ts.denominator,
                    ts.style,
                    line_spacing,
                    staff_color,
                );
            }

            // Staff origin (3rd line)
            let staff_origin = egui::Pos2::new(measures_x, staff_top_y + line_spacing * 2.0);

            // Render measures
            let measure_start_x = measures_x;
            let measure_usable_width = max_width
                - LEFT_MARGIN
                - system.left_margin
                - INITIAL_BAR_GAP
                - CLEF_WIDTH
                - key_sig_width
                - TIME_SIG_WIDTH;

            let mut prev_key: Option<&KeySignature> = None;

            // Compute proportional measure widths
            let measure_widths = {
                // Use `super::layout::compute_measure_widths` or the imported one
                // Since we're in render::score, use crate::render::layout
                crate::render::layout::compute_measure_widths(
                    &staff.measures,
                    measure_usable_width,
                    DEFAULT_MEASURE_WIDTH * 0.5,
                    DEFAULT_MEASURE_WIDTH * 2.0,
                )
            };

            let mut measure_x = measure_start_x;
            for (mi, measure) in staff.measures.iter().enumerate() {
                let measure_width = measure_widths[mi];

                // Bar line between measures (with style from previous measure)
                if mi > 0 {
                    let prev_measure = &staff.measures[mi - 1];
                    render_bar_line(
                        painter,
                        measure_x,
                        staff_top_y,
                        line_spacing,
                        prev_measure.barline.style,
                        staff_color,
                    );
                }

                // Key signature change mid-staff
                if mi > 0 {
                    if let Some(pk) = prev_key {
                        if measure.key_signature.fifths != pk.fifths {
                            let key_x = measure_x + line_spacing;
                            render_key_or_default(
                                painter,
                                measure,
                                key_x,
                                staff_top_y,
                                line_spacing,
                                staff.clef,
                                staff_color,
                                Some(pk),
                            );
                        }
                    }
                }

                render_measure_elements(
                    painter,
                    measure,
                    staff_origin,
                    staff.clef,
                    measure_x,
                    measure_width,
                    style,
                );

                // Render staff-level directions below the staff
                let staff_end_x = measure_start_x + measure_usable_width;
                for direction in &measure.directions {
                    render_direction(
                        painter,
                        measure_x + measure_width * 0.5,
                        staff_top_y + line_spacing * 2.0,
                        staff_end_x,
                        direction,
                        line_spacing,
                        staff_color,
                    );
                }

                prev_key = Some(&measure.key_signature);
                measure_x += measure_width;
            }
            // Final bar line
            let end_x = measure_x;
            if let Some(last_measure) = staff.measures.last() {
                render_bar_line(
                    painter,
                    end_x,
                    staff_top_y,
                    line_spacing,
                    last_measure.barline.style,
                    staff_color,
                );
            }

            // Render lyrics below the staff
            let mut note_lyrics: Vec<(f32, &[crate::notation::Lyric])> = Vec::new();
            {
                let mut mx = measure_start_x;
                for (mi, measure) in staff.measures.iter().enumerate() {
                    for elem in &measure.elements {
                        let lyrics: &[crate::notation::Lyric] = match elem {
                            crate::notation::MeasureElement::Note(n) => &n.lyrics,
                            crate::notation::MeasureElement::Chord(notes) if !notes.is_empty() => {
                                &notes[0].lyrics
                            }
                            _ => continue,
                        };
                        if !lyrics.is_empty() {
                            note_lyrics.push((mx, lyrics));
                        }
                    }
                    mx += measure_widths[mi];
                }
            }
            if !note_lyrics.is_empty() {
                render_lyrics(
                    painter,
                    &note_lyrics,
                    staff_top_y + line_spacing * 4.0,
                    line_spacing,
                    staff_color,
                );
            }
            // Render cross-note elements (ties, slurs, glissando)
            {
                let mut m_xs: Vec<f32> = Vec::with_capacity(measure_widths.len());
                let mut mx = measure_start_x;
                for &w in &measure_widths {
                    m_xs.push(mx);
                    mx += w;
                }
                let staff_origin =
                    egui::Pos2::new(measure_start_x, staff_top_y + line_spacing * 2.0);
                render_cross_note_elements(
                    painter,
                    staff_origin,
                    staff.clef,
                    &staff.measures,
                    &m_xs,
                    &measure_widths,
                    line_spacing,
                    staff_color,
                );
            }

            y_offset += staff_height(line_spacing) + STAFF_PADDING;
        }
    }

    y_offset
}

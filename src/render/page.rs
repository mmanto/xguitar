use eframe::egui;

use super::constants::{
    A4_HEIGHT_PT, A4_WIDTH_PT, CLEF_WIDTH, DEFAULT_MEASURE_WIDTH, INITIAL_BAR_GAP, PAGE_GAP,
    PAGE_MARGIN_BOTTOM, PAGE_MARGIN_LEFT, PAGE_MARGIN_RIGHT, STAFF_LINE_COUNT, STAFF_LINE_SPACING,
    SYSTEM_SPACING, TIME_SIG_WIDTH,
};
use super::score::get_clef_config;
use super::{
    RenderStyle, compute_measure_widths, render_bar_line, render_clef, render_cross_note_elements,
    render_direction, render_key_signature, render_lyrics, render_measure_elements,
    render_staff_lines, render_time_signature,
};
use crate::notation::{BarStyle, DirectionKind, Metronome, NoteFigure, Score};

/// Una hoja A4 con los índices de los sistemas y staves que contiene.
#[derive(Clone, Debug)]
pub struct StaffRef {
    pub system_idx: usize,
    pub staff_idx: usize,
}

#[derive(Clone, Debug)]
pub struct Page {
    /// Referencias a staves que pertenecen a esta página.
    pub staff_refs: Vec<StaffRef>,
}

/// Layout completo de paginación para una partitura.
#[derive(Clone, Debug)]
pub struct PageLayout {
    pub pages: Vec<Page>,
    /// Ancho de página escalado por zoom.
    pub page_width: f32,
    /// Alto de página escalado por zoom.
    pub page_height: f32,
    /// Altura total de todas las páginas apiladas (incluye gaps).
    pub total_height: f32,
}

/// Distribuye los staves de la partitura en páginas A4.
pub fn compute_pages(
    score: &Score,
    zoom: f32,
    _max_measures_per_line: usize,
    header_height: f32,
) -> PageLayout {
    let page_width = A4_WIDTH_PT * zoom;
    let page_height = A4_HEIGHT_PT * zoom;
    let line_spacing = STAFF_LINE_SPACING * zoom;

    let usable_height = page_height - header_height - PAGE_MARGIN_BOTTOM * zoom;
    let staff_height_val = line_spacing * (STAFF_LINE_COUNT as f32 - 1.0);
    let staff_slot_height = staff_height_val + SYSTEM_SPACING * zoom;

    let max_staves_per_page = if staff_slot_height > 0.0 {
        (usable_height / staff_slot_height) as usize
    } else {
        1
    }
    .max(1);

    let mut pages: Vec<Page> = Vec::new();
    let mut current_refs: Vec<StaffRef> = Vec::new();

    for (si, system) in score.systems.iter().enumerate() {
        for (sti, _staff) in system.staves.iter().enumerate() {
            current_refs.push(StaffRef {
                system_idx: si,
                staff_idx: sti,
            });
            if current_refs.len() >= max_staves_per_page {
                pages.push(Page {
                    staff_refs: std::mem::take(&mut current_refs),
                });
            }
        }
    }

    if !current_refs.is_empty() {
        pages.push(Page {
            staff_refs: current_refs,
        });
    }

    let total_height = pages.len() as f32 * (page_height + PAGE_GAP * zoom);

    PageLayout {
        pages,
        page_width,
        page_height,
        total_height,
    }
}

/// Renderiza una sola página A4 en `top_left`.
pub fn render_page(
    painter: &egui::Painter,
    top_left: egui::Pos2,
    page: &Page,
    layout: &PageLayout,
    score: &Score,
    style: &RenderStyle,
) {
    let zoom = style.line_spacing / STAFF_LINE_SPACING;
    let line_spacing = style.line_spacing;
    let sheet = &style.sheet;

    // Drop shadow
    if sheet.page.shadow.enabled {
        let shadow_offset = egui::Vec2::new(
            sheet.page.shadow.offset_x * zoom,
            sheet.page.shadow.offset_y * zoom,
        );
        let shadow_color = egui::Color32::from_black_alpha(sheet.page.shadow.alpha);
        painter.rect_filled(
            egui::Rect::from_min_size(
                top_left + shadow_offset,
                egui::Vec2::new(layout.page_width, layout.page_height),
            ),
            2.0,
            shadow_color,
        );
    }

    // Page background
    let bg_color = sheet.page_bg(style.dark_mode);
    painter.rect_filled(
        egui::Rect::from_min_size(
            top_left,
            egui::Vec2::new(layout.page_width, layout.page_height),
        ),
        0.0,
        bg_color,
    );

    // Page border
    let border_color = sheet.page_border(style.dark_mode);
    painter.rect_stroke(
        egui::Rect::from_min_size(
            top_left,
            egui::Vec2::new(layout.page_width, layout.page_height),
        ),
        0.0,
        egui::Stroke::new(sheet.page.border_width, border_color),
        egui::StrokeKind::Inside,
    );

    // Page header: title, composer, tempo
    let header_cx = top_left.x + layout.page_width * 0.5;
    let title_y = top_left.y + sheet.header.title_top_offset * zoom;
    let composer_y = title_y + (sheet.header.title_size + sheet.header.row_gap) * zoom;
    let tempo_y = composer_y + (sheet.header.composer_size + sheet.header.row_gap * 2.0) * zoom;

    let title_color = sheet.title_color(style.dark_mode);
    painter.text(
        egui::Pos2::new(header_cx, title_y),
        egui::Align2::CENTER_TOP,
        &score.title,
        egui::FontId::new(
            sheet.header.title_size * zoom,
            egui::FontFamily::Proportional,
        ),
        title_color,
    );

    let subtitle_color = sheet.subtitle_color(style.dark_mode);
    painter.text(
        egui::Pos2::new(header_cx, composer_y),
        egui::Align2::CENTER_TOP,
        &score.composer,
        egui::FontId::new(
            sheet.header.composer_size * zoom,
            egui::FontFamily::Proportional,
        ),
        subtitle_color,
    );

    // Tempo (BPM): extract from first measure's metronome direction
    let tempo_row_height = if let Some(metro) = first_metronome(score) {
        // Use Unicode note symbols visible at text sizes (not SMuFL PUA)
        let note_char = metronome_note_char(metro.beat_unit);
        let glyph_size = line_spacing * 1.6;
        let text = format!("{} = {}", note_char, metro.per_minute);
        painter.text(
            egui::Pos2::new(header_cx, tempo_y),
            egui::Align2::CENTER_TOP,
            text,
            egui::FontId::new(glyph_size, egui::FontFamily::Proportional),
            title_color,
        );
        glyph_size + sheet.header.row_gap * zoom
    } else {
        0.0
    };
    let header_bottom = top_left.y
        + (sheet.header.title_top_offset
            + sheet.header.title_size
            + sheet.header.row_gap
            + sheet.header.composer_size
            + sheet.header.row_gap
            + sheet.header.header_staff_gap)
            * zoom
        + tempo_row_height;
    // Staves
    let staff_height_val = line_spacing * (STAFF_LINE_COUNT as f32 - 1.0);
    let staff_slot_height = staff_height_val + SYSTEM_SPACING * zoom;

    let usable_width = layout.page_width - PAGE_MARGIN_LEFT * zoom - PAGE_MARGIN_RIGHT * zoom;
    let left_x = top_left.x + PAGE_MARGIN_LEFT * zoom;

    let _prev_key: Option<&crate::notation::KeySignature> = None;

    for (i, sref) in page.staff_refs.iter().enumerate() {
        let system = &score.systems[sref.system_idx];
        let staff = &system.staves[sref.staff_idx];

        let staff_top_y = header_bottom + i as f32 * staff_slot_height;
        let left_x_bar = left_x + system.left_margin * zoom;
        let staff_color = sheet.staff_color(style.dark_mode);

        // Estimate key sig width
        let key_sig_width = if let Some(first_m) = staff.measures.first() {
            let fifths_abs = first_m.key_signature.fifths.unsigned_abs() as f32;
            line_spacing * 1.1 * fifths_abs + line_spacing * 0.5
        } else {
            0.0
        };

        // Staff lines
        render_staff_lines(
            painter,
            egui::Pos2::new(left_x_bar, staff_top_y),
            usable_width,
            line_spacing,
            staff_color,
        );

        // Initial bar line
        render_bar_line(
            painter,
            left_x_bar,
            staff_top_y,
            line_spacing,
            BarStyle::Regular,
            staff_color,
        );

        // Clef
        let clef_x = left_x_bar + INITIAL_BAR_GAP * zoom;
        let (glyph_scale, fine_offset) = get_clef_config(sheet, staff.clef);
        render_clef(
            painter,
            egui::Pos2::new(clef_x, staff_top_y),
            staff.clef,
            line_spacing,
            sheet.clef_color(style.dark_mode),
            glyph_scale,
            fine_offset,
        );

        // Key signature
        if let Some(first_m) = staff.measures.first() {
            let key_x = clef_x + CLEF_WIDTH * zoom;
            render_key_signature(
                painter,
                key_x,
                staff_top_y,
                &first_m.key_signature,
                line_spacing,
                staff.clef,
                staff_color,
            );

            // Time signature
            // Time signature — closer to clef
            let ts_x = key_x + key_sig_width - line_spacing * 0.5;
            let ts = &first_m.time_signature;
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

        let measures_x = clef_x + CLEF_WIDTH * zoom + key_sig_width + TIME_SIG_WIDTH * zoom * 0.7;
        let staff_origin = egui::Pos2::new(measures_x, staff_top_y + line_spacing * 2.0);

        let measure_usable_width = usable_width
            - (INITIAL_BAR_GAP + CLEF_WIDTH + TIME_SIG_WIDTH * 0.7) * zoom
            - key_sig_width
            - system.left_margin * zoom;

        // Compute proportional measure widths
        let measure_widths = compute_measure_widths(
            &staff.measures,
            measure_usable_width,
            DEFAULT_MEASURE_WIDTH * zoom * 0.5,
            DEFAULT_MEASURE_WIDTH * zoom * 2.0,
        );

        let mut measure_x = measures_x;
        for (mi, measure) in staff.measures.iter().enumerate() {
            let measure_width = measure_widths[mi];
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

            render_measure_elements(
                painter,
                measure,
                staff_origin,
                staff.clef,
                measure_x,
                measure_width,
                style,
            );

            // Render staff-level directions (dynamics, text, etc.) below the staff.
            // Metronome is rendered in the page header — skip it here.
            let staff_end_x = measures_x + measure_usable_width;
            for direction in &measure.directions {
                if matches!(direction.kind, DirectionKind::Metronome(_)) {
                    continue;
                }
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
            let mut mx = measures_x;
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
            let mut mx = measures_x;
            for &w in &measure_widths {
                m_xs.push(mx);
                mx += w;
            }
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
    }
}

/// Extrae el metrónomo de la primera medida del primer staff.
fn first_metronome(score: &Score) -> Option<&Metronome> {
    let staff = score.systems.first()?.staves.first()?;
    let measure = staff.measures.first()?;
    for dir in &measure.directions {
        if let DirectionKind::Metronome(metro) = &dir.kind {
            return Some(metro);
        }
    }
    None
}

/// Símbolo de nota visible para el metrónomo (usa Unicode, no SMuFL PUA).
fn metronome_note_char(figure: NoteFigure) -> char {
    match figure {
        NoteFigure::Quarter => '\u{2669}',  // ♩
        NoteFigure::Eighth => '\u{266A}',   // ♪
        NoteFigure::Half => '\u{2669}',     // sin símbolo half en Unicode, usa ♩
        NoteFigure::Whole => '\u{2669}',    // ídem
        _ => '\u{2669}',                    // default: ♩
    }
}

/// Renderiza todas las páginas del layout en una grilla.
pub fn render_pages(
    painter: &egui::Painter,
    top_left: egui::Pos2,
    layout: &PageLayout,
    score: &Score,
    style: &RenderStyle,
    pages_per_row: usize,
) {
    let mut y = top_left.y;
    let mut x = top_left.x;

    for (i, page) in layout.pages.iter().enumerate() {
        if i > 0 && i % pages_per_row == 0 {
            y += layout.page_height + PAGE_GAP * style.line_spacing / STAFF_LINE_SPACING;
            x = top_left.x;
        }

        render_page(painter, egui::Pos2::new(x, y), page, layout, score, style);
        x += layout.page_width + PAGE_GAP * style.line_spacing / STAFF_LINE_SPACING;
    }
}

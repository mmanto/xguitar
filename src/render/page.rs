use eframe::egui;

use super::constants::{
    A4_HEIGHT_PT, A4_WIDTH_PT, CLEF_WIDTH, DEFAULT_MEASURE_WIDTH, INITIAL_BAR_GAP, PAGE_GAP,
    PAGE_MARGIN_BOTTOM, PAGE_MARGIN_LEFT, PAGE_MARGIN_RIGHT, STAFF_LINE_COUNT, STAFF_LINE_SPACING,
    SYSTEM_SPACING, TIME_SIG_WIDTH,
};
use super::score::get_clef_config;
use super::{
    RenderStyle, break_measures_into_lines, compute_measure_widths, element_offsets,
    render_bar_line, render_clef, render_cross_note_elements, render_direction,
    render_key_signature, render_lyrics, render_measure_elements, render_measure_number,
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


fn staff_line_count(staff: &crate::notation::Staff, zoom: f32, system_left_margin: f32) -> usize {
    let line_spacing = STAFF_LINE_SPACING * zoom;
    let page_width = A4_WIDTH_PT * zoom;
    let usable_width = page_width - PAGE_MARGIN_LEFT * zoom - PAGE_MARGIN_RIGHT * zoom;

    // Replicate key_sig_width calculation from render_page
    let key_sig_width = staff
        .measures
        .first()
        .map(|m| {
            let fifths_abs = m.key_signature.fifths.unsigned_abs() as f32;
            line_spacing * 1.1 * fifths_abs + line_spacing * 0.5
        })
        .unwrap_or(0.0);

    let measure_usable_width = usable_width
        - (INITIAL_BAR_GAP + CLEF_WIDTH + TIME_SIG_WIDTH * 0.7) * zoom
        - key_sig_width
        - system_left_margin * zoom;

    let lines = break_measures_into_lines(&staff.measures, measure_usable_width, line_spacing, DEFAULT_MEASURE_WIDTH * zoom * 0.5);
    lines.len().max(1)
}
/// Distribuye los staves de la partitura en páginas A4.
pub fn compute_pages(
    score: &Score,
    zoom: f32,
    header_height: f32,
) -> PageLayout {
    let page_width = A4_WIDTH_PT * zoom;
    let page_height = A4_HEIGHT_PT * zoom;
    let line_spacing = STAFF_LINE_SPACING * zoom;

    let usable_height = page_height - header_height - PAGE_MARGIN_BOTTOM * zoom;
    let staff_height_val = line_spacing * (STAFF_LINE_COUNT as f32 - 1.0);
    let staff_slot_height = staff_height_val + SYSTEM_SPACING * zoom;

    let max_slots_per_page = if staff_slot_height > 0.0 {
        (usable_height / staff_slot_height) as usize
    } else {
        1
    }
    .max(1);

    let mut pages: Vec<Page> = Vec::new();
    let mut current_refs: Vec<StaffRef> = Vec::new();
    let mut current_slots: usize = 0;

    for (si, system) in score.systems.iter().enumerate() {
        for (sti, staff) in system.staves.iter().enumerate() {
            let lines = staff_line_count(staff, zoom, system.left_margin);

            // If this staff doesn't fit in current page, start new page
            if !current_refs.is_empty() && current_slots + lines > max_slots_per_page {
                pages.push(Page {
                    staff_refs: std::mem::take(&mut current_refs),
                });
                current_slots = 0;
            }

            current_refs.push(StaffRef {
                system_idx: si,
                staff_idx: sti,
            });
            current_slots += lines;
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
    // Left edge of the staves below (see `left_x` further down) — the tempo
    // mark is anchored here instead of page-centered, since conventional
    // engraving (and the MusicXML reference this was checked against) places
    // it above the left margin of the first system, not centered on the page.
    let left_x = top_left.x + PAGE_MARGIN_LEFT * zoom;
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
            egui::Pos2::new(left_x, tempo_y),
            egui::Align2::LEFT_TOP,
            text,
            egui::FontId::new(glyph_size, egui::FontFamily::Proportional),
            title_color,
        );
        glyph_size + sheet.header.row_gap * zoom
    } else {
        0.0
    };

    // Additional page credits (e.g. an arranger byline like "Music by X")
    // not already shown as title/composer above.
    //
    // Only skip *centered* credits matching the title/composer text — our
    // hardcoded title/composer block above is always center-justified, so a
    // center credit with the same text is that same element duplicated in
    // the MusicXML source. A left/right-justified credit is a distinct
    // side element (e.g. an arranger byline) even if its text happens to
    // coincide with the composer's name, and should still be drawn.
    //
    // Vertical position deliberately ignores `credit.default_y`: MusicXML
    // expresses it as a fraction of the source document's real print page
    // height, but our header block has its own fixed (stylesheet-driven)
    // height that doesn't match that proportion — using the raw fraction
    // routinely lands extra credits on top of the staves below. Byline-type
    // credits belong in the header regardless, so they're pinned to the
    // composer row; only the horizontal position (`default_x`, a page
    // fraction that IS proportion-independent) is honored.
    for credit in &score.credits {
        let crate::notation::CreditKind::Words(text) = &credit.kind else {
            continue;
        };
        if credit.justify == crate::notation::CreditJustify::Center
            && (text == &score.title || text == &score.composer)
        {
            continue;
        }
        let align = match credit.justify {
            crate::notation::CreditJustify::Left => egui::Align2::LEFT_TOP,
            crate::notation::CreditJustify::Center => egui::Align2::CENTER_TOP,
            crate::notation::CreditJustify::Right => egui::Align2::RIGHT_TOP,
        };
        let pos = egui::Pos2::new(top_left.x + credit.default_x * layout.page_width, composer_y);
        painter.text(
            pos,
            align,
            text,
            egui::FontId::new(sheet.header.composer_size * zoom, egui::FontFamily::Proportional),
            subtitle_color,
        );
    }

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

    let _prev_key: Option<&crate::notation::KeySignature> = None;

        for (i, sref) in page.staff_refs.iter().enumerate() {
            let system = &score.systems[sref.system_idx];
            let staff = &system.staves[sref.staff_idx];

            let staff_first_line_y = header_bottom + i as f32 * staff_slot_height;
            let left_x_bar = left_x + system.left_margin * zoom;
            let staff_color = sheet.staff_color(style.dark_mode);


            // Estimate key sig width
            let key_sig_width = if let Some(first_m) = staff.measures.first() {
                let fifths_abs = first_m.key_signature.fifths.unsigned_abs() as f32;
                line_spacing * 1.1 * fifths_abs + line_spacing * 0.5
            } else {
                0.0
            };

            // Clef
            let clef_x = left_x_bar + INITIAL_BAR_GAP * zoom;
            let measures_x_first =
                clef_x + CLEF_WIDTH * zoom + key_sig_width + TIME_SIG_WIDTH * zoom * 0.7;

            let measure_usable_width = usable_width
                - (INITIAL_BAR_GAP + CLEF_WIDTH + TIME_SIG_WIDTH * 0.7) * zoom
                - key_sig_width
                - system.left_margin * zoom;

            // Los renglones siguientes repiten la clave (convención de grabado
            // estándar) pero no la armadura ni el compás, así que solo reservan
            // el ancho de la clave.
            let measure_usable_continuation = usable_width
                - (INITIAL_BAR_GAP + CLEF_WIDTH) * zoom
                - system.left_margin * zoom;

            let lines =
                break_measures_into_lines(&staff.measures, measure_usable_width, line_spacing, DEFAULT_MEASURE_WIDTH * zoom * 0.5);

            // Collect lyrics with absolute (x, y) positions across all lines
            let mut all_note_lyrics: Vec<(f32, f32, &[crate::notation::Lyric])> = Vec::new();

            for (line_idx, &(start, end)) in lines.iter().enumerate() {
                let line_measures = &staff.measures[start..end];
                let line_y_offset = line_idx as f32 * staff_slot_height;
                let staff_top_y = staff_first_line_y + line_y_offset;

                // Staff lines for this line segment
                render_staff_lines(
                    painter,
                    egui::Pos2::new(left_x_bar, staff_top_y),
                    usable_width,
                    line_spacing,
                    staff_color,
                );

                // Barra inicial: solo en el primer renglón (los siguientes la
                // reciben del bucle de compases, que usa el estilo real del
                // compás anterior — p. ej. una barra de repetición).
                if line_idx == 0 {
                    render_bar_line(
                        painter,
                        left_x_bar,
                        staff_top_y,
                        line_spacing,
                        BarStyle::Regular,
                        staff_color,
                    );
                }

                // Clave: se repite al inicio de cada renglón (convención de
                // grabado estándar), a diferencia de la armadura y el compás
                // que solo van en el primero.
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

                if line_idx == 0 {
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
                }

                // Compute left-aligned widths: last measure stretches to fill line
                let line_available = if line_idx == 0 {
                    measure_usable_width
                } else {
                    measure_usable_continuation
                };
                let line_widths = compute_measure_widths(
                    line_measures,
                    line_available,
                    DEFAULT_MEASURE_WIDTH * zoom * 0.5,
                    DEFAULT_MEASURE_WIDTH * zoom * 2.0,
                    line_spacing,
                );

                let measures_x = if line_idx == 0 {
                    measures_x_first
                } else {
                    clef_x + CLEF_WIDTH * zoom
                };
                let staff_origin =
                    egui::Pos2::new(measures_x, staff_top_y + line_spacing * 2.0);
                let mut measure_x = measures_x;

                for (local_idx, measure) in line_measures.iter().enumerate() {
                    let actual_idx = start + local_idx;
                    let mw = line_widths[local_idx];

                    // Bar line: draw if not the first measure overall
                    // At line breaks, this draws the separator between the last measure
                    // of the previous line and the first measure of this line — before
                    // the clef, not before the shifted start of measure content.
                    if actual_idx > 0 {
                        let prev_measure = &staff.measures[actual_idx - 1];
                        let bar_x = if local_idx == 0 { left_x_bar } else { measure_x };
                        render_bar_line(
                            painter,
                            bar_x,
                            staff_top_y,
                            line_spacing,
                            prev_measure.barline.style,
                            staff_color,
                        );
                    }

                    // Número de compás
                    render_measure_number(
                        painter,
                        measure_x,
                        staff_top_y,
                        line_spacing,
                        &measure.number,
                        staff_color,
                    );

                    render_measure_elements(
                        painter,
                        measure,
                        staff_origin,
                        staff.clef,
                        measure_x,
                        mw,
                        style,
                        sref.system_idx,
                        sref.staff_idx,
                        actual_idx,
                    );

                    // Directions
                    let staff_end_x = measures_x + line_available;
                    let direction_offsets = element_offsets(&measure.elements, mw);
                    for direction in &measure.directions {
                        if matches!(direction.kind, DirectionKind::Metronome(_)) {
                            continue;
                        }
                        let dx = direction_offsets
                            .get(direction.element_index)
                            .copied()
                            .unwrap_or(mw * 0.5);
                        render_direction(
                            painter,
                            measure_x + dx,
                            staff_top_y + line_spacing * 2.0,
                            staff_end_x,
                            direction,
                            line_spacing,
                            staff_color,
                        );
                    }

                    // Collect lyrics with absolute (x, y) for this line
                    for elem in &measure.elements {
                        let lyrics: &[crate::notation::Lyric] = match elem {
                            crate::notation::MeasureElement::Note(n) => &n.lyrics,
                            crate::notation::MeasureElement::Chord(notes) if !notes.is_empty() => {
                                &notes[0].lyrics
                            }
                            _ => continue,
                        };
                        if !lyrics.is_empty() {
                            all_note_lyrics.push((
                                measure_x,
                                staff_top_y + line_spacing * 4.0,
                                lyrics,
                            ));
                        }
                    }

                    measure_x += mw;
                }

                // Barra de cierre al final de cada línea
                let line_last = &staff.measures[end - 1];
                render_bar_line(
                    painter,
                    measure_x,
                    staff_top_y,
                    line_spacing,
                    line_last.barline.style,
                    staff_color,
                );

                // Cross-note elements per line
                {
                    let mut m_xs: Vec<f32> = Vec::with_capacity(line_widths.len());
                    let mut mx = measures_x;
                    for &w in &line_widths {
                        m_xs.push(mx);
                        mx += w;
                    }
                    render_cross_note_elements(
                        painter,
                        staff_origin,
                        staff.clef,
                        line_measures,
                        &m_xs,
                        &line_widths,
                        line_spacing,
                        staff_color,
                    );
                }
            }

            // Render lyrics collected from all lines
            if !all_note_lyrics.is_empty() {
                // Group by staff_top_y to render per line
                let mut by_y: std::collections::BTreeMap<i64, Vec<(f32, &[crate::notation::Lyric])>> =
                    std::collections::BTreeMap::new();
                for (x, y, lyrics) in &all_note_lyrics {
                    let key = (*y / 1.0) as i64;
                    by_y.entry(key).or_default().push((*x, *lyrics));
                }
                for (key, note_lyrics) in by_y {
                    render_lyrics(painter, &note_lyrics, key as f32, line_spacing, staff_color);
                }
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

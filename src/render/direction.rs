use crate::notation::{
    Direction, DirectionKind, Metronome, OctaveShift, OctaveShiftKind, Pedal, PedalKind, Wedge,
    WedgeKind,
};
use eframe::egui;

/// Renderiza una dirección (dinámica, texto, wedge, etc.) en el staff.
pub fn render_direction(
    painter: &egui::Painter,
    x: f32,
    y: f32,      // staff middle line y
    next_x: f32, // end x for spanning elements
    dir: &Direction,
    line_spacing: f32,
    color: egui::Color32,
) {
    let below_y = y + line_spacing * 2.5;

    match &dir.kind {
        DirectionKind::Dynamics(dms) => {
            let text = dynamics_text(dms);
            let font_size = line_spacing * 1.6;
            painter.text(
                egui::Pos2::new(x, below_y),
                egui::Align2::CENTER_TOP,
                text,
                egui::FontId::new(font_size, egui::FontFamily::Proportional),
                color,
            );
        }
        DirectionKind::Wedge(wedge) => {
            render_wedge(painter, x, below_y, next_x, wedge, line_spacing, color);
        }
        DirectionKind::Words(text) => {
            let font_size = line_spacing * 1.3;
            painter.text(
                egui::Pos2::new(x, below_y),
                egui::Align2::LEFT_TOP,
                text,
                egui::FontId::new(font_size, egui::FontFamily::Proportional),
                color,
            );
        }
        DirectionKind::Rehearsal(text) => {
            let font_size = line_spacing * 1.8;
            let text_galley = painter.layout_no_wrap(
                text.clone(),
                egui::FontId::new(font_size, egui::FontFamily::Proportional),
                color,
            );
            let rect = egui::Rect::from_center_size(
                egui::Pos2::new(x, below_y - line_spacing * 0.5),
                text_galley.size() + egui::Vec2::new(line_spacing * 0.8, line_spacing * 0.4),
            );
            painter.rect_stroke(
                rect,
                0.0,
                egui::Stroke::new(1.5, color),
                egui::StrokeKind::Inside,
            );
            painter.text(
                rect.center(),
                egui::Align2::CENTER_CENTER,
                text,
                egui::FontId::new(font_size, egui::FontFamily::Proportional),
                color,
            );
        }
        DirectionKind::Metronome(metro) => {
            render_metronome(painter, x, below_y, metro, line_spacing, color);
        }
        DirectionKind::OctaveShift(shift) => {
            render_octave_shift(painter, x, below_y, next_x, shift, line_spacing, color);
        }
        DirectionKind::Pedal(pedal) => {
            render_pedal(painter, x, below_y, next_x, pedal, line_spacing, color);
        }
        DirectionKind::Dashes(_) | DirectionKind::Bracket(_) => {
            // Rendered as line segments between start/stop
        }
    }
}

fn dynamics_text(dms: &[crate::notation::DynamicMark]) -> String {
    dms.iter()
        .map(|dm| match dm {
            crate::notation::DynamicMark::PPPP => "pppp",
            crate::notation::DynamicMark::PPP => "ppp",
            crate::notation::DynamicMark::PP => "pp",
            crate::notation::DynamicMark::P => "p",
            crate::notation::DynamicMark::MP => "mp",
            crate::notation::DynamicMark::MF => "mf",
            crate::notation::DynamicMark::F => "f",
            crate::notation::DynamicMark::FF => "ff",
            crate::notation::DynamicMark::FFF => "fff",
            crate::notation::DynamicMark::FFFF => "ffff",
            crate::notation::DynamicMark::SF => "sf",
            crate::notation::DynamicMark::SFZ => "sfz",
            crate::notation::DynamicMark::SFFZ => "sffz",
            crate::notation::DynamicMark::SFP => "sfp",
            crate::notation::DynamicMark::SFPP => "sfpp",
            crate::notation::DynamicMark::FP => "fp",
            crate::notation::DynamicMark::RF => "rf",
            crate::notation::DynamicMark::RFZ => "rfz",
            crate::notation::DynamicMark::FZ => "fz",
            crate::notation::DynamicMark::N => "n",
            crate::notation::DynamicMark::PF => "pf",
            crate::notation::DynamicMark::Other(s) => s.as_str(),
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn render_wedge(
    painter: &egui::Painter,
    start_x: f32,
    y: f32,
    end_x: f32,
    wedge: &Wedge,
    line_spacing: f32,
    color: egui::Color32,
) {
    let open_height = line_spacing * 1.0;
    let closed_height = if wedge.niente {
        0.0
    } else {
        line_spacing * 0.2
    };
    let stroke = egui::Stroke::new(line_spacing * 0.08, color);

    match wedge.kind {
        WedgeKind::Crescendo => {
            // Opens to the right: <
            painter.line_segment(
                [
                    egui::Pos2::new(start_x, y - closed_height / 2.0),
                    egui::Pos2::new(end_x, y - open_height / 2.0),
                ],
                stroke,
            );
            painter.line_segment(
                [
                    egui::Pos2::new(start_x, y + closed_height / 2.0),
                    egui::Pos2::new(end_x, y + open_height / 2.0),
                ],
                stroke,
            );
            if wedge.niente {
                let niente = '\u{E52A}';
                let font_id =
                    egui::FontId::new(line_spacing * 1.0, egui::FontFamily::Name("Leland".into()));
                painter.text(
                    egui::Pos2::new(start_x, y),
                    egui::Align2::CENTER_CENTER,
                    niente.to_string(),
                    font_id,
                    color,
                );
            }
        }
        WedgeKind::Diminuendo => {
            // Closes to the right: >
            painter.line_segment(
                [
                    egui::Pos2::new(start_x, y - open_height / 2.0),
                    egui::Pos2::new(end_x, y - closed_height / 2.0),
                ],
                stroke,
            );
            painter.line_segment(
                [
                    egui::Pos2::new(start_x, y + open_height / 2.0),
                    egui::Pos2::new(end_x, y + closed_height / 2.0),
                ],
                stroke,
            );
            if wedge.niente {
                let niente = '\u{E52A}';
                let font_id =
                    egui::FontId::new(line_spacing * 1.0, egui::FontFamily::Name("Leland".into()));
                painter.text(
                    egui::Pos2::new(end_x, y),
                    egui::Align2::CENTER_CENTER,
                    niente.to_string(),
                    font_id,
                    color,
                );
            }
        }
    }
}

fn render_metronome(
    painter: &egui::Painter,
    x: f32,
    y: f32,
    metro: &Metronome,
    line_spacing: f32,
    color: egui::Color32,
) {
    let font_size = line_spacing * 1.3;
    let font_id = egui::FontId::new(font_size, egui::FontFamily::Proportional);
    let note_glyph = metro.beat_unit.notehead_glyph();
    let text = if metro.parentheses {
        format!("({} = {})", note_glyph, metro.per_minute)
    } else {
        format!("{} = {}", note_glyph, metro.per_minute)
    };
    painter.text(
        egui::Pos2::new(x, y),
        egui::Align2::LEFT_TOP,
        text,
        font_id,
        color,
    );
}

fn render_octave_shift(
    painter: &egui::Painter,
    start_x: f32,
    y: f32,
    end_x: f32,
    shift: &OctaveShift,
    line_spacing: f32,
    color: egui::Color32,
) {
    let text = match (shift.size, shift.kind) {
        (8, OctaveShiftKind::Up) => "8va",
        (8, OctaveShiftKind::Down) => "8vb",
        (15, OctaveShiftKind::Up) => "15ma",
        (15, OctaveShiftKind::Down) => "15mb",
        _ => "8va",
    };
    let is_stop = matches!(shift.kind, OctaveShiftKind::Stop);

    let font_size = line_spacing * 1.0;
    let font_id = egui::FontId::new(font_size, egui::FontFamily::Proportional);

    if is_stop {
        // Just the text at the stop point
        painter.text(
            egui::Pos2::new(start_x, y),
            egui::Align2::LEFT_TOP,
            text,
            font_id,
            color,
        );
    } else {
        // Text + dashed line + optional descender
        let text_end_x = start_x + line_spacing * 2.0;
        painter.text(
            egui::Pos2::new(start_x, y),
            egui::Align2::LEFT_TOP,
            text,
            font_id,
            color,
        );

        // Dashed line
        let dash_len = line_spacing * 0.3;
        let stroke = egui::Stroke::new(line_spacing * 0.06, color);
        let mut cx = text_end_x;
        while cx < end_x {
            let seg_end = (cx + dash_len).min(end_x);
            painter.line_segment(
                [egui::Pos2::new(cx, y), egui::Pos2::new(seg_end, y)],
                stroke,
            );
            cx += dash_len * 2.0;
        }

        // Descender at end
        painter.line_segment(
            [
                egui::Pos2::new(end_x, y),
                egui::Pos2::new(end_x, y - line_spacing * 0.6),
            ],
            stroke,
        );
    }
}

fn render_pedal(
    painter: &egui::Painter,
    start_x: f32,
    y: f32,
    end_x: f32,
    pedal: &Pedal,
    line_spacing: f32,
    color: egui::Color32,
) {
    let font_size = line_spacing * 1.0;
    let font_id = egui::FontId::new(font_size, egui::FontFamily::Proportional);

    match pedal.kind {
        PedalKind::Start => {
            painter.text(
                egui::Pos2::new(start_x, y),
                egui::Align2::LEFT_TOP,
                "Ped.",
                font_id,
                color,
            );
        }
        PedalKind::Stop => {
            painter.text(
                egui::Pos2::new(start_x, y),
                egui::Align2::LEFT_TOP,
                "*",
                font_id,
                color,
            );
        }
        PedalKind::Change | PedalKind::Continue | PedalKind::Resume => {
            if pedal.line {
                let stroke = egui::Stroke::new(line_spacing * 0.06, color);
                painter.line_segment(
                    [egui::Pos2::new(start_x, y), egui::Pos2::new(end_x, y)],
                    stroke,
                );
            }
        }
    }
}

use crate::notation::{Clef, KeySignature};
use eframe::egui;

/// Orden de sostenidos en armaduras: F, C, G, D, A, E, B.
const SHARP_ORDER: [i8; 7] = [3, 0, 4, 1, 5, 2, 6]; // staff positions (C=0)
/// Orden de bemoles en armaduras: B, E, A, D, G, C, F.
const FLAT_ORDER: [i8; 7] = [6, 2, 5, 1, 4, 0, 3]; // staff positions

/// Dibuja la armadura de clave después de la clave.
///
/// `x` — posición x inicial (después de la clave).
/// `staff_top_y` — coordenada y de la primera línea del staff.
/// `line_spacing` — espacio entre líneas.
/// `clef` — clave actual (afecta posiciones de octava para sostenidos/bemoles).
pub fn render_key_signature(
    painter: &egui::Painter,
    x: f32,
    staff_top_y: f32,
    key: &KeySignature,
    line_spacing: f32,
    clef: Clef,
    color: egui::Color32,
) -> f32 {
    if key.fifths == 0 && key.cancel.is_none() {
        return x; // C major / A minor — nothing to draw
    }

    let half_spacing = line_spacing / 2.0;
    let staff_origin_y = match clef {
        Clef::Treble => staff_top_y + line_spacing * 3.0, // 2nd line from bottom
        Clef::Bass => staff_top_y + line_spacing * 1.0,   // 2nd line from top
        _ => staff_top_y + line_spacing * 2.0,            // middle line
    };

    // Key sig accidentals are slightly smaller than regular accidentals
    let font_size = line_spacing * 3.4; // ~85% of 4.0
    let font_id = egui::FontId::new(font_size, egui::FontFamily::Name("Leland".into()));

    // Horizontal spacing between accidentals
    let accidental_spacing = line_spacing * 1.1;

    let mut cursor_x = x;

    // First: draw cancellation naturals if changing key
    if let Some(cancel_fifths) = key.cancel {
        if cancel_fifths != 0 {
            let cancel_order: &[i8] = if cancel_fifths > 0 {
                &SHARP_ORDER
            } else {
                &FLAT_ORDER
            };
            let count = cancel_fifths.unsigned_abs() as usize;
            for &sp in cancel_order.iter().take(count) {
                let y = staff_origin_y - sp as f32 * half_spacing;
                // Natural sign SMuFL U+E261
                let glyph = char::from_u32(0xE261).unwrap();
                painter.text(
                    egui::Pos2::new(cursor_x, y),
                    egui::Align2::CENTER_CENTER,
                    glyph.to_string(),
                    font_id.clone(),
                    color,
                );
                cursor_x += accidental_spacing;
            }
        }
    }

    // Then: draw the new key signature
    if key.fifths != 0 {
        let acc_order: &[i8] = if key.fifths > 0 {
            &SHARP_ORDER
        } else {
            &FLAT_ORDER
        };
        let count = key.fifths.unsigned_abs() as usize;

        // SMuFL glyphs: sharp U+E262, flat U+E260
        let glyph = if key.fifths > 0 {
            char::from_u32(0xE262).unwrap()
        } else {
            char::from_u32(0xE260).unwrap()
        };

        // For 7 accidentals, two octaves may be needed: add -7 offset for lower octave
        for (acc_idx, &sp) in acc_order.iter().take(count).enumerate() {
            // Standard octave; for 7 sharps use lower octave on some
            let octave_offset = if count >= 7 {
                // Sharps: F# C# G# D# A# E# B# — last two need lower octave
                if key.fifths > 0 && acc_idx >= 5 {
                    -7i8
                }
                // Flats: Bb Eb Ab Db Gb Cb Fb — last two need lower octave
                else if key.fifths < 0 && acc_idx >= 5 {
                    7i8
                } else {
                    0i8
                }
            } else {
                0i8
            };

            let y = staff_origin_y - (sp as f32 + octave_offset as f32) * half_spacing;
            painter.text(
                egui::Pos2::new(cursor_x, y),
                egui::Align2::CENTER_CENTER,
                glyph.to_string(),
                font_id.clone(),
                color,
            );
            cursor_x += accidental_spacing;
        }
    }

    // Return new x position after the key signature
    cursor_x + accidental_spacing * 0.5
}

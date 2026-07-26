use crate::notation::{Lyric, Syllabic};
use eframe::egui;

/// Renderiza letras debajo del staff.
pub fn render_lyrics(
    painter: &egui::Painter,
    notes: &[(f32, &[Lyric])], // (note_x, lyrics for that note)
    staff_bottom_y: f32,
    line_spacing: f32,
    color: egui::Color32,
) {
    let font_size = line_spacing * 0.67; // ~8pt
    let font_id = egui::FontId::new(font_size, egui::FontFamily::Proportional);

    // Group lyrics by verse number
    let mut verses: std::collections::BTreeMap<u8, Vec<(f32, &Lyric)>> =
        std::collections::BTreeMap::new();
    for (x, lyrics) in notes {
        for lyric in *lyrics {
            verses.entry(lyric.number).or_default().push((*x, lyric));
        }
    }

    for (verse_num, verse_lyrics) in verses.iter() {
        let verse_y = staff_bottom_y + line_spacing * 1.5 + *verse_num as f32 * line_spacing * 0.8;

        let _prev_x: Option<f32> = None;
        let _prev_syllabic: Option<Syllabic> = None;

        for (i, (x, lyric)) in verse_lyrics.iter().enumerate() {
            let mut text = lyric.text.clone();

            // Syllabic hyphenation
            match lyric.syllabic {
                Syllabic::Begin | Syllabic::Middle => {
                    text.push('-');
                }
                Syllabic::End | Syllabic::Single => {
                    // No hyphen
                }
            }

            painter.text(
                egui::Pos2::new(*x, verse_y),
                egui::Align2::CENTER_TOP,
                &text,
                font_id.clone(),
                color,
            );

            // Melisma extension line
            if lyric.extend && i + 1 < verse_lyrics.len() {
                let next_x = verse_lyrics[i + 1].0;
                let ext_y = verse_y + line_spacing * 0.3;
                let stroke = egui::Stroke::new(line_spacing * 0.05, color);
                painter.line_segment(
                    [
                        egui::Pos2::new(*x + line_spacing * 0.5, ext_y),
                        egui::Pos2::new(next_x - line_spacing * 0.5, ext_y),
                    ],
                    stroke,
                );
            }
        }
    }
}

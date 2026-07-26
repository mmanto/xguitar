use crate::notation::{Measure, MeasureElement, NoteFigure};

/// Compute proportionate measure widths based on rhythmic content.
///
/// Each measure receives a width proportional to its fill density:
/// shorter notes → narrower default spacing → wider measure.
/// Returns widths in the same order as `measures`.
pub fn compute_measure_widths(
    measures: &[Measure],
    total_width: f32,
    min_width: f32,
    max_width: f32,
) -> Vec<f32> {
    if measures.is_empty() {
        return Vec::new();
    }

    // Weight per measure: sum of duration weight for each note/rest
    let weights: Vec<f32> = measures
        .iter()
        .map(|m| {
            let w: f32 = m.elements.iter().map(|elem| elem_weight(elem)).sum();
            if w > 0.0 { w } else { 1.0 } // empty measure = minimum
        })
        .collect();

    let total_weight: f32 = weights.iter().sum();
    let mut widths: Vec<f32> = weights
        .iter()
        .map(|&w| {
            let raw = total_width * (w / total_weight);
            raw.clamp(min_width, max_width)
        })
        .collect();

    // Redistribute any leftover space proportionally
    let allocated: f32 = widths.iter().sum();
    let leftover = total_width - allocated;
    if leftover.abs() > 0.5 {
        let distrib = leftover / widths.len() as f32;
        for w in &mut widths {
            *w = (*w + distrib).clamp(min_width, max_width);
        }
    }

    widths
}

fn elem_weight(elem: &MeasureElement) -> f32 {
    match elem {
        MeasureElement::Note(note) => duration_weight(&note.figure) * dot_multiplier(&note.figure),
        MeasureElement::Rest(rest) => duration_weight(&rest.figure) * dot_multiplier(&rest.figure),
        MeasureElement::Chord(notes) => notes
            .first()
            .map(|n| duration_weight(&n.figure) * dot_multiplier(&n.figure))
            .unwrap_or(1.0),
        // Backup/Forward are spacers, skip them
        MeasureElement::Backup(_) | MeasureElement::Forward(_) => 0.0,
        MeasureElement::MultipleRest(mr) => mr.count as f32 * 2.0, // wide for multi-measure rest
    }
}

fn duration_weight(figure: &NoteFigure) -> f32 {
    match figure {
        NoteFigure::Breve => 4.0,
        NoteFigure::Whole => 2.0,
        NoteFigure::Half => 1.5,
        NoteFigure::Quarter => 1.0,
        NoteFigure::Eighth => 0.8,
        NoteFigure::Sixteenth => 0.7,
        NoteFigure::ThirtySecond => 0.6,
        NoteFigure::SixtyFourth => 0.5,
        NoteFigure::HundredTwentyEighth => 0.5,
    }
}

fn dot_multiplier(figure: &NoteFigure) -> f32 {
    match figure {
        NoteFigure::Breve => 1.0, // Breve doesn't take dots in practice
        _ => 1.0, // dots handled by Note.dotted but doesn't significantly affect width
    }
}

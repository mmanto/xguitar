use crate::notation::{Measure, MeasureElement};

// ────────────────────────────────────────────────────────────
//  Espaciado proporcional a la duración — ver ADR-006
// ────────────────────────────────────────────────────────────
//
// El ancho ideal de un elemento crece con la raíz cuadrada de su duración
// (fracción de negra), no linealmente: duplicar la duración agrega un
// incremento de ancho decreciente, no lo duplica. Es una forma más simple
// del mismo principio que un modelo logarítmico (concava, sin necesitar una
// "duración de referencia" arbitraria como ancla), y coincide con la
// convención de grabado de que las figuras más largas necesitan más aire
// pero no proporcionalmente a su duración real.
//
// `WIDTH_SHAPE_MIN`/`WIDTH_SHAPE_SCALE` están calibrados para que una negra
// (fracción 1.0) dé la misma forma que el antiguo espaciado uniforme
// (`line_spacing * 1.5`), de modo que una partitura compuesta solo por
// negras se vea casi igual que antes del cambio.
const WIDTH_SHAPE_MIN: f32 = 0.9;
const WIDTH_SHAPE_SCALE: f32 = 0.6;

/// Forma adimensional (sin `line_spacing`) del ancho ideal de un elemento
/// según su duración en fracción de negra. Se multiplica por `line_spacing`
/// en `measure_natural_width`; en `element_offsets` se usa directamente ya
/// que solo importan las proporciones relativas entre elementos.
fn width_shape(duration_quarters: f32) -> f32 {
    WIDTH_SHAPE_MIN + WIDTH_SHAPE_SCALE * duration_quarters.max(0.0).sqrt()
}

/// Multiplicador de duración por puntillos (0–3): 1.0, 1.5, 1.75, 1.875.
fn dot_multiplier(dots: u8) -> f32 {
    2.0 - 0.5f32.powi(dots as i32)
}

/// Forma de ancho ideal de un elemento de compás. `MultipleRest` recibe un
/// ancho fijo generoso (no participa del modelo de duración); `Backup`/
/// `Forward` no ocupan espacio propio.
fn element_width_shape(elem: &MeasureElement) -> f32 {
    match elem {
        MeasureElement::Backup(_) | MeasureElement::Forward(_) => 0.0,
        MeasureElement::MultipleRest(_) => 6.0,
        MeasureElement::Note(n) => {
            width_shape(n.figure.quarter_fraction() * dot_multiplier(n.dotted))
        }
        MeasureElement::Rest(r) => {
            width_shape(r.figure.quarter_fraction() * dot_multiplier(r.dotted))
        }
        MeasureElement::Chord(notes) => notes
            .first()
            .map(|n| width_shape(n.figure.quarter_fraction() * dot_multiplier(n.dotted)))
            .unwrap_or(width_shape(1.0)),
    }
}

/// Offsets horizontales (relativos al inicio del compás, en `0..measure_width`)
/// de cada elemento de `elements`, en el mismo orden. `Backup`/`Forward` no
/// consumen espacio propio y su offset no debe leerse (los llamadores ya los
/// saltean al iterar).
///
/// Cada elemento ocupa una porción de `measure_width` proporcional a su
/// `element_width_shape` (que ya refleja su duración — ver arriba) y se
/// posiciona en el centro de esa porción, igual que el modelo uniforme
/// anterior posicionaba cada nota en el centro de su slot de ancho fijo.
pub fn element_offsets(elements: &[MeasureElement], measure_width: f32) -> Vec<f32> {
    let shapes: Vec<f32> = elements.iter().map(element_width_shape).collect();
    let total: f32 = shapes.iter().sum();

    if total <= 0.0 {
        return vec![0.0; elements.len()];
    }

    let scale = measure_width / total;
    let mut offsets = Vec::with_capacity(elements.len());
    let mut cumulative = 0.0;
    for (elem, shape) in elements.iter().zip(shapes.iter()) {
        if matches!(elem, MeasureElement::Backup(_) | MeasureElement::Forward(_)) {
            offsets.push(0.0);
            continue;
        }
        let slot = shape * scale;
        offsets.push(cumulative + slot / 2.0);
        cumulative += slot;
    }
    offsets
}

/// Natural width of a measure based on the duration of its content (see
/// ADR-006). A measure with short notes is narrower; one with long notes or
/// many notes is wider — but width no longer grows linearly with note count
/// alone, it reflects how much rhythmic content actually needs to be shown.
pub fn measure_natural_width(measure: &Measure, line_spacing: f32) -> f32 {
    let base: f32 = measure
        .elements
        .iter()
        .filter(|e| !matches!(e, MeasureElement::Backup(_) | MeasureElement::Forward(_)))
        .map(|e| line_spacing * element_width_shape(e))
        .sum();
    // Extra space for accidentals
    let accidental_extra: f32 = measure
        .elements
        .iter()
        .filter_map(|e| match e {
            MeasureElement::Note(n) => n.accidental_override.map(|_| line_spacing * 1.2),
            _ => None,
        })
        .sum();
    // Small fixed margin at both ends of the measure
    let padding = line_spacing * 1.0;
    (base + accidental_extra + padding).max(line_spacing)
}

/// Compute measure widths with proportional justification (see ADR-006).
///
/// When a line has slack (natural widths sum to less than `total_width`),
/// every measure is stretched by the same factor — not just the last one —
/// so systems fill the available width evenly. Measures that would exceed
/// `max_width` are clamped and their leftover slack is redistributed across
/// the remaining unclamped measures. When there's no slack (natural widths
/// already meet or exceed `total_width`), measures keep their natural
/// (clamped to `min_width`) widths unscaled.
/// Returns widths in the same order as `measures`.
pub fn compute_measure_widths(
    measures: &[Measure],
    total_width: f32,
    min_width: f32,
    max_width: f32,
    line_spacing: f32,
) -> Vec<f32> {
    if measures.is_empty() {
        return Vec::new();
    }

    // Single measure: use full available width
    if measures.len() == 1 {
        return vec![total_width.max(min_width)];
    }

    let naturals: Vec<f32> = measures
        .iter()
        .map(|m| measure_natural_width(m, line_spacing).max(min_width))
        .collect();
    let natural_total: f32 = naturals.iter().sum();

    if natural_total >= total_width {
        // No slack to distribute: keep natural (clamped) widths, matching
        // the previous overflow behavior.
        return naturals;
    }

    // Slack available: stretch every measure proportionally, clamping to
    // max_width and redistributing clamped-away slack across the rest.
    let mut widths = naturals.clone();
    let mut locked = vec![false; widths.len()];
    let mut remaining_width = total_width;
    let mut remaining_natural = natural_total;

    loop {
        if remaining_natural <= 0.0 {
            break;
        }
        let stretch = remaining_width / remaining_natural;
        let mut newly_locked = false;
        for i in 0..widths.len() {
            if locked[i] {
                continue;
            }
            let candidate = naturals[i] * stretch;
            if candidate > max_width {
                widths[i] = max_width;
                locked[i] = true;
                remaining_width -= max_width;
                remaining_natural -= naturals[i];
                newly_locked = true;
            }
        }
        if !newly_locked {
            for i in 0..widths.len() {
                if !locked[i] {
                    widths[i] = naturals[i] * stretch;
                }
            }
            break;
        }
    }

    widths
}

/// Pack measures into lines that fit within `available_width`.
/// Returns Vec of (start_idx, end_idx_exclusive) per line.
///
/// Each measure's width for line-breaking is at least `min_measure_width`
/// to match the minimum enforced by `compute_measure_widths`.
/// If a single measure exceeds available_width, it gets its own line
/// (the stretching pass will compress it via max_width clamp).
pub fn break_measures_into_lines(
    measures: &[Measure],
    available_width: f32,
    line_spacing: f32,
    min_measure_width: f32,
) -> Vec<(usize, usize)> {
    let mut lines: Vec<(usize, usize)> = Vec::new();
    let mut line_start = 0;
    let mut accumulated = 0.0;

    for (i, measure) in measures.iter().enumerate() {
        let mw = measure_natural_width(measure, line_spacing).max(min_measure_width);

        if i > line_start && accumulated + mw > available_width {
            lines.push((line_start, i));
            line_start = i;
            accumulated = mw;
        } else {
            accumulated += mw;
        }
    }

    if line_start < measures.len() {
        lines.push((line_start, measures.len()));
    }

    lines
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::notation::{Note, NoteFigure, Rest};

    fn note(figure: NoteFigure) -> MeasureElement {
        MeasureElement::Note(Note {
            figure,
            ..Default::default()
        })
    }

    fn measure_with(elements: Vec<MeasureElement>) -> Measure {
        Measure {
            number: "1".into(),
            time_signature: crate::notation::TimeSignature {
                numerator: 4,
                denominator: 4,
                style: crate::notation::TimeSignatureStyle::Numeric,
            },
            key_signature: crate::notation::KeySignature::default(),
            elements,
            barline: crate::notation::Barline::default(),
            ending: None,
            directions: vec![],
            divisions: 1,
        }
    }

    #[test]
    fn ideal_width_grows_with_duration() {
        let ls = 12.0;
        let w = |fig: NoteFigure| line_spacing_width(fig, ls);
        assert!(w(NoteFigure::Eighth) < w(NoteFigure::Quarter));
        assert!(w(NoteFigure::Quarter) < w(NoteFigure::Half));
        assert!(w(NoteFigure::Half) < w(NoteFigure::Whole));
    }

    fn line_spacing_width(fig: NoteFigure, line_spacing: f32) -> f32 {
        line_spacing * width_shape(fig.quarter_fraction())
    }

    #[test]
    fn whole_note_measure_wider_than_single_eighth_but_narrower_than_eight_eighths() {
        let ls = 12.0;
        let whole_measure = measure_with(vec![note(NoteFigure::Whole)]);
        let one_eighth = measure_with(vec![note(NoteFigure::Eighth)]);
        let eight_eighths = measure_with((0..8).map(|_| note(NoteFigure::Eighth)).collect());

        let whole_w = measure_natural_width(&whole_measure, ls);
        let one_eighth_w = measure_natural_width(&one_eighth, ls);
        let eight_eighths_w = measure_natural_width(&eight_eighths, ls);

        assert!(
            whole_w > one_eighth_w,
            "whole note measure ({whole_w}) should be wider than a single eighth ({one_eighth_w})"
        );
        assert!(
            whole_w < eight_eighths_w,
            "whole note measure ({whole_w}) should be narrower than 8 eighths ({eight_eighths_w}), same total duration but more visual content"
        );
    }

    #[test]
    fn rest_uses_same_width_model_as_note() {
        let ls = 12.0;
        let note_measure = measure_with(vec![note(NoteFigure::Quarter)]);
        let rest_measure = measure_with(vec![MeasureElement::Rest(Rest {
            figure: NoteFigure::Quarter,
            dotted: 0,
            display_step: None,
            display_octave: None,
            measure: false,
        })]);
        assert_eq!(
            measure_natural_width(&note_measure, ls),
            measure_natural_width(&rest_measure, ls),
            "a quarter rest should take the same natural width as a quarter note"
        );
    }

    #[test]
    fn compute_measure_widths_stretches_every_measure_when_slack_available() {
        let ls = 12.0;
        let measures = vec![
            measure_with(vec![note(NoteFigure::Quarter), note(NoteFigure::Quarter)]),
            measure_with(vec![note(NoteFigure::Whole)]),
        ];
        let naturals: Vec<f32> = measures
            .iter()
            .map(|m| measure_natural_width(m, ls))
            .collect();
        let total_width = naturals.iter().sum::<f32>() * 2.0; // plenty of slack
        let widths = compute_measure_widths(&measures, total_width, ls * 0.5, ls * 20.0, ls);

        assert!(
            (widths[0] - naturals[0]).abs() > 0.01,
            "first measure should stretch, not just the last one"
        );
        assert!(
            (widths[1] - naturals[1]).abs() > 0.01,
            "second (last) measure should also stretch"
        );
        assert!(
            (widths.iter().sum::<f32>() - total_width).abs() < 1.0,
            "stretched widths should fill the available width"
        );
    }

    #[test]
    fn compute_measure_widths_keeps_natural_when_no_slack() {
        let ls = 12.0;
        let measures = vec![
            measure_with(vec![note(NoteFigure::Quarter)]),
            measure_with(vec![note(NoteFigure::Quarter)]),
        ];
        let naturals: Vec<f32> = measures
            .iter()
            .map(|m| measure_natural_width(m, ls))
            .collect();
        let total_width: f32 = naturals.iter().sum(); // exactly zero slack
        let widths = compute_measure_widths(&measures, total_width, ls * 0.5, ls * 20.0, ls);
        for (w, n) in widths.iter().zip(naturals.iter()) {
            assert!(
                (w - n).abs() < 0.01,
                "no slack: widths should match naturals"
            );
        }
    }

    #[test]
    fn element_offsets_length_matches_elements_and_ends_within_measure_width() {
        let elements = vec![
            note(NoteFigure::Quarter),
            note(NoteFigure::Eighth),
            note(NoteFigure::Eighth),
        ];
        let measure_width = 90.0;
        let offsets = element_offsets(&elements, measure_width);
        assert_eq!(offsets.len(), elements.len());
        assert!(
            offsets[2] < measure_width,
            "last offset should land within the measure"
        );
        assert!(
            offsets[0] < offsets[1] && offsets[1] < offsets[2],
            "offsets should be increasing"
        );
    }
}

use crate::notation::MeasureElement;

/// Un grupo de notas unidas por una barra de corchea.
#[derive(Clone, Debug)]
pub struct BeamGroup {
    /// Índice de inicio en el slice de elementos.
    pub start_idx: usize,
    /// Índice de fin (inclusive) en el slice de elementos.
    pub end_idx: usize,
    /// Nivel de beam (0 = octavo, 1 = dieciseisavo, etc.).
    pub level: u8,
}

/// Agrupa notas consecutivas (corcheas o más cortas) en grupos de beam,
/// respetando los pulsos del compás. En 4/4, las corcheas se agrupan de a 2.
///
/// `divisions` = duración de una negra en unidades de duration.
/// Un pulso = `divisions` unidades. El grupo se rompe al cruzar un pulso.
pub fn compute_beams(elements: &[MeasureElement], divisions: u32) -> Vec<BeamGroup> {
    let mut groups: Vec<BeamGroup> = Vec::new();
    let mut current_start: Option<usize> = None;
    let mut max_flags: u8 = 0;
    let mut accumulated: u32 = 0; // duration units since group start

    // Effective divisions: at least 1 to avoid division-by-zero / integer underflow
    let divs = if divisions == 0 { 4 } else { divisions };

    for (i, elem) in elements.iter().enumerate() {
        match elem {
            MeasureElement::Note(note) if note.figure.flag_count() > 0 && !note.grace => {
                let fc = note.figure.flag_count();
                let dur = note.figure.duration_divisions(divs);
                let effective_dur = if dur == 0 { 1 } else { dur };

                // Check if adding this note would cross a beat boundary
                // within an existing group.
                if current_start.is_some() && accumulated > 0 && accumulated.is_multiple_of(divs) {
                    // Beat boundary: close current group, start new one
                    let end = i - 1;
                    if end > current_start.unwrap() {
                        for level in 0..max_flags {
                            groups.push(BeamGroup {
                                start_idx: current_start.unwrap(),
                                end_idx: end,
                                level,
                            });
                        }
                    }
                    current_start = Some(i);
                    max_flags = fc;
                    accumulated = effective_dur;
                } else if current_start.is_none() {
                    current_start = Some(i);
                    max_flags = fc;
                    accumulated = effective_dur;
                } else {
                    max_flags = max_flags.max(fc);
                    accumulated += effective_dur;
                }
            }
            _ => {
                // Rests, chords, backup/forward — break beam group
                if let Some(start) = current_start {
                    let end = i - 1;
                    if end > start {
                        for level in 0..max_flags {
                            groups.push(BeamGroup {
                                start_idx: start,
                                end_idx: end,
                                level,
                            });
                        }
                    }
                    current_start = None;
                    max_flags = 0;
                    accumulated = 0;
                }
            }
        }
    }

    // Close final group
    if let Some(start) = current_start {
        let end = elements.len() - 1;
        if end > start {
            for level in 0..max_flags {
                groups.push(BeamGroup {
                    start_idx: start,
                    end_idx: end,
                    level,
                });
            }
        }
    }

    groups
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::notation::{Note, NoteFigure, StemDirection};

    fn eighth() -> MeasureElement {
        MeasureElement::Note(Note {
            figure: NoteFigure::Eighth,
            ..Default::default()
        })
    }

    fn quarter() -> MeasureElement {
        MeasureElement::Note(Note {
            figure: NoteFigure::Quarter,
            ..Default::default()
        })
    }

    #[test]
    fn eight_eighths_in_4_4_make_four_pairs() {
        // 8 corcheas en 4/4 → 4 grupos de 2
        let elements: Vec<MeasureElement> = (0..8).map(|_| eighth()).collect();
        let groups = compute_beams(&elements, 4); // divisions=4 (4/4)
        assert_eq!(groups.len(), 4, "4 beam groups for 8 eighths in 4/4");
        assert_eq!(groups[0].start_idx, 0);
        assert_eq!(groups[0].end_idx, 1);
        assert_eq!(groups[1].start_idx, 2);
        assert_eq!(groups[1].end_idx, 3);
        assert_eq!(groups[2].start_idx, 4);
        assert_eq!(groups[2].end_idx, 5);
        assert_eq!(groups[3].start_idx, 6);
        assert_eq!(groups[3].end_idx, 7);
    }

    #[test]
    fn quarter_breaks_beam_group() {
        // corchea + negra + corchea → un solo grupo no se forma (negra no lleva beam)
        let elements = vec![eighth(), quarter(), eighth()];
        let groups = compute_beams(&elements, 4);
        assert!(groups.is_empty(), "no beam groups: quarter breaks the run");
    }
    #[test]
    fn mixed_eighths_sixteenths_in_one_beat() {
        // corchea + semicorchea + semicorchea dentro de un pulso → 1 grupo,
        // pero con 2 niveles de beam (max_flags=2: uno para corchea, otro para semicorchea)
        let mut elements = vec![eighth()];
        elements.push(MeasureElement::Note(Note {
            figure: NoteFigure::Sixteenth,
            ..Default::default()
        }));
        elements.push(MeasureElement::Note(Note {
            figure: NoteFigure::Sixteenth,
            ..Default::default()
        }));
        let groups = compute_beams(&elements, 4);
        // eighth(2) + 16th(1) + 16th(1) = 4 = 1 beat, single group
        // Two beam levels: level 0 (eighth beam) + level 1 (sixteenth beam)
        assert_eq!(
            groups.len(),
            2,
            "two beam levels for mixed eighths+sixteenths"
        );
        // Both groups span the same note range
        assert_eq!(groups[0].start_idx, 0);
        assert_eq!(groups[0].end_idx, 2);
        assert_eq!(groups[0].level, 0);
        assert_eq!(groups[1].start_idx, 0);
        assert_eq!(groups[1].end_idx, 2);
        assert_eq!(groups[1].level, 1);
    }
}

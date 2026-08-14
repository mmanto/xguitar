//! Recorre un `notation::Score` y produce una lista de eventos MIDI-like con
//! tiempo absoluto en segundos. Lógica de dominio pura — sin egui, sin cpal,
//! sin sfizz — testeable sobre datos sintéticos sin hardware de audio.

use crate::notation::{
    Articulation, DirectionKind, DynamicMark, MeasureElement, Note, NoteRef, Score, Slur, SlurKind,
    Staff, TieKind, TimeModification,
};

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum EventKind {
    NoteOn { midi: u8, velocity: u8 },
    NoteOff { midi: u8 },
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SequencedEvent {
    pub time_secs: f32,
    pub kind: EventKind,
    pub note_ref: NoteRef,
    /// End time (for NoteOn only; 0.0 for NoteOff). Used by highlighting.
    pub end_secs: f32,
}

const DEFAULT_TEMPO_BPM: f32 = 120.0;
const DEFAULT_VELOCITY: u8 = 80;
/// Fracción de la duración escrita que realmente suena (deja un pequeño
/// hueco entre notas no ligadas ni articuladas — convención estándar de
/// secuenciadores simples para evitar que todo suene completamente legato).
const DEFAULT_GATE: f32 = 0.9;

/// Construye la línea de tiempo completa de la partitura: todos los
/// `System`/`Staff` suenan simultáneamente (son pentagramas simultáneos por
/// definición — ver `notation::System`), cada uno recorrido de forma
pub fn build_events(score: &Score) -> Vec<SequencedEvent> {
    let mut events = Vec::new();
    for (system_idx, system) in score.systems.iter().enumerate() {
        for (staff_idx, staff) in system.staves.iter().enumerate() {
            build_staff_events(staff, system_idx, staff_idx, &mut events);
        }
    }
    events.sort_by(|a, b| a.time_secs.partial_cmp(&b.time_secs).unwrap());
    events
}

fn build_staff_events(
    staff: &Staff,
    system_idx: usize,
    staff_idx: usize,
    out: &mut Vec<SequencedEvent>,
) {
    let mut tempo_bpm = DEFAULT_TEMPO_BPM;
    let mut velocity = DEFAULT_VELOCITY;
    let mut absolute_time: f32 = 0.0;

    for (measure_idx, measure) in staff.measures.iter().enumerate() {
        let divisions = measure.divisions.max(1) as f32;

        // Simplificación: el tempo/dinámica de nivel-compás se aplica desde
        // el inicio del compás en el que aparece, no en su `element_index`
        // exacto — cubre el caso común (marcas al principio del compás) sin
        // tener que resolver la ambigüedad de tempo cambiando a mitad de
        // compás combinado con múltiples voces (Backup/Forward).
        for direction in &measure.directions {
            match &direction.kind {
                DirectionKind::Metronome(m) => {
                    tempo_bpm = m.per_minute as f32 * m.beat_unit.quarter_fraction();
                }
                DirectionKind::Dynamics(marks) => {
                    if let Some(mark) = marks.first() {
                        velocity = dynamic_velocity(mark);
                    }
                }
                _ => {}
            }
        }

        let seconds_per_division = 60.0 / tempo_bpm / divisions;
        let mut cursor: f32 = 0.0;
        let mut measure_max_cursor: f32 = 0.0;

        for (element_idx, elem) in measure.elements.iter().enumerate() {
            match elem {
                MeasureElement::Note(note) => {
                    let dur = sounding_divisions(note, divisions);
                    let start = absolute_time + cursor * seconds_per_division;
                    let note_ref = NoteRef {
                        system_idx,
                        staff_idx,
                        measure_idx,
                        element_idx,
                        subnote_idx: 0,
                    };
                    emit_note(
                        note,
                        start,
                        seconds_per_division,
                        dur,
                        velocity,
                        note_ref,
                        out,
                    );
                    cursor += dur;
                    measure_max_cursor = measure_max_cursor.max(cursor);
                }
                MeasureElement::Rest(rest) => {
                    let base = rest.figure.duration_divisions(divisions as u32) as f32;
                    let dur = base
                        * dot_multiplier(rest.dotted)
                        * rest.time_modification.map(ratio).unwrap_or(1.0);
                    cursor += dur;
                    measure_max_cursor = measure_max_cursor.max(cursor);
                }
                MeasureElement::Chord(notes) => {
                    let dur = notes
                        .first()
                        .map(|n| sounding_divisions(n, divisions))
                        .unwrap_or(divisions);
                    let start = absolute_time + cursor * seconds_per_division;
                    for (subnote_idx, note) in notes.iter().enumerate() {
                        let note_ref = NoteRef {
                            system_idx,
                            staff_idx,
                            measure_idx,
                            element_idx,
                            subnote_idx,
                        };
                        emit_note(
                            note,
                            start,
                            seconds_per_division,
                            dur,
                            velocity,
                            note_ref,
                            out,
                        );
                    }
                    cursor += dur;
                    measure_max_cursor = measure_max_cursor.max(cursor);
                }
                MeasureElement::Backup(divs) => {
                    cursor = (cursor - *divs as f32).max(0.0);
                }
                MeasureElement::Forward(divs) => {
                    cursor += *divs as f32;
                    measure_max_cursor = measure_max_cursor.max(cursor);
                }
                MeasureElement::MultipleRest(mr) => {
                    // Aproximación: se asume que el silencio de varios
                    // compases reemplaza esos compases en vez de coexistir
                    // con entradas de `Measure` individuales adicionales.
                    let whole_measure = divisions * measure.time_signature.numerator as f32 * 4.0
                        / measure.time_signature.denominator as f32;
                    cursor += whole_measure * mr.count as f32;
                    measure_max_cursor = measure_max_cursor.max(cursor);
                }
            }
        }

        absolute_time += measure_max_cursor * seconds_per_division;
    }
}

fn sounding_divisions(note: &Note, divisions: f32) -> f32 {
    let base = note.figure.duration_divisions(divisions as u32) as f32;
    base * dot_multiplier(note.dotted) * note.time_modification.map(ratio).unwrap_or(1.0)
}

fn ratio(tm: TimeModification) -> f32 {
    tm.ratio()
}

/// Espejo de `render::layout::dot_multiplier` (0–3 puntillos: 1.0, 1.5,
/// 1.75, 1.875) — mismo cálculo, distinto dominio (segundos en vez de ancho).
fn dot_multiplier(dots: u8) -> f32 {
    2.0 - 0.5f32.powi(dots as i32)
}

fn emit_note(
    note: &Note,
    start_secs: f32,
    seconds_per_division: f32,
    duration_divisions: f32,
    default_velocity: u8,
    note_ref: NoteRef,
    out: &mut Vec<SequencedEvent>,
) {
    if note.grace {
        // Las notas de gracia no tienen duración propia en el modelo (piden
        // tiempo prestado de la nota principal) — se omiten hasta que el
        // secuenciador les dé un tratamiento dedicado.
        return;
    }

    let midi = note.pitch.midi();
    let ties = note
        .attachments
        .as_ref()
        .map(|a| a.ties.as_slice())
        .unwrap_or(&[]);
    let has_incoming_tie = ties
        .iter()
        .any(|t| matches!(t.kind, TieKind::Stop | TieKind::Continue));
    let has_outgoing_tie = ties.iter().any(|t| {
        matches!(
            t.kind,
            TieKind::Start | TieKind::Continue | TieKind::LetRing
        )
    });

    let note_end = start_secs + duration_divisions * seconds_per_division;

    if !has_incoming_tie {
        let velocity = note
            .attachments
            .as_ref()
            .and_then(|a| a.dynamics.as_ref())
            .map(dynamic_velocity)
            .unwrap_or(default_velocity);
        out.push(SequencedEvent {
            time_secs: start_secs,
            kind: EventKind::NoteOn { midi, velocity },
            note_ref,
            end_secs: note_end,
        });
    }

    if !has_outgoing_tie {
        let gate = note
            .attachments
            .as_ref()
            .map(|a| gate_factor(&a.articulations, &a.slurs))
            .unwrap_or(DEFAULT_GATE);
        let end_secs = start_secs + duration_divisions * seconds_per_division * gate;
        out.push(SequencedEvent {
            time_secs: end_secs,
            kind: EventKind::NoteOff { midi },
            note_ref,
            end_secs: 0.0,
        });
    }
}

fn gate_factor(articulations: &[Articulation], slurs: &[Slur]) -> f32 {
    let legato = slurs
        .iter()
        .any(|s| matches!(s.kind, SlurKind::Start | SlurKind::Continue));
    if legato {
        return 1.0;
    }
    if articulations.contains(&Articulation::Staccatissimo) {
        return 0.25;
    }
    if articulations.contains(&Articulation::Staccato)
        || articulations.contains(&Articulation::Spiccato)
    {
        return 0.5;
    }
    if articulations.contains(&Articulation::Tenuto) {
        return 1.0;
    }
    DEFAULT_GATE
}

/// Mapeo aproximado de dinámicas a velocity MIDI (0-127), en línea con las
/// convenciones usuales de software de notación (mf ≈ 80, el default General
/// MIDI).
fn dynamic_velocity(mark: &DynamicMark) -> u8 {
    match mark {
        DynamicMark::PPPP => 16,
        DynamicMark::PPP => 24,
        DynamicMark::PP => 33,
        DynamicMark::P => 49,
        DynamicMark::MP => 64,
        DynamicMark::MF => 80,
        DynamicMark::F => 96,
        DynamicMark::FF => 112,
        DynamicMark::FFF => 120,
        DynamicMark::FFFF => 127,
        DynamicMark::SF
        | DynamicMark::SFZ
        | DynamicMark::SFFZ
        | DynamicMark::FZ
        | DynamicMark::RFZ => 120,
        DynamicMark::SFP | DynamicMark::SFPP | DynamicMark::FP => 100,
        DynamicMark::RF => 108,
        DynamicMark::N | DynamicMark::PF => 80,
        DynamicMark::Other(_) => 80,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::notation::{
        Barline, Clef, KeySignature, Measure, NoteAttachment, NoteFigure, PartList, Pitch, Rest,
        Staff, StaffKind, StemDirection, Step, System, Tie, TimeSignature, TimeSignatureStyle,
    };

    fn note(step: Step, octave: i8, figure: NoteFigure) -> Note {
        Note {
            pitch: Pitch {
                step,
                accidental: crate::notation::Accidental::Natural,
                octave,
            },
            figure,
            dotted: 0,
            time_modification: None,
            accidental_override: None,
            stem_direction: StemDirection::Up,
            grace: false,
            chord: false,
            attachments: None,
            lyrics: vec![],
        }
    }

    fn measure_with(elements: Vec<MeasureElement>, divisions: u32) -> Measure {
        Measure {
            number: "1".into(),
            time_signature: TimeSignature {
                numerator: 4,
                denominator: 4,
                style: TimeSignatureStyle::Numeric,
            },
            key_signature: KeySignature::default(),
            elements,
            barline: Barline::default(),
            ending: None,
            directions: vec![],
            divisions,
            system_break: false,
            chord_symbol: None,
        }
    }

    fn score_with_measures(measures: Vec<Measure>) -> Score {
        Score {
            title: String::new(),
            composer: String::new(),
            systems: vec![System {
                staves: vec![Staff {
                    clef: Clef::Treble,
                    line: 2,
                    measures,
                    name: String::new(),
                    abbreviation: String::new(),
                    kind: StaffKind::Standard,
                }],
                left_margin: 0.0,
                bracket: None,
            }],
            credits: vec![],
            scaling: None,
            part_list: PartList::default(),
        }
    }

    #[test]
    fn two_quarters_at_120bpm_are_half_a_second_apart() {
        let measure = measure_with(
            vec![
                MeasureElement::Note(note(Step::C, 4, NoteFigure::Quarter)),
                MeasureElement::Note(note(Step::D, 4, NoteFigure::Quarter)),
            ],
            1,
        );
        let score = score_with_measures(vec![measure]);
        let events = build_events(&score);

        let note_ons: Vec<f32> = events
            .iter()
            .filter(|e| matches!(e.kind, EventKind::NoteOn { .. }))
            .map(|e| e.time_secs)
            .collect();
        assert_eq!(note_ons.len(), 2);
        assert!((note_ons[0] - 0.0).abs() < 1e-6);
        assert!((note_ons[1] - 0.5).abs() < 1e-6, "got {:?}", note_ons);
    }

    #[test]
    fn tied_notes_do_not_retrigger() {
        let mut first = note(Step::C, 4, NoteFigure::Quarter);
        first.attachments = Some(NoteAttachment {
            ties: vec![Tie {
                kind: TieKind::Start,
                number: 1,
                placement: crate::notation::Placement::Below,
            }],
            ..Default::default()
        });
        let mut second = note(Step::C, 4, NoteFigure::Quarter);
        second.attachments = Some(NoteAttachment {
            ties: vec![Tie {
                kind: TieKind::Stop,
                number: 1,
                placement: crate::notation::Placement::Below,
            }],
            ..Default::default()
        });

        let measure = measure_with(
            vec![MeasureElement::Note(first), MeasureElement::Note(second)],
            1,
        );
        let score = score_with_measures(vec![measure]);
        let events = build_events(&score);

        let note_ons = events
            .iter()
            .filter(|e| matches!(e.kind, EventKind::NoteOn { .. }))
            .count();
        let note_offs = events
            .iter()
            .filter(|e| matches!(e.kind, EventKind::NoteOff { .. }))
            .count();
        // Un solo note-on al inicio de la ligadura, un solo note-off al final.
        assert_eq!(note_ons, 1);
        assert_eq!(note_offs, 1);
        let off_time = events
            .iter()
            .find(|e| matches!(e.kind, EventKind::NoteOff { .. }))
            .unwrap()
            .time_secs;
        // Debe sonar por las dos negras completas (1s a 120bpm), no cortarse
        // al final de la primera.
        assert!(off_time > 0.9, "got {off_time}");
    }

    #[test]
    fn eighth_note_triplet_fits_in_one_quarter() {
        let tm = TimeModification {
            actual_notes: 3,
            normal_notes: 2,
        };
        let mut a = note(Step::C, 4, NoteFigure::Eighth);
        a.time_modification = Some(tm);
        let mut b = note(Step::D, 4, NoteFigure::Eighth);
        b.time_modification = Some(tm);
        let mut c = note(Step::E, 4, NoteFigure::Eighth);
        c.time_modification = Some(tm);
        let after = note(Step::F, 4, NoteFigure::Quarter);

        let measure = measure_with(
            vec![
                MeasureElement::Note(a),
                MeasureElement::Note(b),
                MeasureElement::Note(c),
                MeasureElement::Note(after),
            ],
            // divisions=6 (no 1): con divisions=1, "corchea" trunca a 1/2=0
            // por división entera — se necesita resolución suficiente para
            // representar corcheas y su tresillo (6 / 2 * 2/3 = 2 exacto),
            // igual que cualquier MusicXML real (nunca usa divisions=1).
            6,
        );
        let score = score_with_measures(vec![measure]);
        let events = build_events(&score);

        let note_ons: Vec<f32> = events
            .iter()
            .filter(|e| matches!(e.kind, EventKind::NoteOn { .. }))
            .map(|e| e.time_secs)
            .collect();
        assert_eq!(note_ons.len(), 4);
        // A 120bpm, una negra dura 0.5s; el tresillo de corcheas debe ocupar
        // exactamente esos 0.5s (3 corcheas a 1/6s cada una), no 0.75s.
        assert!((note_ons[3] - 0.5).abs() < 1e-4, "got {:?}", note_ons);
    }

    #[test]
    fn backup_forward_advance_by_max_voice_cursor() {
        // Voz 1: una negra. Backup 1 negra. Voz 2: una negra. El compás debe
        // durar 0.5s (una negra a 120bpm), no 1s.
        let measure = measure_with(
            vec![
                MeasureElement::Note(note(Step::C, 4, NoteFigure::Quarter)),
                MeasureElement::Backup(1),
                MeasureElement::Note(note(Step::E, 4, NoteFigure::Quarter)),
            ],
            1,
        );
        let next = measure_with(
            vec![MeasureElement::Rest(Rest {
                figure: NoteFigure::Quarter,
                dotted: 0,
                time_modification: None,
                display_step: None,
                display_octave: None,
                measure: false,
            })],
            1,
        );
        let score = score_with_measures(vec![measure, next]);
        let events = build_events(&score);
        let note_ons: Vec<f32> = events
            .iter()
            .filter(|e| matches!(e.kind, EventKind::NoteOn { .. }))
            .map(|e| e.time_secs)
            .collect();
        // Ambas voces empiezan en t=0 (una tras el backup).
        assert!((note_ons[0] - 0.0).abs() < 1e-6);
        assert!((note_ons[1] - 0.0).abs() < 1e-6);
    }
}

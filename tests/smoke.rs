use m_guitar::notation::{MeasureElement, Note, TieKind};
use m_guitar::parse_musicxml;

#[test]
fn parse_simple_musicxml() {
    let xml = std::fs::read_to_string("test-data/simple.musicxml").expect("read test file");
    let score = parse_musicxml(&xml).expect("parse musicxml");

    assert_eq!(score.systems.len(), 1);
    let system = &score.systems[0];
    assert_eq!(system.staves.len(), 1);
    let staff = &system.staves[0];
    assert_eq!(staff.measures.len(), 5, "5 measures");

    // Measure 1: 4 quarter notes + direction
    let m1 = &staff.measures[0];
    assert_eq!(m1.directions.len(), 1, "measure 1: direction");
    assert_eq!(m1.elements.len(), 4, "measure 1: 4 elements");

    // Measure 2: chord (3 notes → single Chord element)
    let m2 = &staff.measures[1];
    assert!(
        matches!(m2.elements[0], MeasureElement::Chord(_)),
        "measure 2: chord"
    );

    // Measure 3: whole rest
    let m3 = &staff.measures[2];
    assert!(
        matches!(m3.elements[0], MeasureElement::Rest(_)),
        "measure 3: rest"
    );

    // Measures 4-5: tied notes
    let m4_notes = extract_notes(&staff.measures[3]);
    let m5_notes = extract_notes(&staff.measures[4]);
    assert!(!m4_notes.is_empty(), "measure 4: notes");
    assert!(!m5_notes.is_empty(), "measure 5: notes");

    let has_start = m4_notes
        .iter()
        .filter_map(|n| n.attachments.as_ref())
        .flat_map(|a| &a.ties)
        .any(|t| matches!(t.kind, TieKind::Start));
    let has_stop = m5_notes
        .iter()
        .filter_map(|n| n.attachments.as_ref())
        .flat_map(|a| &a.ties)
        .any(|t| matches!(t.kind, TieKind::Stop));

    assert!(has_start, "measure 4: tie start");
    assert!(has_stop, "measure 5: tie stop");
}

fn extract_notes(measure: &m_guitar::notation::Measure) -> Vec<&Note> {
    measure
        .elements
        .iter()
        .filter_map(|e| match e {
            MeasureElement::Note(n) => Some(n),
            _ => None,
        })
        .collect()
}

#[test]
fn parse_test_1_xml() {
    let xml = std::fs::read_to_string("test-data/test.1.xml").expect("read test file");
    let score = parse_musicxml(&xml).expect("parse musicxml");

    assert_eq!(score.systems.len(), 1);
    let staff = &score.systems[0].staves[0];

    // Three measures
    assert_eq!(staff.measures.len(), 3);
    // Clef: Treble (G clef)
    assert_eq!(staff.clef, m_guitar::Clef::Treble, "treble clef");

    let m1 = &staff.measures[0];

    // 4/4 time signature
    assert_eq!(m1.time_signature.numerator, 4);
    assert_eq!(m1.time_signature.denominator, 4);

    // Key signature: C major (no accidentals)
    assert_eq!(m1.key_signature.fifths, 0);

    // Measure 1: four quarter notes: C5, D5, E5, F5 (with octave-shift -1)
    assert_eq!(m1.elements.len(), 4);
    let expected = [
        (m_guitar::Step::C, 5),
        (m_guitar::Step::D, 5),
        (m_guitar::Step::E, 5),
        (m_guitar::Step::F, 5),
    ];
    for (i, (step, octave)) in expected.iter().enumerate() {
        let note = match &m1.elements[i] {
            MeasureElement::Note(n) => n,
            other => panic!("element {i} is not a Note: {other:?}"),
        };
        assert_eq!(note.pitch.step, *step, "m1 note {i} step");
        assert_eq!(note.pitch.octave, *octave, "m1 note {i} octave");
        assert_eq!(
            note.figure,
            m_guitar::NoteFigure::Quarter,
            "m1 note {i} quarter"
        );
    }

    // Measure 2: eight eighth notes: C4, D4, E4, F4, G4, A4, B4, C5
    let m2 = &staff.measures[1];
    assert_eq!(m2.elements.len(), 8, "measure 2: 8 elements");
    let m2_expected = [
        (m_guitar::Step::C, 4),
        (m_guitar::Step::D, 4),
        (m_guitar::Step::E, 4),
        (m_guitar::Step::F, 4),
        (m_guitar::Step::G, 4),
        (m_guitar::Step::A, 4),
        (m_guitar::Step::B, 4),
        (m_guitar::Step::C, 5),
    ];
    for (i, (step, octave)) in m2_expected.iter().enumerate() {
        let note = match &m2.elements[i] {
            MeasureElement::Note(n) => n,
            other => panic!("m2 element {i} is not a Note: {other:?}"),
        };
        assert_eq!(note.pitch.step, *step, "m2 note {i} step");
        assert_eq!(note.pitch.octave, *octave, "m2 note {i} octave");
        assert_eq!(
            note.figure,
            m_guitar::NoteFigure::Eighth,
            "m2 note {i} eighth"
        );
    }

    // Measure 3: quarter + eighth + eighth + quarter + eighth + eighth
    // D5(negra) E5 F5(corcheas) G5(negra) A5 B5(corcheas)
    let m3 = &staff.measures[2];
    assert_eq!(m3.elements.len(), 6, "measure 3: 6 elements");
    let m3_expected = [
        (m_guitar::Step::D, 5, m_guitar::NoteFigure::Quarter),
        (m_guitar::Step::E, 5, m_guitar::NoteFigure::Eighth),
        (m_guitar::Step::F, 5, m_guitar::NoteFigure::Eighth),
        (m_guitar::Step::G, 5, m_guitar::NoteFigure::Quarter),
        (m_guitar::Step::A, 5, m_guitar::NoteFigure::Eighth),
        (m_guitar::Step::B, 5, m_guitar::NoteFigure::Eighth),
    ];
    for (i, (step, octave, fig)) in m3_expected.iter().enumerate() {
        let note = match &m3.elements[i] {
            MeasureElement::Note(n) => n,
            other => panic!("m3 element {i} is not a Note: {other:?}"),
        };
        assert_eq!(note.pitch.step, *step, "m3 note {i} step");
        assert_eq!(note.pitch.octave, *octave, "m3 note {i} octave");
        assert_eq!(note.figure, *fig, "m3 note {i} {:?}", fig);
    }
}

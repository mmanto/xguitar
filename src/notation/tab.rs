use super::{Accidental, NoteFigure, Step, TimeSignature};

// ────────────────────────────────────────────────────────────
//  TablatureStaff — dedicated tab representation
// ────────────────────────────────────────────────────────────

/// Un pentagrama de tablatura con afinación y compases.
#[derive(Clone, Debug)]
pub struct TablatureStaff {
    pub string_count: u8,
    pub tuning: Vec<TuningStep>,
    /// Cejilla (capo) en traste 0–12.
    pub capo: Option<u8>,
    pub measures: Vec<TabMeasure>,
}

impl Default for TablatureStaff {
    fn default() -> Self {
        Self {
            string_count: 6,
            tuning: standard_guitar_tuning(),
            capo: None,
            measures: Vec::new(),
        }
    }
}

/// Afinación estándar de guitarra: E2 A2 D3 G3 B3 E4.
pub fn standard_guitar_tuning() -> Vec<TuningStep> {
    vec![
        TuningStep {
            step: Step::E,
            alter: Accidental::Natural,
            octave: 2,
        },
        TuningStep {
            step: Step::A,
            alter: Accidental::Natural,
            octave: 2,
        },
        TuningStep {
            step: Step::D,
            alter: Accidental::Natural,
            octave: 3,
        },
        TuningStep {
            step: Step::G,
            alter: Accidental::Natural,
            octave: 3,
        },
        TuningStep {
            step: Step::B,
            alter: Accidental::Natural,
            octave: 3,
        },
        TuningStep {
            step: Step::E,
            alter: Accidental::Natural,
            octave: 4,
        },
    ]
}

/// Un paso de afinación: grado + alteración + octava.
#[derive(Clone, Copy, Debug)]
pub struct TuningStep {
    pub step: Step,
    pub alter: Accidental,
    pub octave: i8,
}

// ────────────────────────────────────────────────────────────
//  TabMeasure and TabElement
// ────────────────────────────────────────────────────────────

/// Compás de tablatura.
#[derive(Clone, Debug)]
pub struct TabMeasure {
    pub time_signature: TimeSignature,
    pub elements: Vec<TabElement>,
}

/// Elemento dentro de un compás de tablatura.
#[derive(Clone, Debug)]
pub enum TabElement {
    TabNote(TabNote),
    TabRest(TabRest),
    TabChord { notes: Vec<TabNote> },
}

/// Nota de tablatura.
#[derive(Clone, Debug)]
pub struct TabNote {
    /// Cuerda: 1 = más aguda, 6 = más grave.
    pub string: u8,
    /// Traste: 0 = al aire, 1–24 = pisado.
    pub fret: u8,
    pub figure: NoteFigure,
    pub dotted: u8,
    pub technique: Vec<TabTechnique>,
}

impl TabNote {
    pub fn new(string: u8, fret: u8, figure: NoteFigure) -> Self {
        Self {
            string,
            fret,
            figure,
            dotted: 0,
            technique: Vec::new(),
        }
    }
}

/// Silencio en tablatura.
#[derive(Clone, Debug)]
pub struct TabRest {
    pub figure: NoteFigure,
}

// ────────────────────────────────────────────────────────────
//  TabTechnique — técnicas de guitarra
// ────────────────────────────────────────────────────────────

/// Técnica específica de tablatura/guitarra.
#[derive(Clone, Debug)]
pub enum TabTechnique {
    /// Ligado ascendente (hammer-on).
    HammerOn,
    /// Ligado descendente (pull-off).
    PullOff,
    /// Bend con alteración en semitonos.
    Bend {
        alter: f32,
        pre_bend: bool,
        release: bool,
    },
    /// Slide entre trastes.
    Slide { kind: SlideKind },
    /// Vibrato normal.
    Vibrato,
    /// Vibrato amplio.
    WideVibrato,
    /// Tapping.
    Tap,
    /// Armónico.
    Harmonic { kind: HarmonicKind },
    /// Palm mute.
    PalmMute,
    /// Let ring (dejar sonar).
    LetRing,
    /// Nota fantasma (parenthesized).
    GhostNote,
    /// Nota muerta (X).
    DeadNote,
    /// Trino hasta un traste específico.
    Trill { fret: u8 },
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum SlideKind {
    /// Slide desde una nota indeterminada hacia la nota.
    Into,
    /// Slide desde la nota hacia una nota indeterminada.
    OutOf,
    /// Slide entre dos posiciones definidas.
    Shift,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum HarmonicKind {
    Natural,
    Artificial,
    Pinch,
    Tap,
    Semi,
}

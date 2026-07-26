use super::tuplet::Tuplet;

// ────────────────────────────────────────────────────────────
//  NoteAttachment — markings attached to a note
// ────────────────────────────────────────────────────────────

/// Marcas y ornamentos adjuntos a una nota.
#[derive(Clone, Debug, Default)]
pub struct NoteAttachment {
    pub articulations: Vec<Articulation>,
    pub ornaments: Vec<Ornament>,
    pub technical: Vec<Technical>,
    pub dynamics: Option<DynamicMark>,
    pub fermata: Option<Fermata>,
    pub ties: Vec<Tie>,
    pub slurs: Vec<Slur>,
    pub glissando: Option<Glissando>,
    pub arpeggiate: Option<Arpeggiate>,
    pub tremolo: Option<Tremolo>,
    /// Tresillo, quintillo, etc.
    pub tuplet: Option<Tuplet>,
}

// ────────────────────────────────────────────────────────────
//  Articulations
// ────────────────────────────────────────────────────────────

/// Articulación: modifica el ataque o duración de una nota.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Articulation {
    /// > — SMuFL U+E4A0
    Accent,
    /// ^ — SMuFL U+E4AC (marcato)
    StrongAccent,
    /// . — SMuFL U+E4A2 (staccato)
    Staccato,
    /// ▼ — SMuFL U+E4AA (staccatissimo)
    Staccatissimo,
    /// — — SMuFL U+E4A4 (tenuto)
    Tenuto,
    /// . + — (portato / mezzo-staccato)
    DetachedLegato,
    /// ' — SMuFL U+E4A8 (spiccato)
    Spiccato,
    /// ' — SMuFL U+E4B6 (breath mark / comma)
    BreathMark,
    /// // — SMuFL U+E4B8 (caesura / railroad tracks)
    Caesura,
    /// <> — SMuFL U+E4A2 + U+E4A3 (soft accent)
    SoftAccent,
    /// Jazz: scoop up to note
    Scoop,
    /// Jazz: plop / drop
    Plop,
    /// Jazz: doit (ascending fall-off)
    Doit,
    /// Jazz: fall-off (descending)
    Falloff,
}

impl Articulation {
    /// Glifo SMuFL principal para esta articulación.
    pub fn glyph(self) -> Option<char> {
        match self {
            Articulation::Accent => Some('\u{E4A0}'),
            Articulation::StrongAccent => Some('\u{E4AC}'),
            Articulation::Staccato => Some('\u{E4A2}'),
            Articulation::Staccatissimo => Some('\u{E4AA}'),
            Articulation::Tenuto => Some('\u{E4A4}'),
            Articulation::DetachedLegato => None, // composite: . + —
            Articulation::Spiccato => Some('\u{E4A8}'),
            Articulation::BreathMark => Some('\u{E4B6}'),
            Articulation::Caesura => Some('\u{E4B8}'),
            Articulation::SoftAccent => None, // composite
            Articulation::Scoop
            | Articulation::Plop
            | Articulation::Doit
            | Articulation::Falloff => None, // drawn as curves
        }
    }

    /// Posición preferida respecto a la nota.
    pub fn default_placement(self) -> Placement {
        match self {
            Articulation::Staccato
            | Articulation::Staccatissimo
            | Articulation::Tenuto
            | Articulation::DetachedLegato
            | Articulation::Spiccato
            | Articulation::Accent
            | Articulation::StrongAccent => {
                // If stem is up, place below; if stem is down, place above.
                // Simplified: default to above.
                Placement::Above
            }
            Articulation::SoftAccent
            | Articulation::BreathMark
            | Articulation::Caesura
            | Articulation::Scoop
            | Articulation::Plop
            | Articulation::Doit
            | Articulation::Falloff => Placement::Above,
        }
    }
}

// ────────────────────────────────────────────────────────────
//  Ornaments
// ────────────────────────────────────────────────────────────

/// Ornamento musical.
#[derive(Clone, PartialEq, Debug)]
pub enum Ornament {
    /// tr — SMuFL U+E566
    Trill,
    /// ~ — SMuFL U+E567
    Turn,
    /// Inverted turn — SMuFL U+E568
    InvertedTurn,
    /// Mordent — SMuFL U+E56C
    Mordent,
    /// Prall / inverted mordent — SMuFL U+E56B
    InvertedMordent,
    /// Trémolo de barras (1–8 marcas)
    Tremolo { marks: u8 },
    /// Línea ondulada (trill extension)
    WavyLine { start: bool, stop: bool },
}

impl Ornament {
    /// Glifo SMuFL para este ornamento.
    pub fn glyph(&self) -> Option<char> {
        match self {
            Ornament::Trill => Some('\u{E566}'),
            Ornament::Turn => Some('\u{E567}'),
            Ornament::InvertedTurn => Some('\u{E568}'),
            Ornament::Mordent => Some('\u{E56C}'),
            Ornament::InvertedMordent => Some('\u{E56B}'),
            Ornament::Tremolo { .. } | Ornament::WavyLine { .. } => None,
        }
    }
}

// ────────────────────────────────────────────────────────────
//  Technical
// ────────────────────────────────────────────────────────────

/// Técnica instrumental (dedos, arco, pedales, etc.).
#[derive(Clone, PartialEq, Debug)]
pub enum Technical {
    UpBow,
    DownBow,
    Harmonic,
    OpenString,
    ThumbPosition,
    Fingering(String),
    Pluck,
    DoubleTongue,
    TripleTongue,
    Stopped,
    SnapPizzicato,
    Fret(u8),
    String(u8),
    HammerOn,
    PullOff,
    Bend {
        alter: f32,
        pre_bend: bool,
        release: bool,
    },
    Tap,
    Heel,
    Toe,
    Fingernails,
    Other(String),
}

// ────────────────────────────────────────────────────────────
//  Dynamics
// ────────────────────────────────────────────────────────────

/// Marca dinámica.
#[derive(Clone, PartialEq, Debug)]
pub enum DynamicMark {
    PPPP,
    PPP,
    PP,
    P,
    MP,
    MF,
    F,
    FF,
    FFF,
    FFFF,
    SF,
    SFZ,
    SFFZ,
    SFP,
    SFPP,
    FP,
    RF,
    RFZ,
    FZ,
    N,
    PF,
    Other(String),
}

// ────────────────────────────────────────────────────────────
//  Fermata
// ────────────────────────────────────────────────────────────

/// Calderón (fermata).
#[derive(Clone, PartialEq, Debug)]
pub struct Fermata {
    pub shape: FermataShape,
    pub upright: bool,
    pub placement: Placement,
}

impl Default for Fermata {
    fn default() -> Self {
        Self {
            shape: FermataShape::Normal,
            upright: true,
            placement: Placement::Above,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum FermataShape {
    Normal,
    Angled,
    Square,
    DoubleAngled,
    DoubleSquare,
    Long,
    Short,
    Henze,
}

// ────────────────────────────────────────────────────────────
//  Ties and Slurs
// ────────────────────────────────────────────────────────────

/// Ligadura de prolongación (une dos notas del mismo pitch).
#[derive(Clone, PartialEq, Debug)]
pub struct Tie {
    pub kind: TieKind,
    pub number: u8,
    pub placement: Placement,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum TieKind {
    Start,
    Stop,
    Continue,
    LetRing,
}

/// Ligadura de expresión (une notas de distinto pitch).
#[derive(Clone, PartialEq, Debug)]
pub struct Slur {
    pub kind: SlurKind,
    pub number: u8,
    pub placement: Placement,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum SlurKind {
    Start,
    Stop,
    Continue,
}

// ────────────────────────────────────────────────────────────
//  Glissando / Arpeggiate
// ────────────────────────────────────────────────────────────

/// Glissando / portamento entre notas.
#[derive(Clone, PartialEq, Debug)]
pub struct Glissando {
    pub kind: GlissandoKind,
    pub line_type: LineType,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum GlissandoKind {
    Start,
    Stop,
}

/// Arpegio de acorde.
#[derive(Clone, PartialEq, Debug)]
pub struct Arpeggiate {
    pub direction: Option<ArpeggioDirection>,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ArpeggioDirection {
    Up,
    Down,
}

/// Trémolo entre dos notas (barras diagonales).
#[derive(Clone, PartialEq, Debug)]
pub struct Tremolo {
    pub marks: u8,
}

// ────────────────────────────────────────────────────────────
//  Placement and LineType
// ────────────────────────────────────────────────────────────

/// Colocación relativa al pentagrama.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Placement {
    Above,
    Below,
}

/// Tipo de línea para glissando y otros conectores.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum LineType {
    Solid,
    Dashed,
    Dotted,
    Wavy,
}

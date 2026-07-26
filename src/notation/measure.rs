use super::direction::Direction;
use super::{Barline, Ending, KeySignature, Note, Rest};

/// Estilo de indicación de compás.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum TimeSignatureStyle {
    Numeric, // 4/4, 3/4
    Common,  // C (SMuFL U+E08A)
    Cut,     // ₵ (SMuFL U+E08B)
}

/// Compás (time signature).
#[derive(Clone, Copy, Debug)]
pub struct TimeSignature {
    pub numerator: u8,
    pub denominator: u8,
    pub style: TimeSignatureStyle,
}

/// Elemento dentro de un compás.
#[derive(Clone, Debug)]
pub enum MeasureElement {
    Note(Note),
    Rest(Rest),
    Chord(Vec<Note>),
    Backup(u32),
    Forward(u32),
    MultipleRest(super::multirest::MultipleRest),
}

/// Un compás con su métrica, armadura y elementos.
#[derive(Clone, Debug)]
pub struct Measure {
    pub number: String,
    pub time_signature: TimeSignature,
    pub key_signature: KeySignature,
    pub elements: Vec<MeasureElement>,
    pub barline: Barline,
    pub ending: Option<Ending>,
    /// Direcciones a nivel de compás: dinámicas, texto, marcas de ensayo, etc.
    pub directions: Vec<Direction>,
}

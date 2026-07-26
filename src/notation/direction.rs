use super::Placement;
use crate::notation::NoteFigure;

// ────────────────────────────────────────────────────────────
//  Direction — attached to a measure
// ────────────────────────────────────────────────────────────

/// Una dirección musical (dinámicas, texto, marcas de ensayo, etc.).
#[derive(Clone, Debug)]
pub struct Direction {
    pub placement: Placement,
    pub staff: u8,
    pub kind: DirectionKind,
}

#[derive(Clone, Debug)]
pub enum DirectionKind {
    Dynamics(Vec<DynamicMark>),
    Wedge(Wedge),
    Words(String),
    Rehearsal(String),
    Metronome(Metronome),
    OctaveShift(OctaveShift),
    Pedal(Pedal),
    Dashes(Dashes),
    Bracket(Bracket),
}

// ────────────────────────────────────────────────────────────
//  DynamicMark (Phase 2 — referenced here for Direction)
// ────────────────────────────────────────────────────────────

pub use crate::notation::DynamicMark;

// ────────────────────────────────────────────────────────────
//  Wedge (hairpin: crescendo / diminuendo)
// ────────────────────────────────────────────────────────────

#[derive(Clone, Debug)]
pub struct Wedge {
    pub kind: WedgeKind,
    pub niente: bool,
    /// Spread in tenths (MusicXML default-y relative positioning)
    pub spread: f32,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum WedgeKind {
    Crescendo,
    Diminuendo,
}

// ────────────────────────────────────────────────────────────
//  Metronome
// ────────────────────────────────────────────────────────────

#[derive(Clone, Debug)]
pub struct Metronome {
    pub beat_unit: NoteFigure,
    pub per_minute: u16,
    pub parentheses: bool,
}

// ────────────────────────────────────────────────────────────
//  OctaveShift (8va, 8vb, 15ma, 15mb)
// ────────────────────────────────────────────────────────────

#[derive(Clone, Debug)]
pub struct OctaveShift {
    /// 8 or 15
    pub size: u8,
    pub kind: OctaveShiftKind,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum OctaveShiftKind {
    Up,
    Down,
    Stop,
}

// ────────────────────────────────────────────────────────────
//  Pedal
// ────────────────────────────────────────────────────────────

#[derive(Clone, Debug)]
pub struct Pedal {
    pub kind: PedalKind,
    pub line: bool,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum PedalKind {
    Start,
    Stop,
    Change,
    Continue,
    Resume,
}

// ────────────────────────────────────────────────────────────
//  Dashes
// ────────────────────────────────────────────────────────────

#[derive(Clone, Debug)]
pub struct Dashes {
    pub kind: DashesKind,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum DashesKind {
    Start,
    Stop,
    Continue,
}

// ────────────────────────────────────────────────────────────
//  Bracket
// ────────────────────────────────────────────────────────────

#[derive(Clone, Debug)]
pub struct Bracket {
    pub kind: BracketKind,
    pub line_end: LineEnd,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum BracketKind {
    Start,
    Stop,
    Continue,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum LineEnd {
    Up,
    Down,
    Both,
    None,
    Arrow,
}

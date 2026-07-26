use super::{NoteFigure, Step};

/// Un silencio musical.
#[derive(Clone, Debug)]
pub struct Rest {
    pub figure: NoteFigure,
    /// Cantidad de puntillos (0–3).
    pub dotted: u8,
    /// Posición visual en el staff (opcional).
    pub display_step: Option<Step>,
    pub display_octave: Option<i8>,
    /// Silencio de compás completo (centrado, ignora otros elementos).
    pub measure: bool,
}

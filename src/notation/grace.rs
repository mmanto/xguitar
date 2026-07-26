use super::Pitch;

/// Nota de adorno (gracia).
#[derive(Clone, Debug)]
pub struct GraceNote {
    pub pitch: Pitch,
    /// true = acciaccatura (slash), false = appoggiatura (no slash).
    pub slash: bool,
    /// Porcentaje robado de la nota anterior (0.0–1.0).
    pub steal_previous: f32,
    /// Porcentaje robado de la nota siguiente (0.0–1.0).
    pub steal_following: f32,
    /// La nota de gracia tiene su propia duración rítmica.
    pub make_time: bool,
}

impl Default for GraceNote {
    fn default() -> Self {
        Self {
            pitch: Pitch {
                step: super::Step::C,
                accidental: super::Accidental::Natural,
                octave: 4,
            },
            slash: true,
            steal_previous: 0.5,
            steal_following: 0.5,
            make_time: false,
        }
    }
}

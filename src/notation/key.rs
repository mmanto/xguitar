/// Armadura de clave.
#[derive(Clone, Debug)]
pub struct KeySignature {
    /// Número de sostenidos (+) o bemoles (-). Rango: -7 (Cb) a +7 (C#).
    pub fifths: i8,
    pub mode: KeyMode,
    /// Armadura previa a cancelar con becuadros antes de la nueva.
    pub cancel: Option<i8>,
}

impl Default for KeySignature {
    fn default() -> Self {
        Self {
            fifths: 0,
            mode: KeyMode::None,
            cancel: None,
        }
    }
}

/// Modo de la armadura.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum KeyMode {
    Major,
    Minor,
    Dorian,
    Phrygian,
    Lydian,
    Mixolydian,
    Aeolian,
    Ionian,
    Locrian,
    None,
}

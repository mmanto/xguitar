use crate::notation::Clef;

/// Alteración de una nota.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Accidental {
    Natural,
    Sharp,
    Flat,
    DoubleSharp,
    DoubleFlat,
}

impl Accidental {
    /// Glifo SMuFL para esta alteración.
    pub fn glyph(self) -> char {
        match self {
            Accidental::Natural => '\u{E261}',
            Accidental::Sharp => '\u{E262}',
            Accidental::Flat => '\u{E260}',
            Accidental::DoubleSharp => '\u{E264}',
            Accidental::DoubleFlat => '\u{E266}',
        }
    }

    /// Semitonos de alteración: Natural=0, Sharp=+1, Flat=-1, DoubleSharp=+2, DoubleFlat=-2.
    pub fn alter_semitones(self) -> i8 {
        match self {
            Accidental::Natural => 0,
            Accidental::Sharp => 1,
            Accidental::Flat => -1,
            Accidental::DoubleSharp => 2,
            Accidental::DoubleFlat => -2,
        }
    }
}

/// Grado de la escala (C, D, E, F, G, A, B).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Step {
    C,
    D,
    E,
    F,
    G,
    A,
    B,
}

/// Altura absoluta: clase + alteración + octava.
///
/// La octava 4 es la central (`middle C` = C4).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Pitch {
    pub step: Step,
    pub accidental: Accidental,
    /// 0 = sub-contra, 4 = middle C
    pub octave: i8,
}

impl Pitch {
    /// Posición en el staff relativa a la línea de referencia de la clave.
    ///
    /// - Clave de Sol: B4 (tercera línea) = 0.
    /// - Clave de Fa: D3 (tercera línea) = 0.
    ///
    /// Valores positivos indican notas más agudas (más arriba en el staff).
    /// La unidad es el paso entre línea y espacio adyacentes.
    pub fn staff_position(&self, clef: Clef) -> i8 {
        let diatonic = self.diatonic_index();
        let reference = clef.reference_diatonic();
        (diatonic - reference) as i8
    }

    /// Índice MIDI (middle C = 60).
    pub fn midi(&self) -> u8 {
        let step_semitones: i8 = match self.step {
            Step::C => 0,
            Step::D => 2,
            Step::E => 4,
            Step::F => 5,
            Step::G => 7,
            Step::A => 9,
            Step::B => 11,
        };
        let accidental_offset: i8 = match self.accidental {
            Accidental::Natural => 0,
            Accidental::Sharp => 1,
            Accidental::Flat => -1,
            Accidental::DoubleSharp => 2,
            Accidental::DoubleFlat => -2,
        };
        (12 * (self.octave + 1) + step_semitones + accidental_offset) as u8
    }

    /// Nombre en español (ej. "Do sostenido").
    pub fn name_es(&self) -> String {
        let step_name = match self.step {
            Step::C => "Do",
            Step::D => "Re",
            Step::E => "Mi",
            Step::F => "Fa",
            Step::G => "Sol",
            Step::A => "La",
            Step::B => "Si",
        };
        let accidental_name = match self.accidental {
            Accidental::Natural => "",
            Accidental::Sharp => " sostenido",
            Accidental::Flat => " bemol",
            Accidental::DoubleSharp => " doble sostenido",
            Accidental::DoubleFlat => " doble bemol",
        };
        format!("{}{}", step_name, accidental_name)
    }

    /// Índice diatónico absoluto: C0 = 0, D0 = 1, …, B0 = 6, C1 = 7, …
    fn diatonic_index(&self) -> i32 {
        let step_idx: i32 = match self.step {
            Step::C => 0,
            Step::D => 1,
            Step::E => 2,
            Step::F => 3,
            Step::G => 4,
            Step::A => 5,
            Step::B => 6,
        };
        step_idx + 7 * self.octave as i32
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn midi_middle_c() {
        let c4 = Pitch {
            step: Step::C,
            accidental: Accidental::Natural,
            octave: 4,
        };
        assert_eq!(c4.midi(), 60);
    }

    #[test]
    fn midi_a4() {
        let a4 = Pitch {
            step: Step::A,
            accidental: Accidental::Natural,
            octave: 4,
        };
        assert_eq!(a4.midi(), 69);
    }

    #[test]
    fn midi_c_sharp_4() {
        let cs4 = Pitch {
            step: Step::C,
            accidental: Accidental::Sharp,
            octave: 4,
        };
        assert_eq!(cs4.midi(), 61);
    }

    #[test]
    fn midi_b_flat_3() {
        let bb3 = Pitch {
            step: Step::B,
            accidental: Accidental::Flat,
            octave: 3,
        };
        assert_eq!(bb3.midi(), 58);
    }

    #[test]
    fn staff_position_treble_b4() {
        let b4 = Pitch {
            step: Step::B,
            accidental: Accidental::Natural,
            octave: 4,
        };
        assert_eq!(b4.staff_position(Clef::Treble), 0);
    }

    #[test]
    fn staff_position_treble_c4() {
        let c4 = Pitch {
            step: Step::C,
            accidental: Accidental::Natural,
            octave: 4,
        };
        // C4 is one ledger line below the treble staff (position -6)
        assert_eq!(c4.staff_position(Clef::Treble), -6);
    }

    #[test]
    fn staff_position_treble_e4() {
        let e4 = Pitch {
            step: Step::E,
            accidental: Accidental::Natural,
            octave: 4,
        };
        // E4 is the bottom line of treble staff (position -4)
        assert_eq!(e4.staff_position(Clef::Treble), -4);
    }

    #[test]
    fn staff_position_treble_f5() {
        let f5 = Pitch {
            step: Step::F,
            accidental: Accidental::Natural,
            octave: 5,
        };
        // F5 is the top line of treble staff (position +4)
        assert_eq!(f5.staff_position(Clef::Treble), 4);
    }

    #[test]
    fn staff_position_bass_d3() {
        let d3 = Pitch {
            step: Step::D,
            accidental: Accidental::Natural,
            octave: 3,
        };
        assert_eq!(d3.staff_position(Clef::Bass), 0);
    }

    #[test]
    fn staff_position_bass_middle_c() {
        let c4 = Pitch {
            step: Step::C,
            accidental: Accidental::Natural,
            octave: 4,
        };
        // Middle C in bass clef: one ledger line above (position +6)
        assert_eq!(c4.staff_position(Clef::Bass), 6);
    }

    #[test]
    fn staff_position_bass_g2() {
        let g2 = Pitch {
            step: Step::G,
            accidental: Accidental::Natural,
            octave: 2,
        };
        // G2 is the bottom line of bass staff (position -4)
        assert_eq!(g2.staff_position(Clef::Bass), -4);
    }
}

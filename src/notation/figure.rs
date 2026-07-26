/// Figura rítmica que define la duración relativa de una nota.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum NoteFigure {
    /// Cuadrada / breve (double whole note) — SMuFL U+E0A1
    Breve,
    /// Redonda (whole note) — SMuFL U+E1D2
    Whole,
    /// Blanca (half note) — SMuFL U+E1D3
    Half,
    /// Negra (quarter note) — SMuFL U+E1D5
    Quarter,
    /// Corchea (eighth note) — SMuFL U+E1D7
    Eighth,
    /// Semicorchea (sixteenth note) — SMuFL U+E1D9
    Sixteenth,
    /// Fusa (thirty-second note) — SMuFL U+E1DB
    ThirtySecond,
    /// Semifusa (sixty-fourth note) — SMuFL U+E1DD
    SixtyFourth,
    /// Garrapatea (hundred-twenty-eighth note) — same notehead as Quarter
    HundredTwentyEighth,
}

impl NoteFigure {
    /// Glifo SMuFL correspondiente a esta figura (renderizar con fuente Leland).
    pub fn glyph(self) -> char {
        match self {
            NoteFigure::Breve => '\u{E1D1}',
            NoteFigure::Whole => '\u{E1D2}',
            NoteFigure::Half => '\u{E1D3}',
            NoteFigure::Quarter => '\u{E1D5}',
            NoteFigure::Eighth => '\u{E1D7}',
            NoteFigure::Sixteenth => '\u{E1D9}',
            NoteFigure::ThirtySecond => '\u{E1DB}',
            NoteFigure::SixtyFourth => '\u{E1DD}',
            NoteFigure::HundredTwentyEighth => '\u{E1DD}', // same glyph as 64th, 5 flags
        }
    }

    /// Glifo SMuFL solo de la cabeza de nota (sin plica ni corchete).
    pub fn notehead_glyph(self) -> char {
        match self {
            NoteFigure::Breve => '\u{E0A1}',
            NoteFigure::Whole => '\u{E0A2}',
            NoteFigure::Half => '\u{E0A3}',
            _ => '\u{E0A4}', // Quarter, Eighth, etc. — misma cabeza negra
        }
    }

    /// Nombre en español.
    pub fn name_es(self) -> &'static str {
        match self {
            NoteFigure::Breve => "Cuadrada",
            NoteFigure::Whole => "Redonda",
            NoteFigure::Half => "Blanca",
            NoteFigure::Quarter => "Negra",
            NoteFigure::Eighth => "Corchea",
            NoteFigure::Sixteenth => "Semicorchea",
            NoteFigure::ThirtySecond => "Fusa",
            NoteFigure::SixtyFourth => "Semifusa",
            NoteFigure::HundredTwentyEighth => "Garrapatea",
        }
    }

    /// Nombre en inglés.
    pub fn name_en(self) -> &'static str {
        match self {
            NoteFigure::Breve => "Breve",
            NoteFigure::Whole => "Whole",
            NoteFigure::Half => "Half",
            NoteFigure::Quarter => "Quarter",
            NoteFigure::Eighth => "Eighth",
            NoteFigure::Sixteenth => "Sixteenth",
            NoteFigure::ThirtySecond => "Thirty-second",
            NoteFigure::SixtyFourth => "Sixty-fourth",
            NoteFigure::HundredTwentyEighth => "Hundred-twenty-eighth",
        }
    }

    /// Todas las figuras, en orden de mayor a menor duración.
    pub fn all() -> [NoteFigure; 9] {
        [
            NoteFigure::Breve,
            NoteFigure::Whole,
            NoteFigure::Half,
            NoteFigure::Quarter,
            NoteFigure::Eighth,
            NoteFigure::Sixteenth,
            NoteFigure::ThirtySecond,
            NoteFigure::SixtyFourth,
            NoteFigure::HundredTwentyEighth,
        ]
    }

    /// Cantidad de corchetes/banderas que lleva esta figura.
    /// 0 = solo plica, 1+ = plica con N banderas.
    pub fn flag_count(self) -> u8 {
        match self {
            NoteFigure::Breve => 0,
            NoteFigure::Whole => 0,
            NoteFigure::Half => 0,
            NoteFigure::Quarter => 0,
            NoteFigure::Eighth => 1,
            NoteFigure::Sixteenth => 2,
            NoteFigure::ThirtySecond => 3,
            NoteFigure::SixtyFourth => 4,
            NoteFigure::HundredTwentyEighth => 5,
        }
    }

    /// Duración en divisiones (quarters=divisions, others proportional).
    pub fn duration_divisions(self, divisions: u32) -> u32 {
        match self {
            NoteFigure::Breve => divisions * 8,
            NoteFigure::Whole => divisions * 4,
            NoteFigure::Half => divisions * 2,
            NoteFigure::Quarter => divisions,
            NoteFigure::Eighth => divisions / 2,
            NoteFigure::Sixteenth => divisions / 4,
            NoteFigure::ThirtySecond => divisions / 8,
            NoteFigure::SixtyFourth => divisions / 16,
            NoteFigure::HundredTwentyEighth => divisions / 32,
        }
    }
}

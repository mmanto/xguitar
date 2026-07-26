/// Clave musical que define la posición tonal en el pentagrama.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Clef {
    /// Clave de Sol (G clef) — SMuFL U+E050
    Treble,
    /// Clave de Fa (F clef) — SMuFL U+E062
    Bass,
    /// Clave de Do en tercera (C clef) — SMuFL U+E05C
    Alto,
    /// Clave de Do en cuarta (C clef) — SMuFL U+E05C
    Tenor,
    /// Clave de percusión (neutral clef) — SMuFL U+E069
    Percussion,
    /// Clave de tablatura (TAB) — SMuFL U+E06D
    Tab,
}

impl Clef {
    /// Glifo SMuFL correspondiente a esta clave (renderizar con fuente Leland).
    pub fn glyph(self) -> char {
        match self {
            Clef::Treble => '\u{E050}',
            Clef::Bass => '\u{E062}',
            Clef::Alto => '\u{E05C}',
            Clef::Tenor => '\u{E05C}', // Same glyph as Alto, different line position
            Clef::Percussion => '\u{E069}',
            Clef::Tab => '\u{E06D}',
        }
    }

    /// Nombre en español.
    pub fn name_es(self) -> &'static str {
        match self {
            Clef::Treble => "Sol",
            Clef::Bass => "Fa",
            Clef::Alto => "Do en tercera",
            Clef::Tenor => "Do en cuarta",
            Clef::Percussion => "Percusión",
            Clef::Tab => "Tablatura",
        }
    }

    /// Nombre en inglés.
    pub fn name_en(self) -> &'static str {
        match self {
            Clef::Treble => "Treble",
            Clef::Bass => "Bass",
            Clef::Alto => "Alto",
            Clef::Tenor => "Tenor",
            Clef::Percussion => "Percussion",
            Clef::Tab => "Tablature",
        }
    }

    /// Línea por defecto para esta clave (1-5, contando desde abajo).
    pub fn default_line(self) -> i8 {
        match self {
            Clef::Treble => 2,
            Clef::Bass => 4,
            Clef::Alto => 3,
            Clef::Tenor => 4,
            Clef::Percussion => 3,
            Clef::Tab => 5,
        }
    }

    /// Índice diatónico de la línea de referencia.
    ///
    /// - Clave de Sol (line 2, B4 on middle line): diatónico 34
    /// - Clave de Fa (line 4, D3 on middle line): diatónico 22
    /// - Clave de Do en 3ra (middle C = C4): diatónico 28
    /// - Clave de Do en 4ta (C4): diatónico 28
    /// - Percusión (line 3): diatónico 28
    /// - Tab (line 5): diatónico 28
    pub(crate) fn reference_diatonic(self) -> i32 {
        match self {
            Clef::Treble => 34,     // B4 = 6 + 7*4 (middle line, line 2 is G4)
            Clef::Bass => 22,       // D3 = 1 + 7*3 (middle line, line 4 is F3)
            Clef::Alto => 28,       // C4 = 0 + 7*4 (middle C)
            Clef::Tenor => 28,      // C4
            Clef::Percussion => 28, // neutral
            Clef::Tab => 28,        // neutral
        }
    }
}

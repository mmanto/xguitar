/// Una sílaba de letra/canción asociada a una nota.
#[derive(Clone, Debug)]
pub struct Lyric {
    /// Número de verso (1, 2, 3...).
    pub number: u8,
    pub syllabic: Syllabic,
    pub text: String,
    /// Línea de extensión de melisma (underscore).
    pub extend: bool,
}

/// Tipo de sílaba en una letra.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Syllabic {
    Single,
    Begin,
    End,
    Middle,
}

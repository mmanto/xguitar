use super::Staff;

/// Un sistema: agrupa pentagramas que suenan simultáneamente.
#[derive(Clone, Debug)]
pub struct System {
    pub staves: Vec<Staff>,
    /// Margen izquierdo del sistema (indentación).
    pub left_margin: f32,
    /// Corchete o llave para grupos de partes.
    pub bracket: Option<GroupBracket>,
}

/// Tipo de corchete/llave para agrupar pentagramas.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum GroupBracket {
    Brace,   // llave { — piano, arpa
    Bracket, // corchete [ — grupos de instrumentos
    Line,    // línea simple
    Square,  // corchete cuadrado
}

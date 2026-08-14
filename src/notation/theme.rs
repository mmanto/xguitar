/// Símbolo de acorde para mostrar (ej. "Am", "G7", "F#m7b5").
/// String simple por flexibilidad; no se hace análisis armónico automático.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ChordSymbol(pub String);

/// Una progresión: secuencia de acordes, cada uno con duración en compases.
#[derive(Clone, Debug)]
pub struct ChordProgression {
    pub chords: Vec<ChordStep>,
}

#[derive(Clone, Debug)]
pub struct ChordStep {
    pub symbol: ChordSymbol,
    /// Duración en cantidad de compases que ocupa este acorde.
    pub measures: u32,
}

/// Tipos predefinidos de sección musical.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SectionKind {
    Intro,
    Verse,
    Chorus,
    Bridge,
    Solo,
    Outro,
    /// Sección con nombre libre.
    Custom,
}

impl SectionKind {
    /// Color por defecto para el bloque de la sección en el mapa de temas.
    pub fn default_color(&self) -> &'static str {
        match self {
            SectionKind::Intro => "#4A90D9",
            SectionKind::Verse => "#50C878",
            SectionKind::Chorus => "#E8A838",
            SectionKind::Bridge => "#9B59B6",
            SectionKind::Solo => "#E74C3C",
            SectionKind::Outro => "#95A5A6",
            SectionKind::Custom => "#5DADE2",
        }
    }
}

/// Una sección de un tema: rango de compases + progresión de acordes.
#[derive(Clone, Debug)]
pub struct Section {
    pub kind: SectionKind,
    /// Nombre visible; para `Custom` es libre, para los demás tiene default i18n.
    pub label: String,
    /// Índice del primer compás (0-based, absoluto dentro del Score).
    /// El índice es sobre la secuencia plana de todos los compases del Score
    /// (iterando `score.systems[].staves[].measures[]` en orden).
    pub start_measure: usize,
    /// Índice del último compás (inclusivo).
    pub end_measure: usize,
    /// Progresión de acordes de esta sección.
    pub progression: ChordProgression,
    /// Color del bloque en el mapa (almacenado como string de color egui, ej. "#4A90D9").
    pub color: String,
}

impl Section {
    /// Crea una nueva sección validando que `start_measure <= end_measure`.
    pub fn new(
        kind: SectionKind,
        label: String,
        start_measure: usize,
        end_measure: usize,
        progression: ChordProgression,
        color: String,
    ) -> Self {
        debug_assert!(
            start_measure <= end_measure,
            "Section start_measure ({}) must be <= end_measure ({})",
            start_measure,
            end_measure
        );
        Self {
            kind,
            label,
            start_measure,
            end_measure,
            progression,
            color,
        }
    }
}

/// Un tema: agrupación de secciones sobre una partitura.
#[derive(Clone, Debug)]
pub struct Theme {
    pub name: String,
    pub sections: Vec<Section>,
}

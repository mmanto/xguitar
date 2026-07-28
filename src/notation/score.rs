use super::system::System;

/// Partitura completa.
#[derive(Clone, Debug)]
pub struct Score {
    pub title: String,
    pub composer: String,
    pub systems: Vec<System>,
    pub credits: Vec<Credit>,
    pub scaling: Option<Scaling>,
    pub part_list: PartList,
}

/// Crédito de página (título, compositor, pie de página, etc.).
#[derive(Clone, Debug)]
pub struct Credit {
    pub page: u8,
    pub kind: CreditKind,
    /// Posición horizontal como fracción de página (0.0–1.0), no tenths de MusicXML.
    pub default_x: f32,
    /// Posición vertical como fracción de página (0.0–1.0, crece hacia arriba
    /// como en MusicXML), no tenths.
    pub default_y: f32,
    pub justify: CreditJustify,
}

#[derive(Clone, Debug)]
pub enum CreditKind {
    Words(String),
    Symbol(char),
}

/// Alineación horizontal de un crédito (atributo `justify` de MusicXML).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum CreditJustify {
    #[default]
    Left,
    Center,
    Right,
}

/// Escala de MusicXML (mm por tenth).
#[derive(Clone, Debug)]
pub struct Scaling {
    pub millimeters: f32,
    pub tenths: f32,
}

/// Lista de partes e instrumentos.
#[derive(Clone, Debug)]
pub struct PartList {
    pub parts: Vec<PartInfo>,
    pub groups: Vec<PartGroup>,
}

impl Default for PartList {
    fn default() -> Self {
        Self {
            parts: Vec::new(),
            groups: Vec::new(),
        }
    }
}

/// Información de una parte/instrumento.
#[derive(Clone, Debug)]
pub struct PartInfo {
    pub id: String,
    pub name: String,
    pub abbreviation: String,
}

/// Grupo de partes (ej. sección de vientos).
#[derive(Clone, Debug)]
pub struct PartGroup {
    pub number: u8,
    pub kind: super::system::GroupBracket,
    pub name: String,
}

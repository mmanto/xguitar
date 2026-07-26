use super::Placement;

/// Tresillo, quintillo, etc. — agrupación rítmica irregular.
#[derive(Clone, Debug)]
pub struct Tuplet {
    /// Número real de notas en el grupo (ej. 3 para tresillo).
    pub number: u8,
    /// Mostrar corchete sobre/bajo el grupo.
    pub bracket: bool,
    pub show_number: Option<TupletShow>,
    pub show_type: Option<TupletShow>,
    pub placement: Placement,
}

impl Default for Tuplet {
    fn default() -> Self {
        Self {
            number: 3,
            bracket: true,
            show_number: Some(TupletShow::Actual),
            show_type: None,
            placement: Placement::Above,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum TupletShow {
    Actual,
    Both,
    None,
}

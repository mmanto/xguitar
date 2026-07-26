use super::tab::TablatureStaff;
use super::{Clef, Measure};

/// Tipo de pentagrama.
#[derive(Clone, Debug)]
pub enum StaffKind {
    Standard,
    Tablature(TablatureStaff),
    GrandStaff {
        upper: Box<Staff>,
        lower: Box<Staff>,
    },
}

/// Un pentagrama con su clave y compases.
#[derive(Clone, Debug)]
pub struct Staff {
    pub clef: Clef,
    pub line: i8,
    pub measures: Vec<Measure>,
    pub name: String,
    pub abbreviation: String,
    pub kind: StaffKind,
}

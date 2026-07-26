/// Configuración de layout para sistemas.
#[derive(Clone, Debug)]
pub struct SystemLayout {
    /// Distancia vertical entre sistemas consecutivos.
    pub system_distance: f32,
    /// Distancia desde el margen superior al primer sistema.
    pub top_system_distance: f32,
    /// Divisores entre sistemas (// al inicio de línea).
    pub system_dividers: Option<SystemDividers>,
}

#[derive(Clone, Copy, Debug)]
pub struct SystemDividers {
    pub left: bool,
    pub right: bool,
}

impl Default for SystemLayout {
    fn default() -> Self {
        Self {
            system_distance: 60.0,
            top_system_distance: 80.0,
            system_dividers: None,
        }
    }
}

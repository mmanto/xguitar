/// Barra de compás.
#[derive(Clone, Debug)]
pub struct Barline {
    pub style: BarStyle,
    /// Dirección de repetición (forward/backward).
    pub repeat: Option<RepeatDirection>,
    /// Casilla de repetición (1ra/2da vez).
    pub ending: Option<Ending>,
}

impl Default for Barline {
    fn default() -> Self {
        Self {
            style: BarStyle::Regular,
            repeat: None,
            ending: None,
        }
    }
}

/// Estilo visual de la barra de compás.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum BarStyle {
    Regular,
    Dotted,
    Dashed,
    Heavy,
    LightLight, // double thin
    LightHeavy, // final
    HeavyLight, // reverse final
    HeavyHeavy,
    Tick,
    Short,
    None,
}

/// Dirección de repetición.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum RepeatDirection {
    Forward,
    Backward,
}

/// Casilla de primera/segunda vez.
#[derive(Clone, Debug)]
pub struct Ending {
    pub number: String,       // "1", "1,2", "1-3"
    pub text: Option<String>, // "D.C."
    pub length: Option<i32>,  // número de compases en la casilla
}

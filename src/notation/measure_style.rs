use super::NoteFigure;

/// Notación de compás repetido o slash.
#[derive(Clone, Debug)]
pub enum MeasureStyle {
    /// Símbolo de repetición de compás: % (SMuFL U+E500).
    RepeatPercent,
    /// Repetición de pulso: barra simple.
    BeatRepeat,
    /// Notación slash: figuras sin altura definida.
    Slash(Vec<SlashNote>),
}

/// Nota slash (cabeza diagonal, sin plica normalmente).
#[derive(Clone, Copy, Debug)]
pub struct SlashNote {
    pub figure: NoteFigure,
    pub dotted: bool,
}

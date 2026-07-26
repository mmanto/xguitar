/// Silencio de múltiples compases.
#[derive(Clone, Debug)]
pub struct MultipleRest {
    /// Cantidad de compases de silencio.
    pub count: u8,
    /// Usar símbolos de silencio compuesto (Kirchenpause).
    pub use_symbols: bool,
}

//! Reproducción de partituras: secuenciador + motor de síntesis.
//!
//! `sequencer` es lógica de dominio pura (recorre un `notation::Score` y
//! produce una lista de eventos con tiempo absoluto) sin depender de egui ni
//! de ningún backend de audio concreto — testeable sin hardware. `player`
//! (nativo) conecta esos eventos a un `PlaybackEngine` real vía `cpal`.

pub mod sequencer;

#[cfg(not(target_arch = "wasm32"))]
pub mod player;
#[cfg(not(target_arch = "wasm32"))]
pub mod sfizz;

use std::path::Path;

#[derive(Debug)]
pub enum AudioError {
    /// No se pudo crear el motor de síntesis.
    EngineInit,
    /// El archivo de instrumento no existe o no se pudo cargar.
    InstrumentLoad(String),
    /// No se pudo abrir el dispositivo de salida de audio.
    OutputDevice(String),
}

impl std::fmt::Display for AudioError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AudioError::EngineInit => write!(f, "no se pudo inicializar el motor de audio"),
            AudioError::InstrumentLoad(path) => {
                write!(f, "no se pudo cargar el instrumento: {path}")
            }
            AudioError::OutputDevice(msg) => write!(f, "error de dispositivo de audio: {msg}"),
        }
    }
}

impl std::error::Error for AudioError {}

/// Motor de síntesis: recibe eventos MIDI-like y renderiza audio interleaved.
/// Implementado hoy por `sfizz::SfizzEngine` (nativo, sample-based). La
/// abstracción existe para poder enchufar a futuro un motor más simple
/// (osciladores en Rust puro) para el build WASM, donde `sfizz` — una
/// librería C++ nativa — no puede correr.
pub trait PlaybackEngine: Send {
    fn load_instrument(&mut self, path: &Path) -> Result<(), AudioError>;
    /// Configura el motor para el sample rate y tamaño de bloque real del
    /// dispositivo de salida — se llama una vez, antes del primer `render`.
    /// Default no-op: no todos los motores necesitan preparación previa.
    fn configure(&mut self, sample_rate: f32, max_block_frames: usize) {
        let _ = (sample_rate, max_block_frames);
    }
    fn note_on(&mut self, midi: u8, velocity: u8);
    fn note_off(&mut self, midi: u8);
    /// Renderiza `buffer.len() / channels` frames interleaved.
    fn render(&mut self, buffer: &mut [f32], channels: usize);
}

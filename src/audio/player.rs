//! Conecta el `sequencer` (lógica pura) a un `PlaybackEngine` real vía
//! `cpal`. Todo el trabajo de audio (crear el stream, correrlo, y disparar
//! los eventos de la partitura en su momento) vive en un hilo dedicado —
//! el primer hilo de background nativo de la app. La UI solo lee estado a
//! través de atómicos (`Arc<AtomicBool>`/`Arc<AtomicU32>`), nunca toca el
//! motor de audio directamente.

use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::mpsc::{self, Receiver, RecvTimeoutError};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};

use super::sequencer::{self, EventKind, SequencedEvent};
use super::sfizz::SfizzEngine;
use super::PlaybackEngine;
use crate::notation::Score;

enum Control {
    Stop,
}

/// Servicio de reproducción a nivel de aplicación — uno solo, no por
/// documento (no tiene sentido reproducir dos partituras a la vez).
pub struct AudioService {
    instrument_path: Option<PathBuf>,
    is_playing: Arc<AtomicBool>,
    position_secs: Arc<AtomicU32>,
    last_error: Arc<Mutex<Option<String>>>,
    control_tx: Option<mpsc::Sender<Control>>,
    thread: Option<std::thread::JoinHandle<()>>,
}

impl AudioService {
    pub fn new() -> Self {
        Self {
            instrument_path: None,
            is_playing: Arc::new(AtomicBool::new(false)),
            position_secs: Arc::new(AtomicU32::new(0)),
            last_error: Arc::new(Mutex::new(None)),
            control_tx: None,
            thread: None,
        }
    }

    pub fn set_instrument_path(&mut self, path: Option<PathBuf>) {
        self.instrument_path = path;
    }

    pub fn instrument_path(&self) -> Option<&PathBuf> {
        self.instrument_path.as_ref()
    }

    pub fn is_playing(&self) -> bool {
        self.is_playing.load(Ordering::Relaxed)
    }

    pub fn position_secs(&self) -> f32 {
        f32::from_bits(self.position_secs.load(Ordering::Relaxed))
    }

    /// Consume el último error de reproducción (creación de instrumento,
    /// dispositivo de audio, etc.), si hubo alguno. Pensado para que la UI
    /// lo muestre una vez en `status_message` y lo descarte.
    pub fn take_last_error(&self) -> Option<String> {
        self.last_error.lock().unwrap().take()
    }

    pub fn play(&mut self, score: &Score) {
        self.stop();

        let events = sequencer::build_events(score);
        let instrument_path = self.instrument_path.clone();
        let is_playing = self.is_playing.clone();
        let position_secs = self.position_secs.clone();
        let last_error = self.last_error.clone();

        let (tx, rx) = mpsc::channel();
        self.control_tx = Some(tx);
        is_playing.store(true, Ordering::Relaxed);
        position_secs.store(0f32.to_bits(), Ordering::Relaxed);

        self.thread = Some(std::thread::spawn(move || {
            run_playback(
                events,
                instrument_path,
                rx,
                is_playing,
                position_secs,
                last_error,
            );
        }));
    }

    pub fn stop(&mut self) {
        if let Some(tx) = self.control_tx.take() {
            let _ = tx.send(Control::Stop);
        }
        if let Some(handle) = self.thread.take() {
            let _ = handle.join();
        }
        self.is_playing.store(false, Ordering::Relaxed);
    }
}

impl Default for AudioService {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for AudioService {
    fn drop(&mut self) {
        self.stop();
    }
}

fn run_playback(
    events: Vec<SequencedEvent>,
    instrument_path: Option<PathBuf>,
    control_rx: Receiver<Control>,
    is_playing: Arc<AtomicBool>,
    position_secs: Arc<AtomicU32>,
    last_error: Arc<Mutex<Option<String>>>,
) {
    let host = cpal::default_host();
    let device = match host.default_output_device() {
        Some(d) => d,
        None => {
            *last_error.lock().unwrap() =
                Some("no se encontró dispositivo de salida de audio".into());
            is_playing.store(false, Ordering::Relaxed);
            return;
        }
    };
    let config = match device.default_output_config() {
        Ok(c) => c,
        Err(err) => {
            *last_error.lock().unwrap() = Some(err.to_string());
            is_playing.store(false, Ordering::Relaxed);
            return;
        }
    };
    let channels = config.channels() as usize;
    let sample_rate = config.sample_rate() as f32;
    let stream_config: cpal::StreamConfig = config.into();

    // Cota generosa para el tamaño de bloque que va a pedir el backend de
    // audio por callback — sfizz exige preconfigurar el máximo esperado
    // (`sfizz_set_samples_per_block`); `render()` igual redimensiona su
    // scratch si algún callback llegara a pedir más.
    const MAX_BLOCK_FRAMES: usize = 8192;

    let mut engine = match SfizzEngine::new() {
        Ok(e) => e,
        Err(err) => {
            *last_error.lock().unwrap() = Some(err.to_string());
            is_playing.store(false, Ordering::Relaxed);
            return;
        }
    };
    engine.configure(sample_rate, MAX_BLOCK_FRAMES);

    match &instrument_path {
        Some(path) => {
            if let Err(err) = engine.load_instrument(path) {
                *last_error.lock().unwrap() = Some(err.to_string());
                // Seguimos igual: sfizz sin instrumento cargado simplemente
                // no va a sonar, pero no hace falta abortar la reproducción.
            }
        }
        None => {
            *last_error.lock().unwrap() = Some(
                "sin instrumento configurado — elegí un archivo .sfz en Preferencias".into(),
            );
        }
    }

    let engine: Arc<Mutex<Box<dyn PlaybackEngine>>> = Arc::new(Mutex::new(Box::new(engine)));

    let stream_engine = engine.clone();
    let stream = device.build_output_stream(
        stream_config,
        move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
            let mut engine = stream_engine.lock().unwrap();
            engine.render(data, channels);
        },
        |err| log::error!("error de audio: {err}"),
        None,
    );
    let stream = match stream {
        Ok(s) => s,
        Err(err) => {
            *last_error.lock().unwrap() = Some(err.to_string());
            is_playing.store(false, Ordering::Relaxed);
            return;
        }
    };
    if let Err(err) = stream.play() {
        *last_error.lock().unwrap() = Some(err.to_string());
        is_playing.store(false, Ordering::Relaxed);
        return;
    }

    let start = Instant::now();
    for event in events {
        let deadline = start + Duration::from_secs_f32(event.time_secs.max(0.0));
        let now = Instant::now();
        if now < deadline {
            // `recv_timeout` bloquea hasta el mensaje o hasta que se cumpla
            // el plazo completo (no retorna antes espontáneamente), así que
            // no hace falta reintentar en un loop.
            match control_rx.recv_timeout(deadline - now) {
                Ok(Control::Stop) | Err(RecvTimeoutError::Disconnected) => {
                    is_playing.store(false, Ordering::Relaxed);
                    return;
                }
                Err(RecvTimeoutError::Timeout) => {}
            }
        }
        position_secs.store(event.time_secs.to_bits(), Ordering::Relaxed);
        let mut engine = engine.lock().unwrap();
        match event.kind {
            EventKind::NoteOn { midi, velocity } => engine.note_on(midi, velocity),
            EventKind::NoteOff { midi } => engine.note_off(midi),
        }
    }

    // Deja sonar el release de las últimas notas antes de cerrar el stream
    // (o corta antes si llega Stop).
    let _ = control_rx.recv_timeout(Duration::from_millis(500));
    is_playing.store(false, Ordering::Relaxed);
}

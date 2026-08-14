//! Conecta el `sequencer` (lógica pura) a un `PlaybackEngine` real vía
//! `cpal`. Un único stream de audio persistente creado en `ensure_stream()`,
//! con un hilo dedicado que recibe comandos Play/Stop por un canal mpsc.
//! El stream **nunca se destruye** — solo alterna entre reproducir eventos
//! y renderizar silencio — para que el nodo PipeWire/JACK sea permanente
//! y los parches en Carla/qpwgraph/helvum no se pierdan.

use std::collections::HashSet;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::mpsc::{self, Receiver, RecvTimeoutError};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};

use super::PlaybackEngine;
use super::sequencer::{self, EventKind, SequencedEvent};
use super::sfizz::SfizzEngine;
use crate::notation::{NoteRef, Score};

/// Intenta hosts de audio en orden de preferencia: PipeWire > JACK > default.
/// Usa lookup por string para que funcione en runtime sin necesidad de feature
/// flags en nuestro crate — si cpal no se compiló con `pipewire`/`jack`,
/// `from_str` simplemente falla y se degrada al host por defecto.
fn select_host() -> cpal::Host {
    for name in ["pipewire", "jack"] {
        if let Ok(id) = name.parse::<cpal::HostId>() {
            if let Ok(host) = cpal::host_from_id(id) {
                log::info!("usando host de audio: {name} (nativo)");
                return host;
            }
        }
    }
    log::info!("usando host de audio por defecto");
    cpal::default_host()
}

/// Comandos que la UI envía al hilo de audio persistente.
enum Command {
    /// Reproducir una secuencia de eventos (partitura completa).
    Play(Vec<SequencedEvent>),
    /// Detener la reproducción en curso (el stream sigue vivo).
    Stop,
}

/// Servicio de reproducción a nivel de aplicación.
///
/// Mantiene un único stream de audio (`cpal`) y un hilo de scheduling
/// que recibe comandos por canal. El stream se crea una sola vez en
/// `ensure_stream()` y persiste hasta que se dropea el `AudioService`.
pub struct AudioService {
    instrument_path: Option<PathBuf>,
    is_playing: Arc<AtomicBool>,
    is_paused: Arc<AtomicBool>,
    position_secs: Arc<AtomicU32>,
    last_error: Arc<Mutex<Option<String>>>,
    command_tx: Option<mpsc::Sender<Command>>,
    thread: Option<std::thread::JoinHandle<()>>,
    /// Copy of the last-played event list (with NoteRefs) for highlighting.
    note_events: Arc<Vec<SequencedEvent>>,
}
impl AudioService {
    pub fn new() -> Self {
        Self {
            instrument_path: None,
            is_playing: Arc::new(AtomicBool::new(false)),
            is_paused: Arc::new(AtomicBool::new(false)),
            position_secs: Arc::new(AtomicU32::new(0)),
            last_error: Arc::new(Mutex::new(None)),
            command_tx: None,
            thread: None,
            note_events: Arc::new(Vec::new()),
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

    pub fn is_paused(&self) -> bool {
        self.is_paused.load(Ordering::Relaxed)
    }

    pub fn position_secs(&self) -> f32 {
        f32::from_bits(self.position_secs.load(Ordering::Relaxed))
    }

    /// Consume el último error de reproducción, si hubo alguno.
    pub fn take_last_error(&self) -> Option<String> {
        self.last_error.lock().unwrap().take()
    }

    /// Crea el stream de audio persistente y lanza el hilo de scheduling.
    /// Idempotente: si ya hay un stream vivo, no hace nada.
    pub fn ensure_stream(&mut self) {
        if self.command_tx.is_some() {
            return;
        }
        let instrument_path = self.instrument_path.clone();
        let is_playing = self.is_playing.clone();
        let is_paused = self.is_paused.clone();
        let position_secs = self.position_secs.clone();
        let last_error = self.last_error.clone();

        let (tx, rx) = mpsc::channel();
        self.command_tx = Some(tx);

        self.thread = Some(std::thread::spawn(move || {
            run_audio_thread(
                instrument_path,
                rx,
                is_playing,
                is_paused,
                position_secs,
                last_error,
            );
        }));
    }

    /// Envía los eventos de una partitura al hilo de audio.
    /// Si el stream no se inicializó todavía, no hace nada.
    pub fn play(&mut self, score: &Score) {
        let events = sequencer::build_events(score);
        self.note_events = Arc::new(events.clone());
        self.is_paused.store(false, Ordering::Relaxed);
        if let Some(tx) = &self.command_tx {
            let _ = tx.send(Command::Play(events));
            self.is_playing.store(true, Ordering::Relaxed);
            self.position_secs.store(0f32.to_bits(), Ordering::Relaxed);
        }
    }

    /// Detiene la reproducción en curso. El stream y el hilo siguen vivos
    /// (produciendo silencio) — el nodo PipeWire/JACK no desaparece.
    pub fn stop(&mut self) {
        if let Some(tx) = &self.command_tx {
            let _ = tx.send(Command::Stop);
        }
        self.is_playing.store(false, Ordering::Relaxed);
        self.is_paused.store(false, Ordering::Relaxed);
    }

    /// Pausa la reproducción sin detener el stream.
    /// Las notas ya disparadas siguen sonando en sfizz.
    pub fn pause(&mut self) {
        self.is_paused.store(true, Ordering::Relaxed);
        self.is_playing.store(false, Ordering::Relaxed);
    }

    /// Reanuda desde la posición de pausa.
    pub fn resume(&mut self) {
        self.is_paused.store(false, Ordering::Relaxed);
        self.is_playing.store(true, Ordering::Relaxed);
    }

    /// Vuelve al inicio de la partitura.
    pub fn seek_start(&mut self) {
        if let Some(tx) = &self.command_tx {
            let _ = tx.send(Command::Stop);
        }
        self.is_playing.store(false, Ordering::Relaxed);
        self.is_paused.store(false, Ordering::Relaxed);
        self.position_secs.store(0f32.to_bits(), Ordering::Relaxed);
    }
    /// Returns NoteRefs for all notes that are currently sounding.
    pub fn active_note_refs(&self) -> HashSet<NoteRef> {
        let pos = self.position_secs();
        self.note_events
            .iter()
            .filter(|e| {
                matches!(e.kind, EventKind::NoteOn { .. }) && e.time_secs <= pos && pos < e.end_secs
            })
            .map(|e| e.note_ref)
            .collect()
    }
}

impl Default for AudioService {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for AudioService {
    fn drop(&mut self) {
        // Soltar el sender cierra el canal → el hilo sale de recv() y termina.
        self.command_tx = None;
        if let Some(handle) = self.thread.take() {
            let _ = handle.join();
        }
    }
}

/// Hilo permanente de audio: crea el stream una sola vez y entra en un
/// loop de comandos (Play/Stop). El stream se mantiene vivo durante toda
/// la vida de la aplicación.
fn run_audio_thread(
    instrument_path: Option<PathBuf>,
    command_rx: Receiver<Command>,
    is_playing: Arc<AtomicBool>,
    is_paused: Arc<AtomicBool>,
    position_secs: Arc<AtomicU32>,
    last_error: Arc<Mutex<Option<String>>>,
) {
    // ── Setup único del stream ──
    let host = select_host();
    let device = match host.default_output_device() {
        Some(d) => d,
        None => {
            *last_error.lock().unwrap() =
                Some("no se encontró dispositivo de salida de audio".into());
            return;
        }
    };
    let config = match device.default_output_config() {
        Ok(c) => c,
        Err(err) => {
            *last_error.lock().unwrap() = Some(err.to_string());
            return;
        }
    };
    let channels = config.channels() as usize;
    let sample_rate = config.sample_rate() as f32;
    let stream_config: cpal::StreamConfig = config.into();

    const MAX_BLOCK_FRAMES: usize = 8192;

    let mut engine = match SfizzEngine::new() {
        Ok(e) => e,
        Err(err) => {
            *last_error.lock().unwrap() = Some(err.to_string());
            return;
        }
    };
    engine.configure(sample_rate, MAX_BLOCK_FRAMES);

    match &instrument_path {
        Some(path) => {
            if let Err(err) = engine.load_instrument(path) {
                *last_error.lock().unwrap() = Some(err.to_string());
            }
        }
        None => {
            *last_error.lock().unwrap() =
                Some("sin instrumento configurado — elegí un archivo .sfz en Preferencias".into());
        }
    }

    let engine: Arc<Mutex<Box<dyn PlaybackEngine>>> = Arc::new(Mutex::new(Box::new(engine)));

    let stream_engine = engine.clone();
    let stream = device.build_output_stream(
        stream_config,
        move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
            let mut eng = stream_engine.lock().unwrap();
            eng.render(data, channels);
        },
        |err| log::error!("error de audio: {err}"),
        None,
    );
    let stream = match stream {
        Ok(s) => s,
        Err(err) => {
            *last_error.lock().unwrap() = Some(err.to_string());
            return;
        }
    };
    if let Err(err) = stream.play() {
        *last_error.lock().unwrap() = Some(err.to_string());
        return;
    }

    // ── Loop de comandos ──
    loop {
        let events = match command_rx.recv() {
            Ok(Command::Play(ev)) => ev,
            Ok(Command::Stop) => {
                is_playing.store(false, Ordering::Relaxed);
                continue;
            }
            Err(_) => break, // Canal cerrado → shutdown
        };

        is_playing.store(true, Ordering::Relaxed);
        position_secs.store(0f32.to_bits(), Ordering::Relaxed);

        let start = Instant::now();

        for event in &events {
            // Pause: spin until resumed or stopped.
            while is_paused.load(Ordering::Relaxed) {
                match command_rx.recv_timeout(Duration::from_millis(50)) {
                    Ok(Command::Stop) | Err(RecvTimeoutError::Disconnected) => break,
                    _ => {}
                }
            }
            // Exit event loop if stopped during pause.
            if !is_playing.load(Ordering::Relaxed) {
                break;
            }

            let deadline = start + Duration::from_secs_f32(event.time_secs.max(0.0));
            let now = Instant::now();
            if now < deadline {
                match command_rx.recv_timeout(deadline - now) {
                    Ok(Command::Stop) | Err(RecvTimeoutError::Disconnected) => break,
                    Ok(Command::Play(_)) => break,
                    Err(RecvTimeoutError::Timeout) => {}
                }
            }
            position_secs.store(event.time_secs.to_bits(), Ordering::Relaxed);
            let mut eng = engine.lock().unwrap();
            match event.kind {
                EventKind::NoteOn { midi, velocity } => eng.note_on(midi, velocity),
                EventKind::NoteOff { midi } => eng.note_off(midi),
            }
        }

        is_playing.store(false, Ordering::Relaxed);
    }
}

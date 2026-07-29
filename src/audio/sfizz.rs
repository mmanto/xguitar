//! Bindings FFI mínimos a `libsfizz` (sampler SFZ) — nativo únicamente.
//! Subconjunto de la API C de `/usr/include/sfizz.h` (paquete de sistema
//! `sfizz`/`sfizz-lib`). No existe un crate publicado que envuelva sfizz;
//! ver ADR-008.
//!
//! sfizz solo renderiza estéreo, en formato *planar* (`float**` — un puntero
//! por canal, no interleaved: `sfizz_render_block`, ver cabecera línos
//! ~706-727). `cpal` entrega/espera buffers *interleaved* — `render()` hace
//! la conversión usando un scratch buffer reutilizado entre llamadas para
//! no asignar memoria en el callback de audio realtime.

use super::{AudioError, PlaybackEngine};
use std::ffi::CString;
use std::path::Path;

#[repr(C)]
#[allow(non_camel_case_types)]
struct sfizz_synth_t {
    _private: [u8; 0],
}

unsafe extern "C" {
    fn sfizz_create_synth() -> *mut sfizz_synth_t;
    fn sfizz_free(synth: *mut sfizz_synth_t);
    fn sfizz_load_file(synth: *mut sfizz_synth_t, path: *const libc::c_char) -> bool;
    fn sfizz_set_sample_rate(synth: *mut sfizz_synth_t, sample_rate: f32);
    fn sfizz_set_samples_per_block(synth: *mut sfizz_synth_t, samples_per_block: i32);
    fn sfizz_send_note_on(synth: *mut sfizz_synth_t, delay: i32, note_number: i32, velocity: i32);
    fn sfizz_send_note_off(synth: *mut sfizz_synth_t, delay: i32, note_number: i32, velocity: i32);
    /// `channels`: puntero a un array de 2 punteros (izquierdo, derecho).
    fn sfizz_render_block(
        synth: *mut sfizz_synth_t,
        channels: *mut *mut f32,
        num_channels: i32,
        num_frames: i32,
    );
}

pub struct SfizzEngine {
    inner: *mut sfizz_synth_t,
    /// Scratch planar reutilizado entre `render()` — evita asignar en el
    /// hilo realtime de audio salvo la primera vez (o si crece el bloque).
    left: Vec<f32>,
    right: Vec<f32>,
}

impl SfizzEngine {
    pub fn new() -> Result<Self, AudioError> {
        let inner = unsafe { sfizz_create_synth() };
        if inner.is_null() {
            Err(AudioError::EngineInit)
        } else {
            Ok(Self {
                inner,
                left: Vec::new(),
                right: Vec::new(),
            })
        }
    }
}

impl PlaybackEngine for SfizzEngine {
    fn load_instrument(&mut self, path: &Path) -> Result<(), AudioError> {
        let path_str = path.to_string_lossy().into_owned();
        let cpath = CString::new(path_str.as_str())
            .map_err(|_| AudioError::InstrumentLoad(path_str.clone()))?;
        let ok = unsafe { sfizz_load_file(self.inner, cpath.as_ptr()) };
        if ok {
            Ok(())
        } else {
            Err(AudioError::InstrumentLoad(path_str))
        }
    }

    fn configure(&mut self, sample_rate: f32, max_block_frames: usize) {
        unsafe {
            sfizz_set_sample_rate(self.inner, sample_rate);
            sfizz_set_samples_per_block(self.inner, max_block_frames as i32);
        }
        self.left.resize(max_block_frames, 0.0);
        self.right.resize(max_block_frames, 0.0);
    }

    fn note_on(&mut self, midi: u8, velocity: u8) {
        unsafe { sfizz_send_note_on(self.inner, 0, midi as i32, velocity as i32) }
    }

    fn note_off(&mut self, midi: u8) {
        unsafe { sfizz_send_note_off(self.inner, 0, midi as i32, 0) }
    }

    fn render(&mut self, buffer: &mut [f32], channels: usize) {
        let channels = channels.max(1);
        let num_frames = buffer.len() / channels;
        if self.left.len() < num_frames {
            self.left.resize(num_frames, 0.0);
            self.right.resize(num_frames, 0.0);
        }

        let mut channel_ptrs: [*mut f32; 2] = [self.left.as_mut_ptr(), self.right.as_mut_ptr()];
        unsafe {
            sfizz_render_block(self.inner, channel_ptrs.as_mut_ptr(), 2, num_frames as i32);
        }

        for frame in 0..num_frames {
            let l = self.left[frame];
            let r = self.right[frame];
            let base = frame * channels;
            if channels == 1 {
                buffer[base] = 0.5 * (l + r);
            } else {
                buffer[base] = l;
                buffer[base + 1] = r;
                for extra in buffer.iter_mut().skip(base + 2).take(channels - 2) {
                    *extra = 0.0;
                }
            }
        }
    }
}

impl Drop for SfizzEngine {
    fn drop(&mut self) {
        unsafe { sfizz_free(self.inner) }
    }
}

// SAFETY: sfizz_synth_t es opaco para nosotros; la librería está diseñada
// para recibir llamadas desde el hilo de audio realtime (render) y desde
// otro hilo (note_on/note_off) — es el mismo patrón que ya usaba el
// prototipo `guitarra-sfizz`, donde el synth se comparte tras un Mutex.
unsafe impl Send for SfizzEngine {}

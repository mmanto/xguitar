use super::attachment::NoteAttachment;
use super::lyric::Lyric;
use super::{Accidental, NoteFigure, Pitch, Step};

/// Dirección de la plica: hacia arriba o hacia abajo.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum StemDirection {
    Up,
    Down,
}

/// Una nota musical con altura, figura rítmica y puntillo.
#[derive(Clone, Debug)]
pub struct Note {
    pub pitch: Pitch,
    pub figure: NoteFigure,
    pub dotted: u8,
    pub accidental_override: Option<Accidental>,
    pub stem_direction: StemDirection,
    pub grace: bool,
    pub chord: bool,
    pub attachments: Option<NoteAttachment>,
    /// Letra/canción asociada a esta nota.
    pub lyrics: Vec<Lyric>,
}

impl Default for Note {
    fn default() -> Self {
        Self {
            pitch: Pitch {
                step: Step::C,
                accidental: Accidental::Natural,
                octave: 4,
            },
            figure: NoteFigure::Quarter,
            dotted: 0,
            accidental_override: None,
            stem_direction: StemDirection::Up,
            grace: false,
            chord: false,
            attachments: None,
            lyrics: Vec::new(),
        }
    }
}

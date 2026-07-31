/// Identifies a specific note in the score for playback highlighting.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct NoteRef {
    pub system_idx: usize,
    pub staff_idx: usize,
    pub measure_idx: usize,
    pub element_idx: usize,
    /// 0 for bare `Note` elements; index within `Chord` for chord notes.
    pub subnote_idx: usize,
}

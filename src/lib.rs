pub mod app;

pub mod fonts;
pub mod i18n;
pub mod musicxml;
pub mod notation;
pub mod render;

pub use musicxml::parser::parse_musicxml;

pub use fonts::configure_fonts;
pub use i18n::{I18n, Lang};
pub use notation::{
    Accidental, Arpeggiate, ArpeggioDirection, Articulation, BarStyle, Barline, Bracket,
    BracketKind, Clef, Credit, CreditJustify, CreditKind, Dashes, DashesKind, Direction,
    DirectionKind, DynamicMark, Ending, Fermata, FermataShape, Glissando, GlissandoKind, GraceNote,
    GroupBracket, HarmonicKind, KeyMode, KeySignature, LineEnd, LineType, Lyric, Measure,
    MeasureElement, MeasureStyle, Metronome, MultipleRest, Note, NoteAttachment, NoteFigure,
    OctaveShift, OctaveShiftKind, Ornament, PartGroup, PartInfo, PartList, Pitch, Placement,
    RepeatDirection, Rest, Scaling, Score, SlashNote, SlideKind, Slur, SlurKind, Staff, StaffKind,
    StemDirection, Step, Syllabic, System, SystemDividers, SystemLayout, TabElement, TabMeasure,
    TabNote, TabRest, TabTechnique, TablatureStaff, Technical, Tie, TieKind, TimeSignature,
    TimeSignatureStyle, Tremolo, TuningStep, Tuplet, TupletShow, Wedge, WedgeKind,
};
pub use render::{
    PAGE_GAP, Page, PageLayout, RenderStyle, STAFF_LINE_SPACING, ScoreStylesheet, compute_pages,
    render_clef, render_measure_elements, render_notehead, render_page, render_pages, render_score,
    render_staff_lines, score_total_height,
};

// ── WASM entry point ──────────────────────────────────────────────────────

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen::prelude::wasm_bindgen(start)]
pub fn wasm_main() {
    use crate::app::MGuitarApp;
    use eframe::wasm_bindgen::JsCast;

    std::panic::set_hook(Box::new(console_error_panic_hook::hook));
    web_sys::console::log_1(&"xguitar: WASM start".into());

    let web_options = eframe::WebOptions::default();
    web_sys::console::log_1(&format!("xguitar: renderer={:?}", web_options.renderer).into());

    wasm_bindgen_futures::spawn_local(async move {
        web_sys::console::log_1(&"xguitar: async init".into());
        let window = web_sys::window().expect("no window");
        let document = window.document().expect("no document");
        let canvas = document
            .get_element_by_id("xguitar-canvas")
            .expect("canvas not found")
            .dyn_into::<web_sys::HtmlCanvasElement>()
            .expect("not a canvas");

        web_sys::console::log_1(&format!("canvas {}x{}", canvas.width(), canvas.height()).into());

        eframe::WebRunner::new()
            .start(
                canvas,
                web_options,
                Box::new(|cc| {
                    web_sys::console::log_1(&"creating app".into());
                    Ok(Box::new(MGuitarApp::new(cc)))
                }),
            )
            .await
            .expect("WebRunner failed");

        web_sys::console::log_1(&"xguitar: running!".into());
    });
}

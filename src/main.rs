use m_guitar::app::MGuitarApp;

#[cfg(not(target_arch = "wasm32"))]
fn main() {
    let native_options = eframe::NativeOptions {
        viewport: eframe::egui::ViewportBuilder::default().with_decorations(false),
        ..Default::default()
    };
    let _ = eframe::run_native(
        "m-guitar",
        native_options,
        Box::new(|cc| Ok(Box::new(MGuitarApp::new(cc)))),
    );
}

#[cfg(target_arch = "wasm32")]
fn main() {}

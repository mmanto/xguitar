use eframe::egui;

pub fn configure_fonts(ctx: &egui::Context) {
    let mut fonts = egui::FontDefinitions::default();

    // Leland music font — needed on ALL platforms for notation glyphs
    fonts.font_data.insert(
        "Leland".into(),
        egui::FontData::from_static(include_bytes!("../lib/MusicFonts/Leland/Leland.otf")).into(),
    );
    fonts.families.insert(
        egui::FontFamily::Name("Leland".into()),
        vec!["Leland".into()],
    );

    // LelandText — only on native. On WASM, keep egui default for text.
    #[cfg(not(target_arch = "wasm32"))]
    {
        fonts.font_data.insert(
            "LelandText".into(),
            egui::FontData::from_static(include_bytes!("../lib/MusicFonts/Leland/LelandText.otf"))
                .into(),
        );
        fonts
            .families
            .get_mut(&egui::FontFamily::Proportional)
            .expect("Proportional font family missing")
            .push("LelandText".into());
    }

    ctx.set_fonts(fonts);
}

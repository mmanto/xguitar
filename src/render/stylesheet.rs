use eframe::egui;
use serde::Deserialize;

// ────────────────────────────────────────────────────────────
//  ScoreStylesheet — visual preset for score rendering
// ────────────────────────────────────────────────────────────

/// A complete visual stylesheet for score rendering.
///
/// Loaded from TOML files; missing fields inherit sensible defaults
/// matching a clean printed-score look.
#[derive(Deserialize, Clone, Debug)]
pub struct ScoreStylesheet {
    pub name: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub page: PageStyle,
    #[serde(default)]
    pub header: HeaderStyle,
    #[serde(default)]
    pub notation: NotationStyle,
}

#[derive(Deserialize, Clone, Debug)]
pub struct PageStyle {
    #[serde(default = "default_bg_light")]
    pub bg_light: String,
    #[serde(default = "default_bg_dark")]
    pub bg_dark: String,
    #[serde(default = "default_border_light")]
    pub border_light: String,
    #[serde(default = "default_border_dark")]
    pub border_dark: String,
    #[serde(default = "default_border_width")]
    pub border_width: f32,
    #[serde(default)]
    pub shadow: ShadowStyle,
}

#[derive(Deserialize, Clone, Debug)]
pub struct ShadowStyle {
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default = "default_shadow_offset")]
    pub offset_x: f32,
    #[serde(default = "default_shadow_offset")]
    pub offset_y: f32,
    #[serde(default = "default_shadow_alpha")]
    pub alpha: u8,
}

#[derive(Deserialize, Clone, Debug)]
pub struct HeaderStyle {
    #[serde(default = "default_title_size")]
    pub title_size: f32,
    #[serde(default = "default_composer_size")]
    pub composer_size: f32,
    #[serde(default = "default_tempo_size")]
    pub tempo_size: f32,
    #[serde(default = "default_row_gap")]
    pub row_gap: f32,
    #[serde(default = "default_title_top_offset")]
    pub title_top_offset: f32,
    #[serde(default = "default_header_staff_gap")]
    pub header_staff_gap: f32,
    #[serde(default = "default_title_light")]
    pub title_light: String,
    #[serde(default = "default_title_dark")]
    pub title_dark: String,
    #[serde(default = "default_subtitle_light")]
    pub subtitle_light: String,
    #[serde(default = "default_subtitle_dark")]
    pub subtitle_dark: String,
}

#[derive(Deserialize, Clone, Debug)]
pub struct NotationStyle {
    #[serde(default = "default_staff_light")]
    pub staff_light: String,
    #[serde(default = "default_staff_dark")]
    pub staff_dark: String,
    #[serde(default = "default_note_light")]
    pub note_light: String,
    #[serde(default = "default_note_dark")]
    pub note_dark: String,
    #[serde(default = "default_clef_light")]
    pub clef_light: String,
    #[serde(default = "default_clef_dark")]
    pub clef_dark: String,
    #[serde(default)]
    pub clefs: ClefStyle,
    #[serde(default = "default_staff_scale")]
    pub staff_scale: f32,
}

#[derive(Deserialize, Default, Clone, Debug)]
pub struct ClefStyle {
    #[serde(default)]
    pub treble: ClefVariant,
    #[serde(default)]
    pub bass: ClefVariant,
    #[serde(default)]
    pub alto: ClefVariant,
    #[serde(default)]
    pub tenor: ClefVariant,
    #[serde(default)]
    pub percussion: ClefVariant,
    #[serde(default)]
    pub tab: ClefVariant,
}

#[derive(Deserialize, Clone, Debug)]
pub struct ClefVariant {
    #[serde(default = "default_treble_glyph_scale")]
    pub glyph_scale: f32,
    #[serde(default)]
    pub fine_offset: f32,
}

// ── Default impls for serde #[serde(default)] ──────────────

impl Default for PageStyle {
    fn default() -> Self {
        Self {
            bg_light: default_bg_light(),
            bg_dark: default_bg_dark(),
            border_light: default_border_light(),
            border_dark: default_border_dark(),
            border_width: default_border_width(),
            shadow: ShadowStyle::default(),
        }
    }
}

impl Default for ShadowStyle {
    fn default() -> Self {
        Self {
            enabled: default_true(),
            offset_x: default_shadow_offset(),
            offset_y: default_shadow_offset(),
            alpha: default_shadow_alpha(),
        }
    }
}

impl Default for HeaderStyle {
    fn default() -> Self {
        Self {
            title_size: default_title_size(),
            composer_size: default_composer_size(),
            tempo_size: default_tempo_size(),
            row_gap: default_row_gap(),
            title_top_offset: default_title_top_offset(),
            header_staff_gap: default_header_staff_gap(),
            title_light: default_title_light(),
            title_dark: default_title_dark(),
            subtitle_light: default_subtitle_light(),
            subtitle_dark: default_subtitle_dark(),
        }
    }
}

impl Default for NotationStyle {
    fn default() -> Self {
        Self {
            staff_light: default_staff_light(),
            staff_dark: default_staff_dark(),
            note_light: default_note_light(),
            note_dark: default_note_dark(),
            clef_light: default_clef_light(),
            clef_dark: default_clef_dark(),
            clefs: ClefStyle::default(),
            staff_scale: default_staff_scale(),
        }
    }
}

impl Default for ClefVariant {
    fn default() -> Self {
        Self {
            glyph_scale: default_treble_glyph_scale(),
            fine_offset: 0.0,
        }
    }
}

// ── Default value functions ─────────────────────────────────

fn default_bg_light() -> String {
    "#FFFFFF".into()
}
fn default_bg_dark() -> String {
    "#262626".into()
}
fn default_border_light() -> String {
    "#B4B4B4".into()
}
fn default_border_dark() -> String {
    "#505050".into()
}
fn default_border_width() -> f32 {
    0.0
}
fn default_true() -> bool {
    true
}
fn default_shadow_offset() -> f32 {
    4.0
}
fn default_shadow_alpha() -> u8 {
    60
}
fn default_title_size() -> f32 {
    18.0
}
fn default_composer_size() -> f32 {
    8.5
}
fn default_tempo_size() -> f32 {
    6.0
}
fn default_row_gap() -> f32 {
    8.0
}
fn default_title_top_offset() -> f32 {
    47.0
}
fn default_header_staff_gap() -> f32 {
    20.0
}
fn default_title_light() -> String {
    "#000000".into()
}
fn default_title_dark() -> String {
    "#DCDCDC".into()
}
fn default_subtitle_light() -> String {
    "#7C7C7C".into()
}
fn default_subtitle_dark() -> String {
    "#B4B4B4".into()
}
fn default_staff_light() -> String {
    "#000000".into()
}
fn default_staff_dark() -> String {
    "#E0E0E0".into()
}
fn default_note_light() -> String {
    "#000000".into()
}
fn default_note_dark() -> String {
    "#E0E0E0".into()
}
fn default_clef_light() -> String {
    "#000000".into()
}
fn default_clef_dark() -> String {
    "#E0E0E0".into()
}

fn default_staff_scale() -> f32 {
    1.0
}
fn default_treble_glyph_scale() -> f32 {
    4.2
}

// ── ScoreStylesheet methods ─────────────────────────────────

impl ScoreStylesheet {
    /// Loads all stylesheets from a directory.
    ///
    /// Scans `dir` for `*.toml` files, parses each one.
    /// Returns vector sorted alphabetically by name.
    /// If the directory doesn't exist or yields no valid sheets,
    /// returns a vec with only [`Self::builtin_default`].
    pub fn load_all(dir: &std::path::Path) -> Vec<Self> {
        let mut sheets: Vec<Self> = Vec::new();

        if let Ok(entries) = std::fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().and_then(|e| e.to_str()) == Some("toml") {
                    match std::fs::read_to_string(&path) {
                        Ok(content) => match toml::from_str::<Self>(&content) {
                            Ok(sheet) => sheets.push(sheet),
                            Err(e) => {
                                eprintln!(
                                    "WARN: failed to parse stylesheet '{}': {e}",
                                    path.display()
                                );
                            }
                        },
                        Err(e) => {
                            eprintln!("WARN: cannot read stylesheet '{}': {e}", path.display());
                        }
                    }
                }
            }
        } else {
            eprintln!("INFO: stylesheet directory not found: {}", dir.display());
        }

        sheets.sort_by(|a, b| a.name.cmp(&b.name));

        if sheets.is_empty() {
            sheets.push(Self::builtin_default());
        }

        sheets
    }

    /// Built-in fallback stylesheet matching the reference printed-score look.
    pub fn builtin_default() -> Self {
        Self {
            name: "Clásico".into(),
            description: "Estilo de partitura impresa — limpio, sin sombras ni bordes".into(),
            page: PageStyle {
                bg_light: default_bg_light(),
                bg_dark: default_bg_dark(),
                border_light: default_border_light(),
                border_dark: default_border_dark(),
                border_width: default_border_width(),
                shadow: ShadowStyle {
                    enabled: false,
                    offset_x: default_shadow_offset(),
                    offset_y: default_shadow_offset(),
                    alpha: default_shadow_alpha(),
                },
            },
            header: HeaderStyle {
                title_size: default_title_size(),
                composer_size: default_composer_size(),
                tempo_size: default_tempo_size(),
                row_gap: default_row_gap(),
                title_top_offset: default_title_top_offset(),
                header_staff_gap: default_header_staff_gap(),
                title_light: default_title_light(),
                title_dark: default_title_dark(),
                subtitle_light: default_subtitle_light(),
                subtitle_dark: default_subtitle_dark(),
            },
            notation: NotationStyle {
                staff_light: default_staff_light(),
                staff_dark: default_staff_dark(),
                note_light: default_note_light(),
                note_dark: default_note_dark(),
                clef_light: default_clef_light(),
                clef_dark: default_clef_dark(),
                clefs: ClefStyle {
                    treble: ClefVariant {
                        glyph_scale: 4.2,
                        fine_offset: -0.3,
                    },
                    bass: ClefVariant {
                        glyph_scale: 5.0,
                        fine_offset: 0.1,
                    },
                    alto: ClefVariant {
                        glyph_scale: 4.2,
                        fine_offset: 0.0,
                    },
                    tenor: ClefVariant {
                        glyph_scale: 4.2,
                        fine_offset: 0.0,
                    },
                    percussion: ClefVariant {
                        glyph_scale: 4.0,
                        fine_offset: 0.0,
                    },
                    tab: ClefVariant {
                        glyph_scale: 4.0,
                        fine_offset: 0.0,
                    },
                },
                staff_scale: default_staff_scale(),
            },
        }
    }

    /// Obtiene la configuración de glifo para la clave especificada.
    pub fn clef_variant(&self, clef: crate::notation::Clef) -> &ClefVariant {
        match clef {
            crate::notation::Clef::Treble => &self.notation.clefs.treble,
            crate::notation::Clef::Bass => &self.notation.clefs.bass,
            crate::notation::Clef::Alto => &self.notation.clefs.alto,
            crate::notation::Clef::Tenor => &self.notation.clefs.tenor,
            crate::notation::Clef::Percussion => &self.notation.clefs.percussion,
            crate::notation::Clef::Tab => &self.notation.clefs.tab,
        }
    }

    // ── Color resolvers ──

    fn resolve(hex_light: &str, hex_dark: &str, dark_mode: bool) -> egui::Color32 {
        let hex = if dark_mode { hex_dark } else { hex_light };
        parse_hex(hex)
    }

    pub fn page_bg(&self, dark_mode: bool) -> egui::Color32 {
        Self::resolve(&self.page.bg_light, &self.page.bg_dark, dark_mode)
    }

    pub fn page_border(&self, dark_mode: bool) -> egui::Color32 {
        Self::resolve(&self.page.border_light, &self.page.border_dark, dark_mode)
    }

    pub fn title_color(&self, dark_mode: bool) -> egui::Color32 {
        Self::resolve(&self.header.title_light, &self.header.title_dark, dark_mode)
    }

    pub fn subtitle_color(&self, dark_mode: bool) -> egui::Color32 {
        Self::resolve(
            &self.header.subtitle_light,
            &self.header.subtitle_dark,
            dark_mode,
        )
    }

    pub fn staff_color(&self, dark_mode: bool) -> egui::Color32 {
        Self::resolve(
            &self.notation.staff_light,
            &self.notation.staff_dark,
            dark_mode,
        )
    }

    pub fn note_color(&self, dark_mode: bool) -> egui::Color32 {
        Self::resolve(
            &self.notation.note_light,
            &self.notation.note_dark,
            dark_mode,
        )
    }

    pub fn clef_color(&self, dark_mode: bool) -> egui::Color32 {
        Self::resolve(
            &self.notation.clef_light,
            &self.notation.clef_dark,
            dark_mode,
        )
    }
}

// ── Helpers ─────────────────────────────────────────────────

/// Parses "#RRGGBB" or "#RRGGBBAA" into `Color32`.
/// Returns `Color32::BLACK` on invalid input.
fn parse_hex(s: &str) -> egui::Color32 {
    let s = s.trim();
    if s.len() < 7 || !s.starts_with('#') {
        return egui::Color32::BLACK;
    }
    let r = u8::from_str_radix(&s[1..3], 16).unwrap_or(0);
    let g = u8::from_str_radix(&s[3..5], 16).unwrap_or(0);
    let b = u8::from_str_radix(&s[5..7], 16).unwrap_or(0);
    let a = if s.len() >= 9 {
        u8::from_str_radix(&s[7..9], 16).unwrap_or(255)
    } else {
        255
    };
    egui::Color32::from_rgba_premultiplied(r, g, b, a)
}

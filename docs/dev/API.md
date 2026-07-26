# API.md — API Interna de Módulos

m-guitar es una aplicación de escritorio sin API de red.
Este documento describe la interfaz interna entre módulos una vez que el código se estructure.

---

## Estado actual

La aplicación reside completamente en `src/main.rs` (~250 líneas).
No hay API de módulos definida — toda la lógica está en el struct `MyEguiApp` y su implementación de `eframe::App`.

---

## Módulos actuales (inline en main.rs)

### `Lang` enum

```rust
enum Lang { Es, En }
```

Idiomas soportados. Se usa como estado en `I18n`.

### `I18n`

```rust
struct I18n { lang: Lang }

impl I18n {
    fn new(lang: Lang) -> Self;
    fn t<'a>(&self, key: &'a str) -> &'a str;
}
```

Traducción por lookup de claves. Las claves válidas son las documentadas en `DATA_MODEL.md > i18n`.

### `MyEguiApp`

```rust
struct MyEguiApp {
    window_open: bool,
    dark_mode: bool,
    i18n: I18n,
    first_frame: bool,
}

impl MyEguiApp {
    fn new(cc: &eframe::CreationContext<'_>) -> Self;
}

impl eframe::App for MyEguiApp {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame);
}
```

Estado global de la aplicación y loop de renderizado.

---

## Proyección — API de módulos futura

Cuando `main.rs` supere ~500 líneas, se extraerán los siguientes módulos:

### `src/i18n.rs`

```rust
pub enum Lang { Es, En }

pub struct I18n { /* ... */ }

impl I18n {
    pub fn new(lang: Lang) -> Self;
    pub fn t(&self, key: &str) -> &str;
    pub fn toggle(&mut self);
}
```

### `src/notation/mod.rs`

```rust
pub mod clef;
pub mod note;
pub mod staff;
```

### `src/notation/note.rs`

```rust
pub enum Pitch { C, D, E, F, G, A, B }

pub enum Accidental { Natural, Sharp, Flat }

pub enum NoteFigure { Whole, Half, Quarter, Eighth, Sixteenth }

pub struct Note {
    pub pitch: Pitch,
    pub octave: u8,
    pub accidental: Accidental,
    pub figure: NoteFigure,
}

impl Note {
    pub fn glyph(&self) -> char;      // glifo Leland
    pub fn name(&self) -> &str;       // nombre en español
}
```

### `src/fonts.rs`

```rust
pub fn configure_fonts(ctx: &egui::Context);
```

Centraliza la carga de fuentes (actualmente en `MyEguiApp::new`).

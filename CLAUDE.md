# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project

m-guitar (crate name `m-guitar`, binary/lib name `m_guitar`) is a native music notation editor written in Rust, built on `egui`/`eframe`. It renders SMuFL music glyphs using the embedded Leland font and can import scores from MusicXML. It targets native (Linux/macOS/Windows) and WebAssembly (browser) via the same `eframe` codebase.

## Commands

```bash
cargo build                    # compile (debug)
cargo build --release          # compile, optimized — use for actually running the app, egui is slow in debug
cargo run --release            # run the native app
cargo test                     # run all tests (unit tests in src/, integration tests in tests/smoke.rs)
cargo test --test smoke        # run only the MusicXML import integration tests
cargo test <name> -- --nocapture   # run a single test by name with output
cargo fmt                      # format (rustfmt defaults)
cargo clippy -- -D warnings    # lint, must be warning-free before a PR
cargo check                    # fast type-check without codegen
```

WASM build (requires `rustup target add wasm32-unknown-unknown` and `cargo install wasm-bindgen-cli`):
```bash
./build-wasm.sh [release|debug]   # builds, runs wasm-bindgen, stages output in web-dist/
cd web-dist && python3 serve.py   # serve at http://localhost:8080
```

System dependencies for building (Linux) are in `ENV.md`.

## Architecture

### Three-layer pipeline: MusicXML → domain model → renderer

1. **`src/musicxml/`** — `parser.rs` (~1600 lines) parses MusicXML 4.0 (via `roxmltree`, no allocation-heavy DOM) into the domain model in `src/notation/`. `error.rs` defines `MusicXmlError`. The parser aims for strict XSD 4.0 compliance (clef-octave-change, senza-misura, compound time, score-timewise, etc.) — see `docs/dev/DATA_MODEL.md` for the glyph/field mapping this was built against. The local copy of the spec/XSD lives in `lib/musicxml-4.0/`.

2. **`src/notation/`** — the domain model: `Score` → `System` (simultaneous staves, e.g. a grand staff or a guitar's standard+tab pair) → `Staff` → `Measure` → `MeasureElement` (`Note`/`Rest`/`Chord`/`Backup`/`Forward`). Each concept (clef, pitch, key signature, barline, tuplet, tablature, note attachments/ornaments/articulations, etc.) is its own small module, all re-exported flat from `notation::mod`. This is pure data — no rendering or egui types leak in here. `docs/dev/DATA_MODEL.md` documents each type's fields and their SMuFL glyph/Unicode mapping.

3. **`src/render/`** — turns the domain model into `egui::Painter` draw calls. `score.rs` (`render_score`) walks a `Score` and draws it as one continuous vertical strip; `page.rs` (`compute_pages`/`render_pages`) is the paginated A4 layout used by the app (see ADR-005 in `docs/dev/DECISIONS.md`) — staves flow across virtual A4 pages with zoom. Other modules render one concern each (`clef.rs`, `note.rs`/`stem.rs`/`beam.rs`, `key.rs`, `staff.rs`/`rest.rs`, `attachment.rs` for slurs/ties/ornaments, `tab.rs` for tablature, `direction.rs` for dynamics/wedges/octave shifts, `lyric.rs`). Note: several render submodules are re-exported under a `_render` suffix in `render/mod.rs` (e.g. `clef.rs` → `clef_render`) to avoid name clashes with the `notation` types of the same name — check `render/mod.rs` when looking for where something lives. `stylesheet.rs` defines `ScoreStylesheet`, a serde/TOML-deserialized visual preset (colors, sizes, shadows) loaded from `assets/stylesheets/*.toml` (bundled) or `~/.config/m-guitar/stylesheets/` (user-added), selectable at runtime.

### `src/app.rs` — UI and app state

`MGuitarApp` is the `eframe::App` implementation; it's tab-based, holding `Vec<Document>` (one per open score) plus the active tab index. Each `Document` owns a `Score`, zoom level, dirty flag, and note-entry state (`pending_step`/`pending_figure_digits` for the keyboard note-entry mini-DSL: a letter C–B picks the step, then digits 1/2/4/8/6/32/33 pick the figure). Session state (open documents/paths) persists via `dirs::config_dir()`. The window is borderless (`with_decorations(false)`) with a custom top bar standing in for menu + titlebar (ADR-003) — there's no native title bar or window chrome.

### Fonts and i18n

`src/fonts.rs` embeds the Leland (`FontFamily::Name("Leland")`, SMuFL glyphs) and LelandText (`FontFamily::Proportional`, doubles as the UI font) fonts at compile time via `include_bytes!` — there is no fallback UI font. `src/i18n.rs` is a zero-allocation `match`-based lookup (`I18n::t(key) -> &'static str`) over `Lang::{Es, En}`; adding a string requires adding the key to both language arms. UI strings must never be hardcoded — always go through `I18n::t()`.

## Conventions

- Keep `main.rs` thin; put logic in modules (`app.rs`, `notation/`, `render/`, `musicxml/`) — extract when a file starts approaching ~500 lines.
- Group UI state into structs; use egui's builder/closure pattern (`ui.horizontal(|ui| { ... })`) and avoid nesting more than 3-4 levels deep.
- Domain code (`notation/`, `musicxml/`) stays free of `egui` types; only `render/` and `app.rs` should depend on egui.

## Documentation upkeep

This repo tracks design/architecture decisions in `docs/`. `AGENTS.md` has the full change→doc mapping table; the ones most relevant to code changes:
- Non-obvious technical decision → add an ADR to `docs/dev/DECISIONS.md` (see existing ADRs there for the format and rationale style).
- New/changed domain type or SMuFL glyph mapping → `docs/dev/DATA_MODEL.md`.
- New Cargo dependency, embedded font/resource, or i18n key set → `docs/dev/SETUP.md`.
- User-visible feature/fix/breaking change → `CHANGELOG.md` (under `[Sin publicar]`, Keep a Changelog format, Spanish).

Commit messages follow Conventional Commits with a Spanish, imperative description and a scope, e.g. `feat(notation): agregar renderizado de pentagrama` — see `docs/dev/CONTRIBUTING.md` for the full type list and branching model (`main`/`develop`/`feature/*`/`fix/*`).

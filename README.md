# m-guitar

A native music notation application built with Rust and [egui](https://github.com/emilk/egui).

Uses the [Leland](https://github.com/MuseScoreFonts/Leland) music engraving font (SIL Open Font License) for high-quality glyph rendering.

## Features

- **Native GUI** via `eframe`/`egui` — no browser, no Electron
- **Leland font** embedded at compile time for notation glyphs and UI text
- **Dark/light mode** toggle
- **i18n** — Spanish and English
- **Borderless window** — custom titlebar area with integrated menu bar
- Clef rendering — G clef (treble), F clef (bass)
- Note figure display — whole, half, quarter, eighth, sixteenth

## Build

```sh
cargo build --release
```

## Run

```sh
cargo run --release
```

## License

Source code: TBD.

The Leland font (`lib/MusicFonts/Leland/`) is licensed under the [SIL Open Font License 1.1](lib/MusicFonts/Leland/LICENSE.txt).

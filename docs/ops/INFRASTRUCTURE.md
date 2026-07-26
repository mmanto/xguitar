# INFRASTRUCTURE.md — Infraestructura

m-guitar es una aplicación de escritorio autocontenida. No tiene infraestructura de servidores, bases de datos ni servicios externos.

---

## Arquitectura de ejecución

```
┌─────────────────────────────────────────┐
│              Sistema Operativo           │
│  ┌───────────────────────────────────┐  │
│  │         m-guitar (binario)        │  │
│  │  ┌─────────────────────────────┐  │  │
│  │  │  eframe (winit + egui)      │  │  │
│  │  │  ┌───────────────────────┐  │  │  │
│  │  │  │  Wgpu / OpenGL        │  │  │  │
│  │  │  │  (GPU rendering)      │  │  │  │
│  │  │  └───────────────────────┘  │  │  │
│  │  └─────────────────────────────┘  │  │
│  │  ┌─────────────────────────────┐  │  │
│  │  │  Leland.otf  (embebido)     │  │  │
│  │  │  LelandText.otf (embebido)  │  │  │
│  │  └─────────────────────────────┘  │  │
│  └───────────────────────────────────┘  │
└─────────────────────────────────────────┘
```

Un solo binario. Sin procesos externos, sin red, sin bases de datos.
Las fuentes se embeben en tiempo de compilación con `include_bytes!`.

---

## Build pipeline

```
src/main.rs ──┐
              ├──► rustc ──► m-guitar (binario estático)
Leland.otf ───┘
```

- **Compiler:** rustc vía Cargo
- **Linker:** default de la plataforma
- **Optimización:** LTO en release (`cargo build --release`)

---

## Distribución

### Linux

```bash
cargo build --release
# Binario en target/release/m-guitar
```

**Dependencias runtime:**
- `libxcb`, `libxkbcommon`, `libgtk-3` (X11)
- `wayland-client` (Wayland)

### macOS

```bash
cargo build --release
# Bundle .app con cargo-bundle
cargo install cargo-bundle
cargo bundle --release
```

### Windows

```bash
cargo build --release
# Binario en target/release/m-guitar.exe
```

---

## CI/CD (proyección)

```yaml
# .github/workflows/build.yml (propuesto)
on: [push, pull_request]

jobs:
  build:
    runs-on: ${{ matrix.os }}
    strategy:
      matrix:
        os: [ubuntu-latest, macos-latest, windows-latest]
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - run: cargo build --release
      - run: cargo test
      - run: cargo clippy -- -D warnings
```

---

## Sin estado persistente (actual)

La aplicación no guarda configuración ni archivos. A futuro se usará:

- `dirs::config_dir()` para preferencias (idioma, tema)
- `rfd::FileDialog` para abrir/guardar archivos de partitura

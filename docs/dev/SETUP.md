# SETUP.md — Setup de Entorno Local

Guía paso a paso para levantar m-guitar en modo desarrollo.

---

## Prerrequisitos

| Herramienta | Versión mínima | Instalación |
|---|---|---|
| Rust | 1.85+ | [rustup.rs](https://rustup.rs) |
| Cargo | incluido con Rust | |
| Git | 2.40+ | |

Ver `ENV.md` para dependencias de sistema por plataforma.

---

## Clonar y compilar

```bash
# 1. Clonar repositorio
git clone <repo-url> && cd m-guitar

# 2. Compilar
cargo build

# 3. Ejecutar en modo desarrollo
cargo run

# 4. (Recomendado) Ejecutar en release para uso diario
cargo run --release
```

---

## Comandos frecuentes

```bash
# Compilar
cargo build

# Compilar optimizado
cargo build --release

# Ejecutar
cargo run

# Ejecutar optimizado
cargo run --release

# Formatear código
cargo fmt

# Lint
cargo clippy

# Lint estricto (sin warnings)
cargo clippy -- -D warnings

# Ejecutar tests
cargo test

# Ejecutar tests con output
cargo test -- --nocapture

# Verificar que compila sin generar binario
cargo check
```

---

## Estructura del proyecto

```
.
├── src/
│   ├── main.rs              # Entry point, UI, estado de la app
│   ├── lib.rs               # Re-exports públicos
│   ├── i18n.rs              # Internacionalización
│   ├── fonts.rs             # Carga de fuentes
│   ├── notation/            # Lógica de dominio musical
│   ├── render/              # Renderizado de partituras
│   │   ├── constants.rs     # Constantes visuales
│   │   ├── stylesheet.rs    # ScoreStylesheet (TOML presets)
│   │   ├── page.rs          # Layout A4, paginación
│   │   ├── score.rs         # RenderStyle, render_score
│   │   ├── clef.rs          # Renderizado de claves
│   │   ├── staff.rs         # Líneas del pentagrama
│   │   └── note.rs          # Cabezas de nota
│   └── musicxml/            # Importación de archivos MusicXML
│       ├── mod.rs
│       ├── error.rs
│       └── parser.rs
├── assets/
│   └── stylesheets/         # Presets de estilo embebidos
│       ├── classical.toml   # Estilo clásico (default)
│       └── dark.toml        # Fondo oscuro
├── docs/                    # Documentación técnica
│   ├── dev/
│   ├── design/
│   ├── ops/
│   └── qa/
├── AGENTS.md
├── CHANGELOG.md
├── ENV.md
├── Cargo.toml
└── Cargo.lock
```

---

## Troubleshooting

**Error de linkeo en Linux (libraries no encontradas)**
```bash
# Debian/Ubuntu
sudo apt install build-essential cmake pkg-config \
  libxcb-render0-dev libxcb-shape0-dev libxcb-xfixes0-dev \
  libxkbcommon-dev libssl-dev libgtk-3-dev libclang-dev

# Arch
sudo pacman -S cmake gtk3 base-devel
```

**Error: "could not find native static library"**
```bash
# Limpiar y recompilar
cargo clean
cargo build
```

**La ventana no aparece o crashea al iniciar**
```bash
# Ejecutar con logs
RUST_LOG=debug cargo run
```

**egui se ve lento en modo debug**
Esperado. egui usa GPU por software en debug. Compilar con `--release` para rendimiento nativo:

```bash
cargo run --release
```

---

## Dependencias del proyecto

| Crate | Versión | Uso |
|---|---|---|
| `eframe` | 0.35.0 | Framework de ventana + egui integrado |
| `serde` | 1 | (de)serialización con derive macros |
| `toml` | 0.8 | Parseo de archivos TOML para stylesheets |
| `dirs` | 6 | Directorios de configuración cross-platform |
| `roxmltree` | 0.20 | Parseo de XML sin allocaciones (MusicXML) |
| `rfd` | 0.15 | Diálogo de archivos nativo cross-platform |
| `cpal` | 0.18 | I/O de audio cross-platform (solo nativo) — ver ADR-008 |
| `libc` | 0.2 | Tipos C para el FFI a `sfizz` (solo nativo) |
| `pkg-config` | 0.3 | Build-dependency: ubica `libsfizz` en `build.rs` |

`sfizz` (librería de sistema, no crate — no existe uno publicado) se linkea
dinámicamente en build nativo. Ver `ENV.md` para instalarla por plataforma y
para cómo apuntar la app a un instrumento `.sfz` de guitarra.

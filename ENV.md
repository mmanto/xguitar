# ENV.md — Entorno de desarrollo

Requisitos de toolchain y entorno para compilar y ejecutar m-guitar.

---

## Toolchain

| Herramienta | Versión mínima | Instalación |
|---|---|---|
| Rust | 1.85+ (edition 2024) | [rustup.rs](https://rustup.rs) |
| Cargo | incluido con Rust | |
| Git | 2.40+ | |

---

## Dependencias del sistema

`eframe` requiere algunas bibliotecas nativas según la plataforma. La
reproducción de audio (`cpal` + `sfizz`, ver ADR-008) agrega, solo en build
nativo, la librería de sistema `sfizz` (con su `.pc` de pkg-config) más las
libs de audio habituales de Linux (ALSA, que `cpal` usa como backend — en
sistemas con PipeWire esto ya se resuelve de forma transparente vía la capa
de compatibilidad `pipewire-alsa`/`pipewire-pulse`, sin dependencias extra).

### Linux (Debian/Ubuntu)

```bash
sudo apt install libxcb-render0-dev libxcb-shape0-dev libxcb-xfixes0-dev \
  libxkbcommon-dev libssl-dev libgtk-3-dev libclang-dev \
  libasound2-dev pkg-config
```

`sfizz` no tiene paquete oficial en los repos de Debian/Ubuntu — hay que
compilarlo desde [sfztools/sfizz](https://github.com/sfztools/sfizz) (o usar
el repo OBS de terceros `home:sfztools:sfizz`) para tener `libsfizz.so` y su
`.pc` de pkg-config disponibles.

### Linux (Arch)

```bash
sudo pacman -S cmake gtk3 alsa-lib pkgconf sfizz-lib
```

### macOS

Xcode Command Line Tools:

```bash
xcode-select --install
```

`sfizz` no está confirmado como fórmula oficial de Homebrew — probar
`brew search sfizz` y, si no aparece, compilarlo desde
[sfztools/sfizz](https://github.com/sfztools/sfizz) (el build nativo de
xguitar lo busca vía pkg-config igual que en Linux).

### Windows

Además de Rust, hace falta `sfizz` instalado con su `.pc` de pkg-config
disponible (o su equivalente vía vcpkg) para que `build.rs` pueda linkearlo.

### Instrumento SFZ para reproducción

La reproducción no trae un instrumento de guitarra embebido en el repo (los
samples pesan decenas de MB, no se versionan en git). Por defecto la app
busca `~/.config/m-guitar/instruments/default.sfz` (`%APPDATA%\m-guitar\instruments\default.sfz`
en Windows). Cualquier instrumento SFZ de guitarra sirve — se recomienda
[VCSL](https://github.com/sgossner/VCSL) (CC0, carpeta `Chordophones`) por
ser de dominio público y no requerir atribución. Sin instrumento configurado,
el botón Play igual funciona pero no se escucha nada (se muestra un aviso).

---

## Variables de entorno

m-guitar no requiere variables de entorno en tiempo de ejecución.
Toda la configuración está embebida en el binario.

Para desarrollo:

| Variable | Requerida | Descripción |
|---|---|---|
| `RUST_LOG` | ❌ | Nivel de log (`debug`, `info`, `warn`) |

---

## Perfil de release

Compilar en release para uso diario (mejora drásticamente el rendimiento de egui):

```bash
cargo build --release
cargo run --release
```

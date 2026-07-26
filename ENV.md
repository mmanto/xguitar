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

`eframe` requiere algunas bibliotecas nativas según la plataforma.

### Linux (Debian/Ubuntu)

```bash
sudo apt install libxcb-render0-dev libxcb-shape0-dev libxcb-xfixes0-dev \
  libxkbcommon-dev libssl-dev libgtk-3-dev libclang-dev
```

### Linux (Arch)

```bash
sudo pacman -S cmake gtk3
```

### macOS

Xcode Command Line Tools:

```bash
xcode-select --install
```

### Windows

No requiere dependencias adicionales más allá de Rust.

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

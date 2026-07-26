# RUNBOOK.md — Runbook Operacional

Procedimientos de desarrollo, troubleshooting y respuesta a problemas.

---

## Comandos rápidos

| Qué | Comando |
|---|---|
| Compilar | `cargo build` |
| Compilar release | `cargo build --release` |
| Ejecutar | `cargo run --release` |
| Ver warnings | `cargo clippy` |
| Formatear | `cargo fmt` |
| Tests | `cargo test` |
| Limpiar build artifacts | `cargo clean` |
| Ver dependencias | `cargo tree` |
| Actualizar dependencias | `cargo update` |

---

## Procedimientos comunes

### Agregar una dependencia

```bash
cargo add <crate>
# o editar Cargo.toml manualmente y luego:
cargo update
```

### Actualizar eframe

```bash
cargo update -p eframe
cargo build --release  # verificar que no rompe
```

### Agregar un nuevo glifo musical

1. Buscar el código Unicode en la [especificación SMuFL](https://w3c.github.io/smufl/latest/)
2. Verificar que Leland incluye el glifo (abrir `.otf` en un visor de fuentes)
3. Usar el código en `RichText::new("\u{XXXX}")` con `FontFamily::Name("Leland")`

### Agregar una nueva clave i18n

1. Agregar la clave en ambos brazos del `match` en `I18n::t()`
2. Si la clave se usa en UI, referenciarla como `self.i18n.t("nueva_clave")`
3. Actualizar la tabla de i18n en `docs/dev/DATA_MODEL.md`

---

## Troubleshooting

### La app crashea al iniciar

```bash
RUST_LOG=debug cargo run
```

Causas comunes:
- Falta una dependencia del sistema (ver `ENV.md`)
- `include_bytes!` apunta a un archivo que no existe (verificar `lib/MusicFonts/Leland/`)

### Los glifos no se ven (cuadrados o espacios)

Los glifos `\u{E000}`–`\u{EFFF}` requieren la fuente Leland cargada.
Verificar que:
1. `Leland.otf` y `LelandText.otf` existen en `lib/MusicFonts/Leland/`
2. `FontDefinitions` se configura en `MyEguiApp::new()` antes del primer frame
3. Se usa `FontFamily::Name("Leland")` (no `FontFamily::Proportional`)

### egui está muy lento

Modo debug de egui renderiza por software. La diferencia con release es de 10-50x.
Usar siempre `cargo run --release` para desarrollo diario de UI.

```bash
cargo run --release
```

### Error: "could not find Leland in fonts"

La fuente se carga en `MyEguiApp::new()`. Si el error ocurre, es porque `include_bytes!` falló.
Verificar la ruta relativa: `"../lib/MusicFonts/Leland/Leland.otf"` desde `src/main.rs`.

---

## Plataformas

| Plataforma | Backend | Estado |
|---|---|---|
| Linux (X11/Wayland) | Wgpu / OpenGL | Soportado |
| macOS | Metal | Teórico (no testeado) |
| Windows | DirectX / OpenGL | Teórico (no testeado) |

Probar en otras plataformas:

```bash
# Compilar para Windows desde Linux
rustup target add x86_64-pc-windows-msvc
cargo build --release --target x86_64-pc-windows-msvc
```

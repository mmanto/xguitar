# AGENTS.md

Instrucciones para agentes de IA que trabajen en este repositorio.
Leer este archivo antes de realizar cualquier tarea.

---

## Regla principal

> **Nunca cerrar una tarea sin verificar si algún documento de `/docs` debe actualizarse.**

---

## Mapa de cambios → documentos

| Tipo de cambio | Archivos a actualizar |
|---|---|
| Nueva pantalla, vista o cambio de layout | `docs/design/SCREENS.md` |
| Cambio en componentes visuales o estilos | `docs/design/DESIGN.md` |
| Cambio en la estructura de layouts | `docs/design/LAYOUT.md` |
| Decisión técnica relevante (ADR) | `docs/dev/DECISIONS.md` |
| Cambio en el modelo de dominio (notas, compases, etc.) | `docs/dev/DATA_MODEL.md` |
| Feature completada, bug corregido, breaking change | `CHANGELOG.md` |
| Cambio en dependencias de Cargo | `docs/dev/SETUP.md` |
| Nueva fuente o recurso embebido | `docs/dev/SETUP.md` |
| Cambio en convenciones de código | `docs/dev/CONTRIBUTING.md` |
| Cambio en i18n (nuevas claves, idiomas) | `docs/dev/SETUP.md` |

---

## Stack del proyecto

- **Lenguaje:** Rust (edition 2024)
- **GUI:** egui / eframe 0.35
- **Fuente musical:** Leland (SIL Open Font License)
- **Build:** Cargo
- **Plataformas:** Linux, macOS, Windows (eframe cross-platform)

---

## Convenciones de código

### Rust
- `cargo fmt` antes de commitear (rustfmt con defaults)
- `cargo clippy` sin warnings antes de abrir PR
- Módulos planos mientras el proyecto sea pequeño; refactorizar a `lib.rs` + submódulos cuando `main.rs` supere ~500 líneas
- Strings de UI via `I18n::t()` — nunca hardcodear texto visible en español o inglés
- Fuentes embebidas con `include_bytes!` en `main.rs`
- Estados de UI en `MyEguiApp`; lógica de dominio en módulos separados

### Estructura esperada a futuro
```
src/
├── main.rs          # Entry point, app state, UI
├── i18n.rs          # Internacionalización
├── notation/        # Lógica de notación musical
│   ├── mod.rs
│   ├── note.rs
│   ├── clef.rs
│   └── staff.rs
└── fonts.rs         # Carga de fuentes
```

---

## Formato de commits

```
<tipo>(<scope>): <descripción breve en español>

feat: nueva funcionalidad
fix: corrección de bug
docs: actualización de documentación
refactor: reestructuración sin cambio funcional
chore: tareas de mantenimiento
test: agregar o modificar tests
```

---

## Antes de finalizar cualquier tarea

1. ¿Se agregó o modificó una pantalla o vista? → Actualizar `SCREENS.md`
2. ¿Se tomó una decisión técnica no obvia? → Agregar ADR en `DECISIONS.md`
3. ¿El cambio es visible para el usuario final? → Actualizar `CHANGELOG.md`
4. ¿Se agregó una dependencia? → Actualizar `SETUP.md`
5. ¿Cambió la estructura de layouts o componentes? → Actualizar `LAYOUT.md` o `DESIGN.md`

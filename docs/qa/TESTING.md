# TESTING.md — Estrategia de Testing

Guía de testing para m-guitar (Rust).

---

## Pirámide de tests

```
         /\
        /E2E\          ← Tests manuales de UI (abrir app, verificar glifos)
       /──────\
      /Integración\    ← Tests de renderizado con egui test harness
     /────────────\
    /  Unitarios   \   ← Tests de lógica de dominio (i18n, parsing de notas)
   /────────────────\
```

**Cobertura objetivo (proyección):**
- Lógica de dominio (`src/notation/`): 80%
- i18n: 100% (pocas claves, alta criticidad)
- UI: validación manual + smoke tests con `eframe` test harness

---

## Tests unitarios

### Estructura (proyección)

```
src/
├── main.rs
├── i18n.rs
├── notation/
│   ├── mod.rs
│   ├── note.rs
│   └── note_test.rs     # Tests inline o en archivo separado
```

### Ejemplo: test de i18n

```rust
// En src/i18n.rs o src/main.rs
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_i18n_es() {
        let i18n = I18n::new(Lang::Es);
        assert_eq!(i18n.t("file"), "Archivo");
        assert_eq!(i18n.t("new"), "Nuevo");
        assert_eq!(i18n.t("unknown_key"), "unknown_key"); // fallback
    }

    #[test]
    fn test_i18n_en() {
        let i18n = I18n::new(Lang::En);
        assert_eq!(i18n.t("file"), "File");
        assert_eq!(i18n.t("close"), "Close");
    }

    #[test]
    fn test_i18n_all_keys_have_translations() {
        // Verificar que las mismas claves existen en ambos idiomas
        let es = I18n::new(Lang::Es);
        let en = I18n::new(Lang::En);
        let keys = ["file", "new", "open", "close", "exit", "app_title", "lang_toggle"];
        for key in keys {
            let es_val = es.t(key);
            let en_val = en.t(key);
            assert_ne!(es_val, key, "key '{}' missing ES translation", key);
            assert_ne!(en_val, key, "key '{}' missing EN translation", key);
            assert_ne!(es_val, en_val, "key '{}' has same value in ES and EN", key);
        }
    }
}
```

### Ejemplo: test de dominio (proyección)

```rust
// src/notation/note.rs
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_note_glyph() {
        let note = Note {
            pitch: Pitch::C,
            octave: 4,
            accidental: Accidental::Natural,
            figure: NoteFigure::Quarter,
        };
        assert_eq!(note.glyph(), '\u{E1D5}'); // negra
    }

    #[test]
    fn test_note_name_es() {
        let note = Note {
            pitch: Pitch::G,
            octave: 4,
            accidental: Accidental::Sharp,
            figure: NoteFigure::Quarter,
        };
        assert_eq!(note.name_es(), "Sol sostenido");
    }
}
```

---

## Tests de integración

### egui test harness

egui no tiene un test runner headless oficial, pero se puede testear el state:

```rust
#[test]
fn test_app_creates_with_fonts() {
    // Usar eframe::CreationContext de test
    // Verificar que las fuentes se cargaron sin pánico
}

#[test]
fn test_dark_mode_toggle() {
    let mut app = MyEguiApp::new(/* test cc */);
    assert!(app.dark_mode);
    app.dark_mode = false;
    assert!(!app.dark_mode);
}

#[test]
fn test_lang_toggle() {
    let mut app = MyEguiApp::new(/* test cc */);
    assert_eq!(app.i18n.lang, Lang::Es);
    app.i18n.lang = Lang::En;
    assert_eq!(app.i18n.lang, Lang::En);
}
```

---

## Smoke test manual

Antes de cada release, verificar manualmente:

1. `cargo run --release` → la ventana aparece maximizada
2. Glifos de claves: `\u{E050}` (Sol) y `\u{E062}` (Fa) se ven como símbolos musicales, no cuadrados
3. Glifos de figuras: los 5 glifos se ven correctamente
4. Toggle EN/ES: cambia textos del menú ("Archivo" ↔ "File")
5. Toggle dark mode: alterna entre tema oscuro y claro
6. Menú Archivo: despliega opciones sin crashear
7. Cerrar ventana con "Salir" del menú

---

## Ejecutar tests

```bash
# Todos los tests
cargo test

# Con output
cargo test -- --nocapture

# Un test específico
cargo test test_i18n_es

# Tests de un módulo
cargo test notation::

# Con cobertura (requiere cargo-tarpaulin)
cargo install cargo-tarpaulin
cargo tarpaulin --out Html
```

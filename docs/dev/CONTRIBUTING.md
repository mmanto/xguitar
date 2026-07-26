# CONTRIBUTING.md — Guía de Contribución

Convenciones de desarrollo para m-guitar.

---

## Branching strategy (Git Flow simplificado)

```
main           → producción (protegida, solo merge via PR)
develop        → integración (base para feature branches)
feature/xxx    → nuevas funcionalidades
fix/xxx        → correcciones de bugs
hotfix/xxx     → fixes urgentes directo a main
chore/xxx      → mantenimiento (deps, config)
docs/xxx       → solo documentación
```

### Flujo de trabajo

```bash
# 1. Partir siempre desde develop actualizado
git checkout develop && git pull

# 2. Crear branch
git checkout -b feature/nombre-descriptivo

# 3. Desarrollar con commits atómicos
git add -p  # staging parcial
git commit -m "feat(notation): agregar pentagrama interactivo"

# 4. Push y abrir PR hacia develop
git push origin feature/nombre-descriptivo
```

---

## Convención de commits (Conventional Commits)

```
<tipo>(<scope>): <descripción en español, imperativo>

feat(notation): agregar renderizado de pentagrama
fix(i18n): corregir clave de traducción de "file"
docs(readme): actualizar stack del proyecto
refactor(ui): extraer top_bar a función separada
chore(deps): actualizar eframe a 0.36
test(notation): agregar tests de parsing de notas
```

**Tipos válidos:**
- `feat` — nueva funcionalidad
- `fix` — corrección de bug
- `docs` — solo documentación
- `refactor` — reestructuración sin cambio funcional
- `test` — agregar o modificar tests
- `chore` — mantenimiento, dependencias
- `perf` — mejora de rendimiento

---

## Pull Requests

### Checklist antes de abrir un PR

- [ ] `cargo fmt` sin cambios pendientes
- [ ] `cargo clippy` sin warnings
- [ ] `cargo build` exitoso
- [ ] `cargo test` pasando
- [ ] Documentación actualizada si corresponde (`AGENTS.md` como guía)
- [ ] `CHANGELOG.md` actualizado si hay cambios visibles al usuario

### Template de descripción de PR

```markdown
## ¿Qué hace este PR?
Breve descripción del cambio.

## Tipo de cambio
- [ ] Nueva funcionalidad
- [ ] Corrección de bug
- [ ] Refactor
- [ ] Documentación

## Testing
Cómo probar el cambio.

## Screenshots (si aplica)
```

---

## Estándares de código

### Rust

- **Formatter:** `rustfmt` (defaults)
- **Linter:** `clippy` (`cargo clippy -- -D warnings`)
- **Idioma:** inglés para código (nombres de tipos, funciones, variables); español para UI strings via i18n
- **Módulos:** mantener `main.rs` por debajo de ~500 líneas; extraer a módulos cuando crezca
- **UI strings:** nunca hardcodear texto visible — usar `I18n::t()`

```rust
// ✅ Correcto — texto via i18n
ui.button(self.i18n.t("file"));

// ❌ Incorrecto — texto hardcodeado
ui.button("Archivo");
```

- **Fuentes:** embebidas con `include_bytes!` en tiempo de compilación
- **Estados:** agrupar estado de UI en structs con `Default` o constructores explícitos
- **Patrones egui:** usar el patrón de builder de egui (`ui.horizontal(|ui| { ... })`); no anidar más de 3-4 niveles

---

## Code review

- Mínimo 1 aprobación para merge a `develop`
- Mínimo 2 aprobaciones para merge a `main`
- Usar comentarios sugestivos, no imperativos: "¿Qué pensás de hacer X?" > "Hacé X"
- Resolver todos los comentarios antes de hacer merge

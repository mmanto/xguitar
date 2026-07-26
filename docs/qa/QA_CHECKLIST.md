# QA_CHECKLIST.md — Checklist de Entrega

Verificar todos los ítems antes de cada release o merge a `main`.

---

## Checklist de Pull Request

### Código
- [ ] `cargo fmt` sin cambios pendientes
- [ ] `cargo clippy` sin warnings
- [ ] `cargo build` y `cargo build --release` exitosos
- [ ] `cargo test` pasando
- [ ] Sin `println!`, `dbg!` o `todo!()` olvidados
- [ ] Sin credenciales o tokens en el código

### Documentación
- [ ] `CHANGELOG.md` actualizado si hay cambios visibles al usuario
- [ ] `DATA_MODEL.md` actualizado si se agregaron glifos o entidades
- [ ] `SCREENS.md` actualizado si cambió la UI
- [ ] `DECISIONS.md` actualizado si se tomó una decisión técnica no obvia

---

## Checklist de Release

### Funcionalidad
- [ ] Glifos de claves se renderizan correctamente (Sol, Fa)
- [ ] Glifos de figuras se renderizan correctamente (5 figuras)
- [ ] Toggle de idioma (ES ↔ EN) cambia todos los textos de UI
- [ ] Toggle de modo oscuro/claro alterna correctamente
- [ ] Menú Archivo despliega y cierra correctamente
- [ ] La ventana se maximiza al iniciar (primer frame)
- [ ] La ventana muestra la barra superior sin decoraciones nativas

### UX/UI
- [ ] Sin texto cortado o glifos superpuestos
- [ ] Consistencia visual entre modo claro y oscuro
- [ ] Fuente LelandText usada en toda la UI (no hay fallback a sistema)
- [ ] Espaciado entre secciones y glifos consistente

### Multi-plataforma
- [ ] Compila en Linux
- [ ] Compila en macOS (si hay acceso)
- [ ] Compila en Windows (si hay acceso)

---

## Checklist pre-merge a main

- [ ] Todos los tests pasan
- [ ] PR aprobado por al menos 1 revisor
- [ ] Branch actualizada con `main` (sin conflictos)
- [ ] Commits squasheados si son muchos micro-commits
- [ ] Mensaje de merge en español, formato conventional commits

---

## Verificaciones manuales de release

| Verificación | Responsable | Fecha |
|---|---|---|
| Glifos de claves visibles | | |
| Glifos de figuras visibles | | |
| Toggle ES/EN funcional | | |
| Toggle dark/light funcional | | |
| Menú Archivo funcional | | |
| Ventana maximizada al iniciar | | |

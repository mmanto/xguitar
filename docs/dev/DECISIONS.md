# DECISIONS.md — Architecture Decision Records (ADRs)

Registro de decisiones técnicas significativas con su contexto y razonamiento.
Una decisión documentada aquí no debe cuestionarse sin agregar un nuevo ADR.

---

## Formato de ADR

```
## ADR-XXX: Título
**Estado:** Propuesto | Aceptado | Obsoleto | Reemplazado por ADR-YYY
**Fecha:** YYYY-MM-DD
**Autores:** Nombre(s)

### Contexto
¿Qué problema o situación motiva esta decisión?

### Opciones consideradas
1. Opción A — ventajas / desventajas
2. Opción B — ventajas / desventajas

### Decisión
Opción elegida y motivo.

### Consecuencias
Qué implica esta decisión a futuro.
```

---

## ADR-001: egui/eframe como framework de UI

**Estado:** Aceptado
**Fecha:** 2026-07

### Contexto
Necesitamos un framework de GUI nativo para una aplicación de notación musical.
Requisitos: cross-platform (Linux, macOS, Windows), rendering eficiente, embebible, sin dependencias de navegador.

### Opciones consideradas
1. **egui/eframe** — immediate mode, Rust nativo, OpenGL/Wgpu backend, cross-platform, hot-reload amigable
2. **Tauri + React** — web-based, más pesado, dependencia de system webview
3. **Iced** — Elm-like, nativo, pero ecosistema más chico y menos maduro para gráficos custom
4. **GTK (gtk-rs)** — nativo Linux-first, curva de aprendizaje alta, cross-platform débil

### Decisión
egui/eframe. Immediate mode simplifica el renderizado de glifos musicales (cada frame re-renderiza desde el estado), el ecosistema es activo, y la compilación a un solo binario sin runtime externo es ideal para distribución.

### Consecuencias
- Estilo visual de egui (look "técnico" por defecto) — se puede customizar con `Visuals`
- Sin soporte nativo de fuentes musicales — se cargan como fuentes custom via `FontDefinitions`
- Renderizado por software en modo debug → compilar en `--release` para desarrollo diario
- No hay components reutilizables como en React — la UI se define imperativamente en `update()`

---

## ADR-002: Leland como fuente de notación musical

**Estado:** Aceptado
**Fecha:** 2026-07

### Contexto
Necesitamos una fuente que implemente el estándar SMuFL para renderizar glifos de notación musical.
Requisitos: licencia abierta, embedible, cobertura completa del estándar SMuFL.

### Opciones consideradas
1. **Leland** (MuseScore) — SIL Open Font License, cobertura SMuFL completa, diseñada para engraving de alta calidad
2. **Bravura** (Steinberg) — SIL Open Font License, también completa, más conocida
3. **Gonville** — licencia abierta pero cobertura parcial

### Decisión
Leland. Es la fuente oficial de MuseScore 4, tiene excelente calidad de engraving, y viene con variante `LelandText` que funciona bien como fuente de UI (reemplazando la necesidad de una segunda fuente para texto).

### Consecuencias
- Ambos archivos `.otf` se embeben en el binario con `include_bytes!`
- LelandText se usa como `FontFamily::Proportional` principal para toda la UI
- Leland se registra como `FontFamily::Name("Leland")` para glifos musicales
- Licencia OFL-1.1 documentada en `lib/MusicFonts/Leland/LICENSE.txt`

---

## ADR-003: Ventana borderless con decoraciones custom

**Estado:** Aceptado
**Fecha:** 2026-07

### Contexto
Para una aplicación de notación musical, la barra de título nativa rompe la estética y desperdicia espacio vertical que podría usarse para el pentagrama.

### Opciones consideradas
1. **Borderless** (`with_decorations(false)`) + panel superior egui como titlebar
2. **Decoraciones nativas** — estándar del SO
3. **Fullscreen** — inmersivo pero pierde acceso a otras ventanas

### Decisión
Borderless con panel superior egui del 5% de altura. El panel integra menú Archivo, toggle de idioma y dark mode en el mismo espacio que ocuparía una titlebar nativa, pero con estilo consistente con el resto de la app. Maximizado automático en primer frame para experiencia inmersiva.

### Consecuencias
- El usuario no puede arrastrar la ventana desde la barra de título (limitación actual de egui con decoraciones nativas deshabilitadas)
- Necesario implementar botones minimizar/maximizar/cerrar manualmente si se requieren
- En Windows, el snapping Aero no funciona sin decoraciones

---

## ADR-004: i18n con lookup table en el binario

**Estado:** Aceptado
**Fecha:** 2026-07

### Contexto
La aplicación debe soportar español e inglés desde el inicio. Se necesita un mecanismo simple sin dependencias externas.

### Opciones consideradas
1. **Lookup table en `match`** — zero-cost, sin allocation, compila a código
2. **Fluent / ICU4X** — estándar profesional, pero agrega dependencias y complejidad para 2 idiomas
3. **Archivos JSON externos** — fácil de editar pero requiere incluirlos en el binario o distribuirlos

### Decisión
Lookup table con `match` anidado en `I18n::t()`. Para 2 idiomas y ~10 claves actuales, es la solución más simple y rápida. No hay allocation por string (retorna `&'static str`).

### Consecuencias
- Agregar un idioma requiere modificar `I18n::t()` y el enum `Lang`
- Las claves nuevas deben agregarse en ambos `match` arms
- Si el número de claves crece más de ~30, considerar migrar a Fluent o un hashmap estático con `phf`


## ADR-005: Renderizado de partitura en páginas A4

**Estado:** Aceptado
**Fecha:** 2026-07

### Contexto
La partitura se renderizaba como una tira vertical continua (`render_score`). Para una experiencia de lectura musical natural, se necesita paginación en hojas de referencia A4, con staves fluyendo entre páginas.

### Opciones consideradas
1. **Páginas A4 virtuales con zoom** — hojas de proporción real A4 (210×297 mm a 96 DPI), scroll vertical, 1-2 páginas por fila según ancho de ventana
2. **Scroll continuo con marcas de página** — mantener el scroll continuo pero dibujar líneas de corte de página
3. **Una página por vez con navegación** — una sola página visible, botones next/prev

### Decisión
Opción 1: páginas A4 virtuales. Es la experiencia estándar de software de notación (MuseScore, Sibelius, Dorico). El zoom escala todas las dimensiones proporcionalmente. Layout responsive: 1 o 2 columnas según el ancho disponible.

### Consecuencias
- `render_score` se preserva como utilidad (posible toggle "vista continua" futuro)
- Las dimensiones A4 son proporciones de referencia, no previsualización de impresión exacta (96 DPI ≠ DPI físico del monitor)
- El algoritmo de paginación distribuye staves por altura; no considera densidad de notas
- `max_measures_per_line` está reservado para control futuro de layout horizontal
---

## Próximos ADRs sugeridos

- ADR-005: Representación interna de notas y pentagramas (structs vs bitfields)
- ADR-006: Estrategia de renderizado del pentagrama (egui painter vs OpenGL custom)
- ADR-007: Formato de archivo para partituras (MusicXML vs formato propio vs MIDI)
- ADR-008: Reproducción de audio (MIDI synth vs samples vs TFM)

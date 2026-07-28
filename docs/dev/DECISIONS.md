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

## ADR-006: Espaciado proporcional a duración + justificación por sistema

**Estado:** Aceptado
**Fecha:** 2026-07-28

### Contexto
Comparando el render contra un PDF de referencia (export de Guitar Pro) del mismo archivo (`test-data/simple.musicxml.xml`), se detectó que el espaciado horizontal de notas era puramente por **cantidad de elementos** (`measure_natural_width` en `render/layout.rs`), ignorando la duración: una redonda y una corchea ocupaban el mismo ancho. Esto le daba al render un aspecto de "grilla de máquina de escribir" en vez de grabado musical. Además, `compute_measure_widths` solo estiraba el **último** compás de cada línea para llenar el espacio sobrante — no era un modelo de justificación real, así que el espaciado entre compases de una misma línea se veía desparejo. El síntoma más visible: los compases 6 y 7 del fixture (corridas largas de semifusas) se renderizaban extremadamente densos e ilegibles comparados con el PDF de referencia.

### Opciones consideradas
1. **Modelo de resortes ópticos completo (Gourlay)** — el estándar de la industria (Finale, Dorico), pero implica un sistema de "springs" con rigidez por elemento y un solver de optimización; complejidad desproporcionada para el tamaño actual del proyecto.
2. **Ancho ideal logarítmico + estiramiento uniforme** — ancho de cada elemento como `base + escala * log2(duración/referencia)`, justificación estirando todos los compases de la línea proporcionalmente. Necesita una constante de "duración de referencia" arbitraria como ancla del logaritmo.
3. **Ancho ideal con raíz cuadrada + estiramiento uniforme (elegida)** — mismo espíritu cóncavo que la opción logarítmica (duplicar la duración no duplica el ancho) pero sin necesitar una duración de referencia arbitraria: `ancho = line_spacing * (MIN + ESCALA * sqrt(duración_en_negras))`. Se ancla en que una negra dé el mismo ancho que el espaciado uniforme anterior (`line_spacing * 1.5`), así una partitura de solo negras se ve casi igual que antes.

### Decisión
Opción 3. Está implementada en `src/render/layout.rs`:
- `element_width_shape`/`width_shape` calculan una forma adimensional de ancho por elemento según su duración (`NoteFigure::quarter_fraction`, nuevo método en `src/notation/figure.rs`, independiente de `divisions` de MusicXML).
- `measure_natural_width` suma esas formas (× `line_spacing`) más un margen fijo y el extra por accidentales.
- `compute_measure_widths` calcula los anchos naturales de todos los compases de una línea; si hay espacio sobrante, estira **todos** los compases por el mismo factor (no solo el último), clampeando a `max_width` y redistribuyendo el sobrante de los compases clampeados entre los que no lo están (dos pasadas). Sin espacio sobrante, mantiene el comportamiento de overflow anterior (ancho natural sin escalar).
- `element_offsets` distribuye el ancho asignado a un compás entre sus elementos proporcionalmente a esa misma forma, centrando cada elemento en su porción — generaliza el modelo uniforme anterior (que era el caso particular de todas las formas iguales).
- `break_measures_into_lines` no cambió: al llamar a `measure_natural_width` internamente, hereda el comportamiento consciente de duración sin necesitar modificaciones.

### Consecuencias
- Es un escalado proporcional uniforme dentro de cada compás, **no** un modelo de resortes ópticos real — no hay rigidez distinta por tipo de elemento ni justificación óptima global entre compases de distinto contenido. Suficiente para el objetivo actual (evitar el aspecto de grilla), pero un candidato futuro si se necesita paridad visual más fina con motores profesionales.
- Las constantes `WIDTH_SHAPE_MIN`/`WIDTH_SHAPE_SCALE` (`render/layout.rs`) fueron calibradas visualmente contra el PDF de referencia con `test-data/simple.musicxml.xml`; pueden necesitar reajuste con partituras de contenido rítmico más variado.
- `MultipleRest` no participa del modelo de duración (ancho fijo, `element_width_shape` le asigna una forma constante) — no está parseado desde MusicXML actualmente, así que no hay caso de prueba real que lo ejercite.
- El campo `divisions` de `Measure` (ver más abajo en este documento el fix del bug de beaming) sigue siendo necesario para el agrupado de barras, pero el espaciado horizontal ya no depende de él — usa `NoteFigure::quarter_fraction`, una proporción fija por tipo de figura.

---

## ADR-007: BASE_SCALE para calibrar el 100% de zoom

**Estado:** Aceptado
**Fecha:** 2026-07-28

### Contexto
Comparando el render contra el mismo PDF de referencia de ADR-006 (export de Guitar Pro de `test-data/simple.musicxml.xml`), la partitura solo se veía visualmente equivalente a ese PDF (a "100%" en el visor de PDF) cuando el zoom de la app llegaba a ~220%. Se investigaron dos hipótesis:
1. Que el 220% aportara más calidad de anti-aliasing/nitidez de trazo — descartada: un recorte del render al 100% escalado 2.2× con nearest-neighbor (sin suavizado) resultó **idéntico píxel a píxel** al render nativo a 220%, confirmando que `zoom` ya escala todo proporcionalmente y no hay diferencia de calidad, solo de tamaño.
2. Que hubiera que auto-detectar el DPI real de pantalla (`ctx.native_pixels_per_point()`) y usarlo para corregir el 100% — descartada tras revisar el código fuente de egui (`context.rs`, egui 0.35.0): egui ya trabaja en puntos lógicos independientes de resolución, y aplica `pixels_per_point` internamente solo al rasterizar (nitidez de fuente/anti-aliasing), nunca para el tamaño de layout. Multiplicar manualmente por ese valor en la app duplica el escalado en vez de corregirlo. Además, `native_pixels_per_point` refleja la preferencia de escala de UI del sistema operativo (en una máquina de prueba real: KDE configuraba `Xft.dpi=120`, dando factor 1.25), no la densidad física real del panel (medida vía EDID con `xrandr`: ~189 PPI en esa misma máquina, que requeriría factor ~1.96 para tamaño físico exacto). No existe una forma portable (Linux/macOS/Windows/WASM) de consultar la densidad física real — el hack de EDID vía `xrandr` es específico de X11.

### Opciones consideradas
1. **Auto-detectar `native_pixels_per_point`** — portable y es lo que hacen navegadores/visores de PDF, pero solo refleja la preferencia de escala del SO, no el tamaño físico real; en la máquina de prueba solo hubiera cerrado ~25% de la brecha necesaria (1.25× de 2.2× objetivo), y depende de que cada usuario tenga su DE bien calibrado.
2. **Multiplicar las constantes de tamaño en `constants.rs`** — simple pero incompleto: el texto de encabezado (título/compositor/tempo) viene del stylesheet y se escala solo por `zoom`, no por las constantes de notación, así que quedaba desproporcionadamente chico frente a la partitura ya agrandada.
3. **`BASE_SCALE` aplicado una sola vez sobre `zoom` en el punto de renderizado (`app.rs`)** — cubre por igual la geometría de notación y el encabezado porque ambos derivan del mismo `zoom` en ese punto. Valor fijo (2.2), determinístico en cualquier máquina/plataforma.

### Decisión
Opción 3. `BASE_SCALE: f32 = 2.2` (`render/constants.rs`) se multiplica por el `zoom` del documento en el único lugar donde se lee para renderizar (`app.rs`, dentro del panel central), produciendo `render_zoom`. El zoom "lógico" que ve y controla el usuario (dropdown, atajos de teclado, persistencia de sesión) no cambia de significado — sigue siendo 100% = "tamaño por defecto". El zoom por defecto de un documento nuevo bajó de 1.30 a 1.0, ya que con `BASE_SCALE` un zoom lógico de 1.0 ya reproduce el resultado visual que antes requería 2.2 (220%).

### Consecuencias
- El "100%" de esta app es una calibración visual contra un PDF de referencia, no una promesa de tamaño físico exacto de hoja A4 — ninguna app cross-platform puede prometer eso sin un paso de calibración manual del usuario (regla en pantalla), que está fuera de alcance.
- Si en el futuro se agrega un paso de calibración manual (ej. "ajustá esta regla a 10 cm reales"), debería reemplazar o combinarse con `BASE_SCALE`, no apilarse ciegamente.
- Cambiar `BASE_SCALE` es un solo número en `constants.rs`; recalibrar contra un nuevo PDF de referencia solo requiere ajustar ese valor.
- La sesión persistida (`session.json`) solo guarda rutas de archivo, no el zoom por documento (`restore_session` siempre crea `Document`s con el zoom por defecto) — no hay estado viejo de zoom que migrar.

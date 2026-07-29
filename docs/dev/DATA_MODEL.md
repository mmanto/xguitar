# DATA_MODEL.md — Modelo de Dominio

Descripción de las entidades del dominio de notación musical.
Define los conceptos que la aplicación modela, renderiza y eventualmente editará.

---

## Dominio actual (Phase 1 — implementado)

### `Clef` (Clave)

Define la posición tonal en el pentagrama.

| Variante | Glifo Leland | Unicode | Línea |
|---|---|---|---|
| Treble (Sol) | `\u{E050}` | U+E050 | 2 |
| Bass (Fa) | `\u{E062}` | U+E062 | 4 |
| Alto (Do en 3ra) | `\u{E05C}` | U+E05C | 3 |
| Tenor (Do en 4ta) | `\u{E05C}` | U+E05C | 4 |
| Percussion | `\u{E069}` | U+E069 | 3 |
| Tab | `\u{E06D}` | U+E06D | 5 |

### `Accidental` (Alteración)

| Variante | Glifo Leland | Unicode | Semitonos |
|---|---|---|---|
| Natural | `\u{E261}` | U+E261 | 0 |
| Sharp | `\u{E262}` | U+E262 | +1 |
| Flat | `\u{E260}` | U+E260 | -1 |
| DoubleSharp | `\u{E264}` | U+E264 | +2 |
| DoubleFlat | `\u{E266}` | U+E266 | -2 |

### `NoteFigure` (Figura rítmica)

| Figura | Glifo Leland | Unicode | Banderas |
|---|---|---|---|
| Breve | `\u{E1D1}` | U+E1D1 | 0 |
| Whole | `\u{E1D2}` | U+E1D2 | 0 |
| Half | `\u{E1D3}` | U+E1D3 | 0 |
| Quarter | `\u{E1D5}` | U+E1D5 | 0 |
| Eighth | `\u{E1D7}` | U+E1D7 | 1 |
| Sixteenth | `\u{E1D9}` | U+E1D9 | 2 |
| ThirtySecond | `\u{E1DB}` | U+E1DB | 3 |
| SixtyFourth | `\u{E1DD}` | U+E1DD | 4 |
| HundredTwentyEighth | `\u{E1DD}` | U+E1DD | 5 |

### `TimeModification` (tresillos/quintillos — duración)

| Campo | Tipo | Descripción |
|---|---|---|
| `actual_notes` | `u8` | Notas realmente presentes en el grupo (ej. 3 en un tresillo) |
| `normal_notes` | `u8` | Notas del valor normal en ese mismo tiempo (ej. 2) |

`ratio()` devuelve `normal_notes / actual_notes` para escalar la duración en
divisiones. Se parsea desde `<time-modification>`, independiente del `Tuplet`
visual de `NoteAttachment` (corchete/número), que hoy no se parsea desde
MusicXML — sólo existe como dato para quien lo construya manualmente.

### `Note` (Nota musical)

| Campo | Tipo | Descripción |
|---|---|---|
| `pitch` | `Pitch` | Altura: step, accidental, octave |
| `figure` | `NoteFigure` | Duración rítmica |
| `dotted` | `u8` | Cantidad de puntillos (0–3) |
| `time_modification` | `Option<TimeModification>` | Ratio actual/normal de `<time-modification>` (tresillos, etc.) — afecta duración sonante, no el dibujo del corchete (eso lo controla `Tuplet` en `NoteAttachment`) |
| `accidental_override` | `Option<Accidental>` | Alteración explícita (cortesía) |
| `stem_direction` | `StemDirection` | Dirección de plica (Up/Down) |
| `grace` | `bool` | Nota de gracia |
| `chord` | `bool` | Miembro de acorde |

### `Rest` (Silencio)

| Campo | Tipo | Descripción |
|---|---|---|
| `figure` | `NoteFigure` | Duración |
| `dotted` | `u8` | Puntillos (0–3) |
| `time_modification` | `Option<TimeModification>` | Igual que en `Note` — tresillos de silencios |
| `display_step` | `Option<Step>` | Posición visual |
| `display_octave` | `Option<i8>` | Octava visual |
| `measure` | `bool` | Silencio de compás completo |

### `MeasureElement` (Elemento de compás)

Enumeración: `Note(Note)`, `Rest(Rest)`, `Chord(Vec<Note>)`, `Backup(u32)`, `Forward(u32)`.

### `Measure` (Compás)

| Campo | Tipo | Descripción |
|---|---|---|
| `number` | `String` | Número (puede ser "1a", "X") |
| `time_signature` | `TimeSignature` | Fórmula de compás |
| `key_signature` | `KeySignature` | Armadura de clave |
| `elements` | `Vec<MeasureElement>` | Contenido del compás |
| `barline` | `Barline` | Barra divisoria |
| `ending` | `Option<Ending>` | Casilla de repetición |
| `divisions` | `u32` | Divisiones por negra (MusicXML `<divisions>`), heredado del último valor visto en compases anteriores |
| `system_break` | `bool` | Salto de sistema explícito del origen (MusicXML `<print new-system="yes"/>`) antes de este compás. `break_measures_into_lines` (render/layout.rs) lo respeta como corte forzado de renglón, además del reflow automático por ancho disponible |

### `KeySignature` (Armadura)

| Campo | Tipo | Descripción |
|---|---|---|
| `fifths` | `i8` | Sostenidos (+) o bemoles (-), -7 a +7 |
| `mode` | `KeyMode` | Major, Minor, Dorian, etc. |
| `cancel` | `Option<i8>` | Armadura previa a cancelar |

### `Barline` (Barra de compás)

| Campo | Tipo | Descripción |
|---|---|---|
| `style` | `BarStyle` | Regular, Dotted, Dashed, Heavy, LightLight, LightHeavy, HeavyLight, HeavyHeavy, Tick, Short, None |
| `repeat` | `Option<RepeatDirection>` | Forward/Backward |
| `ending` | `Option<Ending>` | Casilla |

### `Staff` (Pentagrama)

| Campo | Tipo | Descripción |
|---|---|---|
| `clef` | `Clef` | Clave |
| `line` | `i8` | Línea de la clave (1–5) |
| `measures` | `Vec<Measure>` | Compases |
| `name` | `String` | Nombre de parte |
| `abbreviation` | `String` | Abreviatura |

### `System` (Sistema)

| Campo | Tipo | Descripción |
|---|---|---|
| `staves` | `Vec<Staff>` | Pentagramas simultáneos |
| `left_margin` | `f32` | Indentación |
| `bracket` | `Option<GroupBracket>` | Corchete/llave |

### `Score` (Partitura)

| Campo | Tipo | Descripción |
|---|---|---|
| `title` | `String` | Título |
| `composer` | `String` | Compositor |
| `systems` | `Vec<System>` | Sistemas |
| `credits` | `Vec<Credit>` | Créditos de página |
| `scaling` | `Option<Scaling>` | Escala MusicXML |
| `part_list` | `PartList` | Partes e instrumentos |

### `Credit` (Crédito de página)

Créditos adicionales de MusicXML (`<credit><credit-words>`) — título, compositor y otros
(p. ej. "Music by X" en la esquina superior derecha). Parseados en `parse_credits`
(`src/musicxml/parser.rs`), usando `<defaults><page-layout>` para convertir las
coordenadas absolutas en tenths de MusicXML a fracciones de página. Si el
documento no declara `page-layout`, los créditos se omiten (no hay espacio de
coordenadas confiable para posicionarlos).

| Campo | Tipo | Descripción |
|---|---|---|
| `page` | `u8` | Número de página (atributo `page` de `<credit>`) |
| `kind` | `CreditKind` | `Words(String)` o `Symbol(char)` |
| `default_x` | `f32` | Posición horizontal como fracción de página (0.0–1.0) |
| `default_y` | `f32` | Posición vertical como fracción de página (0.0–1.0, crece hacia arriba como en MusicXML). Parseado pero **no usado** por el render — ver nota abajo. |
| `justify` | `CreditJustify` | `Left` (default) \| `Center` \| `Right` — atributo `justify` de `<credit-words>` |

El render (`render_page` en `src/render/page.rs`) dibuja los créditos que no
coinciden en texto con `score.title`/`score.composer` centrados (ya mostrados
con su propio layout), usando `default_x` para la posición horizontal y
`justify` para la alineación. `default_y` se ignora deliberadamente: expresa
una fracción de la altura de página *real* del documento fuente, que no
coincide con la altura fija del bloque de header de este renderer (usar la
fracción cruda hacía que créditos "extra" cayeran encima del pentagrama) — en
cambio, se posicionan en la misma fila que el compositor.

---

## Mapeo de glifos Leland

Los glifos se referencian por su código Unicode del estándar SMuFL (Standard Music Font Layout).
La fuente Leland implementa el rango `U+E000`–`U+EFFF`.

| Categoría | Rango SMuFL |
|---|---|
| Clefs | U+E050–U+E07F |
| Time signatures | U+E080–U+E09F |
| Noteheads | U+E0A0–U+E0FF |
| Flags | U+E240–U+E25F |
| Accidentals | U+E260–U+E27F |
| Rests | U+E4E0–U+E4FF |
| Articulations | U+E4A0–U+E4BF |

Referencia completa: [SMuFL Specification](https://w3c.github.io/smufl/latest/)

---

## Dominio Phase 2 — Articulaciones, Ornamentos, Conectores

### `NoteAttachment` (Marcas de nota)

| Campo | Tipo | Descripción |
|---|---|---|
| `articulations` | `Vec<Articulation>` | Articulaciones (acento, staccato, etc.) |
| `ornaments` | `Vec<Ornament>` | Ornamentos (trino, mordente, etc.) |
| `technical` | `Vec<Technical>` | Técnicas instrumentales |
| `dynamics` | `Option<DynamicMark>` | Marca dinámica (p, f, mf, etc.) |
| `fermata` | `Option<Fermata>` | Calderón |
| `ties` | `Vec<Tie>` | Ligaduras de prolongación |
| `slurs` | `Vec<Slur>` | Ligaduras de expresión |
| `glissando` | `Option<Glissando>` | Glissando/portamento |
| `arpeggiate` | `Option<Arpeggiate>` | Arpegio |
| `tremolo` | `Option<Tremolo>` | Trémolo |

### `Articulation` — 14 variantes

`Accent` (>), `StrongAccent` (^), `Staccato` (.), `Staccatissimo`, `Tenuto`, `DetachedLegato`, `Spiccato`, `BreathMark`, `Caesura`, `SoftAccent`, `Scoop`, `Plop`, `Doit`, `Falloff`.

### `Ornament` — 7 variantes

`Trill`, `Turn`, `InvertedTurn`, `Mordent`, `InvertedMordent`, `Tremolo { marks }`, `WavyLine { start, stop }`.

### `DynamicMark` — 21 variantes

`PPPP`–`FFFF`, `SF`, `SFZ`, `SFFZ`, `SFP`, `SFPP`, `FP`, `RF`, `RFZ`, `FZ`, `N`, `PF`, `Other(String)`.

### `Fermata` / `FermataShape` — 8 formas

`Normal`, `Angled`, `Square`, `DoubleAngled`, `DoubleSquare`, `Long`, `Short`, `Henze`.

### `Tie` / `Slur`

Ligaduras con `kind` (Start/Stop/Continue), `number`, y `placement` (Above/Below).

### Renderizado Phase 2

| Elemento | Método | SMuFL / Técnica |
|---|---|---|
| Articulaciones | Glifos apilados sobre la nota | U+E4A0–U+E4B8 |
| Ornamentos | Glifos sobre la nota | U+E566–U+E56C |
| Calderón | Glifo SMuFL | U+E4C0–U+E4C5 |
| Dinámicas | Texto proporcional | LelandText |
| Slurs/Ties | Curvas Bezier cúbicas | `Painter::line_segment` |
| Glissando | Línea + "gliss." | `Painter::line_segment` |
| Arpegio | Línea ondulada vertical | `Painter::line_segment` |
| Trémolo | Barras diagonales | `Painter::line_segment` |

---

## `src/audio/` — Reproducción (ver ADR-008)

No forma parte del modelo de dominio de notación (`notation::`) — vive en su
propio módulo, también libre de tipos de egui, que consume `Score` para
producir eventos de audio.

### `sequencer::SequencedEvent`

| Campo | Tipo | Descripción |
|---|---|---|
| `time_secs` | `f32` | Tiempo absoluto desde el inicio de la partitura |
| `kind` | `EventKind` | `NoteOn { midi, velocity }` / `NoteOff { midi }` |

`sequencer::build_events(&Score) -> Vec<SequencedEvent>` es la única función
pública del módulo — pura, testeable sin audio real (ver tests en el mismo
archivo).

### `PlaybackEngine` (trait, `audio::mod`)

Interfaz que desacopla el secuenciador del motor de síntesis concreto:
`load_instrument`, `configure(sample_rate, max_block_frames)` (default
no-op), `note_on`, `note_off`, `render(buffer, channels)` (interleaved).
Implementada hoy por `sfizz::SfizzEngine` (nativo únicamente).

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

### `Note` (Nota musical)

| Campo | Tipo | Descripción |
|---|---|---|
| `pitch` | `Pitch` | Altura: step, accidental, octave |
| `figure` | `NoteFigure` | Duración rítmica |
| `dotted` | `u8` | Cantidad de puntillos (0–3) |
| `accidental_override` | `Option<Accidental>` | Alteración explícita (cortesía) |
| `stem_direction` | `StemDirection` | Dirección de plica (Up/Down) |
| `grace` | `bool` | Nota de gracia |
| `chord` | `bool` | Miembro de acorde |

### `Rest` (Silencio)

| Campo | Tipo | Descripción |
|---|---|---|
| `figure` | `NoteFigure` | Duración |
| `dotted` | `u8` | Puntillos (0–3) |
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

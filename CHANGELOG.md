# CHANGELOG

Todos los cambios notables de este proyecto se documentan aquí.
Formato basado en [Keep a Changelog](https://keepachangelog.com/es/1.0.0/).
Versionado semántico: `MAJOR.MINOR.PATCH`.

---

## [Sin publicar]

### Added
- Módulo Fingerboard (`src/fingerboard/`): diapasón interactivo de guitarra (6 cuerdas) y bajo (4 cuerdas) con afinación estándar. Ventana flotante invocable desde un icono 🎸 en la barra de herramientas. Soporta selección de posiciones, toggle de intervalos, slider de trastes (5–24), control de tamaño (escala 0.5x–3.0x que amplía el dibujo para facilitar la lectura), y cambio entre guitarra/bajo.
- Soporte nativo de PipeWire para salida de audio: `select_host()` intenta PipeWire → JACK → default (ALSA), habilitado con `cpal = { features = ["pipewire"] }`. La app aparece como cliente PipeWire enrutable en qpwgraph/helvum/Carla.
- Reproducción de partituras: botón Play/Stop en la barra de herramientas, secuenciador que recorre la partitura respetando tempo, ligaduras, tresillos, articulaciones (staccato, tenuto) y dinámicas, motor de síntesis sfizz (formato de instrumento SFZ) sobre `cpal` — nativo únicamente, ver ADR-008
- Parseo de `<time-modification>` (ratio de tresillos/quintillos) en notas y silencios
- Ventana borderless con barra superior personalizada (5% altura)
- Fuente Leland embebida para notación musical y UI
- Renderizado de claves: Sol (G clef) y Fa (F clef)
- Renderizado de figuras: redonda, blanca, negra, corchea, semicorchea
- Internacionalización (i18n) español/inglés
- Toggle de modo oscuro/claro
- Menú Archivo con opciones (Nuevo, Abrir, Cerrar, Salir — placeholders)
- Renderizado de partitura paginada en hojas A4 con scroll vertical
- Zoom de partitura con rango 0.25x–4.0x (Ctrl+/Ctrl-)
- Maximizado automático en primer frame
- Sistema de hojas de estilo (stylesheets) externas en TOML
- Selector de estilo visual en la barra superior
- Entrada de notas por teclado: letra (C–B) para grado, dígitos (1,2,4,8,6,32,33) para figura
- Barra de estado inferior con feedback de entrada y mensajes
- Acción "Nuevo" funcional: limpia la partitura a un pentagrama vacío en clave de Sol
- Soporte para estilos custom en ~/.config/m-guitar/stylesheets/
- Importación de archivos MusicXML (.xml, .musicxml) vía botón "Abrir"
- Soporte para notas, alteraciones, puntillos, claves (Sol/Fa), y compases desde MusicXML
- Cumplimiento estricto con XSD de MusicXML 4.0: clef-octave-change, senza-misura, compases compuestos, clefs adicionales, score-timewise, atributos por compás, valores por defecto de línea de clave, fallback de compositor
- Tests de importación MusicXML: 12 casos cubriendo parsing básico, claves alternativas, alteraciones, notas de gracia, XSD edge cases
- **Phase 1 — Domain Model & Rendering Strategy:** Modelo de dominio completo (~12 tipos nuevos/extendidos) y pipeline de renderizado SMuFL
  - Clef: Alto, Tenor, Percussion, Tab
  - Accidental: DoubleSharp, DoubleFlat con glifos SMuFL
  - NoteFigure: Breve, HundredTwentyEighth
  - Note: dotted u8, accidental_override, stem_direction, grace, chord
  - Rest, KeySignature, KeyMode, Barline, BarStyle, RepeatDirection, Ending
  - System, GroupBracket, Credit, Scaling, PartList, PartInfo, PartGroup
  - MeasureElement: Note, Rest, Chord, Backup, Forward
  - StemDirection, TimeSignatureStyle (Numeric/Common/Cut)
  - Renderizado de plicas, banderas, beams, alteraciones, puntillos, silencios
  - Armaduras de clave con cancelación de becuadros
  - Barras de compás con 11 estilos (incluyendo repeticiones)
  - Indicación de compás con glifos SMuFL (numérico, común, cortado)
  - Acordes: notas múltiples con plica compartida
  - MusicXML parser: rest, chord, accidental, key, barline, ending, part-name, stem, dots
  - **Phase 2 — Articulations, Ornaments, Connecting Marks:** Marcas de nota, ornamentos y conectores (~25 tipos)
  - NoteAttachment: struct con articulations, ornaments, technical, dynamics, fermata, ties, slurs, glissando, arpeggiate, tremolo
  - Articulation: 14 variantes (Accent, StrongAccent, Staccato, Staccatissimo, Tenuto, DetachedLegato, Spiccato, BreathMark, Caesura, SoftAccent, Scoop, Plop, Doit, Falloff)
  - Ornament: 7 variantes (Trill, Turn, InvertedTurn, Mordent, InvertedMordent, Tremolo, WavyLine)
  - Technical: 20+ variantes (UpBow, DownBow, Harmonic, OpenString, Fingering, Bend, HammerOn, PullOff, Tap, etc.)
  - DynamicMark: 21 marcas dinámicas (pppp–ffff, sf, sfz, fp, etc.)
  - Fermata: 8 formas (Normal, Angled, Square, DoubleAngled, etc.)
  - Tie/Slur con start/stop/continue + Placement
  - Glissando, Arpeggiate, Tremolo
  - Renderizado SMuFL de articulaciones y ornamentos apilados
  - Curvas Bezier cúbicas para slurs y ties
  - Líneas de glissando con texto "gliss."
  - Línea ondulada vertical para arpegios
  - Barras diagonales para trémolos
  - MusicXML parser: articulations, ornaments, technical, dynamics, fermata, tied, slur, glissando, arpeggiate, tremolo
  - **Phase 3 — Dynamics, Text, Directions:** Direcciones, dinámicas, texto y letras (~20 tipos)
  - Direction/DirectionKind: 9 variantes (Dynamics, Wedge, Words, Rehearsal, Metronome, OctaveShift, Pedal, Dashes, Bracket)
  - Wedge/WedgeKind: crescendo/diminuendo con niente
  - Metronome: beat_unit + per_minute + parentheses
  - OctaveShift: 8va/8vb/15ma/15mb con línea punteada y descendente
  - Pedal/PedalKind: Start/Stop/Change/Continue/Resume con línea
  - Lyric/Syllabic: Single/Begin/End/Middle con guiones y melisma extenders
  - Renderizado de dinámicas como texto, wedges como líneas angulares
  - Renderizado de marcas de ensayo en recuadro
  - Renderizado de metrónomo, octava shift, pedal
  - Renderizado de letras debajo del staff con guiones silábicos
  - **Phase 4 — Tablature:** Dominio y renderizado de tablatura para guitarra (~12 tipos)
  - TablatureStaff: N cuerdas, afinación, capo
  - TabMeasure/TabElement: TabNote, TabRest, TabChord
  - TabNote: string, fret, figure, dotted, technique
  - TabTechnique: 14 técnicas (HammerOn, PullOff, Bend, Slide, Vibrato, WideVibrato, Tap, Harmonic, PalmMute, LetRing, GhostNote, DeadNote, Trill)
  - SlideKind: Into/OutOf/Shift, HarmonicKind: Natural/Artificial/Pinch/Tap/Semi
  - StaffKind enum: Standard, Tablature(TablatureStaff), GrandStaff
  - Renderizado de líneas de tablatura (N cuerdas)
  - Números de traste con glifos SMuFL circled-fret (U+EBD0–U+EBDE) para 0–14
  - Notas fantasma entre paréntesis, notas muertas con X
  - Hammer-on "H", Pull-off "P", Tap "T"
  - Bend con flecha curva y texto de cantidad ("full", "1/2", "1/4")
  - Slide con línea diagonal (Into/OutOf/Shift)
  - Vibrato/WideVibrato con línea ondulada
  - Armónicos con texto (N.H./A.H./P.H./T.H.) y brackets SMuFL <> (U+EAB0/U+EAB1)
  - Palm mute "P.M.", Let Ring con línea punteada, Trino con texto "tr{fret}"
  - **Phase 5 — Advanced Notation, Layout, Polish:** Notación avanzada y layout (~8 tipos)
  - GraceNote: acciaccatura/appoggiatura con steal_previous/following/make_time
  - Tuplet/TupletShow: tresillos, quintillos con bracket y número
  - Tuplet añadido a NoteAttachment
  - MultipleRest: silencios de múltiples compases (Kirchenpause)
  - MeasureElement::MultipleRest variant
  - MeasureStyle/SlashNote: RepeatPercent (%), BeatRepeat, Slash notation
  - SystemLayout/SystemDividers: system_distance, top_system_distance, divisores //
  - **MusicXML Render Strategy:** Integración completa del pipeline MusicXML → render
  - Parser: direcciones (dynamics, wedge, words, rehearsal, metronome, octave-shift, pedal)
  - Parser: letras/canción (lyric, syllabic, extend)
  - Render: direcciones y marcas dinámicas debajo del staff
  - Render: letras con hyphenación silábica y melisma extenders
  - Render: ligaduras de expresión (slur) y prolongación (tie) entre notas
  - Render: líneas de glissando entre notas
  - Render: corchetes de tresillo/quintillo con número
  - Render: MultipleRest con símbolo de Kirchenpause y número
  - Render: anchos de compás proporcionales al contenido rítmico
  - Test data: `test-data/simple.musicxml` con direcciones, acordes, ligaduras y letra
  - Test: parser assertion `direction_and_lyric_parsed`
- Quiebre automático de compases en múltiples líneas cuando exceden el ancho disponible de página
- Estiramiento proporcional de compases por línea para llenar el ancho completo del pentagrama
- Repetición de la clave al inicio de cada renglón cuando un pentagrama se parte en varias líneas (convención de grabado estándar)
- Resaltado de notas activas durante la reproducción: las notas que están sonando se muestran en color ámbar cálido (#D46A04 / #FFB347), configurable via stylesheet (`note_highlight_light` / `note_highlight_dark` en `[notation]`)
- Vista de mapa de temas: grilla de bloques por compás con aspecto de hoja de partitura (colores del stylesheet activo), agrupados por sección musical (Intro, Verso, Estribillo, Puente, Solo, Outro) con acorde visible por bloque, más un minimapa horizontal de secciones (ancho proporcional a compases) para navegación rápida — ver ADR-009 en `docs/dev/DECISIONS.md`
- Modelo de dominio para temas musicales: `Theme`, `Section`, `ChordProgression`, `ChordSymbol`, `SectionKind` en `src/notation/theme.rs`
- Toggle 🗺️/🎼 en la toolbar para alternar entre vista de partitura y mapa de temas
- Scroll programático: al clickear una sección en el mapa, la vista de partitura scrollea al compás correspondiente
- Método `PageLayout::measure_position()` para mapear índice de compás a posición de página

### Fixed
- Los saltos de sistema explícitos del origen (`<print new-system="yes"/>` de MusicXML) ahora se respetan: antes se ignoraban y el layout paginado solo partía renglones por ancho disponible, produciendo una distribución de compases por línea distinta a la del archivo original (y de programas como Guitar Pro/MuseScore)
- El tempo (metrónomo) ahora se alinea al margen izquierdo del pentagrama en vez de centrarse en el ancho completo de la página; antes quedaba "flotando" sin relación visual con el primer compás, sobre todo en sistemas con varios compases por renglón
- Parser MusicXML: `chord_buffer` no se limpiaba entre acordes consecutivos cuando `held_note` era `None`, causando que todas las notas de un compás con múltiples acordes se colapsaran en un solo `Chord` de 24+ notas (en vez de N acordes separados)
- Beam rendering: `compute_beams()` rompía los grupos de beam al encontrar un `MeasureElement::Chord`; ahora los acordes con figuras de corchea o más cortas participan en los grupos de beam igual que las notas sueltas. `beam_meta` y el renderizado de acordes también usan `is_beamed`/`stem_beam_y` del grupo
- Beam rendering: las plicas de acordes ahora se extienden hasta la altura del beam cuando forman parte de un grupo (antes siempre eran plicas sueltas con `is_beamed = false`)

- Beam rendering: las plicas de acordes no beamados (blancas, negras sin barra) ahora usan la misma fórmula de altura que los acordes beamados (`top_y - 3.5 * line_spacing`), alcanzando la misma posición vertical que alcanzaría el beam si existiera
### Changed
- Calibrado el 100% de zoom para que coincida visualmente con partituras bien grabadas (comparado contra un PDF de referencia): antes había que subir a ~220% para lograr ese resultado. Zoom por defecto de un documento nuevo: 1.0 (antes 1.30). Ver ADR-007 en `docs/dev/DECISIONS.md`.

## Categorías de cambios

- **Added** — nuevas funcionalidades
- **Changed** — cambios en funcionalidades existentes
- **Deprecated** — funcionalidades que serán eliminadas próximamente
- **Removed** — funcionalidades eliminadas
- **Fixed** — correcciones de bugs
- **Security** — correcciones de vulnerabilidades

---

<!-- Ejemplo de entrada:

## [0.2.0] — 2025-08-15

### Added
- Pentagrama interactivo con posicionamiento de notas
- Reproducción de audio MIDI

### Fixed
- Glifos de Leland que no renderizaban en Windows (#12)

### Changed
- Extracción de `I18n` a `src/i18n.rs`

-->

# CHANGELOG

Todos los cambios notables de este proyecto se documentan aquí.
Formato basado en [Keep a Changelog](https://keepachangelog.com/es/1.0.0/).
Versionado semántico: `MAJOR.MINOR.PATCH`.

---

## [Sin publicar]

### Added
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

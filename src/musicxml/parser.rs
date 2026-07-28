use roxmltree::{Document, Node};

use crate::notation::{
    Accidental, BarStyle, Barline, Clef, Ending, KeyMode, KeySignature, Measure, MeasureElement,
    Note, NoteFigure, PartGroup, PartInfo, PartList, Pitch, RepeatDirection, Rest, Score, Staff,
    StemDirection, Step, System, TimeSignature, TimeSignatureStyle,
};

use super::MusicXmlError;
/// Parse a MusicXML string into a Score.
pub fn parse_musicxml(xml: &str) -> Result<Score, MusicXmlError> {
    let doc = Document::parse_with_options(
        xml,
        roxmltree::ParsingOptions {
            allow_dtd: true,
            ..Default::default()
        },
    )?;
    let root = doc.root_element();

    match root.tag_name().name() {
        "score-partwise" => parse_partwise(root),
        "score-timewise" => Err(MusicXmlError::Unsupported(
            "score-timewise format is not supported. Use score-partwise instead.".into(),
        )),
        _ => Err(MusicXmlError::Unsupported(format!(
            "Root element must be <score-partwise> or <score-timewise>, got <{}>",
            root.tag_name().name()
        ))),
    }
}
fn parse_partwise(root: Node) -> Result<Score, MusicXmlError> {
    let title = parse_title(&root);
    let composer = parse_composer(&root);
    let part_list = parse_part_list(&root);
    let staves = parse_parts(&root, &part_list)?;
    let page_layout = parse_page_layout(&root);
    let credits = parse_credits(&root, page_layout);

    let system = System {
        staves,
        left_margin: 0.0,
        bracket: None,
    };

    Ok(Score {
        title,
        composer,
        systems: vec![system],
        credits,
        scaling: None,
        part_list,
    })
}

/// Lee `<defaults><page-layout><page-width>/<page-height>` (en tenths de
/// MusicXML). Necesario para convertir las coordenadas absolutas de
/// `<credit>` (también en tenths) a fracciones de página.
fn parse_page_layout(root: &Node) -> Option<(f32, f32)> {
    let defaults = root.children().find(|n| n.has_tag_name("defaults"))?;
    let page_layout = defaults
        .children()
        .find(|n| n.has_tag_name("page-layout"))?;
    let width: f32 = first_child_text(&page_layout, "page-width")?.parse().ok()?;
    let height: f32 = first_child_text(&page_layout, "page-height")?
        .parse()
        .ok()?;
    Some((width, height))
}

/// Parsea los `<credit>` de nivel de partitura (título, compositor, y
/// créditos adicionales como "Music by X"). Sin `page_wh_tenths` no hay
/// espacio de coordenadas confiable para posicionarlos, así que se omiten
/// en vez de adivinar.
fn parse_credits(root: &Node, page_wh_tenths: Option<(f32, f32)>) -> Vec<crate::notation::Credit> {
    let Some((page_w, page_h)) = page_wh_tenths else {
        return Vec::new();
    };
    if page_w <= 0.0 || page_h <= 0.0 {
        return Vec::new();
    }

    let mut credits = Vec::new();
    for credit_node in root.children().filter(|n| n.has_tag_name("credit")) {
        let Some(words_node) = credit_node
            .children()
            .find(|n| n.has_tag_name("credit-words"))
        else {
            continue;
        };
        let Some(text) = words_node.text().map(|s| s.trim().to_string()) else {
            continue;
        };
        if text.is_empty() {
            continue;
        }
        let default_x: f32 = words_node
            .attribute("default-x")
            .and_then(|s| s.parse().ok())
            .unwrap_or(0.0);
        let default_y: f32 = words_node
            .attribute("default-y")
            .and_then(|s| s.parse().ok())
            .unwrap_or(0.0);
        let justify = match words_node.attribute("justify") {
            Some("center") => crate::notation::CreditJustify::Center,
            Some("right") => crate::notation::CreditJustify::Right,
            _ => crate::notation::CreditJustify::Left,
        };
        let page: u8 = credit_node
            .attribute("page")
            .and_then(|s| s.parse().ok())
            .unwrap_or(1);

        credits.push(crate::notation::Credit {
            page,
            kind: crate::notation::CreditKind::Words(text),
            default_x: (default_x / page_w).clamp(0.0, 1.0),
            default_y: (default_y / page_h).clamp(0.0, 1.0),
            justify,
        });
    }
    credits
}

fn parse_parts(root: &Node, part_list: &PartList) -> Result<Vec<Staff>, MusicXmlError> {
    let mut staves = Vec::new();
    for part in root.children().filter(|n| n.has_tag_name("part")) {
        let part_id = part.attribute("id").unwrap_or("");
        let part_info = part_list.parts.iter().find(|p| p.id == part_id);
        staves.push(parse_part(&part, part_info)?);
    }
    Ok(staves)
}

fn parse_part(part: &Node, part_info: Option<&PartInfo>) -> Result<Staff, MusicXmlError> {
    let mut measures = Vec::new();
    let mut current_clef = Clef::Treble;
    let mut current_key = KeySignature::default();
    let mut current_time = TimeSignature {
        numerator: 4,
        denominator: 4,
        style: TimeSignatureStyle::Numeric,
    };
    let mut current_octave_shift: i8 = 0;
    let mut current_divisions: u32 = 1;

    let name = part_info.map(|p| p.name.clone()).unwrap_or_default();
    let abbreviation = part_info
        .map(|p| p.abbreviation.clone())
        .unwrap_or_else(|| name.clone());

    for measure_node in part.children().filter(|n| n.has_tag_name("measure")) {
        let measure_number = measure_node.attribute("number").unwrap_or("1").to_string();

        // Process ALL <attributes> elements within the measure
        for attrs in measure_node
            .children()
            .filter(|n| n.has_tag_name("attributes"))
        {
            if let Some(clef) = parse_clef(&attrs) {
                current_clef = clef;
            }
            if let Some(time) = parse_time(&attrs) {
                current_time = time;
            }
            if let Some(key) = parse_key(&attrs) {
                current_key = key;
            }
            if let Some(shift) = parse_clef_octave_shift(&attrs) {
                current_octave_shift = shift;
            }
            if let Some(divisions) = parse_divisions(&attrs) {
                current_divisions = divisions;
            }
        }

        let elements = parse_measure_elements(&measure_node, current_octave_shift)?;
        let barline = parse_barline(&measure_node);

        measures.push(Measure {
            number: measure_number,
            time_signature: current_time,
            key_signature: current_key.clone(),
            elements,
            barline,
            ending: None,
            directions: parse_directions(&measure_node),
            divisions: current_divisions,
        });
    }

    // Last measure: use final double barline if not explicitly set
    if let Some(last) = measures.last_mut() {
        if last.barline.style == BarStyle::Regular || last.barline.style == BarStyle::None {
            last.barline.style = BarStyle::LightHeavy;
        }
    }

    Ok(Staff {
        clef: current_clef,
        line: current_clef.default_line(),
        measures,
        name,
        abbreviation,
        kind: crate::notation::StaffKind::Standard,
    })
}
fn first_child_text(node: &Node, tag: &str) -> Option<String> {
    node.children()
        .find(|n| n.has_tag_name(tag))
        .and_then(|n| n.text())
        .map(|s| s.trim().to_string())
}

fn parse_title(root: &Node) -> String {
    // Prefer <movement-title>
    if let Some(t) = first_child_text(root, "movement-title").filter(|s| !s.is_empty()) {
        return t;
    }
    // Fallback: <work><work-title>
    root.children()
        .find(|n| n.has_tag_name("work"))
        .and_then(|w| first_child_text(&w, "work-title"))
        .unwrap_or_default()
}

fn parse_composer(root: &Node) -> String {
    root.children()
        .find(|n| n.has_tag_name("identification"))
        .map(|id| {
            // Prefer creator with type="composer"
            let composers: Vec<_> = id
                .children()
                .filter(|n| n.has_tag_name("creator"))
                .collect();

            // First try exact type="composer" match
            composers
                .iter()
                .find(|n| n.attribute("type") == Some("composer"))
                .or_else(|| {
                    // Fallback: first creator regardless of type
                    composers.first()
                })
                .and_then(|n| n.text().map(|s| s.trim().to_string()))
                .unwrap_or_default()
        })
        .unwrap_or_default()
}

fn parse_part_list(root: &Node) -> PartList {
    let pl_node = root.children().find(|n| n.has_tag_name("part-list"));

    let mut parts = Vec::new();
    let mut groups = Vec::new();

    if let Some(pl) = pl_node {
        for child in pl.children() {
            match child.tag_name().name() {
                "score-part" => {
                    let id = child.attribute("id").unwrap_or("").to_string();
                    let name = first_child_text(&child, "part-name").unwrap_or_default();
                    let abbreviation = first_child_text(&child, "part-abbreviation")
                        .unwrap_or_else(|| name.clone());
                    parts.push(PartInfo {
                        id,
                        name,
                        abbreviation,
                    });
                }
                "part-group" => {
                    let number: u8 = child
                        .attribute("number")
                        .and_then(|n| n.parse().ok())
                        .unwrap_or(1);
                    let kind = first_child_text(&child, "group-symbol")
                        .map(|s| match s.as_str() {
                            "brace" => crate::notation::GroupBracket::Brace,
                            "bracket" => crate::notation::GroupBracket::Bracket,
                            "line" => crate::notation::GroupBracket::Line,
                            "square" => crate::notation::GroupBracket::Square,
                            _ => crate::notation::GroupBracket::Bracket,
                        })
                        .unwrap_or(crate::notation::GroupBracket::Bracket);
                    let name = first_child_text(&child, "group-name").unwrap_or_default();
                    groups.push(PartGroup { number, kind, name });
                }
                _ => {}
            }
        }
    }

    PartList { parts, groups }
}

fn parse_clef(attrs: &Node) -> Option<Clef> {
    let clef_node = attrs.children().find(|n| n.has_tag_name("clef"))?;

    if clef_node.attribute("additional") == Some("yes") {
        return None;
    }

    let sign = first_child_text(&clef_node, "sign")?;
    let line: i32 = first_child_text(&clef_node, "line")
        .and_then(|s| s.parse().ok())
        .unwrap_or_else(|| default_clef_line(&sign));

    match (sign.as_str(), line) {
        ("G", 2) => Some(Clef::Treble),
        ("F", 4) => Some(Clef::Bass),
        ("C", 3) => Some(Clef::Alto),
        ("C", 4) => Some(Clef::Tenor),
        ("C", _) => {
            eprintln!("Warning: C clef on line {line}, using Alto");
            Some(Clef::Alto)
        }
        ("percussion", _) | ("TAB", _) => {
            if sign == "TAB" {
                Some(Clef::Tab)
            } else {
                Some(Clef::Percussion)
            }
        }
        _ => {
            eprintln!(
                "Warning: unknown clef '{}' on line {}, falling back to Treble",
                sign, line
            );
            Some(Clef::Treble)
        }
    }
}
/// G→2 (treble), F→4 (bass), C→3 (alto).
fn default_clef_line(sign: &str) -> i32 {
    match sign {
        "G" => 2,
        "F" => 4,
        "C" => 3,
        _ => 0, // percussion, TAB, jianpu, none — line is irrelevant
    }
}

/// Extract clef-octave-change from the first clef child of an attributes element.
/// Returns the octave shift (e.g., -1 for tenor G clef).
fn parse_clef_octave_shift(attrs: &Node) -> Option<i8> {
    let clef_node = attrs.children().find(|n| n.has_tag_name("clef"))?;
    first_child_text(&clef_node, "clef-octave-change").and_then(|s| s.parse().ok())
}

/// Divisiones por negra (MusicXML `<divisions>`). Se acumula y hereda entre
/// compases hasta que un nuevo valor aparece (spec de MusicXML).
fn parse_divisions(attrs: &Node) -> Option<u32> {
    first_child_text(attrs, "divisions").and_then(|s| s.parse().ok())
}

fn parse_time(attrs: &Node) -> Option<TimeSignature> {
    let time_node = attrs.children().find(|n| n.has_tag_name("time"))?;

    // Per XSD, the time element contains either:
    //   (beats, beat-type, interchangeable?)   OR   senza-misura
    // Check for senza-misura first
    if time_node.children().any(|n| n.has_tag_name("senza-misura")) {
        eprintln!("Warning: senza-misura time signature found, defaulting to 4/4");
        return None; // Keep current time signature, don't replace
    }

    // Parse the first (beats, beat-type) pair.
    // Per XSD, multiple (beats, beat-type) pairs can appear for composite meters.
    let beats: u8 = first_child_text(&time_node, "beats").and_then(|s| s.parse().ok())?;
    let beat_type: u8 = first_child_text(&time_node, "beat-type").and_then(|s| s.parse().ok())?;

    // Detect composite time signatures (multiple beats/beat-type pairs)
    let beats_count = time_node
        .children()
        .filter(|n| n.has_tag_name("beats"))
        .count();
    if beats_count > 1 {
        eprintln!(
            "Warning: composite time signature detected (multiple beats/beat-type pairs), using first component {beats}/{beat_type}"
        );
    }

    // Warn if interchangeable is present (e.g., 2/2-6/8)
    if time_node
        .children()
        .any(|n| n.has_tag_name("interchangeable"))
    {
        eprintln!(
            "Warning: interchangeable time signature found, using first component {beats}/{beat_type}"
        );
    }
    Some(TimeSignature {
        numerator: beats,
        denominator: beat_type,
        style: TimeSignatureStyle::Numeric,
    })
}

/// Parse key signature from <attributes>.
fn parse_key(attrs: &Node) -> Option<KeySignature> {
    let key_node = attrs.children().find(|n| n.has_tag_name("key"))?;

    let fifths: i8 = first_child_text(&key_node, "fifths")
        .and_then(|s| s.parse().ok())
        .unwrap_or(0);

    let mode_str = first_child_text(&key_node, "mode").unwrap_or_default();
    let mode = match mode_str.as_str() {
        "major" => KeyMode::Major,
        "minor" => KeyMode::Minor,
        "dorian" => KeyMode::Dorian,
        "phrygian" => KeyMode::Phrygian,
        "lydian" => KeyMode::Lydian,
        "mixolydian" => KeyMode::Mixolydian,
        "aeolian" => KeyMode::Aeolian,
        "ionian" => KeyMode::Ionian,
        "locrian" => KeyMode::Locrian,
        "" => {
            if fifths == 0 {
                KeyMode::None
            } else {
                KeyMode::Major
            }
        }
        _ => {
            eprintln!("Warning: unknown mode '{}', using None", mode_str);
            KeyMode::None
        }
    };

    Some(KeySignature {
        fifths,
        mode,
        cancel: None, // Filled by caller when key changes
    })
}

/// Parse barline from <measure>.
fn parse_barline(measure: &Node) -> Barline {
    let barline_node = measure.children().find(|n| n.has_tag_name("barline"));

    if let Some(bl) = barline_node {
        let style = first_child_text(&bl, "bar-style")
            .map(|s| match s.as_str() {
                "regular" => BarStyle::Regular,
                "dotted" => BarStyle::Dotted,
                "dashed" => BarStyle::Dashed,
                "heavy" => BarStyle::Heavy,
                "light-light" => BarStyle::LightLight,
                "light-heavy" => BarStyle::LightHeavy,
                "heavy-light" => BarStyle::HeavyLight,
                "heavy-heavy" => BarStyle::HeavyHeavy,
                "tick" => BarStyle::Tick,
                "short" => BarStyle::Short,
                "none" => BarStyle::None,
                _ => {
                    eprintln!("Warning: unknown bar-style '{}', using Regular", s);
                    BarStyle::Regular
                }
            })
            .unwrap_or(BarStyle::Regular);

        let repeat = bl.children().find(|n| n.has_tag_name("repeat")).map(|r| {
            match r.attribute("direction") {
                Some("forward") => RepeatDirection::Forward,
                Some("backward") => RepeatDirection::Backward,
                _ => RepeatDirection::Forward,
            }
        });

        let ending = bl.children().find(|n| n.has_tag_name("ending")).map(|e| {
            let number = e.attribute("number").unwrap_or("1").to_string();
            let text = e.text().map(|s| s.trim().to_string());
            Ending {
                number,
                text,
                length: None,
            }
        });

        Barline {
            style,
            repeat,
            ending,
        }
    } else {
        Barline::default()
    }
}

fn parse_measure_elements(
    measure: &Node,
    octave_shift: i8,
) -> Result<Vec<MeasureElement>, MusicXmlError> {
    let mut elements: Vec<MeasureElement> = Vec::new();
    let mut chord_buffer: Vec<Note> = Vec::new();
    let mut held_note: Option<Note> = None;

    for child in measure.children() {
        match child.tag_name().name() {
            "note" => {
                if first_child_text(&child, "voice").is_some_and(|v| v != "1") {
                    continue;
                }

                let is_rest = child.children().any(|n| n.has_tag_name("rest"));
                let has_chord = child.children().any(|n| n.has_tag_name("chord"));
                let is_grace = child.children().any(|n| n.has_tag_name("grace"));

                if is_grace {
                    continue;
                }

                if is_rest {
                    // Flush held note and chord buffer before rest
                    if !chord_buffer.is_empty() {
                        elements.push(MeasureElement::Chord(std::mem::take(&mut chord_buffer)));
                    } else if let Some(n) = held_note.take() {
                        elements.push(MeasureElement::Note(n));
                    }
                    if let Some(rest) = parse_rest(&child) {
                        elements.push(MeasureElement::Rest(rest));
                        chord_buffer.clear();
                    }
                } else if let Some(note) = parse_note(&child, octave_shift) {
                    if has_chord {
                        // This note joins the chord. Move held note (first chord note)
                        // into chord_buffer if present.
                        if let Some(held) = held_note.take() {
                            chord_buffer.push(held);
                        }
                        chord_buffer.push(note);
                    } else if let Some(held) = held_note.take() {
                        // Previous note was held. Without a chord note following,
                        // it must be standalone. Flush held note and chord buffer.
                        if !chord_buffer.is_empty() {
                            elements.push(MeasureElement::Chord(std::mem::take(&mut chord_buffer)));
                        }
                        elements.push(MeasureElement::Note(held));
                        held_note = Some(note); // hold the new note
                    } else {
                        // No held note, no chord. Hold this note and wait to see
                        // if the next note has <chord/>.
                        held_note = Some(note);
                    }
                }
            }
            "backup" => {
                // Flush held note and chord buffer
                if !chord_buffer.is_empty() {
                    elements.push(MeasureElement::Chord(std::mem::take(&mut chord_buffer)));
                } else if let Some(n) = held_note.take() {
                    elements.push(MeasureElement::Note(n));
                }
                let duration: u32 = first_child_text(&child, "duration")
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(0);
                elements.push(MeasureElement::Backup(duration));
            }
            "forward" => {
                if !chord_buffer.is_empty() {
                    elements.push(MeasureElement::Chord(std::mem::take(&mut chord_buffer)));
                } else if let Some(n) = held_note.take() {
                    elements.push(MeasureElement::Note(n));
                }
                let duration: u32 = first_child_text(&child, "duration")
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(0);
                elements.push(MeasureElement::Forward(duration));
            }
            _ => {}
        }
    }

    // Flush remaining
    if !chord_buffer.is_empty() {
        elements.push(MeasureElement::Chord(chord_buffer));
    } else if let Some(n) = held_note.take() {
        elements.push(MeasureElement::Note(n));
    }

    Ok(elements)
}

/// Parse all `<direction>` elements from a measure node, together with the
/// index (into the final renderable `Measure::elements` sequence, same
/// indexing as `element_offsets`) of the note/rest/chord each one precedes.
///
/// Mirrors `parse_measure_elements`'s note/chord-flush state machine (without
/// building notes) so the counting stays in sync with the real element
/// indices. `has_pending` tracks whether a held note or in-progress chord
/// buffer exists that hasn't been flushed to `elements` yet — if so, a
/// direction seen "now" points one slot further, at the *next* new note,
/// since the currently-held note/chord will occupy the current slot once
/// flushed.
fn parse_directions(measure_node: &Node) -> Vec<crate::notation::Direction> {
    let mut dirs = Vec::new();
    let mut elements_count: usize = 0;
    let mut has_pending = false;

    for child in measure_node.children() {
        match child.tag_name().name() {
            "note" => {
                if first_child_text(&child, "voice").is_some_and(|v| v != "1") {
                    continue;
                }
                let is_rest = child.children().any(|n| n.has_tag_name("rest"));
                let has_chord = child.children().any(|n| n.has_tag_name("chord"));
                let is_grace = child.children().any(|n| n.has_tag_name("grace"));
                if is_grace {
                    continue;
                }
                if is_rest {
                    if has_pending {
                        elements_count += 1;
                    }
                    elements_count += 1;
                    has_pending = false;
                } else if has_chord {
                    has_pending = true;
                } else if has_pending {
                    elements_count += 1;
                    has_pending = true; // the new note becomes the newly held one
                } else {
                    has_pending = true;
                }
            }
            "backup" | "forward" => {
                if has_pending {
                    elements_count += 1;
                }
                elements_count += 1;
                has_pending = false;
            }
            "direction" => {
                let element_index = elements_count + usize::from(has_pending);
                if let Some(dir) = parse_single_direction(&child) {
                    dirs.push(crate::notation::Direction {
                        element_index,
                        ..dir
                    });
                }
            }
            _ => {}
        }
    }

    dirs
}

/// Parse one `<direction>` node into a `Direction` (with `element_index: 0`,
/// filled in by the caller — see `parse_directions`).
fn parse_single_direction(child: &Node) -> Option<crate::notation::Direction> {
    let placement = child
        .attribute("placement")
        .map(|p| match p {
            "above" => crate::notation::Placement::Above,
            _ => crate::notation::Placement::Below,
        })
        .unwrap_or(crate::notation::Placement::Below);

    let staff: u8 = child
        .attribute("staff")
        .and_then(|s| s.parse().ok())
        .unwrap_or(1);

    // Find <direction-type> child
    let dtype = child
        .children()
        .find(|n| n.has_tag_name("direction-type"))?;

    let kind = if let Some(dyn_node) = dtype.children().find(|n| n.has_tag_name("dynamics")) {
        let marks: Vec<crate::notation::DynamicMark> = dyn_node
            .children()
            .filter_map(|n| parse_dynamic_mark(n.tag_name().name()))
            .collect();
        if marks.is_empty() {
            return None;
        }
        crate::notation::DirectionKind::Dynamics(marks)
    } else if let Some(w_node) = dtype.children().find(|n| n.has_tag_name("wedge")) {
        let wedge_type = w_node.attribute("type").unwrap_or("crescendo");
        // WedgeKind has no Stop variant; skip stop wedges
        if wedge_type == "stop" {
            return None;
        }
        let niente = w_node.attribute("niente").is_some_and(|v| v == "yes");
        let kind = if wedge_type == "diminuendo" {
            crate::notation::WedgeKind::Diminuendo
        } else {
            crate::notation::WedgeKind::Crescendo
        };
        crate::notation::DirectionKind::Wedge(crate::notation::Wedge {
            kind,
            niente,
            spread: 0.0,
        })
    } else if let Some(w_node) = dtype.children().find(|n| n.has_tag_name("words")) {
        let text = w_node.text().unwrap_or_default().to_string();
        crate::notation::DirectionKind::Words(text)
    } else if let Some(r_node) = dtype.children().find(|n| n.has_tag_name("rehearsal")) {
        let text = r_node.text().unwrap_or_default().to_string();
        crate::notation::DirectionKind::Rehearsal(text)
    } else if let Some(m_node) = dtype.children().find(|n| n.has_tag_name("metronome")) {
        // beat-unit: "quarter", "eighth", "half", etc. → NoteFigure
        let beat_unit = first_child_text(&m_node, "beat-unit")
            .map(|s| parse_beat_unit(&s))
            .unwrap_or(crate::notation::NoteFigure::Quarter);
        let per_minute: u16 = first_child_text(&m_node, "per-minute")
            .and_then(|s| s.parse().ok())
            .unwrap_or(120);
        let parentheses = m_node.attribute("parentheses").is_some_and(|v| v == "yes");
        crate::notation::DirectionKind::Metronome(crate::notation::Metronome {
            beat_unit,
            per_minute,
            parentheses,
        })
    } else if let Some(o_node) = dtype.children().find(|n| n.has_tag_name("octave-shift")) {
        let size: u8 = o_node
            .attribute("size")
            .and_then(|s| s.parse().ok())
            .unwrap_or(8);
        let kind = match o_node.attribute("type") {
            Some("down") => crate::notation::OctaveShiftKind::Down,
            Some("stop") => crate::notation::OctaveShiftKind::Stop,
            _ => crate::notation::OctaveShiftKind::Up,
        };
        crate::notation::DirectionKind::OctaveShift(crate::notation::OctaveShift { size, kind })
    } else {
        let p_node = dtype.children().find(|n| n.has_tag_name("pedal"))?;
        let kind = match p_node.attribute("type") {
            Some("stop") => crate::notation::PedalKind::Stop,
            Some("change") => crate::notation::PedalKind::Change,
            _ => crate::notation::PedalKind::Start,
        };
        let line = p_node.attribute("line").is_some_and(|v| v == "yes");
        crate::notation::DirectionKind::Pedal(crate::notation::Pedal { kind, line })
    };

    Some(crate::notation::Direction {
        element_index: 0,
        placement,
        staff,
        kind,
    })
}

fn parse_beat_unit(s: &str) -> crate::notation::NoteFigure {
    match s {
        "breve" | "long" => crate::notation::NoteFigure::Breve,
        "whole" => crate::notation::NoteFigure::Whole,
        "half" => crate::notation::NoteFigure::Half,
        "quarter" => crate::notation::NoteFigure::Quarter,
        "eighth" => crate::notation::NoteFigure::Eighth,
        "16th" => crate::notation::NoteFigure::Sixteenth,
        "32nd" => crate::notation::NoteFigure::ThirtySecond,
        "64th" => crate::notation::NoteFigure::SixtyFourth,
        "128th" => crate::notation::NoteFigure::HundredTwentyEighth,
        _ => crate::notation::NoteFigure::Quarter,
    }
}

fn parse_dynamic_mark(name: &str) -> Option<crate::notation::DynamicMark> {
    Some(match name {
        "pppp" => crate::notation::DynamicMark::PPPP,
        "ppp" => crate::notation::DynamicMark::PPP,
        "pp" => crate::notation::DynamicMark::PP,
        "p" => crate::notation::DynamicMark::P,
        "mp" => crate::notation::DynamicMark::MP,
        "mf" => crate::notation::DynamicMark::MF,
        "f" => crate::notation::DynamicMark::F,
        "ff" => crate::notation::DynamicMark::FF,
        "fff" => crate::notation::DynamicMark::FFF,
        "ffff" => crate::notation::DynamicMark::FFFF,
        "sf" => crate::notation::DynamicMark::SF,
        "sfz" => crate::notation::DynamicMark::SFZ,
        "sffz" => crate::notation::DynamicMark::SFFZ,
        "sfp" => crate::notation::DynamicMark::SFP,
        "sfpp" => crate::notation::DynamicMark::SFPP,
        "fp" => crate::notation::DynamicMark::FP,
        "rf" => crate::notation::DynamicMark::RF,
        "rfz" => crate::notation::DynamicMark::RFZ,
        "fz" => crate::notation::DynamicMark::FZ,
        "n" => crate::notation::DynamicMark::N,
        "pf" => crate::notation::DynamicMark::PF,
        other => crate::notation::DynamicMark::Other(other.to_string()),
    })
}

/// Parse <notations> child of a <note> into NoteAttachment.
fn parse_attachments(note_node: &Node) -> Option<crate::notation::NoteAttachment> {
    let notations = note_node.children().find(|n| n.has_tag_name("notations"))?;

    let mut attachment = crate::notation::NoteAttachment::default();
    let mut has_any = false;

    for child in notations.children() {
        match child.tag_name().name() {
            "articulations" => {
                for art in child.children() {
                    let tag = art.tag_name().name();
                    let articulation = match tag {
                        "accent" => Some(crate::notation::Articulation::Accent),
                        "strong-accent" => Some(crate::notation::Articulation::StrongAccent),
                        "staccato" => Some(crate::notation::Articulation::Staccato),
                        "staccatissimo" => Some(crate::notation::Articulation::Staccatissimo),
                        "tenuto" => Some(crate::notation::Articulation::Tenuto),
                        "detached-legato" => Some(crate::notation::Articulation::DetachedLegato),
                        "spiccato" => Some(crate::notation::Articulation::Spiccato),
                        "breath-mark" => Some(crate::notation::Articulation::BreathMark),
                        "caesura" => Some(crate::notation::Articulation::Caesura),
                        "soft-accent" => Some(crate::notation::Articulation::SoftAccent),
                        "scoop" => Some(crate::notation::Articulation::Scoop),
                        "plop" => Some(crate::notation::Articulation::Plop),
                        "doit" => Some(crate::notation::Articulation::Doit),
                        "falloff" => Some(crate::notation::Articulation::Falloff),
                        _ => None,
                    };
                    if let Some(a) = articulation {
                        attachment.articulations.push(a);
                        has_any = true;
                    }
                }
            }
            "ornaments" => {
                for orn in child.children() {
                    let tag = orn.tag_name().name();
                    let ornament = match tag {
                        "trill-mark" => Some(crate::notation::Ornament::Trill),
                        "turn" => Some(crate::notation::Ornament::Turn),
                        "inverted-turn" => Some(crate::notation::Ornament::InvertedTurn),
                        "mordent" => Some(crate::notation::Ornament::Mordent),
                        "inverted-mordent" => Some(crate::notation::Ornament::InvertedMordent),
                        "tremolo" => {
                            let marks: u8 =
                                orn.text().and_then(|s| s.trim().parse().ok()).unwrap_or(3);
                            Some(crate::notation::Ornament::Tremolo { marks })
                        }
                        "wavy-line" => {
                            let start = orn.attribute("type") == Some("start");
                            let stop = orn.attribute("type") == Some("stop");
                            Some(crate::notation::Ornament::WavyLine { start, stop })
                        }
                        _ => None,
                    };
                    if let Some(o) = ornament {
                        attachment.ornaments.push(o);
                        has_any = true;
                    }
                }
            }
            "technical" => {
                for tech in child.children() {
                    let tag = tech.tag_name().name();
                    let technical = match tag {
                        "up-bow" => Some(crate::notation::Technical::UpBow),
                        "down-bow" => Some(crate::notation::Technical::DownBow),
                        "harmonic" => Some(crate::notation::Technical::Harmonic),
                        "open-string" => Some(crate::notation::Technical::OpenString),
                        "thumb-position" => Some(crate::notation::Technical::ThumbPosition),
                        "fingering" => tech
                            .text()
                            .map(|s| crate::notation::Technical::Fingering(s.trim().into())),
                        "pluck" => Some(crate::notation::Technical::Pluck),
                        "double-tongue" => Some(crate::notation::Technical::DoubleTongue),
                        "triple-tongue" => Some(crate::notation::Technical::TripleTongue),
                        "stopped" => Some(crate::notation::Technical::Stopped),
                        "snap-pizzicato" => Some(crate::notation::Technical::SnapPizzicato),
                        "fret" => tech
                            .text()
                            .and_then(|s| s.trim().parse().ok())
                            .map(crate::notation::Technical::Fret),
                        "string" => tech
                            .text()
                            .and_then(|s| s.trim().parse().ok())
                            .map(crate::notation::Technical::String),
                        "hammer-on" => Some(crate::notation::Technical::HammerOn),
                        "pull-off" => Some(crate::notation::Technical::PullOff),
                        "bend" => {
                            let alter: f32 = first_child_text(&tech, "bend-alter")
                                .and_then(|s| s.parse().ok())
                                .unwrap_or(0.0);
                            let pre_bend = tech.children().any(|n| n.has_tag_name("pre-bend"));
                            let release = tech.children().any(|n| n.has_tag_name("release"));
                            Some(crate::notation::Technical::Bend {
                                alter,
                                pre_bend,
                                release,
                            })
                        }
                        "tap" => Some(crate::notation::Technical::Tap),
                        "heel" => Some(crate::notation::Technical::Heel),
                        "toe" => Some(crate::notation::Technical::Toe),
                        "fingernails" => Some(crate::notation::Technical::Fingernails),
                        _ => tech
                            .text()
                            .map(|s| crate::notation::Technical::Other(s.trim().into())),
                    };
                    if let Some(t) = technical {
                        attachment.technical.push(t);
                        has_any = true;
                    }
                }
            }
            "dynamics" => {
                for dyn_node in child.children() {
                    let tag = dyn_node.tag_name().name();
                    let dyn_mark = match tag {
                        "pppp" => Some(crate::notation::DynamicMark::PPPP),
                        "ppp" => Some(crate::notation::DynamicMark::PPP),
                        "pp" => Some(crate::notation::DynamicMark::PP),
                        "p" => Some(crate::notation::DynamicMark::P),
                        "mp" => Some(crate::notation::DynamicMark::MP),
                        "mf" => Some(crate::notation::DynamicMark::MF),
                        "f" => Some(crate::notation::DynamicMark::F),
                        "ff" => Some(crate::notation::DynamicMark::FF),
                        "fff" => Some(crate::notation::DynamicMark::FFF),
                        "ffff" => Some(crate::notation::DynamicMark::FFFF),
                        "sf" => Some(crate::notation::DynamicMark::SF),
                        "sfz" => Some(crate::notation::DynamicMark::SFZ),
                        "sffz" => Some(crate::notation::DynamicMark::SFFZ),
                        "sfp" => Some(crate::notation::DynamicMark::SFP),
                        "sfpp" => Some(crate::notation::DynamicMark::SFPP),
                        "fp" => Some(crate::notation::DynamicMark::FP),
                        "rf" => Some(crate::notation::DynamicMark::RF),
                        "rfz" => Some(crate::notation::DynamicMark::RFZ),
                        "fz" => Some(crate::notation::DynamicMark::FZ),
                        "n" => Some(crate::notation::DynamicMark::N),
                        "pf" => Some(crate::notation::DynamicMark::PF),
                        "other-dynamics" => dyn_node
                            .text()
                            .map(|s| crate::notation::DynamicMark::Other(s.trim().into())),
                        _ => None,
                    };
                    if let Some(dm) = dyn_mark {
                        attachment.dynamics = Some(dm);
                        has_any = true;
                        break; // Only first dynamic
                    }
                }
            }
            "fermata" => {
                let shape = match child.attribute("type") {
                    Some("angled") => crate::notation::FermataShape::Angled,
                    Some("square") => crate::notation::FermataShape::Square,
                    _ => crate::notation::FermataShape::Normal,
                };
                let placement = match child.attribute("placement") {
                    Some("above") => crate::notation::Placement::Above,
                    Some("below") => crate::notation::Placement::Below,
                    _ => crate::notation::Placement::Above,
                };
                attachment.fermata = Some(crate::notation::Fermata {
                    shape,
                    upright: true,
                    placement,
                });
                has_any = true;
            }
            "tied" => {
                let kind = match child.attribute("type") {
                    Some("start") => crate::notation::TieKind::Start,
                    Some("stop") => crate::notation::TieKind::Stop,
                    Some("continue") => crate::notation::TieKind::Continue,
                    Some("let-ring") => crate::notation::TieKind::LetRing,
                    _ => crate::notation::TieKind::Start,
                };
                let number: u8 = child
                    .attribute("number")
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(1);
                let placement = match child.attribute("placement") {
                    Some("above") => crate::notation::Placement::Above,
                    Some("below") => crate::notation::Placement::Below,
                    _ => crate::notation::Placement::Above,
                };
                attachment.ties.push(crate::notation::Tie {
                    kind,
                    number,
                    placement,
                });
                has_any = true;
            }
            "slur" => {
                let kind = match child.attribute("type") {
                    Some("start") => crate::notation::SlurKind::Start,
                    Some("stop") => crate::notation::SlurKind::Stop,
                    Some("continue") => crate::notation::SlurKind::Continue,
                    _ => crate::notation::SlurKind::Start,
                };
                let number: u8 = child
                    .attribute("number")
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(1);
                let placement = match child.attribute("placement") {
                    Some("above") => crate::notation::Placement::Above,
                    Some("below") => crate::notation::Placement::Below,
                    _ => crate::notation::Placement::Above,
                };
                attachment.slurs.push(crate::notation::Slur {
                    kind,
                    number,
                    placement,
                });
                has_any = true;
            }
            "glissando" => {
                let kind = match child.attribute("type") {
                    Some("start") => crate::notation::GlissandoKind::Start,
                    _ => crate::notation::GlissandoKind::Stop,
                };
                let line_type = match first_child_text(&child, "line-type").as_deref() {
                    Some("dashed") => crate::notation::LineType::Dashed,
                    Some("dotted") => crate::notation::LineType::Dotted,
                    Some("wavy") => crate::notation::LineType::Wavy,
                    _ => crate::notation::LineType::Solid,
                };
                attachment.glissando = Some(crate::notation::Glissando { kind, line_type });
                has_any = true;
            }
            "arpeggiate" => {
                let direction = match child.attribute("direction") {
                    Some("up") => Some(crate::notation::ArpeggioDirection::Up),
                    Some("down") => Some(crate::notation::ArpeggioDirection::Down),
                    _ => None,
                };
                attachment.arpeggiate = Some(crate::notation::Arpeggiate { direction });
                has_any = true;
            }
            "tremolo" => {
                let marks: u8 = child
                    .text()
                    .and_then(|s| s.trim().parse().ok())
                    .unwrap_or(3);
                attachment.tremolo = Some(crate::notation::Tremolo { marks });
                has_any = true;
            }
            _ => {}
        }
    }

    if has_any { Some(attachment) } else { None }
}
fn parse_note(note_node: &Node, octave_shift: i8) -> Option<Note> {
    let note_type = match first_child_text(note_node, "type") {
        Some(t) => t,
        None => {
            eprintln!("Warning: note without <type> element, skipping");
            return None;
        }
    };
    let figure = parse_note_type(&note_type)?;

    // Count dots (0-3)
    let dotted: u8 = note_node
        .children()
        .filter(|n| n.has_tag_name("dot"))
        .count()
        .min(3) as u8;

    // Parse chord flag
    let chord = note_node.children().any(|n| n.has_tag_name("chord"));

    // Parse grace flag
    let grace = note_node.children().any(|n| n.has_tag_name("grace"));

    // Parse stem direction
    let stem_direction = first_child_text(note_node, "stem")
        .map(|s| match s.as_str() {
            "up" => StemDirection::Up,
            "down" => StemDirection::Down,
            _ => StemDirection::Up,
        })
        .unwrap_or(StemDirection::Up);

    // Parse accidental override
    let accidental_override = first_child_text(note_node, "accidental").map(|s| match s.as_str() {
        "natural" => Accidental::Natural,
        "sharp" => Accidental::Sharp,
        "flat" => Accidental::Flat,
        "double-sharp" => Accidental::DoubleSharp,
        "flat-flat" | "double-flat" => Accidental::DoubleFlat,
        _ => Accidental::Natural,
    });

    let pitch_node = note_node.children().find(|n| n.has_tag_name("pitch"))?;

    let step = parse_step(&pitch_node)?;
    let accidental = parse_alter(&pitch_node)?;
    let octave = parse_octave(&pitch_node)?;

    let effective_octave = octave + octave_shift;

    Some(Note {
        pitch: Pitch {
            step,
            accidental,
            octave: effective_octave,
        },
        figure,
        dotted,
        accidental_override,
        stem_direction,
        grace,
        chord,
        attachments: parse_attachments(note_node)
            .map(|mut a| {
                // <tied> in <notations> and <tie> as direct child represent the
                // same logical tie — deduplicate by (kind, number).
                let existing: Vec<(crate::notation::TieKind, u8)> =
                    a.ties.iter().map(|t| (t.kind, t.number)).collect();
                for tie in parse_ties(note_node) {
                    if !existing.contains(&(tie.kind, tie.number)) {
                        a.ties.push(tie);
                    }
                }
                a
            })
            .or_else(|| {
                let ties = parse_ties(note_node);
                if ties.is_empty() {
                    None
                } else {
                    Some(crate::notation::NoteAttachment {
                        ties,
                        ..Default::default()
                    })
                }
            }),
        lyrics: parse_lyrics(note_node),
    })
}

/// Parse `<lyric>` children of a note.
fn parse_lyrics(note_node: &Node) -> Vec<crate::notation::Lyric> {
    let mut lyrics = Vec::new();

    for child in note_node.children() {
        if child.tag_name().name() != "lyric" {
            continue;
        }

        let number: u8 = child
            .attribute("number")
            .and_then(|s| s.parse().ok())
            .unwrap_or(1);

        let syllabic = first_child_text(&child, "syllabic")
            .map(|s| match s.as_str() {
                "single" => crate::notation::Syllabic::Single,
                "begin" => crate::notation::Syllabic::Begin,
                "end" => crate::notation::Syllabic::End,
                "middle" => crate::notation::Syllabic::Middle,
                _ => crate::notation::Syllabic::Single,
            })
            .unwrap_or(crate::notation::Syllabic::Single);

        let text = child
            .children()
            .find(|n| n.has_tag_name("text"))
            .and_then(|n| n.text())
            .map(|s| s.to_string())
            .unwrap_or_default();

        let extend = child.children().any(|n| n.has_tag_name("extend"));

        lyrics.push(crate::notation::Lyric {
            number,
            syllabic,
            text,
            extend,
        });
    }

    lyrics
}

/// Parse `<tie>` children of a `<note>`. These are direct children, not inside `<notations>`.
fn parse_ties(note_node: &Node) -> Vec<crate::notation::Tie> {
    let mut ties = Vec::new();
    for child in note_node.children() {
        if child.tag_name().name() != "tie" {
            continue;
        }
        let kind = match child.attribute("type") {
            Some("start") => crate::notation::TieKind::Start,
            Some("stop") => crate::notation::TieKind::Stop,
            Some("continue") => crate::notation::TieKind::Continue,
            Some("let-ring") => crate::notation::TieKind::LetRing,
            _ => crate::notation::TieKind::Start,
        };
        let number: u8 = child
            .attribute("number")
            .and_then(|s| s.parse().ok())
            .unwrap_or(1);
        let placement = child
            .attribute("placement")
            .map(|p| match p {
                "above" => crate::notation::Placement::Above,
                _ => crate::notation::Placement::Below,
            })
            .unwrap_or(crate::notation::Placement::Below);
        ties.push(crate::notation::Tie {
            kind,
            number,
            placement,
        });
    }
    ties
}
fn parse_rest(note_node: &Node) -> Option<Rest> {
    let note_type = first_child_text(note_node, "type")?;
    let figure = parse_note_type(&note_type)?;

    let dotted: u8 = note_node
        .children()
        .filter(|n| n.has_tag_name("dot"))
        .count()
        .min(3) as u8;

    let measure = note_node
        .children()
        .find(|n| n.has_tag_name("rest"))
        .and_then(|r| r.attribute("measure"))
        == Some("yes");

    // Parse display-step and display-octave for positioned rests
    let display_step = first_child_text(note_node, "display-step").and_then(|s| parse_step_str(&s));
    let display_octave: Option<i8> =
        first_child_text(note_node, "display-octave").and_then(|s| s.parse().ok());

    Some(Rest {
        figure,
        dotted,
        display_step,
        display_octave,
        measure,
    })
}

fn parse_note_type(t: &str) -> Option<NoteFigure> {
    match t {
        "breve" => Some(NoteFigure::Breve),
        "whole" => Some(NoteFigure::Whole),
        "half" => Some(NoteFigure::Half),
        "quarter" => Some(NoteFigure::Quarter),
        "eighth" => Some(NoteFigure::Eighth),
        "16th" => Some(NoteFigure::Sixteenth),
        "32nd" => Some(NoteFigure::ThirtySecond),
        "64th" => Some(NoteFigure::SixtyFourth),
        "128th" => Some(NoteFigure::HundredTwentyEighth),
        "long" | "maxima" | "256th" | "512th" | "1024th" => {
            eprintln!("Warning: unsupported note type '{}', skipping note", t);
            None
        }
        _ => {
            eprintln!("Warning: unknown note type '{}', skipping note", t);
            None
        }
    }
}

fn parse_step_str(s: &str) -> Option<Step> {
    match s {
        "C" => Some(Step::C),
        "D" => Some(Step::D),
        "E" => Some(Step::E),
        "F" => Some(Step::F),
        "G" => Some(Step::G),
        "A" => Some(Step::A),
        "B" => Some(Step::B),
        _ => None,
    }
}

fn parse_step(pitch_node: &Node) -> Option<Step> {
    let step_text = first_child_text(pitch_node, "step")?;
    // Per XSD step type: A, B, C, D, E, F, G (enumerated)
    match step_text.as_str() {
        "A" => Some(Step::A),
        "B" => Some(Step::B),
        "C" => Some(Step::C),
        "D" => Some(Step::D),
        "E" => Some(Step::E),
        "F" => Some(Step::F),
        "G" => Some(Step::G),
        _ => {
            eprintln!("Warning: invalid step '{}', skipping note", step_text);
            None
        }
    }
}

fn parse_alter(pitch_node: &Node) -> Option<Accidental> {
    // Per XSD, alter is of type semitones (xs:decimal, minOccurs="0")
    // -1 = flat, 0 or absent = natural, 1 = sharp
    // Decimal values (e.g., 0.5 for quarter tone) are valid in XSD but we skip them
    let alter_text = first_child_text(pitch_node, "alter");
    match alter_text.as_deref() {
        None | Some("0") => Some(Accidental::Natural),
        Some("-1") => Some(Accidental::Flat),
        Some("1") => Some(Accidental::Sharp),
        Some(v) => {
            eprintln!("Warning: unsupported alter value '{}', skipping note", v);
            None
        }
    }
}

fn parse_octave(pitch_node: &Node) -> Option<i8> {
    // Per XSD, octave is type octave (xs:integer, minInclusive=0, maxInclusive=9)
    first_child_text(pitch_node, "octave").and_then(|s| s.parse().ok())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::notation::MeasureElement;

    /// Helper: extracts notes from MeasureElement vector, skipping rests/backup/forward.
    fn extract_notes(elements: &[MeasureElement]) -> Vec<&crate::notation::Note> {
        elements
            .iter()
            .filter_map(|e| match e {
                MeasureElement::Note(n) => Some(n),
                _ => None,
            })
            .collect()
    }

    /// Helper: gets the first staff from the first system.
    fn first_staff(score: &Score) -> &Staff {
        &score.systems[0].staves[0]
    }

    /// Helper: gets the first measure from the first staff.
    fn first_measure(score: &Score) -> &Measure {
        &first_staff(score).measures[0]
    }

    #[test]
    fn parse_basic_musicxml() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<score-partwise version="4.0">
  <movement-title>Test Piece</movement-title>
  <identification>
    <creator type="composer">J.S. Bach</creator>
  </identification>
  <part-list>
    <score-part id="P1"><part-name>Piano</part-name></score-part>
  </part-list>
  <part id="P1">
    <measure number="1">
      <attributes>
        <divisions>1</divisions>
        <key><fifths>0</fifths></key>
        <time><beats>4</beats><beat-type>4</beat-type></time>
        <clef><sign>G</sign><line>2</line></clef>
      </attributes>
      <note>
        <pitch><step>C</step><octave>4</octave></pitch>
        <duration>1</duration>
        <type>quarter</type>
      </note>
      <note>
        <pitch><step>D</step><octave>4</octave></pitch>
        <duration>1</duration>
        <type>quarter</type>
      </note>
      <note>
        <pitch><step>E</step><alter>1</alter><octave>4</octave></pitch>
        <duration>1</duration>
        <type>quarter</type>
        <dot/>
      </note>
      <note>
        <pitch><step>F</step><alter>-1</alter><octave>4</octave></pitch>
        <duration>2</duration>
        <type>half</type>
      </note>
    </measure>
  </part>
</score-partwise>"#;

        let score = parse_musicxml(xml).expect("should parse successfully");

        assert_eq!(score.title, "Test Piece");
        assert_eq!(score.composer, "J.S. Bach");
        assert_eq!(score.systems.len(), 1);
        assert_eq!(first_staff(&score).clef, Clef::Treble);
        assert_eq!(first_staff(&score).measures.len(), 1);

        let measure = first_measure(&score);
        assert_eq!(measure.time_signature.numerator, 4);
        assert_eq!(measure.time_signature.denominator, 4);

        let notes = extract_notes(&measure.elements);
        assert_eq!(notes.len(), 4);

        assert_eq!(notes[0].pitch.step, Step::C);
        assert_eq!(notes[0].pitch.accidental, Accidental::Natural);
        assert_eq!(notes[0].pitch.octave, 4);
        assert_eq!(notes[0].figure, NoteFigure::Quarter);
        assert_eq!(notes[0].dotted, 0);

        assert_eq!(notes[1].pitch.step, Step::D);
        assert_eq!(notes[2].pitch.step, Step::E);
        assert_eq!(notes[2].pitch.accidental, Accidental::Sharp);
        assert_eq!(notes[2].dotted, 1);
        assert_eq!(notes[3].pitch.step, Step::F);
        assert_eq!(notes[3].pitch.accidental, Accidental::Flat);
        assert_eq!(notes[3].figure, NoteFigure::Half);
    }

    #[test]
    fn parse_bass_clef() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<score-partwise version="4.0">
  <part-list><score-part id="P1"><part-name>Bass</part-name></score-part></part-list>
  <part id="P1">
    <measure number="1">
      <attributes>
        <clef><sign>F</sign><line>4</line></clef>
      </attributes>
      <note>
        <pitch><step>G</step><octave>2</octave></pitch>
        <duration>1</duration>
        <type>whole</type>
      </note>
    </measure>
  </part>
</score-partwise>"#;

        let score = parse_musicxml(xml).expect("should parse");
        assert_eq!(first_staff(&score).clef, Clef::Bass);
        let notes = extract_notes(&first_measure(&score).elements);
        assert_eq!(notes[0].figure, NoteFigure::Whole);
    }

    #[test]
    fn skip_rests_and_grace_notes() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<score-partwise version="4.0">
  <part-list><score-part id="P1"><part-name>Test</part-name></score-part></part-list>
  <part id="P1">
    <measure number="1">
      <attributes>
        <clef><sign>G</sign><line>2</line></clef>
      </attributes>
      <note>
        <rest/>
        <duration>1</duration>
        <type>quarter</type>
      </note>
      <note>
        <pitch><step>C</step><octave>4</octave></pitch>
        <duration>1</duration>
        <type>quarter</type>
      </note>
      <note>
        <grace/>
        <pitch><step>D</step><octave>4</octave></pitch>
        <type>eighth</type>
      </note>
      <note>
        <pitch><step>E</step><octave>4</octave></pitch>
        <duration>1</duration>
        <type>quarter</type>
      </note>
    </measure>
  </part>
</score-partwise>"#;

        let score = parse_musicxml(xml).expect("should parse");
        let notes = extract_notes(&first_measure(&score).elements);
        assert_eq!(notes.len(), 2);
        assert_eq!(notes[0].pitch.step, Step::C);
        assert_eq!(notes[1].pitch.step, Step::E);
    }

    #[test]
    fn breve_note_type_parsed() {
        // Breve is now supported
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<score-partwise version="4.0">
  <part-list><score-part id="P1"><part-name>Test</part-name></score-part></part-list>
  <part id="P1">
    <measure number="1">
      <attributes>
        <clef><sign>G</sign><line>2</line></clef>
      </attributes>
      <note>
        <pitch><step>C</step><octave>4</octave></pitch>
        <duration>1</duration>
        <type>quarter</type>
      </note>
      <note>
        <pitch><step>D</step><octave>4</octave></pitch>
        <duration>8</duration>
        <type>breve</type>
      </note>
      <note>
        <pitch><step>E</step><octave>4</octave></pitch>
        <duration>1</duration>
        <type>quarter</type>
      </note>
    </measure>
  </part>
</score-partwise>"#;

        let score = parse_musicxml(xml).expect("should parse");
        let notes = extract_notes(&first_measure(&score).elements);
        // Breve is now supported, so 3 notes
        assert_eq!(notes.len(), 3);
        assert_eq!(notes[1].figure, NoteFigure::Breve);
    }

    #[test]
    fn invalid_xml_returns_error() {
        let result = parse_musicxml("not valid xml");
        assert!(result.is_err());
    }

    #[test]
    fn wrong_root_element() {
        let xml = "<something-else></something-else>";
        let result = parse_musicxml(xml);
        assert!(result.is_err());
    }

    #[test]
    fn tenor_clef_octave_shift() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<score-partwise version="4.0">
  <part-list><score-part id="P1"><part-name>Tenor</part-name></score-part></part-list>
  <part id="P1">
    <measure number="1">
      <attributes>
        <clef><sign>G</sign><line>2</line><clef-octave-change>-1</clef-octave-change></clef>
      </attributes>
      <note>
        <pitch><step>C</step><octave>5</octave></pitch>
        <duration>1</duration>
        <type>quarter</type>
      </note>
    </measure>
  </part>
</score-partwise>"#;

        let score = parse_musicxml(xml).expect("should parse");
        let notes = extract_notes(&first_measure(&score).elements);
        // clef-octave-change=-1 shifts octave down: 5 + (-1) = 4
        assert_eq!(notes[0].pitch.octave, 4);
        assert_eq!(notes[0].pitch.step, Step::C);
    }

    #[test]
    fn additional_clef_not_primary() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<score-partwise version="4.0">
  <part-list><score-part id="P1"><part-name>Piano</part-name></score-part></part-list>
  <part id="P1">
    <measure number="1">
      <attributes>
        <clef><sign>G</sign><line>2</line></clef>
        <clef additional="yes"><sign>F</sign><line>4</line></clef>
      </attributes>
      <note>
        <pitch><step>C</step><octave>4</octave></pitch>
        <duration>1</duration>
        <type>quarter</type>
      </note>
    </measure>
  </part>
</score-partwise>"#;

        let score = parse_musicxml(xml).expect("should parse");
        assert_eq!(first_staff(&score).clef, Clef::Treble);
    }

    #[test]
    fn senza_misura_keeps_default_time() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<score-partwise version="4.0">
  <part-list><score-part id="P1"><part-name>Test</part-name></score-part></part-list>
  <part id="P1">
    <measure number="1">
      <attributes>
        <time><senza-misura/></time>
        <clef><sign>G</sign><line>2</line></clef>
      </attributes>
      <note>
        <pitch><step>C</step><octave>4</octave></pitch>
        <duration>1</duration>
        <type>quarter</type>
      </note>
    </measure>
  </part>
</score-partwise>"#;

        let score = parse_musicxml(xml).expect("should parse");
        let time = first_measure(&score).time_signature;
        assert_eq!(time.numerator, 4);
        assert_eq!(time.denominator, 4);
    }

    #[test]
    fn score_timewise_rejected() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<score-timewise version="4.0">
  <measure number="1">
    <part id="P1">
      <note><pitch><step>C</step><octave>4</octave></pitch><duration>1</duration><type>quarter</type></note>
    </part>
  </measure>
</score-timewise>"#;

        let result = parse_musicxml(xml);
        assert!(result.is_err());
        match result {
            Err(MusicXmlError::Unsupported(msg)) => {
                assert!(
                    msg.contains("timewise"),
                    "expected timewise error, got: {msg}"
                );
            }
            _ => panic!("expected Unsupported error"),
        }
    }

    #[test]
    fn missing_note_type_skipped() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<score-partwise version="4.0">
  <part-list><score-part id="P1"><part-name>Test</part-name></score-part></part-list>
  <part id="P1">
    <measure number="1">
      <attributes>
        <clef><sign>G</sign><line>2</line></clef>
      </attributes>
      <note>
        <pitch><step>C</step><octave>4</octave></pitch>
        <duration>4</duration>
      </note>
      <note>
        <pitch><step>D</step><octave>4</octave></pitch>
        <duration>1</duration>
        <type>quarter</type>
      </note>
    </measure>
  </part>
</score-partwise>"#;

        let score = parse_musicxml(xml).expect("should parse");
        let notes = extract_notes(&first_measure(&score).elements);
        assert_eq!(notes.len(), 1);
        assert_eq!(notes[0].pitch.step, Step::D);
    }

    #[test]
    fn creator_fallback_no_type_attribute() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<score-partwise version="4.0">
  <identification>
    <creator>Anonymous</creator>
  </identification>
  <part-list><score-part id="P1"><part-name>Test</part-name></score-part></part-list>
  <part id="P1">
    <measure number="1">
      <attributes>
        <clef><sign>G</sign><line>2</line></clef>
      </attributes>
      <note>
        <pitch><step>C</step><octave>4</octave></pitch>
        <duration>1</duration>
        <type>quarter</type>
      </note>
    </measure>
  </part>
</score-partwise>"#;

        let score = parse_musicxml(xml).expect("should parse");
        assert_eq!(score.composer, "Anonymous");
    }

    #[test]
    fn clef_without_line_uses_standard_default() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<score-partwise version="4.0">
  <part-list><score-part id="P1"><part-name>Test</part-name></score-part></part-list>
  <part id="P1">
    <measure number="1">
      <attributes>
        <clef><sign>F</sign></clef>
      </attributes>
      <note>
        <pitch><step>C</step><octave>3</octave></pitch>
        <duration>1</duration>
        <type>quarter</type>
      </note>
    </measure>
  </part>
</score-partwise>"#;

        let score = parse_musicxml(xml).expect("should parse");
        assert_eq!(first_staff(&score).clef, Clef::Bass);
    }

    #[test]
    fn direction_and_lyric_parsed() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<score-partwise version="4.0">
  <part-list><score-part id="P1"><part-name>Test</part-name></score-part></part-list>
  <part id="P1">
    <measure number="1">
      <attributes>
        <divisions>1</divisions>
        <clef><sign>G</sign><line>2</line></clef>
      </attributes>
      <direction placement="below">
        <direction-type>
          <dynamics><f/></dynamics>
        </direction-type>
      </direction>
      <note>
        <pitch><step>C</step><octave>4</octave></pitch>
        <duration>1</duration>
        <type>quarter</type>
        <lyric number="1">
          <syllabic>single</syllabic>
          <text>La</text>
        </lyric>
      </note>
    </measure>
  </part>
</score-partwise>"#;

        let score = parse_musicxml(xml).expect("should parse");
        let measure = &first_staff(&score).measures[0];

        // Direction parsed
        assert!(!measure.directions.is_empty(), "direction should be parsed");

        // Lyric parsed
        let notes: Vec<&crate::notation::Note> = measure
            .elements
            .iter()
            .filter_map(|e| match e {
                crate::notation::MeasureElement::Note(n) => Some(n),
                _ => None,
            })
            .collect();
        assert_eq!(notes.len(), 1);
        assert!(!notes[0].lyrics.is_empty(), "lyric should be parsed");
        assert_eq!(notes[0].lyrics[0].text, "La");
    }

    #[test]
    fn divisions_inherited_across_measures() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<score-partwise version="4.0">
  <part-list><score-part id="P1"><part-name>Test</part-name></score-part></part-list>
  <part id="P1">
    <measure number="1">
      <attributes>
        <divisions>16</divisions>
        <clef><sign>G</sign><line>2</line></clef>
      </attributes>
      <note>
        <pitch><step>C</step><octave>4</octave></pitch>
        <duration>16</duration>
        <type>quarter</type>
      </note>
    </measure>
    <measure number="2">
      <note>
        <pitch><step>D</step><octave>4</octave></pitch>
        <duration>16</duration>
        <type>quarter</type>
      </note>
    </measure>
  </part>
</score-partwise>"#;

        let score = parse_musicxml(xml).expect("should parse");
        let staff = first_staff(&score);
        assert_eq!(
            staff.measures[0].divisions, 16,
            "measure 1: explicit divisions"
        );
        assert_eq!(
            staff.measures[1].divisions, 16,
            "measure 2: divisions inherited from measure 1"
        );
    }

    #[test]
    fn directions_get_distinct_element_index() {
        // "f" precede a la primer nota (index 0), "mf" precede a la segunda (index 1) —
        // reproduce el caso del compás 4 de simple.musicxml.xml, donde antes ambas
        // direcciones colapsaban en la misma posición X al renderizar.
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<score-partwise version="4.0">
  <part-list><score-part id="P1"><part-name>Test</part-name></score-part></part-list>
  <part id="P1">
    <measure number="1">
      <attributes>
        <divisions>1</divisions>
        <clef><sign>G</sign><line>2</line></clef>
      </attributes>
      <direction>
        <direction-type><dynamics><f/></dynamics></direction-type>
      </direction>
      <note>
        <pitch><step>C</step><octave>4</octave></pitch>
        <duration>1</duration>
        <type>quarter</type>
      </note>
      <direction>
        <direction-type><dynamics><mf/></dynamics></direction-type>
      </direction>
      <note>
        <pitch><step>D</step><octave>4</octave></pitch>
        <duration>1</duration>
        <type>quarter</type>
      </note>
    </measure>
  </part>
</score-partwise>"#;

        let score = parse_musicxml(xml).expect("should parse");
        let measure = &first_staff(&score).measures[0];
        assert_eq!(measure.directions.len(), 2);
        assert_eq!(
            measure.directions[0].element_index, 0,
            "f precede a la nota 0"
        );
        assert_eq!(
            measure.directions[1].element_index, 1,
            "mf precede a la nota 1"
        );
        assert_ne!(
            measure.directions[0].element_index, measure.directions[1].element_index,
            "las dos direcciones deben apuntar a posiciones distintas"
        );
    }

    #[test]
    fn credits_parsed_with_page_layout() {
        // Reproduce el fixture real: título centrado, compositor centrado, y un
        // tercer crédito alineado a la derecha (ej. "Music by X") que antes se
        // perdía por completo (credits: Vec::new() hardcodeado).
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<score-partwise version="4.0">
  <defaults>
    <page-layout>
      <page-width>1310</page-width>
      <page-height>1850</page-height>
    </page-layout>
  </defaults>
  <credit>
    <credit-words default-x="655" default-y="1790" justify="center">Título</credit-words>
  </credit>
  <credit>
    <credit-words default-x="1270" default-y="1616.9" justify="right">Music by X</credit-words>
  </credit>
  <part-list><score-part id="P1"><part-name>Test</part-name></score-part></part-list>
  <part id="P1"><measure number="1"><note><rest/><duration>4</duration><type>whole</type></note></measure></part>
</score-partwise>"#;

        let score = parse_musicxml(xml).expect("should parse");
        assert_eq!(score.credits.len(), 2);

        let title_credit = &score.credits[0];
        assert!(
            matches!(title_credit.kind, crate::notation::CreditKind::Words(ref s) if s == "Título")
        );
        assert_eq!(title_credit.justify, crate::notation::CreditJustify::Center);
        assert!((title_credit.default_x - 655.0 / 1310.0).abs() < 0.001);

        let arranger_credit = &score.credits[1];
        assert_eq!(
            arranger_credit.justify,
            crate::notation::CreditJustify::Right
        );
        assert!((arranger_credit.default_x - 1270.0 / 1310.0).abs() < 0.001);
    }

    #[test]
    fn credits_omitted_without_page_layout() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<score-partwise version="4.0">
  <credit>
    <credit-words default-x="655" default-y="1790">Título</credit-words>
  </credit>
  <part-list><score-part id="P1"><part-name>Test</part-name></score-part></part-list>
  <part id="P1"><measure number="1"><note><rest/><duration>4</duration><type>whole</type></note></measure></part>
</score-partwise>"#;

        let score = parse_musicxml(xml).expect("should parse");
        assert!(
            score.credits.is_empty(),
            "sin page-layout no hay espacio de coordenadas confiable, no se debe adivinar"
        );
    }
}

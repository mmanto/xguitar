# LAYOUT.md — Estructura de Layouts

Define la arquitectura visual de la aplicación.

---


m-guitar tiene un layout único de dos paneles: barra superior fija + panel central con scroll de páginas A4.

```
┌──────────────────────────────────────────────────┐
│  TopBar (5% altura)                              │
│  📄 Archivo ▼              🌙 dark    EN idioma  │
├──────────────────────────────────────────────────┤
│                                                  │
│               CentralPanel                       │
│                                                  │
│              Claves                              │
│         𝄞  Sol    𝄢  Fa                         │
│                                                  │
│              Figuras                             │
│    𝅝        𝅗𝅥       𝅘𝅥       𝅘𝅥𝅮       𝅘𝅥𝅯       │
│  Redonda  Blanca  Negra  Corchea  Semicorchea   │
│                                                  │
└──────────────────────────────────────────────────┘
```

### TopBar

```rust
egui::Panel::top("top_bar")
    .exact_size(screen_h * 0.05)   // 5% del viewport
    .frame(
        egui::Frame::default()
            .fill(ui.style().visuals.panel_fill)
            .stroke(egui::Stroke::new(1.0, color))
            .inner_margin(font_h * 0.5),
    )
    .show(ui, |ui| { /* menú + toggles */ });
```

- **Izquierda:** `MenuBar` con menú Archivo
- **Derecha:** `Layout::right_to_left` con toggles de idioma y dark mode

### CentralPanel
### CentralPanel — Visor de páginas A4

```rust
egui::CentralPanel::default().show(ui, |ui| {
    let layout = compute_pages(&self.score, self.zoom, 4);
    egui::ScrollArea::vertical()
        .auto_shrink([false; 2])
        .show(ui, |ui| {
            // Calcular páginas por fila según ancho disponible
            // Centrar horizontalmente
            // Renderizar páginas con render_pages()
            // Manejar zoom con handle_zoom()
        });
});
```

Las páginas A4 se distribuyen en grilla de 1–2 columnas con scroll vertical.


---

## Proyección — Layouts futuros

### Editor de partitura (ScoreEditor)

Layout previsto cuando la app tenga pentagramas editables:

```
┌──────────────────────────────────────────────────┐
│  TopBar                                          │
├──────────┬───────────────────────────────────────┤
│ Toolbox  │                                       │
│ (notas,  │         Área de pentagramas           │
│  figuras,│         (scrollable)                  │
│  claves) │                                       │
│          │                                       │
├──────────┴───────────────────────────────────────┤
│  StatusBar (compás, tonalidad, tempo)            │
└──────────────────────────────────────────────────┘
```

- **Toolbox:** panel lateral izquierdo (~200px) con paleta de símbolos musicales
- **Área de pentagramas:** `CentralPanel` con scroll vertical
- **StatusBar:** panel inferior (~30px) con información del compás actual

### Visor de partitura (ScoreViewer)

Layout de solo lectura para visualización/impresión:

```
┌──────────────────────────────────────────────────┐
│  TopBar                                          │
├──────────────────────────────────────────────────┤
│                                                  │
│            Partitura a página completa           │
│            (renderizado via Painter)             │
│                                                  │
├──────────────────────────────────────────────────┤
│  [◀◀] [▶] [⏸] [▶▶]  Pág 1/3  Zoom: 100%       │
└──────────────────────────────────────────────────┘
```

---

## Breakpoints y responsive

Por ahora la aplicación asume ventana maximizada y escala con `viewport_rect()`.
No hay breakpoints definidos — las fuentes y espaciados usan tamaños fijos en pixeles.

A futuro, se definirá un sistema de escala basado en el tamaño de pantalla:

| Viewport height | Escala |
|---|---|
| < 768px | 0.75x |
| 768–1080px | 1.0x |
| > 1080px | 1.25x |

---

## Convención de paneles egui

- `TopBottomPanel::top` → barra superior, toolbars
- `CentralPanel` → área de trabajo principal (pentagramas)
- `SidePanel::left` / `SidePanel::right` → toolboxes, paletas
- `TopBottomPanel::bottom` → status bar
- No anidar `CentralPanel` dentro de otro `CentralPanel`

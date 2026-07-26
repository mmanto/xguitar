# DESIGN.md — Design System

Sistema de diseño de m-guitar. Toda decisión visual debe registrarse aquí.

---

## Modos de color

La aplicación soporta dos modos, toggleables desde la barra superior con 🌙.

### Modo oscuro (default)

Usa `egui::Visuals::dark()`.

| Rol | Apariencia |
|---|---|
| Fondo de panel | Gris oscuro (#2D2D2D aprox.) |
| Fondo central | Negro/gris muy oscuro |
| Texto principal | Blanco/gris claro |
| Texto secundario | `weak_text_color()` — gris medio |
| Bordes | `noninteractive.fg_stroke` — gris tenue |

### Modo claro

Usa `egui::Visuals::light()`.

---

## Tipografía

| Rol | Fuente | Tamaño | Peso |
|---|---|---|---|
| UI general (menús, botones, labels) | **LelandText** | `Body` (default egui) | Regular |
| Glifos musicales | **Leland** | 48px (`note_size`) | Regular |
| Títulos de sección | LelandText | 18px | Regular |
| Labels de figuras | LelandText | 14px | Regular |

- **LelandText** se establece como `FontFamily::Proportional` principal → toda la UI la usa
- **Leland** se registra como `FontFamily::Name("Leland")` → solo glifos musicales

Nunca usar fuentes del sistema; todo el texto visible debe renderizarse con LelandText.

---

## Espaciado

Escala usada en la UI actual:

| Elemento | Valor |
|---|---|
| Margin interno de top bar | `font_h * 0.5` |
| Altura de top bar | `screen_h * 0.05` (5% del viewport) |
| Espacio entre secciones | 24px (`ui.add_space(24.0)`) |
| Espacio entre título y contenido | 8px (`ui.add_space(8.0)`) |
| Espacio entre glifos | 32px (`ui.add_space(32.0)`) |
| Espacio entre figuras | 16px (`ui.add_space(16.0)`) |
| Margen superior del contenido | 20px (`ui.add_space(20.0)`) |

---

## Componentes

### Barra superior (TopBar)

- Altura: 5% del viewport
- Fondo: `panel_fill`
- Borde inferior: 1px sólido, color `noninteractive.fg_stroke`
- Layout horizontal: menú a la izquierda, toggles a la derecha

```
┌──────────────────────────────────────────────────┐
│ 📄 Archivo ▼                 🌙    EN    ES │
└──────────────────────────────────────────────────┘
```

### Menú Archivo

Dropdown con opciones:
- ✨ Nuevo
- 📂 Abrir
- ───── (separador)
- ✕ Cerrar
- 🚪 Salir

### Toggles

- **Idioma:** `selectable_label` que muestra "EN" o "ES" según el idioma opuesto
- **Modo oscuro:** `selectable_label` con ícono 🌙

### Panel central

- Ocupa el espacio restante bajo la top bar
- Contenido centrado verticalmente (`ui.vertical_centered`)
- Secciones con título + glifos horizontales

### Sección de glifos

Cada sección tiene:
1. Título en `weak_text_color` (18px) — ej: "Claves", "Figuras"
2. Espacio de 8px
3. Fila horizontal con glifos de 48px + labels descriptivos

---

## Iconografía

No se usa librería de íconos externa. Los emojis Unicode se usan como íconos en menús:
- 📄 File, ✨ New, 📂 Open, ✕ Close, 🚪 Exit, 🌙 Dark mode

Los glifos musicales son caracteres Unicode del bloque SMuFL (`U+E000–U+EFFF`) renderizados con la fuente Leland.

---

## Proyección — Tokens de diseño propios

Cuando egui no alcance para el look deseado, se definirán colores propios vía `Visuals` custom:

```rust
let mut visuals = egui::Visuals::dark();
visuals.panel_fill = Color32::from_rgb(30, 30, 30);
visuals.window_fill = Color32::from_rgb(18, 18, 18);
// etc.
ctx.set_visuals(visuals);
```

---

## Hojas A4 (Page)

La partitura se renderiza dentro de hojas A4 virtuales con proporciones reales (210×297 mm a 96 DPI).

### Dimensiones base

| Propiedad | Valor | Nota |
|---|---|---|
| Ancho A4 | 793.7 pt | `210 mm × 96 / 25.4` |
| Alto A4 | 1122.5 pt | `297 mm × 96 / 25.4` |
| Margen superior | 60 pt | ~15.9 mm |
| Margen inferior | 60 pt | ~15.9 mm |
| Margen izquierdo | 80 pt | ~21.2 mm |
| Margen derecho | 60 pt | ~15.9 mm |
| Espacio entre sistemas | 60 pt | `STAFF_LINE_SPACING × 5` |
| Gap entre páginas | 40 pt | Horizontal y vertical |

### Estilo visual

**Modo claro:**
- Fondo de hoja: blanco (`Color32::WHITE`)
- Borde: gris medio (`Color32::from_gray(180)`, 1px)
- Sombra: rectángulo negro con alpha 60, desplazado 4px abajo-derecha

**Modo oscuro:**
- Fondo de hoja: gris oscuro (`Color32::from_gray(38)`)
- Borde: gris (`Color32::from_gray(80)`, 1px)
- Sombra: igual que modo claro (contrasta contra el fondo del panel)

### Layout responsive

- Si el ancho disponible permite 2 páginas + gap → 2 columnas
- Si no hay espacio suficiente → 1 columna centrada
- Recálculo por frame sin estado persistente

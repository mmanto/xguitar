# SCREENS.md — Inventario de Pantallas

Registro de todas las pantallas, vistas y estados de UI del sistema.
Actualizar ante cualquier cambio en la navegación o nuevas vistas.

---

## Pantallas actuales

### Pantalla principal — Visor de glifos

| MAIN-01 | `/` (única vista) | Partitura demo paginada en hojas A4 con pentagramas, claves y notas. Entrada de notas por teclado y barra de estado inferior. | 🟢 Implementado |
|---|---|---|---|

**Estados:**
- **Success:** glifos renderizados, toggles funcionales

**Estados pendientes:**
- **Loading:** N/A (no hay carga asíncrona)
- **Empty:** N/A (los glifos son estáticos)
- **Error:** N/A (no hay fuentes de error actualmente)

---

## Proyección — Pantallas futuras

### Editor de partitura

| ID | Ruta | Componente | Descripción | Estado |
|---|---|---|---|---|
| EDIT-01 | `/editor` | `ScoreEditor` | Editor de partitura con pentagrama, inserción de notas, reproducción | 🔴 Pendiente |

### Configuración

| ID | Ruta | Componente | Descripción | Estado |
|---|---|---|---|---|
| CFG-01 | `/settings` | `SettingsDialog` | Preferencias: idioma, tema, dispositivo MIDI, audio | 🔴 Pendiente |

### Visor de partitura — parcialmente implementado

| ID | Ruta | Componente | Descripción | Estado |
|---|---|---|---|---|
| VIEW-01 | `/` (integrado) | `render_pages` | Hojas A4 con sombra y bordes, scroll, zoom, layout responsive 1-2 páginas por fila | 🟡 Implementado |

### Nuevo archivo

| ID | Ruta | Componente | Descripción | Estado |
|---|---|---|---|---|
| NEW-01 | `/new` | `NewScoreDialog` | Diálogo: título, compositor, compás, tonalidad, armadura | 🔴 Pendiente |

---

## Estados de pantalla (a implementar)

Cada pantalla futura debe manejar y documentar estos estados:

| Estado | Descripción |
|---|---|
| **Loading** | Skeleton o spinner mientras carga fuente o archivo |
| **Empty** | Sin partitura abierta (first-time experience) |
| **Error** | Archivo corrupto, fuente no encontrada |
| **Success** | Estado normal con datos |
| **Unsaved** | Indicador de cambios sin guardar |

---

## Flujos de usuario proyectados

### Flujo principal: Crear partitura

```
Abrir app → Archivo → Nuevo → Completar metadatos → Editor → Guardar
```

### Flujo: Abrir partitura existente

```
Abrir app → Archivo → Abrir → File dialog (.musicxml, .midi) → Visor/Editor
```

### Flujo: Exportar

```
Editor → Archivo → Exportar → Elegir formato (PDF, MIDI, MusicXML) → Guardar
```

---

## Leyenda de estados

| Ícono | Estado |
|---|---|
| 🟢 | Completado e implementado |
| 🟡 | En desarrollo |
| 🟠 | En diseño / revisión |
| 🔴 | Pendiente |
| ⚫ | Descartado |

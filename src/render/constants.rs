/// Espacio entre líneas del pentagrama en puntos egui.
pub const STAFF_LINE_SPACING: f32 = 12.0;

/// Número de líneas del pentagrama.
pub const STAFF_LINE_COUNT: usize = 5;

/// Altura total del pentagrama (entre primera y quinta línea).
pub const STAFF_HEIGHT: f32 = STAFF_LINE_SPACING * (STAFF_LINE_COUNT as f32 - 1.0);

/// Ancho por defecto de una medida.
pub const DEFAULT_MEASURE_WIDTH: f32 = 220.0;

/// Tamaño base de glifo de nota.
pub const NOTEHEAD_SIZE: f32 = STAFF_LINE_SPACING * 2.5;

/// Ancho de línea de ledger.
pub const LEDGER_LINE_WIDTH: f32 = NOTEHEAD_SIZE * 1.02;

/// A4 width in egui points (210 mm at 96 DPI).
pub const A4_WIDTH_PT: f32 = 210.0 * 96.0 / 25.4; // ≈ 793.7

/// A4 height in egui points (297 mm at 96 DPI).
pub const A4_HEIGHT_PT: f32 = 297.0 * 96.0 / 25.4; // ≈ 1122.5

/// Margen superior de la hoja A4 en puntos (~15 mm).
pub const PAGE_MARGIN_TOP: f32 = 60.0;

/// Margen inferior de la hoja A4 en puntos (~15 mm).
pub const PAGE_MARGIN_BOTTOM: f32 = 60.0;

/// Margen izquierdo de la hoja A4 en puntos (~20 mm).
pub const PAGE_MARGIN_LEFT: f32 = 80.0;

/// Margen derecho de la hoja A4 en puntos (~15 mm).
pub const PAGE_MARGIN_RIGHT: f32 = 60.0;

/// Espacio vertical entre pentagramas consecutivos dentro de una página.
pub const SYSTEM_SPACING: f32 = STAFF_LINE_SPACING * 5.0; // 60.0 pt

/// Espacio horizontal entre dos páginas lado a lado.
pub const PAGE_GAP: f32 = 40.0;

/// Ancho reservado para la clave al inicio del staff.
pub const CLEF_WIDTH: f32 = 48.0;

/// Espacio entre la barra inicial y la clave.
pub const INITIAL_BAR_GAP: f32 = 8.0;

/// Ancho reservado para la métrica de compás después de la clave.
pub const TIME_SIG_WIDTH: f32 = 28.0;

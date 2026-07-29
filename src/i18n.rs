/// Idiomas soportados por la aplicación.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Lang {
    Es,
    En,
}

/// Traducción por lookup de claves, sin allocation (retorna `&'static str`).
pub struct I18n {
    pub lang: Lang,
}

impl I18n {
    pub fn new(lang: Lang) -> Self {
        Self { lang }
    }

    /// Obtiene la traducción para `key` en el idioma actual.
    /// Si la clave no existe, devuelve la clave misma como fallback.
    pub fn t<'a>(&self, key: &'a str) -> &'a str {
        match self.lang {
            Lang::Es => match key {
                "file" => "Archivo",
                "new" => "Nuevo",
                "open" => "Abrir",
                "close" => "Cerrar",
                "exit" => "Salir",
                "app_title" => "Mi App",
                "lang_toggle" => "EN",
                "style" => "Estilo",
                "input_hint" => {
                    "Letra (C–B) → nota | 1=Redonda 2=Blanca 4=Negra 8=Corchea 6=Semicorchea 32=Fusa 33=Semifusa | Enter para insertar"
                }
                "new_score" => "Partitura nueva creada",
                "open_success" => "{} cargada",
                "open_error" => "Error al abrir: {}",
                "play" => "▶",
                "stop" => "⏹",
                "play_wasm_unavailable" => "Reproducción no disponible en el navegador todavía",
                "no_instrument" => "Sin instrumento configurado — elegí un archivo .sfz en Preferencias",
                _ => key,
            },
            Lang::En => match key {
                "file" => "File",
                "new" => "New",
                "open" => "Open",
                "close" => "Close",
                "exit" => "Exit",
                "app_title" => "My App",
                "lang_toggle" => "ES",
                "style" => "Style",
                "input_hint" => {
                    "Letter (C–B) → note | 1=Whole 2=Half 4=Quarter 8=Eighth 6=16th 32=32nd 33=64th | Enter to insert"
                }
                "new_score" => "New score created",
                "open_success" => "{} loaded",
                "open_error" => "Error opening: {}",
                "play" => "▶",
                "stop" => "⏹",
                "play_wasm_unavailable" => "Playback isn't available in the browser build yet",
                "no_instrument" => "No instrument configured — pick a .sfz file in Preferences",
                _ => key,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_keys_present_and_distinct() {
        let es = I18n::new(Lang::Es);
        let en = I18n::new(Lang::En);
        let keys = [
            "file",
            "new",
            "open",
            "close",
            "exit",
            "app_title",
            "lang_toggle",
            "style",
            "input_hint",
            "new_score",
            "open_success",
            "open_error",
            "play_wasm_unavailable",
            "no_instrument",
        ];
        for key in keys {
            let es_val = es.t(key);
            let en_val = en.t(key);
            assert_ne!(es_val, key, "missing ES: {key}");
            assert_ne!(en_val, key, "missing EN: {key}");
            assert_ne!(es_val, en_val, "same value ES/EN: {key}");
        }
    }

    #[test]
    fn unknown_key_fallback() {
        let i18n = I18n::new(Lang::Es);
        assert_eq!(i18n.t("nonexistent"), "nonexistent");
    }
}

//! Languages. Each tool keeps its texts in one table per language (English is the
//! base; the other tables use `..EN`, so a missing text falls back to English).
//! To add a language: a variant in `Lang`, an entry in `LANGS`, and a table in
//! every tool (`Lang::pick` falls back to English until then).

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Lang {
    En,
    Pt,
    Es,
}

/// Central list of supported languages (code, language).
pub const LANGS: &[(&str, Lang)] = &[("en", Lang::En), ("pt", Lang::Pt), ("es", Lang::Es)];

impl Lang {
    /// Maps a code or locale such as "pt", "pt-PT" or "es_ES.UTF-8" to a language.
    pub fn parse(code: &str) -> Option<Lang> {
        let prefix = code.split(['-', '_', '.', '@']).next()?.to_ascii_lowercase();
        LANGS.iter().find(|(c, _)| *c == prefix).map(|(_, l)| *l)
    }

    /// Order of preference: `--lang`, RTOOLS_LANG, the system locale, English.
    pub fn detect(cli: Option<&str>) -> Lang {
        cli.and_then(Lang::parse)
            .or_else(|| std::env::var("RTOOLS_LANG").ok().as_deref().and_then(Lang::parse))
            .or_else(|| sys_locale::get_locale().as_deref().and_then(Lang::parse))
            .unwrap_or(Lang::En)
    }

    pub fn pick<T>(self, en: &'static T, pt: &'static T, es: &'static T) -> &'static T {
        match self {
            Lang::En => en,
            Lang::Pt => pt,
            Lang::Es => es,
        }
    }
}

/// Messages shared by all the tools.
pub struct Common {
    pub unknown_option: &'static str,
    pub missing_value: &'static str,
    pub invalid_value: &'static str,
    pub cannot_read: &'static str,
    pub try_help: &'static str,
    pub not_a_terminal: &'static str,
}

const EN: Common = Common {
    unknown_option: "unknown option",
    missing_value: "missing value for",
    invalid_value: "invalid value for",
    cannot_read: "cannot read",
    try_help: "Try '--help' for more information.",
    not_a_terminal: "this program needs a terminal",
};

const PT: Common = Common {
    unknown_option: "opção desconhecida",
    missing_value: "falta o valor de",
    invalid_value: "valor inválido para",
    cannot_read: "não foi possível ler",
    try_help: "Use '--help' para mais informação.",
    not_a_terminal: "este programa precisa de um terminal",
};

const ES: Common = Common {
    unknown_option: "opción desconocida",
    missing_value: "falta el valor de",
    invalid_value: "valor no válido para",
    cannot_read: "no se puede leer",
    try_help: "Use '--help' para más información.",
    not_a_terminal: "este programa necesita un terminal",
};

pub fn common(lang: Lang) -> &'static Common {
    lang.pick(&EN, &PT, &ES)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_locales() {
        assert_eq!(Lang::parse("pt-PT"), Some(Lang::Pt));
        assert_eq!(Lang::parse("es_ES.UTF-8"), Some(Lang::Es));
        assert_eq!(Lang::parse("EN"), Some(Lang::En));
        assert_eq!(Lang::parse("de-DE"), None);
    }
}

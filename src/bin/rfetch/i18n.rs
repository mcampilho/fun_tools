//! Program texts, one table per language.
//!
//! English is the base table; other tables only override what they translate
//! (`..EN`), so any missing text falls back to English.
//! To add a language: add a variant to `Lang`, an entry to `LANGS` and a table.

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Lang {
    En,
    Pt,
    Es,
}

/// Central list of supported languages (code, language).
pub const LANGS: &[(&str, Lang)] = &[("en", Lang::En), ("pt", Lang::Pt), ("es", Lang::Es)];

pub struct Texts {
    pub os: &'static str,
    pub kernel: &'static str,
    pub uptime: &'static str,
    pub shell: &'static str,
    pub cpu: &'static str,
    pub memory: &'static str,
    pub disk: &'static str,
    pub git: &'static str,
    pub ip: &'static str,
    pub location: &'static str,
    pub weather: &'static str,
    pub timezone: &'static str,
    pub locale: &'static str,
    pub keyboard: &'static str,
    pub days: &'static str,
    pub hours: &'static str,
    pub minutes: &'static str,
    pub unknown_option: &'static str,
    pub unknown_logo: &'static str,
    pub logos_available: &'static str,
    pub logos_wsl: &'static str,
    pub help: &'static str,
    // WMO weather codes
    pub w_clear: &'static str,
    pub w_mainly_clear: &'static str,
    pub w_partly_cloudy: &'static str,
    pub w_overcast: &'static str,
    pub w_fog: &'static str,
    pub w_drizzle: &'static str,
    pub w_freezing_drizzle: &'static str,
    pub w_rain: &'static str,
    pub w_freezing_rain: &'static str,
    pub w_snow: &'static str,
    pub w_snow_grains: &'static str,
    pub w_showers: &'static str,
    pub w_snow_showers: &'static str,
    pub w_thunderstorm: &'static str,
    pub w_thunderstorm_hail: &'static str,
    pub w_unknown: &'static str,
}

const EN: Texts = Texts {
    os: "OS",
    kernel: "Kernel",
    uptime: "Uptime",
    shell: "Shell",
    cpu: "CPU",
    memory: "Memory",
    disk: "Disk",
    git: "Git",
    ip: "IP",
    location: "Location",
    weather: "Weather",
    timezone: "Time zone",
    locale: "Locale",
    keyboard: "Keyboard",
    days: "d",
    hours: "h",
    minutes: "min",
    unknown_option: "unknown option",
    unknown_logo: "unknown logo",
    logos_available: "Available logos:",
    logos_wsl: "Any Linux logo also accepts the -wsl suffix (e.g. ubuntu-wsl).",
    help: "\
Usage: rfetch [options]

Options:
  -o, --offline       Do not use the network (no IP, location or weather)
  -r, --refresh       Ignore the cached network data and fetch it again
      --color         Always use colours
      --no-color      Never use colours (also: NO_COLOR environment variable)
  -l, --lang <code>   Language: en, pt, es (also: RFETCH_LANG, RTOOLS_LANG)
      --logo <name>   Use another logo (--logo list shows them all)
  -h, --help          Show this help
  -V, --version       Show the version",
    w_clear: "Clear sky",
    w_mainly_clear: "Mainly clear",
    w_partly_cloudy: "Partly cloudy",
    w_overcast: "Overcast",
    w_fog: "Fog",
    w_drizzle: "Drizzle",
    w_freezing_drizzle: "Freezing drizzle",
    w_rain: "Rain",
    w_freezing_rain: "Freezing rain",
    w_snow: "Snow",
    w_snow_grains: "Snow grains",
    w_showers: "Rain showers",
    w_snow_showers: "Snow showers",
    w_thunderstorm: "Thunderstorm",
    w_thunderstorm_hail: "Thunderstorm with hail",
    w_unknown: "Unknown",
};

const PT: Texts = Texts {
    os: "SO",
    kernel: "Kernel",
    uptime: "Tempo ligado",
    shell: "Shell",
    cpu: "CPU",
    memory: "Memória",
    disk: "Disco",
    git: "Git",
    ip: "IP",
    location: "Localização",
    weather: "Tempo",
    timezone: "Fuso horário",
    locale: "Idioma",
    keyboard: "Teclado",
    days: "d",
    hours: "h",
    minutes: "min",
    unknown_option: "opção desconhecida",
    unknown_logo: "logótipo desconhecido",
    logos_available: "Logótipos disponíveis:",
    logos_wsl: "Qualquer logótipo de Linux aceita também o sufixo -wsl (por exemplo ubuntu-wsl).",
    help: "\
Utilização: rfetch [opções]

Opções:
  -o, --offline       Não usar a rede (sem IP, localização nem tempo)
  -r, --refresh       Ignorar os dados de rede guardados e voltar a obtê-los
      --color         Usar sempre cores
      --no-color      Nunca usar cores (também: variável de ambiente NO_COLOR)
  -l, --lang <código> Idioma: en, pt, es (também: RFETCH_LANG, RTOOLS_LANG)
      --logo <nome>   Usar outro logótipo (--logo list mostra todos)
  -h, --help          Mostrar esta ajuda
  -V, --version       Mostrar a versão",
    w_clear: "Céu limpo",
    w_mainly_clear: "Céu pouco nublado",
    w_partly_cloudy: "Parcialmente nublado",
    w_overcast: "Nublado",
    w_fog: "Nevoeiro",
    w_drizzle: "Chuvisco",
    w_freezing_drizzle: "Chuvisco gelado",
    w_rain: "Chuva",
    w_freezing_rain: "Chuva gelada",
    w_snow: "Neve",
    w_snow_grains: "Grãos de neve",
    w_showers: "Aguaceiros",
    w_snow_showers: "Aguaceiros de neve",
    w_thunderstorm: "Trovoada",
    w_thunderstorm_hail: "Trovoada com granizo",
    w_unknown: "Desconhecido",
    ..EN
};

const ES: Texts = Texts {
    os: "SO",
    kernel: "Kernel",
    uptime: "Tiempo activo",
    shell: "Shell",
    cpu: "CPU",
    memory: "Memoria",
    disk: "Disco",
    git: "Git",
    ip: "IP",
    location: "Ubicación",
    weather: "Clima",
    timezone: "Zona horaria",
    locale: "Idioma",
    keyboard: "Teclado",
    days: "d",
    hours: "h",
    minutes: "min",
    unknown_option: "opción desconocida",
    unknown_logo: "logotipo desconocido",
    logos_available: "Logotipos disponibles:",
    logos_wsl: "Cualquier logotipo de Linux acepta también el sufijo -wsl (por ejemplo ubuntu-wsl).",
    help: "\
Uso: rfetch [opciones]

Opciones:
  -o, --offline       No usar la red (sin IP, ubicación ni clima)
  -r, --refresh       Ignorar los datos de red guardados y volver a obtenerlos
      --color         Usar siempre colores
      --no-color      No usar nunca colores (también: variable de entorno NO_COLOR)
  -l, --lang <código> Idioma: en, pt, es (también: RFETCH_LANG, RTOOLS_LANG)
      --logo <nombre> Usar otro logotipo (--logo list los muestra todos)
  -h, --help          Mostrar esta ayuda
  -V, --version       Mostrar la versión",
    w_clear: "Despejado",
    w_mainly_clear: "Mayormente despejado",
    w_partly_cloudy: "Parcialmente nublado",
    w_overcast: "Cubierto",
    w_fog: "Niebla",
    w_drizzle: "Llovizna",
    w_freezing_drizzle: "Llovizna helada",
    w_rain: "Lluvia",
    w_freezing_rain: "Lluvia helada",
    w_snow: "Nieve",
    w_snow_grains: "Granos de nieve",
    w_showers: "Chubascos",
    w_snow_showers: "Chubascos de nieve",
    w_thunderstorm: "Tormenta",
    w_thunderstorm_hail: "Tormenta con granizo",
    w_unknown: "Desconocido",
    ..EN
};

pub fn texts(lang: Lang) -> &'static Texts {
    match lang {
        Lang::En => &EN,
        Lang::Pt => &PT,
        Lang::Es => &ES,
    }
}

/// Maps a code or locale such as "pt", "pt-PT" or "es_ES.UTF-8" to a language.
pub fn parse(code: &str) -> Option<Lang> {
    let prefix = code
        .split(|c| c == '-' || c == '_' || c == '.' || c == '@')
        .next()?
        .to_ascii_lowercase();
    LANGS.iter().find(|(c, _)| *c == prefix).map(|(_, l)| *l)
}

/// Order of preference: --lang, RFETCH_LANG, RTOOLS_LANG (all the tools), system
/// locale, English.
pub fn detect(cli: Option<&str>, system_locale: Option<&str>) -> Lang {
    cli.and_then(parse)
        .or_else(|| std::env::var("RFETCH_LANG").ok().as_deref().and_then(parse))
        .or_else(|| std::env::var("RTOOLS_LANG").ok().as_deref().and_then(parse))
        .or_else(|| system_locale.and_then(parse))
        .unwrap_or(Lang::En)
}

impl Texts {
    pub fn weather(&self, code: u32) -> &'static str {
        match code {
            0 => self.w_clear,
            1 => self.w_mainly_clear,
            2 => self.w_partly_cloudy,
            3 => self.w_overcast,
            45 | 48 => self.w_fog,
            51 | 53 | 55 => self.w_drizzle,
            56 | 57 => self.w_freezing_drizzle,
            61 | 63 | 65 => self.w_rain,
            66 | 67 => self.w_freezing_rain,
            71 | 73 | 75 => self.w_snow,
            77 => self.w_snow_grains,
            80..=82 => self.w_showers,
            85 | 86 => self.w_snow_showers,
            95 => self.w_thunderstorm,
            96 | 99 => self.w_thunderstorm_hail,
            _ => self.w_unknown,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_locales() {
        assert_eq!(parse("pt-PT"), Some(Lang::Pt));
        assert_eq!(parse("es_ES.UTF-8"), Some(Lang::Es));
        assert_eq!(parse("EN"), Some(Lang::En));
        assert_eq!(parse("de-DE"), None);
        assert_eq!(detect(Some("xx"), Some("de")), Lang::En);
    }
}

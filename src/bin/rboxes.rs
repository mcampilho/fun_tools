//! rboxes: draws a box around text (like boxes). Widths are measured in terminal
//! columns, so wide characters and coloured text keep the box straight.

use fun::cli::{Arg, Cli};
use fun::i18n::{common, Lang};
use fun::term::{self, Depth, Rgb};
use fun::text::{expand_tabs, pad, width, Align};

struct Style {
    name: &'static str,
    /// top-left, top, top-right, left, right, bottom-left, bottom, bottom-right
    parts: [&'static str; 8],
}

const STYLES: &[Style] = &[
    Style { name: "round", parts: ["╭", "─", "╮", "│", "│", "╰", "─", "╯"] },
    Style { name: "single", parts: ["┌", "─", "┐", "│", "│", "└", "─", "┘"] },
    Style { name: "double", parts: ["╔", "═", "╗", "║", "║", "╚", "═", "╝"] },
    Style { name: "bold", parts: ["┏", "━", "┓", "┃", "┃", "┗", "━", "┛"] },
    Style { name: "dashed", parts: ["┌", "╌", "┐", "╎", "╎", "└", "╌", "┘"] },
    Style { name: "block", parts: ["▛", "▀", "▜", "▌", "▐", "▙", "▄", "▟"] },
    Style { name: "shadow", parts: ["┌", "─", "┐", "│", "│▒", "└", "─", "┘▒"] },
    Style { name: "classic", parts: ["+", "-", "+", "|", "|", "+", "-", "+"] },
    Style { name: "stars", parts: ["*", "*", "*", "*", "*", "*", "*", "*"] },
    Style { name: "hash", parts: ["#", "#", "#", "#", "#", "#", "#", "#"] },
    Style { name: "markdown", parts: ["```", "", "", "", "", "```", "", ""] },
];

const COLORS: &[(&str, Rgb)] = &[
    ("red", Rgb(231, 72, 86)),
    ("green", Rgb(22, 198, 12)),
    ("yellow", Rgb(249, 241, 165)),
    ("blue", Rgb(59, 120, 255)),
    ("magenta", Rgb(180, 0, 158)),
    ("cyan", Rgb(97, 214, 214)),
    ("white", Rgb(242, 242, 242)),
    ("orange", Rgb(255, 135, 0)),
    ("rainbow", Rgb(0, 0, 0)),
];

struct Texts {
    help: &'static str,
    styles: &'static str,
    unknown_style: &'static str,
    unknown_color: &'static str,
    default_text: &'static str,
}

const EN: Texts = Texts {
    help: "\
Usage: rboxes [options] [style] [file...]

Draws a box around the text of the files or of stdin.

  -s, --style <name>   Box style (see --list; default round)
  -l, --list           List the styles
  -a, --align <l|c|r>  Align the text: left, centre, right (default left)
  -p, --padding <n>    Spaces between the text and the sides (default 1)
  -P, --vpadding <n>   Empty lines above and below the text (default 0)
  -w, --width <n>      Minimum width inside the box
  -t, --title <text>   Title on the top border
  -c, --color <name>   Border colour: red, green, yellow, blue, magenta, cyan,
                       white, orange, rainbow (terminal only, or with --force)
  -f, --force          Colour even when the output is not a terminal
      --lang <code>    Language: en, pt, es (also: RTOOLS_LANG)
  -h, --help           Show this help
  -V, --version        Show the version
",
    styles: "Styles:",
    unknown_style: "unknown style",
    unknown_color: "unknown colour",
    default_text: "Hello from rboxes!",
};

const PT: Texts = Texts {
    help: "\
Utilização: rboxes [opções] [estilo] [ficheiro...]

Desenha uma caixa à volta do texto dos ficheiros ou do stdin.

  -s, --style <nome>   Estilo da caixa (ver --list; por omissão round)
  -l, --list           Mostrar os estilos
  -a, --align <l|c|r>  Alinhar o texto: esquerda, centro, direita (por omissão esquerda)
  -p, --padding <n>    Espaços entre o texto e os lados (por omissão 1)
  -P, --vpadding <n>   Linhas vazias acima e abaixo do texto (por omissão 0)
  -w, --width <n>      Largura mínima dentro da caixa
  -t, --title <texto>  Título na margem de cima
  -c, --color <nome>   Cor da moldura: red, green, yellow, blue, magenta, cyan,
                       white, orange, rainbow (só num terminal, ou com --force)
  -f, --force          Colorir mesmo quando a saída não é um terminal
      --lang <código>  Idioma: en, pt, es (também: RTOOLS_LANG)
  -h, --help           Mostrar esta ajuda
  -V, --version        Mostrar a versão
",
    styles: "Estilos:",
    unknown_style: "estilo desconhecido",
    unknown_color: "cor desconhecida",
    default_text: "Olá do rboxes!",
};

const ES: Texts = Texts {
    help: "\
Uso: rboxes [opciones] [estilo] [fichero...]

Dibuja una caja alrededor del texto de los ficheros o de stdin.

  -s, --style <nombre> Estilo de la caja (ver --list; por defecto round)
  -l, --list           Mostrar los estilos
  -a, --align <l|c|r>  Alinear el texto: izquierda, centro, derecha (por defecto izquierda)
  -p, --padding <n>    Espacios entre el texto y los lados (por defecto 1)
  -P, --vpadding <n>   Líneas vacías encima y debajo del texto (por defecto 0)
  -w, --width <n>      Anchura mínima dentro de la caja
  -t, --title <texto>  Título en el borde superior
  -c, --color <nombre> Color del borde: red, green, yellow, blue, magenta, cyan,
                       white, orange, rainbow (solo en un terminal, o con --force)
  -f, --force          Colorear aunque la salida no sea un terminal
      --lang <código>  Idioma: en, pt, es (también: RTOOLS_LANG)
  -h, --help           Mostrar esta ayuda
  -V, --version        Mostrar la versión
",
    styles: "Estilos:",
    unknown_style: "estilo desconocido",
    unknown_color: "color desconocido",
    default_text: "¡Hola desde rboxes!",
};

struct Opts {
    align: Align,
    padding: usize,
    vpadding: usize,
    min_width: usize,
    title: Option<String>,
}

/// Paints border pieces; `n` counts the pieces so the rainbow can move along.
struct Painter {
    depth: Depth,
    color: Option<Rgb>,
    rainbow: bool,
    n: usize,
}

impl Painter {
    fn paint(&mut self, s: &str) -> String {
        if s.is_empty() || self.depth == Depth::None || (self.color.is_none() && !self.rainbow) {
            return s.to_string();
        }
        self.n += 1;
        let c = if self.rainbow { term::rainbow(0.1, self.n as f64 * 1.5) } else { self.color.unwrap() };
        format!("{}{s}{}", term::fg(self.depth, c), term::RESET)
    }
}

fn boxed(text: &str, style: &Style, o: &Opts, p: &mut Painter) -> String {
    let lines: Vec<String> = text.lines().map(|l| expand_tabs(l, 8)).collect();
    let [tl, t, tr, l, r, bl, b, br] = style.parts;
    let mut out = String::new();
    if style.name == "markdown" {
        out.push_str(&format!("{}\n", p.paint(tl)));
        for line in &lines {
            out.push_str(&format!("{line}\n"));
        }
        out.push_str(&format!("{}\n", p.paint(bl)));
        return out;
    }
    let title = o.title.as_deref().map(|s| format!(" {s} "));
    let title_w = title.as_deref().map_or(0, |s| width(s) + 2);
    let text_w = lines.iter().map(|s| width(s)).max().unwrap_or(0).max(o.min_width);
    let inner = (text_w + 2 * o.padding).max(title_w);
    let text_w = inner - 2 * o.padding;
    let side_pad = " ".repeat(o.padding);
    // A piece may be wider than one column (e.g. "```"): repeat it to fill `n` columns.
    let fill = |piece: &str, n: usize| piece.repeat(n / width(piece).max(1));

    let mut top = p.paint(tl);
    match &title {
        Some(ts) => {
            top.push_str(&p.paint(t));
            top.push_str(ts);
            top.push_str(&p.paint(&fill(t, inner - width(ts) - 1)));
        }
        None => top.push_str(&p.paint(&fill(t, inner))),
    }
    top.push_str(&p.paint(tr));
    out.push_str(&top);
    out.push('\n');

    let blank = " ".repeat(inner);
    let rows = std::iter::repeat_n(None, o.vpadding)
        .chain(lines.iter().map(Some))
        .chain(std::iter::repeat_n(None, o.vpadding));
    for row in rows {
        let body = match row {
            Some(line) => format!("{side_pad}{}{side_pad}", pad(line, text_w, o.align)),
            None => blank.clone(),
        };
        // Reset after the text, in case it carries its own colours.
        let reset = if row.is_some_and(|s| s.contains('\x1b')) { term::RESET } else { "" };
        out.push_str(&format!("{}{body}{reset}{}\n", p.paint(l), p.paint(r)));
    }
    let shadow_w = if style.name == "shadow" { 1 } else { 0 };
    out.push_str(&format!("{}{}{}\n", p.paint(bl), p.paint(&fill(b, inner)), p.paint(br)));
    if shadow_w == 1 {
        out.push_str(&format!(" {}\n", p.paint(&"▒".repeat(inner + 2))));
    }
    out
}

fn main() {
    let mut cli = Cli::new("rboxes");
    let t = Lang::pick(cli.lang, &EN, &PT, &ES);
    let mut o = Opts { align: Align::Left, padding: 1, vpadding: 0, min_width: 0, title: None };
    let mut style_name: Option<String> = None;
    let mut color: Option<String> = None;
    let mut force = false;
    let mut files = Vec::new();
    while let Some(a) = cli.next() {
        match a {
            Arg::Pos(p) => {
                // The style may be given as the first word, as in `rboxes double`.
                if style_name.is_none() && files.is_empty() && STYLES.iter().any(|s| s.name == p) {
                    style_name = Some(p);
                } else {
                    files.push(p);
                }
            }
            Arg::Opt(opt) => match opt.as_str() {
                "-h" | "--help" => cli.help(t.help),
                "-V" | "--version" => cli.version(),
                "-l" | "--list" => {
                    println!("{}", t.styles);
                    let o = Opts { align: Align::Left, padding: 1, vpadding: 0, min_width: 0, title: None };
                    let mut p = Painter { depth: Depth::None, color: None, rainbow: false, n: 0 };
                    for s in STYLES {
                        print!("{}", boxed(s.name, s, &o, &mut p));
                    }
                    return;
                }
                "-s" | "--style" => style_name = Some(cli.value(&opt)),
                "-a" | "--align" => {
                    let v = cli.value(&opt);
                    o.align = match v.to_ascii_lowercase().as_str() {
                        "l" | "left" => Align::Left,
                        "c" | "center" | "centre" => Align::Center,
                        "r" | "right" => Align::Right,
                        _ => cli.fail(&format!("{} '{opt}': {v}", common(cli.lang).invalid_value)),
                    }
                }
                "-p" | "--padding" => o.padding = cli.number(&opt, 0..=100),
                "-P" | "--vpadding" => o.vpadding = cli.number(&opt, 0..=100),
                "-w" | "--width" => o.min_width = cli.number(&opt, 0..=1000),
                "-t" | "--title" => o.title = Some(cli.value(&opt)),
                "-c" | "--color" => color = Some(cli.value(&opt).to_ascii_lowercase()),
                "-f" | "--force" => force = true,
                _ => cli.unknown(&opt),
            },
        }
    }
    let name = style_name.unwrap_or_else(|| "round".into()).to_ascii_lowercase();
    let style = STYLES
        .iter()
        .find(|s| s.name == name)
        .unwrap_or_else(|| cli.fail(&format!("{} '{name}' (--list)", t.unknown_style)));
    let rgb = match color.as_deref() {
        None => None,
        Some(c) => Some(
            COLORS
                .iter()
                .find(|(n, _)| *n == c)
                .map(|(_, v)| *v)
                .unwrap_or_else(|| cli.fail(&format!("{} '{c}'", t.unknown_color))),
        ),
    };
    let rainbow = color.as_deref() == Some("rainbow");
    let depth = if rgb.is_some() { term::depth(force.then_some(true)) } else { Depth::None };

    let mut text = String::new();
    if files.is_empty() {
        text = fun::input::piped_stdin().unwrap_or_default();
    } else {
        for f in &files {
            match fun::input::read_all(f) {
                Ok(s) => text.push_str(&s),
                Err(e) => fun::cli::die("rboxes", &format!("{} '{f}': {e}", common(cli.lang).cannot_read)),
            }
        }
    }
    let text = text.trim_end_matches('\n');
    let text = if text.trim().is_empty() { t.default_text } else { text };
    let mut p = Painter { depth, color: rgb.filter(|_| !rainbow), rainbow, n: 0 };
    use std::io::Write;
    let _ = std::io::stdout().write_all(boxed(text, style, &o, &mut p).as_bytes());
}

#[cfg(test)]
mod tests {
    use super::*;

    fn plain() -> Painter {
        Painter { depth: Depth::None, color: None, rainbow: false, n: 0 }
    }

    #[test]
    fn straight_with_wide_characters_and_colours() {
        let o = Opts { align: Align::Left, padding: 1, vpadding: 0, min_width: 0, title: Some("T".into()) };
        let s = boxed("Olá\n日本 😀\n\x1b[31mred\x1b[0m", &STYLES[0], &o, &mut plain());
        let widths: Vec<usize> = s.lines().map(width).collect();
        assert!(widths.iter().all(|&w| w == widths[0]), "{s}");
        assert!(s.starts_with("╭─ T ─"));
    }

    #[test]
    fn every_style_is_straight() {
        let o = Opts { align: Align::Center, padding: 2, vpadding: 1, min_width: 5, title: None };
        for st in STYLES.iter().filter(|s| s.name != "markdown" && s.name != "shadow") {
            let s = boxed("a\nbbb", st, &o, &mut plain());
            let widths: Vec<usize> = s.lines().map(width).collect();
            assert!(widths.iter().all(|&w| w == widths[0]), "{}: {s}", st.name);
        }
    }
}

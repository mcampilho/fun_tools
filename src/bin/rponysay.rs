//! rponysay: a pixel-art pony says (or thinks) a message (like ponysay), with
//! original ponies drawn by the program.

use fun::cli::{Arg, Cli};
use fun::cow::balloon;
use fun::i18n::Lang;
use fun::pix::{mix, scale, Pixels};
use fun::rng::Rng;
use fun::term::{self, Depth, Rgb};
use fun::text::{expand_tabs, wrap_text};
use std::io::Write;

struct Texts {
    help: &'static str,
    coats: &'static str,
    kinds: &'static str,
    unknown: &'static str,
    default_say: &'static str,
    default_think: &'static str,
}

const EN: Texts = Texts {
    help: "\
Usage: rponysay [options] [message...]
       command | rponysay [options]

A pixel-art pony says the message (or thinks it, with -t). The ponies are
drawn with half blocks: colours need a terminal (or --color).

  -t, --think          Think instead of saying
  -f, --coat <name>    Colours of the pony (see -l; default: random)
  -k, --kind <name>    pony, unicorn or pegasus (default: random)
  -l, --list           List the coats and kinds
  -W, --width <n>      Wrap the message at this width (default 40)
  -n, --nowrap         Do not wrap the message
  -s, --seed <n>       Seed for the random choices
      --color          Always use colours
      --no-color       Never use colours (also: NO_COLOR)
      --lang <code>    Language: en, pt, es (also: RTOOLS_LANG)
  -h, --help           Show this help
  -V, --version        Show the version
",
    coats: "Coats:",
    kinds: "Kinds:",
    unknown: "unknown pony",
    default_say: "Neigh! Rust says hello.",
    default_think: "Hay or oats... or a new crate?",
};

const PT: Texts = Texts {
    help: "\
Utilização: rponysay [opções] [mensagem...]
            comando | rponysay [opções]

Um pónei em pixel art diz a mensagem (ou pensa-a, com -t). Os póneis são
desenhados com meios blocos: as cores precisam de um terminal (ou --color).

  -t, --think          Pensar em vez de dizer
  -f, --coat <nome>    Cores do pónei (ver -l; por omissão: ao acaso)
  -k, --kind <nome>    pony (pónei), unicorn (unicórnio) ou pegasus (pégaso)
                       (por omissão: ao acaso)
  -l, --list           Mostrar as pelagens e os tipos
  -W, --width <n>      Quebrar a mensagem nesta largura (por omissão 40)
  -n, --nowrap         Não quebrar a mensagem
  -s, --seed <n>       Semente para as escolhas ao acaso
      --color          Usar sempre cores
      --no-color       Nunca usar cores (também: NO_COLOR)
      --lang <código>  Idioma: en, pt, es (também: RTOOLS_LANG)
  -h, --help           Mostrar esta ajuda
  -V, --version        Mostrar a versão
",
    coats: "Pelagens:",
    kinds: "Tipos:",
    unknown: "pónei desconhecido",
    default_say: "Iiih! O Rust diz olá.",
    default_think: "Feno ou aveia... ou uma crate nova?",
};

const ES: Texts = Texts {
    help: "\
Uso: rponysay [opciones] [mensaje...]
     comando | rponysay [opciones]

Un poni en pixel art dice el mensaje (o lo piensa, con -t). Los ponis se
dibujan con medios bloques: los colores necesitan un terminal (o --color).

  -t, --think          Pensar en vez de decir
  -f, --coat <nombre>  Colores del poni (ver -l; por defecto: al azar)
  -k, --kind <nombre>  pony (poni), unicorn (unicornio) o pegasus (pegaso)
                       (por defecto: al azar)
  -l, --list           Mostrar los pelajes y los tipos
  -W, --width <n>      Partir el mensaje a esta anchura (por defecto 40)
  -n, --nowrap         No partir el mensaje
  -s, --seed <n>       Semilla para las elecciones al azar
      --color          Usar siempre colores
      --no-color       No usar nunca colores (también: NO_COLOR)
      --lang <código>  Idioma: en, pt, es (también: RTOOLS_LANG)
  -h, --help           Mostrar esta ayuda
  -V, --version        Mostrar la versión
",
    coats: "Pelajes:",
    kinds: "Tipos:",
    unknown: "poni desconocido",
    default_say: "¡Iiih! Rust te saluda.",
    default_think: "Heno o avena... ¿o un crate nuevo?",
};

/// A coat: body, mane, mane streak, eyes.
struct Coat {
    name: &'static str,
    body: Rgb,
    mane: Rgb,
    streak: Rgb,
    eye: Rgb,
}

const COATS: &[Coat] = &[
    Coat { name: "coral", body: Rgb(250, 170, 150), mane: Rgb(120, 60, 170), streak: Rgb(250, 230, 120), eye: Rgb(60, 150, 90) },
    Coat { name: "lilac", body: Rgb(205, 175, 235), mane: Rgb(45, 70, 150), streak: Rgb(235, 90, 160), eye: Rgb(120, 60, 170) },
    Coat { name: "azure", body: Rgb(150, 205, 245), mane: Rgb(240, 240, 250), streak: Rgb(110, 170, 240), eye: Rgb(200, 80, 60) },
    Coat { name: "lemon", body: Rgb(250, 235, 140), mane: Rgb(240, 120, 60), streak: Rgb(200, 50, 60), eye: Rgb(60, 110, 200) },
    Coat { name: "jade", body: Rgb(150, 220, 180), mane: Rgb(30, 110, 110), streak: Rgb(245, 200, 80), eye: Rgb(150, 90, 40) },
    Coat { name: "plum", body: Rgb(150, 90, 150), mane: Rgb(250, 200, 230), streak: Rgb(120, 220, 240), eye: Rgb(240, 180, 60) },
    Coat { name: "snow", body: Rgb(245, 245, 248), mane: Rgb(90, 160, 235), streak: Rgb(170, 110, 230), eye: Rgb(60, 140, 200) },
    Coat { name: "cocoa", body: Rgb(170, 120, 85), mane: Rgb(250, 230, 200), streak: Rgb(230, 120, 90), eye: Rgb(70, 160, 120) },
];

const KINDS: &[&str] = &["pony", "unicorn", "pegasus"];

const KEY: Rgb = Rgb(30, 22, 30);

/// A filled ellipse, lit from the top left.
fn blob(p: &mut Pixels, cx: f64, cy: f64, rx: f64, ry: f64, c: Rgb) {
    for y in (cy - ry).floor() as i64..=(cy + ry).ceil() as i64 {
        for x in (cx - rx).floor() as i64..=(cx + rx).ceil() as i64 {
            let (dx, dy) = ((x as f64 + 0.5 - cx) / rx, (y as f64 + 0.5 - cy) / ry);
            let d = dx * dx + dy * dy;
            if d <= 1.0 {
                let light = 1.08 - (dx + dy) * 0.08 - d * 0.1;
                p.put(x, y, scale(c, light));
            }
        }
    }
}

fn hair(p: &mut Pixels, pts: &[(f64, f64, f64)], coat: &Coat) {
    for (i, &(x, y, r)) in pts.iter().enumerate() {
        blob(p, x, y, r, r, coat.mane);
        if i % 2 == 1 {
            blob(p, x + 0.3, y + 0.5, r * 0.45, r * 0.55, coat.streak);
        }
    }
}

/// Draws a pony facing left in a 36 × 36 pixel box.
fn pony(coat: &Coat, kind: &str) -> Pixels {
    let mut p = Pixels::new(36, 18);
    let body = coat.body;
    let shade = scale(body, 0.78);
    // Tail, behind everything.
    hair(&mut p, &[(28.5, 19.0, 2.4), (30.5, 21.0, 2.6), (31.5, 24.0, 2.5), (31.0, 27.0, 2.3), (29.5, 29.5, 2.0), (31.8, 30.5, 1.6)], coat);
    // Far legs, darker.
    for x in [16.0, 25.0] {
        for y in 24..33 {
            for dx in 0..3 {
                p.put(x as i64 + dx, y, scale(shade, 0.9));
            }
        }
        for dx in 0..3 {
            p.put(x as i64 + dx, 33, scale(KEY, 1.8));
        }
    }
    // Body and near legs.
    blob(&mut p, 21.5, 22.0, 9.0, 5.5, body);
    for x in [13.0, 22.0] {
        for y in 25..33 {
            for dx in 0..3 {
                p.put(x as i64 + dx, y, if dx == 2 { shade } else { body });
            }
        }
        for dx in 0..3 {
            p.put(x as i64 + dx, 33, scale(KEY, 2.5));
        }
    }
    // Neck and head.
    for (x, y, r) in [(16.0, 19.0, 3.6), (14.5, 16.5, 3.4), (13.0, 14.0, 3.3)] {
        blob(&mut p, x, y, r, r, body);
    }
    blob(&mut p, 9.5, 11.0, 6.2, 5.2, body);
    blob(&mut p, 4.8, 13.2, 3.6, 3.0, mix(body, Rgb(255, 255, 255), 0.15));
    // Ear.
    for (j, w) in [(0, 1), (1, 2), (2, 2), (3, 3)] {
        for i in 0..w {
            p.put(12 + i, 3 + j, if i == 0 && j > 0 { scale(body, 0.85) } else { body });
        }
    }
    p.put(13, 6, mix(body, Rgb(255, 150, 170), 0.5));
    // Mane: a forelock and locks down the neck.
    hair(&mut p, &[(13.5, 6.0, 2.6), (15.5, 8.5, 2.6), (17.0, 11.5, 2.5), (18.0, 14.5, 2.3), (18.6, 17.3, 2.0), (10.5, 6.0, 2.2), (8.8, 6.6, 1.7)], coat);
    // Eye: white, iris, pupil and a highlight.
    blob(&mut p, 7.6, 10.4, 1.8, 2.3, Rgb(250, 250, 250));
    blob(&mut p, 7.0, 10.9, 1.3, 1.7, coat.eye);
    p.put(7, 11, KEY);
    p.put(6, 10, Rgb(255, 255, 255));
    p.put(9, 8, KEY);
    p.put(8, 8, KEY);
    // Muzzle: nostril, smile and a blush.
    p.put(2, 13, scale(body, 0.55));
    p.put(4, 15, scale(body, 0.55));
    p.put(5, 15, scale(body, 0.55));
    p.put(6, 14, scale(body, 0.55));
    p.put(9, 13, mix(body, Rgb(255, 120, 150), 0.45));
    p.put(10, 13, mix(body, Rgb(255, 120, 150), 0.45));
    match kind {
        "unicorn" => {
            // A spiral horn from the forehead.
            for i in 0..6 {
                let (x, y) = (9 - i / 2, 5 - i);
                let c = if i % 2 == 0 { coat.streak } else { mix(coat.streak, Rgb(255, 255, 255), 0.5) };
                p.put(x, y, c);
                if i < 3 {
                    p.put(x + 1, y, scale(c, 0.85));
                }
            }
        }
        "pegasus" => {
            // A folded wing on the side, with feathers.
            let wing = mix(body, Rgb(255, 255, 255), 0.25);
            blob(&mut p, 22.0, 19.5, 5.5, 2.6, wing);
            for (i, x) in [24.0, 26.0, 28.0].iter().enumerate() {
                blob(&mut p, *x, 18.0 - i as f64 * 0.8, 1.6, 1.6, wing);
            }
            for x in [19, 22, 25] {
                p.put(x, 20, scale(wing, 0.82));
                p.put(x + 1, 21, scale(wing, 0.82));
            }
        }
        _ => {}
    }
    p
}

fn main() {
    let mut cli = Cli::new("rponysay");
    let t = Lang::pick(cli.lang, &EN, &PT, &ES);
    let (mut think, mut coat, mut kind, mut width, mut wrap, mut seed, mut color) = (false, None::<String>, None::<String>, 40usize, true, None::<u64>, None::<bool>);
    let mut words = Vec::new();
    while let Some(a) = cli.next() {
        match a {
            Arg::Pos(p) => words.push(p),
            Arg::Opt(opt) => match opt.as_str() {
                "-h" | "--help" => cli.help(t.help),
                "-V" | "--version" => cli.version(),
                "-t" | "--think" => think = true,
                "-f" | "--coat" => coat = Some(cli.value(&opt).to_ascii_lowercase()),
                "-k" | "--kind" => kind = Some(cli.value(&opt).to_ascii_lowercase()),
                "-W" | "--width" => width = cli.number(&opt, 1..=1000),
                "-n" | "--nowrap" => wrap = false,
                "-s" | "--seed" => seed = Some(cli.parse(&opt)),
                "--color" => color = Some(true),
                "--no-color" => color = Some(false),
                "-l" | "--list" => {
                    let names: Vec<_> = COATS.iter().map(|c| c.name).collect();
                    println!("{}\n  {}\n{}\n  {}", t.coats, names.join("\n  "), t.kinds, KINDS.join("\n  "));
                    return;
                }
                _ => cli.unknown(&opt),
            },
        }
    }
    let mut rng = seed.map_or_else(Rng::new, Rng::seeded);
    let coat = match coat {
        None => rng.pick(COATS),
        Some(n) => COATS.iter().find(|c| c.name == n).unwrap_or_else(|| cli.fail(&format!("{} '{n}'", t.unknown))),
    };
    let kind = match kind {
        None => *rng.pick(KINDS),
        Some(k) => *KINDS.iter().find(|n| **n == k).unwrap_or_else(|| cli.fail(&format!("{} '{k}'", t.unknown))),
    };
    let message = if !words.is_empty() {
        words.join(" ")
    } else {
        fun::input::piped_stdin()
            .filter(|s| !s.trim().is_empty())
            .unwrap_or_else(|| (if think { t.default_think } else { t.default_say }).to_string())
    };
    let depth = term::depth(color);
    let _ = std::io::stdout().write_all(render(&message, think, width, wrap, coat, kind, depth).as_bytes());
}

fn render(message: &str, think: bool, width: usize, wrap: bool, coat: &Coat, kind: &str, depth: Depth) -> String {
    let message = expand_tabs(message.trim_end(), 8);
    let lines: Vec<String> = if wrap { wrap_text(&message, width) } else { message.lines().map(str::to_string).collect() };
    let lines = if lines.is_empty() { vec![String::new()] } else { lines };
    let mut out = balloon(&lines, think);
    // The link from the balloon down to the pony's head.
    let link = if think { "o" } else { "\\" };
    for i in 0..2 {
        out.push_str(&format!("{}{link}\n", " ".repeat(4 + i)));
    }
    for l in pony(coat, kind).to_lines(depth) {
        out.push_str("     ");
        out.push_str(&l);
        out.push('\n');
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_pony_renders() {
        for c in COATS {
            for k in KINDS {
                let s = render("Olá, mundo!", false, 40, true, c, k, Depth::None);
                assert!(s.starts_with(" _____________\n< Olá, mundo! >\n"), "{s}");
                assert!(s.lines().count() > 15);
            }
        }
        assert!(render("hm", true, 40, true, &COATS[0], "pony", Depth::None).contains("    o\n"));
    }

    /// `cargo test --bin rponysay snapshot -- --ignored --nocapture`
    #[test]
    #[ignore]
    fn snapshot() {
        let only: Option<usize> = std::env::var("SNAP_I").ok().and_then(|v| v.parse().ok());
        for (i, k) in KINDS.iter().enumerate().filter(|(i, _)| only.is_none_or(|o| o == *i)) {
            print!("{}", render("Olá! Sou um pónei original.", i == 2, 40, true, &COATS[i * 2], k, Depth::True));
        }
    }
}

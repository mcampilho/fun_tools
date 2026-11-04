//! rmatrix: falling "digital rain" (like cmatrix). q, Esc or Ctrl+C to quit.

use fun::cli::{die, Arg, Cli};
use fun::i18n::{common, Lang};
use fun::rng::Rng;
use fun::term::{self, Depth, Rgb};
use fun::tui::{self, Canvas, Input, Screen};
use std::io::IsTerminal;
use std::time::{Duration, Instant};

struct Texts {
    help: &'static str,
    unknown_color: &'static str,
}

const EN: Texts = Texts {
    help: "\
Usage: rmatrix [options]

Falling digital rain. Press q, Esc or Ctrl+C to quit;
1 to 9 change the speed while it runs.

  -s, --speed <n>      Speed, 1 (slow) to 10 (fast) (default 5)
  -c, --color <name>   green, red, blue, yellow, cyan, magenta, white, orange, rainbow
                       (default green)
  -k, --katakana       Use half-width katakana too (needs a font that has them)
  -d, --density <n>    How many columns rain at once, 1 to 10 (default 6)
  -b, --bold           Bold characters
      --lang <code>    Language: en, pt, es (also: RTOOLS_LANG)
  -h, --help           Show this help
  -V, --version        Show the version
",
    unknown_color: "unknown colour",
};

const PT: Texts = Texts {
    help: "\
Utilização: rmatrix [opções]

Chuva digital a cair. Prima q, Esc ou Ctrl+C para sair;
de 1 a 9 muda a velocidade enquanto corre.

  -s, --speed <n>      Velocidade, de 1 (lenta) a 10 (rápida) (por omissão 5)
  -c, --color <nome>   green, red, blue, yellow, cyan, magenta, white, orange, rainbow
                       (por omissão green)
  -k, --katakana       Usar também katakana de meia largura (precisa de uma fonte que os tenha)
  -d, --density <n>    Quantas colunas chovem ao mesmo tempo, de 1 a 10 (por omissão 6)
  -b, --bold           Caracteres a negrito
      --lang <código>  Idioma: en, pt, es (também: RTOOLS_LANG)
  -h, --help           Mostrar esta ajuda
  -V, --version        Mostrar a versão
",
    unknown_color: "cor desconhecida",
};

const ES: Texts = Texts {
    help: "\
Uso: rmatrix [opciones]

Lluvia digital cayendo. Pulse q, Esc o Ctrl+C para salir;
de 1 a 9 cambia la velocidad mientras funciona.

  -s, --speed <n>      Velocidad, de 1 (lenta) a 10 (rápida) (por defecto 5)
  -c, --color <nombre> green, red, blue, yellow, cyan, magenta, white, orange, rainbow
                       (por defecto green)
  -k, --katakana       Usar también katakana de media anchura (necesita una fuente que los tenga)
  -d, --density <n>    Cuántas columnas llueven a la vez, de 1 a 10 (por defecto 6)
  -b, --bold           Caracteres en negrita
      --lang <código>  Idioma: en, pt, es (también: RTOOLS_LANG)
  -h, --help           Mostrar esta ayuda
  -V, --version        Mostrar la versión
",
    unknown_color: "color desconocido",
};

/// (name, bright trail colour, dark trail colour)
const COLORS: &[(&str, Rgb, Rgb)] = &[
    ("green", Rgb(90, 255, 90), Rgb(0, 130, 30)),
    ("red", Rgb(255, 90, 90), Rgb(140, 0, 0)),
    ("blue", Rgb(110, 160, 255), Rgb(20, 50, 160)),
    ("yellow", Rgb(255, 240, 100), Rgb(150, 130, 0)),
    ("cyan", Rgb(100, 255, 255), Rgb(0, 130, 140)),
    ("magenta", Rgb(255, 110, 255), Rgb(130, 0, 130)),
    ("white", Rgb(240, 240, 240), Rgb(120, 120, 120)),
    ("orange", Rgb(255, 170, 60), Rgb(160, 80, 0)),
    ("rainbow", Rgb(0, 0, 0), Rgb(0, 0, 0)),
];

const ASCII: &str = "0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz@#$%&*+=<>?/\\|~:";

struct Drop {
    /// Row of the head (may be above the screen while waiting).
    head: f64,
    speed: f64,
    len: i64,
    active: bool,
}

struct Rain {
    w: usize,
    h: usize,
    chars: Vec<char>,
    glyphs: Vec<char>,
    drops: Vec<Drop>,
    density: f64,
    rng: Rng,
}

impl Rain {
    fn new(w: usize, h: usize, glyphs: Vec<char>, density: f64, mut rng: Rng) -> Rain {
        let chars = (0..w * h).map(|_| *rng.pick(&glyphs)).collect();
        let mut r = Rain { w, h, chars, glyphs, drops: Vec::new(), density, rng };
        r.drops = (0..w).map(|_| Drop { head: 0.0, speed: 1.0, len: 0, active: false }).collect();
        for x in 0..w {
            r.restart(x);
            // Start spread over the screen, not all at the top.
            r.drops[x].head = r.rng.range(-(h as i64), h as i64) as f64;
        }
        r
    }

    fn restart(&mut self, x: usize) {
        let h = self.h as i64;
        let d = &mut self.drops[x];
        d.active = self.rng.chance(self.density);
        d.head = -(self.rng.range(0, h) as f64);
        d.speed = 0.35 + self.rng.float() * 0.75;
        d.len = self.rng.range(4, (h * 2 / 3).max(6));
    }

    fn step(&mut self, factor: f64) {
        for x in 0..self.w {
            let d = &mut self.drops[x];
            d.head += d.speed * factor;
            if d.head as i64 - d.len > self.h as i64 {
                self.restart(x);
            }
        }
        // A few characters change in place, as on an old screen.
        for _ in 0..(self.w * self.h / 60).max(1) {
            let i = self.rng.below(self.chars.len());
            self.chars[i] = *self.rng.pick(&self.glyphs);
        }
    }

    fn draw(&self, c: &mut Canvas, bright: Rgb, dark: Rgb, rainbow: bool, bold: bool) {
        c.clear();
        for (x, d) in self.drops.iter().enumerate() {
            if !d.active {
                continue;
            }
            let head = d.head as i64;
            let (b, k) = if rainbow {
                let base = term::rainbow(0.08, x as f64 * 1.2);
                (base, Rgb(base.0 / 2, base.1 / 2, base.2 / 2))
            } else {
                (bright, dark)
            };
            for i in 0..=d.len {
                let y = head - i;
                if y < 0 || y >= self.h as i64 {
                    continue;
                }
                let ch = self.chars[y as usize * self.w + x];
                let color = if i == 0 {
                    Rgb(235, 255, 235)
                } else if i < 4 {
                    b
                } else {
                    k
                };
                c.set(x as i64, y, ch, Some(color), bold || i == 0);
            }
        }
    }
}

fn main() {
    let mut cli = Cli::new("rmatrix");
    let t = Lang::pick(cli.lang, &EN, &PT, &ES);
    let (mut speed, mut color, mut katakana, mut density, mut bold) = (5u32, "green".to_string(), false, 6u32, false);
    while let Some(a) = cli.next() {
        match a {
            Arg::Pos(p) => cli.unknown(&p),
            Arg::Opt(opt) => match opt.as_str() {
                "-h" | "--help" => cli.help(t.help),
                "-V" | "--version" => cli.version(),
                "-s" | "--speed" => speed = cli.number(&opt, 1..=10),
                "-c" | "--color" => color = cli.value(&opt).to_ascii_lowercase(),
                "-k" | "--katakana" => katakana = true,
                "-d" | "--density" => density = cli.number(&opt, 1..=10),
                "-b" | "--bold" => bold = true,
                _ => cli.unknown(&opt),
            },
        }
    }
    let Some(&(_, bright, dark)) = COLORS.iter().find(|(n, ..)| *n == color) else {
        cli.fail(&format!("{} '{color}'", t.unknown_color));
    };
    let rainbow = color == "rainbow";
    if !std::io::stdout().is_terminal() {
        die("rmatrix", common(cli.lang).not_a_terminal);
    }
    let mut glyphs: Vec<char> = ASCII.chars().collect();
    if katakana {
        glyphs.extend((0xFF66..=0xFF9D).filter_map(char::from_u32));
    }
    // Colour depth for a terminal: truecolor, 256 or 16 (NO_COLOR is not honoured
    // here, an uncoloured rain would make no sense).
    let depth = term::depth(Some(true)).max(Depth::Ansi16);
    if let Err(e) = run(speed, &glyphs, density as f64 / 10.0, bright, dark, rainbow, bold, depth) {
        die("rmatrix", &e.to_string());
    }
}

#[allow(clippy::too_many_arguments)]
fn run(
    mut speed: u32,
    glyphs: &[char],
    density: f64,
    bright: Rgb,
    dark: Rgb,
    rainbow: bool,
    bold: bool,
    depth: Depth,
) -> std::io::Result<()> {
    let _screen = Screen::enter()?;
    let mut out = std::io::stdout().lock();
    let (w, h) = term::size();
    let mut canvas = Canvas::new(w, h);
    let mut rain = Rain::new(w, h, glyphs.to_vec(), density, Rng::new());
    let frame = Duration::from_millis(33);
    loop {
        let start = Instant::now();
        rain.step(0.25 + speed as f64 * 0.15);
        rain.draw(&mut canvas, bright, dark, rainbow, bold);
        canvas.flush(&mut out, depth)?;
        let wait = frame.saturating_sub(start.elapsed());
        match tui::poll(wait)? {
            Input::Quit => return Ok(()),
            Input::Resize => {
                let (w, h) = term::size();
                canvas.resize(w, h);
                rain = Rain::new(w, h, glyphs.to_vec(), density, Rng::new());
            }
            Input::Key(c @ '1'..='9') => speed = c as u32 - '0' as u32,
            Input::Key('0') => speed = 10,
            _ => {}
        }
        // Keep the frame rate even when a key arrives early.
        let left = frame.saturating_sub(start.elapsed());
        if !left.is_zero() {
            std::thread::sleep(left);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rain_stays_on_screen() {
        let mut r = Rain::new(30, 10, ASCII.chars().collect(), 0.6, Rng::seeded(1));
        let mut c = Canvas::new(30, 10);
        for _ in 0..200 {
            r.step(1.0);
            r.draw(&mut c, Rgb(0, 255, 0), Rgb(0, 100, 0), false, false);
        }
        assert!(r.drops.iter().all(|d| d.head as i64 - d.len <= 11));
    }
}

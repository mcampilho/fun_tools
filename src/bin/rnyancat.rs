//! rnyancat: a pixel-art cat with a pastry body flies through space on a
//! rainbow (like nyancat), with original art. q, Esc or Ctrl+C to quit.

use fun::cli::{die, Arg, Cli};
use fun::i18n::{common, Lang};
use fun::pix::Pixels;
use fun::rng::Rng;
use fun::term::{self, Rgb};
use fun::tui::{self, Canvas, Input, Screen};
use std::io::IsTerminal;
use std::time::{Duration, Instant};

struct Texts {
    help: &'static str,
    /// `{}` is the number of seconds.
    counter: &'static str,
}

const EN: Texts = Texts {
    help: "\
Usage: rnyancat [options]

A cat with a pastry body flies through space on a rainbow.
Drawn in colour pixel art with half blocks: needs a font with them (most have)
and looks best with 24-bit colour. Press q, Esc or Ctrl+C to quit.

  -d, --delay <ms>     Time per frame, 20 to 1000 ms (default 90)
  -f, --frames <n>     Stop after this many frames
  -n, --no-counter     Do not show how long it has been flying
  -s, --scale <n>      Size of the art pixels, 1 to 8 (default: fit the terminal)
      --lang <code>    Language: en, pt, es (also: RTOOLS_LANG)
  -h, --help           Show this help
  -V, --version        Show the version
",
    counter: "You have nyaned for {} seconds!",
};

const PT: Texts = Texts {
    help: "\
Utilização: rnyancat [opções]

Um gato com corpo de bolo voa pelo espaço num arco-íris.
Desenhado em pixel art a cores com meios blocos: precisa de uma fonte que os
tenha (quase todas) e fica melhor com cor de 24 bits. Prima q, Esc ou Ctrl+C
para sair.

  -d, --delay <ms>     Tempo de cada imagem, de 20 a 1000 ms (por omissão 90)
  -f, --frames <n>     Parar ao fim deste número de imagens
  -n, --no-counter     Não mostrar há quanto tempo está a voar
  -s, --scale <n>      Tamanho dos pixels do desenho, de 1 a 8 (por omissão: o
                       que couber no terminal)
      --lang <código>  Idioma: en, pt, es (também: RTOOLS_LANG)
  -h, --help           Mostrar esta ajuda
  -V, --version        Mostrar a versão
",
    counter: "Já nyanaste durante {} segundos!",
};

const ES: Texts = Texts {
    help: "\
Uso: rnyancat [opciones]

Un gato con cuerpo de pastel vuela por el espacio sobre un arcoíris.
Dibujado en pixel art en color con medios bloques: necesita una fuente que los
tenga (casi todas) y se ve mejor con color de 24 bits. Pulse q, Esc o Ctrl+C
para salir.

  -d, --delay <ms>     Tiempo de cada imagen, de 20 a 1000 ms (por defecto 90)
  -f, --frames <n>     Parar tras este número de imágenes
  -n, --no-counter     No mostrar cuánto tiempo lleva volando
  -s, --scale <n>      Tamaño de los píxeles del dibujo, de 1 a 8 (por defecto:
                       lo que quepa en el terminal)
      --lang <código>  Idioma: en, pt, es (también: RTOOLS_LANG)
  -h, --help           Mostrar esta ayuda
  -V, --version        Mostrar la versión
",
    counter: "¡Has nyaneado durante {} segundos!",
};

// The art: one character per pixel, `.` transparent.
const BODY: &[&str] = &[
    "..KKKKKKKKKKKKKKKKK..",
    ".KTTTTTTTTTTTTTTTTTK.",
    "KTTTTPPPPPPPPPPPTTTTK",
    "KTTPPPPPPPSPPPPPPPTTK",
    "KTPPSPPPPPPPPPPPSPPTK",
    "KTPPPPPPPPPPPPPPPPPTK",
    "KTPPPPPPPPPPMPPPPPPTK",
    "KTPPPPPMPPPPPPPPPPPTK",
    "KTPPPPPPPPPPPPPPSPPTK",
    "KTPPSPPPPPPPPPPPPPPTK",
    "KTPPPPPPPPPPPPPPPPPTK",
    "KTPPPPPPPSPPPPPPPPPTK",
    "KTPPPPPPPPPPPPMPPPPTK",
    "KTPPPMPPPPPPPPPPPPPTK",
    "KTTPPPPPPPPPPPPPPPTTK",
    "KTTTTPPPPPPPPPPPTTTTK",
    ".KTTTTTTTTTTTTTTTTTK.",
    "..KKKKKKKKKKKKKKKKK..",
];

const HEAD: &[&str] = &[
    "..KK........KK..",
    ".KGGK......KGGK.",
    ".KGRGK....KGRGK.",
    ".KGGGGKKKKGGGGK.",
    "KGGGGGGGGGGGGGGK",
    "KGGGWKGGGGGWKGGK",
    "KGGGKKGGGGGKKGGK",
    "KGCCGGGGKGGGGCCK",
    "KGCCGKGGKGGKGCCK",
    "KGGGGGKKKKKGGGGK",
    ".KGGGGGGGGGGGGK.",
    "..KKKKKKKKKKKK..",
];

const LEG: &[&str] = &["KGGK", "KGGK", "KGGK", ".KK."];

fn color(ch: char) -> Option<Rgb> {
    Some(match ch {
        'K' => Rgb(15, 15, 20),
        'T' => Rgb(250, 205, 150),
        'P' => Rgb(255, 155, 210),
        'S' => Rgb(235, 50, 150),
        'M' => Rgb(140, 70, 230),
        'G' => Rgb(158, 158, 165),
        'W' => Rgb(255, 255, 255),
        'C' => Rgb(255, 150, 155),
        'R' => Rgb(240, 150, 170),
        _ => return None,
    })
}

const SPACE: Rgb = Rgb(16, 38, 88);
const STAR: Rgb = Rgb(255, 255, 255);
const RAINBOW: [Rgb; 6] = [
    Rgb(255, 45, 45),
    Rgb(255, 155, 0),
    Rgb(255, 235, 0),
    Rgb(60, 225, 20),
    Rgb(10, 155, 255),
    Rgb(115, 65, 255),
];

/// Frames in one cycle of the animation.
const CYCLE: usize = 6;

struct Star {
    x: f64,
    y: i64,
    phase: usize,
}

/// One frame at art resolution (each art pixel becomes `scale` × `scale` pixels).
struct Scene {
    w: i64,
    h: i64,
    stars: Vec<Star>,
    rng: Rng,
}

impl Scene {
    fn new(w: i64, h: i64, rng: Rng) -> Scene {
        let mut s = Scene { w, h, stars: Vec::new(), rng };
        let n = (w * h / 160).max(4);
        for _ in 0..n {
            let star = Star { x: s.rng.range(0, w.max(1)) as f64, y: s.rng.range(0, h.max(1)), phase: s.rng.below(CYCLE) };
            s.stars.push(star);
        }
        s
    }

    fn step(&mut self) {
        for st in &mut self.stars {
            st.x -= 2.0;
            st.phase = (st.phase + 1) % CYCLE;
            if st.x < -4.0 {
                st.x = self.w as f64 + 3.0;
                st.y = self.rng.range(0, self.h.max(1));
            }
        }
    }

    fn draw(&self, p: &mut Pixels, frame: usize) {
        for y in 0..self.h {
            for x in 0..self.w {
                p.put(x, y, SPACE);
            }
        }
        // Stars twinkle: a dot, a small cross, a big cross, a sparkle, ...
        for st in &self.stars {
            let (x, y) = (st.x as i64, st.y);
            let arms: &[(i64, i64)] = match st.phase {
                0 | 5 => &[(0, 0)],
                1 | 4 => &[(0, 0), (1, 0), (-1, 0), (0, 1), (0, -1)],
                2 => &[(2, 0), (-2, 0), (0, 2), (0, -2), (1, 0), (-1, 0), (0, 1), (0, -1)],
                _ => &[(3, 0), (-3, 0), (0, 3), (0, -3), (1, 1), (-1, -1), (1, -1), (-1, 1)],
            };
            for &(dx, dy) in arms {
                p.put(x + dx, y + dy, STAR);
            }
        }
        // The cat, centred; it bobs, its legs paddle and its tail swings.
        let f = frame % CYCLE;
        let bob = [0, 0, 1, 1, 1, 0][f];
        let bx = self.w / 2 - 12;
        let by = self.h / 2 - 10 + bob;
        // Rainbow: six bands in steps that go up and down.
        let band = 3;
        for x in 0..bx + 1 {
            let seg = (x - bx).div_euclid(8);
            let wave = (seg + frame as i64 / 2).rem_euclid(2);
            for (k, &c) in RAINBOW.iter().enumerate() {
                for j in 0..band {
                    p.put(x, self.h / 2 - 9 + wave + (k as i64) * band + j, c);
                }
            }
        }
        // Tail: a thick line with an outline, its tip swinging.
        let tip = [-3, -2, 0, 2, 1, -1][f];
        let (t0, t1) = ((bx as f64 + 1.0, by as f64 + 11.0), (bx as f64 - 6.0, by as f64 + 10.0 + tip as f64));
        for (off, c) in [(1.0, color('K').unwrap()), (0.0, color('G').unwrap())] {
            for i in 0..=12 {
                let t = i as f64 / 12.0;
                let (x, y) = (t0.0 + (t1.0 - t0.0) * t, t0.1 + (t1.1 - t0.1) * t);
                for dy in [-off, 0.0, off] {
                    for dx in [-off, 0.0, off] {
                        p.put((x + dx).round() as i64, (y + dy).round() as i64, c);
                    }
                }
            }
        }
        // Legs, behind the body.
        let swing = [0, -1, -1, 0, 1, 1][f];
        for (lx, s) in [(1, 1), (5, 1), (13, -1), (17, -1)] {
            p.sprite(bx + lx + swing * s, by + 16, LEG, color);
        }
        p.sprite(bx, by, BODY, color);
        let (hx, hy) = ([0, 1, 1, 1, 0, 0][f], [0, 0, 0, 1, 1, 0][f]);
        p.sprite(bx + 12 + hx, by + 5 + hy - bob, HEAD, color);
    }
}

/// The art (`small`) enlarged `scale` times into `big`.
fn enlarge(small: &Pixels, big: &mut Pixels, scale: i64) {
    for y in 0..big.h as i64 {
        for x in 0..big.w as i64 {
            if let Some(c) = small.get(x / scale, y / scale) {
                big.put(x, y, c);
            }
        }
    }
}

fn main() {
    let mut cli = Cli::new("rnyancat");
    let t = Lang::pick(cli.lang, &EN, &PT, &ES);
    let (mut delay, mut frames, mut counter, mut scale) = (90u64, None::<u64>, true, None::<i64>);
    while let Some(a) = cli.next() {
        match a {
            Arg::Pos(p) => cli.unknown(&p),
            Arg::Opt(opt) => match opt.as_str() {
                "-h" | "--help" => cli.help(t.help),
                "-V" | "--version" => cli.version(),
                "-d" | "--delay" => delay = cli.number(&opt, 20..=1000),
                "-f" | "--frames" => frames = Some(cli.number(&opt, 1..=u64::MAX)),
                "-n" | "--no-counter" => counter = false,
                "-s" | "--scale" => scale = Some(cli.number(&opt, 1..=8)),
                _ => cli.unknown(&opt),
            },
        }
    }
    if !std::io::stdout().is_terminal() {
        die("rnyancat", common(cli.lang).not_a_terminal);
    }
    if let Err(e) = run(t, delay, frames, counter, scale) {
        die("rnyancat", &e.to_string());
    }
}

/// The art's scale for a terminal: as big as fits, with room around the cat.
fn fit(cols: usize, rows: usize) -> i64 {
    (cols as i64 / 64).min(rows as i64 * 2 / 34).clamp(1, 8)
}

struct View {
    canvas: Canvas,
    big: Pixels,
    small: Pixels,
    scene: Scene,
    scale: i64,
}

impl View {
    fn new(cols: usize, rows: usize, scale: Option<i64>) -> View {
        let s = scale.unwrap_or_else(|| fit(cols, rows));
        let (aw, ah) = (cols as i64 / s + 1, rows as i64 * 2 / s + 1);
        View {
            canvas: Canvas::new(cols, rows),
            big: Pixels::new(cols, rows),
            small: Pixels::new(aw as usize, (ah as usize).div_ceil(2)),
            scene: Scene::new(aw, ah, Rng::new()),
            scale: s,
        }
    }

    fn render(&mut self, frame: usize, text: Option<&str>) {
        self.scene.draw(&mut self.small, frame);
        enlarge(&self.small, &mut self.big, self.scale);
        self.canvas.clear();
        self.big.show(&mut self.canvas);
        if let Some(text) = text {
            let row = self.canvas.h as i64 - 2;
            let x0 = (self.canvas.w as i64 - text.chars().count() as i64) / 2;
            for (i, ch) in text.chars().enumerate() {
                self.big.glyph(&mut self.canvas, x0 + i as i64, row, ch, Rgb(255, 255, 255));
            }
        }
    }
}

fn run(t: &Texts, delay: u64, frames: Option<u64>, counter: bool, scale: Option<i64>) -> std::io::Result<()> {
    let depth = term::depth(Some(true));
    let _screen = Screen::enter()?;
    let mut out = std::io::stdout().lock();
    let (w, h) = term::size();
    let mut view = View::new(w, h, scale);
    let frame_time = Duration::from_millis(delay);
    let begin = Instant::now();
    let mut n = 0u64;
    loop {
        let start = Instant::now();
        let text = counter.then(|| t.counter.replace("{}", &format!("{:.1}", begin.elapsed().as_secs_f64())));
        view.render(n as usize, text.as_deref());
        view.canvas.flush(&mut out, depth)?;
        n += 1;
        if frames.is_some_and(|f| n >= f) {
            return Ok(());
        }
        match tui::poll(frame_time.saturating_sub(start.elapsed()))? {
            Input::Quit => return Ok(()),
            Input::Resize => {
                let (w, h) = term::size();
                view = View::new(w, h, scale);
            }
            _ => {}
        }
        let left = frame_time.saturating_sub(start.elapsed());
        if !left.is_zero() {
            std::thread::sleep(left);
        }
        view.scene.step();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn art_and_frames() {
        for s in [BODY, HEAD, LEG] {
            let w = s[0].chars().count();
            assert!(s.iter().all(|r| r.chars().count() == w), "{}", s[0]);
            assert!(s.iter().all(|r| r.chars().all(|c| c == '.' || color(c).is_some())));
        }
        for (cols, rows) in [(80, 24), (200, 60), (20, 8)] {
            let mut v = View::new(cols, rows, None);
            for f in 0..20 {
                v.render(f, Some(EN.counter));
                v.scene.step();
            }
        }
        assert_eq!(fit(80, 24), 1);
        assert_eq!(fit(200, 60), 3);
    }

    /// `SNAP_F=2 cargo test --bin rnyancat snapshot -- --ignored --nocapture`
    #[test]
    #[ignore]
    fn snapshot() {
        let env = |k: &str, d: usize| std::env::var(k).ok().and_then(|v| v.parse().ok()).unwrap_or(d);
        let mut v = View::new(env("SNAP_W", 100), env("SNAP_H", 30), None);
        v.scene.rng = Rng::seeded(3);
        v.scene = Scene::new(v.scene.w, v.scene.h, Rng::seeded(3));
        let f = env("SNAP_F", 2);
        for _ in 0..f {
            v.scene.step();
        }
        v.render(f, Some(&PT.counter.replace("{}", "12.3")));
        print!("{}", v.canvas.to_ansi(term::Depth::True));
    }
}

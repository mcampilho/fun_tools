//! rfire: a fire burns at the bottom of the terminal (like aafire and cacafire),
//! drawn with half-block pixels, coloured characters or braille dots.
//! q, Esc or Ctrl+C to quit.
//!
//! The flames follow the classic demo-scene fire of aafire: every point takes the
//! average of five points below it and cools a little, so the heat fed in at the
//! bottom rises and fades. On top of that: a scrolling map of smooth noise makes
//! the cooling uneven (tongues of flame), the fuel row wanders and flares up,
//! sparks fly off, the wind bends the flames and a campfire can burn on logs.

use fun::cli::{die, Arg, Cli};
use fun::i18n::{common, Lang};
use fun::pix::{mix, scale, Pixels};
use fun::rng::Rng;
use fun::term::{self, Depth, Rgb};
use fun::tui::{self, Canvas, Cell, Input, Screen};
use std::io::IsTerminal;
use std::time::{Duration, Instant};

struct Texts {
    help: &'static str,
}

const EN: Texts = Texts {
    help: "\
Usage: rfire [options]

A fire burns at the bottom of the terminal, its flames licking upwards and
sparks flying off. Drawn with half-block pixels (default), coloured characters
(like the old aafire and cacafire) or braille dots.
Keys: q/Esc/Ctrl+C quit, space put out or light again, + and - (or up and down)
flame height, left and right wind, p palette, m mode, c campfire, s sparks.

  -m, --mode <mode>    pixels, chars or braille (default pixels)
  -p, --palette <name> fire, gas, toxic or magic (default fire)
  -i, --intensity <n>  Flame height, 1 to 10 (default 6)
  -w, --wind <n>       Wind, -5 (to the left) to 5 (to the right) (default 0)
  -c, --campfire       A campfire: the flames burn on a pile of logs
  -S, --no-sparks      No sparks
  -d, --delay <ms>     Time per frame, 10 to 1000 ms (default 35)
  -f, --frames <n>     Stop after this many frames
  -C, --no-color       No colours: characters only, like aafire
      --lang <code>    Language: en, pt, es (also: RTOOLS_LANG)
  -h, --help           Show this help
  -V, --version        Show the version
",
};

const PT: Texts = Texts {
    help: "\
Utilização: rfire [opções]

Um fogo arde no fundo do terminal, com as chamas a subir e faúlhas a saltar.
Desenhado com píxeis de meio bloco (por omissão), caracteres coloridos (como os
antigos aafire e cacafire) ou pontos braille.
Teclas: q/Esc/Ctrl+C sair, espaço apagar ou voltar a acender, + e - (ou cima e
baixo) altura das chamas, esquerda e direita vento, p paleta, m modo, c fogueira,
s faúlhas.

  -m, --mode <modo>    pixels, chars ou braille (por omissão pixels)
  -p, --palette <nome> fire, gas, toxic ou magic (por omissão fire)
  -i, --intensity <n>  Altura das chamas, de 1 a 10 (por omissão 6)
  -w, --wind <n>       Vento, de -5 (para a esquerda) a 5 (para a direita)
                       (por omissão 0)
  -c, --campfire       Uma fogueira: as chamas ardem num monte de toros
  -S, --no-sparks      Sem faúlhas
  -d, --delay <ms>     Tempo de cada imagem, de 10 a 1000 ms (por omissão 35)
  -f, --frames <n>     Parar ao fim deste número de imagens
  -C, --no-color       Sem cores: só caracteres, como o aafire
      --lang <código>  Idioma: en, pt, es (também: RTOOLS_LANG)
  -h, --help           Mostrar esta ajuda
  -V, --version        Mostrar a versão
",
};

const ES: Texts = Texts {
    help: "\
Uso: rfire [opciones]

Un fuego arde al fondo del terminal, con las llamas subiendo y chispas saltando.
Dibujado con píxeles de medio bloque (por defecto), caracteres de colores (como
los antiguos aafire y cacafire) o puntos braille.
Teclas: q/Esc/Ctrl+C salir, espacio apagar o volver a encender, + y - (o arriba
y abajo) altura de las llamas, izquierda y derecha viento, p paleta, m modo,
c hoguera, s chispas.

  -m, --mode <modo>    pixels, chars o braille (por defecto pixels)
  -p, --palette <nombre> fire, gas, toxic o magic (por defecto fire)
  -i, --intensity <n>  Altura de las llamas, de 1 a 10 (por defecto 6)
  -w, --wind <n>       Viento, de -5 (a la izquierda) a 5 (a la derecha)
                       (por defecto 0)
  -c, --campfire       Una hoguera: las llamas arden sobre un montón de troncos
  -S, --no-sparks      Sin chispas
  -d, --delay <ms>     Tiempo de cada imagen, de 10 a 1000 ms (por defecto 35)
  -f, --frames <n>     Parar tras este número de imágenes
  -C, --no-color       Sin colores: solo caracteres, como aafire
      --lang <código>  Idioma: en, pt, es (también: RTOOLS_LANG)
  -h, --help           Mostrar esta ayuda
  -V, --version        Mostrar la versión
",
};

#[derive(Clone, Copy, PartialEq, Debug)]
enum Mode {
    /// Two square pixels per cell with half blocks.
    Pixels,
    /// One character per cell from a brightness ramp, dithered.
    Chars,
    /// Braille dots: 2 × 4 per cell.
    Braille,
}

const MODES: &[(&str, Mode)] = &[("pixels", Mode::Pixels), ("chars", Mode::Chars), ("braille", Mode::Braille)];

impl Mode {
    /// Simulation points per cell, across and down.
    fn res(self) -> (usize, usize) {
        match self {
            Mode::Pixels | Mode::Chars => (1, 2),
            Mode::Braille => (2, 4),
        }
    }
}

/// Colour ramps from cool to hot: (heat, colour).
const PALETTES: &[(&str, &[(f32, Rgb)])] = &[
    ("fire", &[(0.0, Rgb(20, 0, 0)), (0.18, Rgb(110, 8, 0)), (0.34, Rgb(195, 30, 0)), (0.5, Rgb(240, 85, 0)), (0.66, Rgb(255, 145, 15)), (0.8, Rgb(255, 205, 55)), (0.92, Rgb(255, 240, 150)), (1.0, Rgb(255, 255, 235))]),
    ("gas", &[(0.0, Rgb(0, 0, 30)), (0.2, Rgb(10, 20, 110)), (0.4, Rgb(25, 70, 210)), (0.6, Rgb(55, 145, 255)), (0.8, Rgb(140, 215, 255)), (1.0, Rgb(240, 250, 255))]),
    ("toxic", &[(0.0, Rgb(0, 20, 0)), (0.2, Rgb(10, 80, 10)), (0.4, Rgb(35, 160, 20)), (0.6, Rgb(120, 225, 40)), (0.8, Rgb(210, 255, 110)), (1.0, Rgb(250, 255, 220))]),
    ("magic", &[(0.0, Rgb(25, 0, 35)), (0.2, Rgb(85, 10, 115)), (0.4, Rgb(165, 30, 195)), (0.6, Rgb(230, 80, 225)), (0.8, Rgb(255, 160, 235)), (1.0, Rgb(255, 235, 255))]),
];

/// The colour of `heat` (0 to 1) on a ramp.
fn shade(stops: &[(f32, Rgb)], heat: f32) -> Rgb {
    let v = heat.clamp(0.0, 1.0);
    for pair in stops.windows(2) {
        let ((a, ca), (b, cb)) = (pair[0], pair[1]);
        if v <= b {
            return mix(ca, cb, ((v - a) / (b - a)) as f64);
        }
    }
    stops[stops.len() - 1].1
}

/// Brightness ramp for the character mode, as in the old ASCII-art renderers.
const RAMP: &[char] = &[' ', '.', ':', '-', '=', '+', '*', '#', '%', '@'];

/// 4 × 4 ordered dithering: steps between two levels become patterns.
const BAYER: [[u8; 4]; 4] = [[0, 8, 2, 10], [12, 4, 14, 6], [3, 11, 1, 9], [15, 7, 13, 5]];

fn bayer(x: usize, y: usize) -> f32 {
    (BAYER[y % 4][x % 4] as f32 + 0.5) / 16.0
}

/// Below this the air is empty (the terminal background shows).
const EMPTY: f32 = 0.045;

struct Options {
    mode: Mode,
    palette: usize,
    intensity: u32,
    wind: i32,
    campfire: bool,
    sparks: bool,
    delay: u64,
    frames: Option<u64>,
}

struct Spark {
    x: f32,
    y: f32,
    vx: f32,
    vy: f32,
    life: f32,
    fade: f32,
}

struct Fire {
    cols: usize,
    rows: usize,
    mode: Mode,
    /// Simulation size: `w` × `h` visible points.
    w: usize,
    h: usize,
    /// `w` × (`h` + 2): two hidden rows of fuel below the visible ones.
    heat: Vec<f32>,
    fuel: Vec<f32>,
    /// `w` × `h` smooth noise from 0 to 1, scrolled upwards: where it is high
    /// the flames cool faster, which splits them into tongues.
    cool: Vec<f32>,
    scroll: usize,
    sparks: Vec<Spark>,
    rng: Rng,
    lit: bool,
    tick: u64,
}

impl Fire {
    fn new(mode: Mode, cols: usize, rows: usize, mut rng: Rng) -> Fire {
        let (rx, ry) = mode.res();
        let (w, h) = ((cols * rx).max(1), (rows * ry).max(1));
        let mut cool: Vec<f32> = (0..w * h).map(|_| rng.float() as f32).collect();
        // Blurred with wrap-around, more up and down than across: tall soft
        // streaks, so the cooler channels split the flames into tongues.
        for pass in 0..8 {
            let src = cool.clone();
            let (dx, dy) = if pass % 4 == 0 { (rx, 0) } else { (0, ry) };
            for y in 0..h {
                for x in 0..w {
                    let s = src[y * w + x] * 2.0 + src[((y + dy) % h) * w + (x + dx) % w] + src[((y + h - dy % h) % h) * w + (x + w - dx % w) % w];
                    cool[y * w + x] = s / 4.0;
                }
            }
        }
        let (lo, hi) = cool.iter().fold((f32::MAX, f32::MIN), |(lo, hi), &v| (lo.min(v), hi.max(v)));
        for v in &mut cool {
            *v = if hi > lo { (*v - lo) / (hi - lo) } else { 0.5 };
        }
        Fire { cols, rows, mode, w, h, heat: vec![0.0; w * (h + 2)], fuel: vec![0.8; w], cool, scroll: 0, sparks: Vec::new(), rng, lit: true, tick: 0 }
    }

    /// How much fuel a column gets: everywhere, or only over the logs.
    fn profile(&self, x: usize, campfire: bool) -> f32 {
        if !campfire {
            return 1.0;
        }
        let half = logs_half(self.cols) / self.cols as f64;
        let d = ((x as f64 + 0.5) / self.w as f64 - 0.5).abs() / half;
        if d >= 1.0 {
            0.0
        } else {
            (1.0 - d * d).sqrt() as f32
        }
    }

    fn step(&mut self, o: &Options) {
        let (w, h) = (self.w, self.h);
        self.tick += 1;
        // The fuel row wanders around its level, like aafire's random walk.
        let level = 0.62 + 0.04 * o.intensity as f32;
        for x in 0..w {
            let f = &mut self.fuel[x];
            if self.lit {
                *f += (self.rng.float() as f32 - 0.5) * 0.4 + (level - *f) * 0.12;
            } else {
                *f *= 0.82;
            }
        }
        // Now and then a flare-up.
        if self.lit && self.rng.chance(0.25) {
            let at = self.rng.below(w);
            let n = 2 + self.rng.below(3 + w / 25);
            for x in at..(at + n).min(w) {
                self.fuel[x] = 1.05 + 0.15 * self.rng.float() as f32;
            }
        }
        let f0 = self.fuel.clone();
        for x in 0..w {
            self.fuel[x] = (f0[x.saturating_sub(1)] + 2.0 * f0[x] + f0[(x + 1).min(w - 1)]) / 4.0;
        }
        for x in 0..w {
            let v = (self.fuel[x] * self.profile(x, o.campfire)).clamp(0.0, 1.2);
            self.heat[h * w + x] = v;
            self.heat[(h + 1) * w + x] = v * (0.9 + 0.2 * self.rng.float() as f32);
        }
        // Each point: the five points below it (aafire's pattern), minus the cooling.
        // Top to bottom, so every row reads the rows below as they were.
        let reach = 0.15 + 0.05 * o.intensity as f32;
        let base = 1.0 / (reach * h as f32);
        let drift = o.wind as f32 / 8.0;
        // Across the sides the field wraps around: with wind, no heat piles up
        // against one side.
        let edge = |x: i64| x.rem_euclid(w as i64) as usize;
        for y in 0..h {
            let (r1, r2) = ((y + 1) * w, (y + 2) * w);
            let crow = ((y + self.scroll) % h) * w;
            // With wind the cool channels lean as the flames do.
            let shear = (drift * (h - y) as f32).round() as i64;
            for x in 0..w {
                let mut sx = x as i64;
                if drift != 0.0 && self.rng.chance(drift.abs() as f64) {
                    sx -= drift.signum() as i64;
                }
                let sum = self.heat[r1 + edge(sx - 1)] + self.heat[r1 + edge(sx)] + self.heat[r1 + edge(sx + 1)] + self.heat[r2 + edge(sx - 1)] + self.heat[r2 + edge(sx + 1)];
                let k = self.cool[crow + edge(x as i64 - shear)];
                let cooling = base * (0.3 + 1.9 * k * k.sqrt()) * (0.6 + 0.8 * self.rng.float() as f32);
                self.heat[y * w + x] = (sum / 5.0 - cooling).max(0.0);
            }
        }
        self.scroll += 1;
        self.move_sparks(o);
    }

    fn move_sparks(&mut self, o: &Options) {
        let (w, h) = (self.w, self.h);
        let speed = h as f32 / 48.0 * self.mode.res().1 as f32 / 2.0;
        if self.lit && o.sparks && self.sparks.len() < 300 {
            let want = o.intensity as f32 * w as f32 / (900.0 * self.mode.res().0 as f32);
            let mut n = want.floor() as usize + usize::from(self.rng.float() < (want.fract()) as f64);
            while n > 0 {
                n -= 1;
                let x = self.rng.below(w);
                // From the tip of the flame in that column.
                let mut y = h;
                while y > 0 && self.heat[(y - 1) * w + x] > 0.3 {
                    y -= 1;
                }
                if y >= h {
                    continue;
                }
                let r = &mut self.rng;
                self.sparks.push(Spark {
                    x: x as f32 + 0.5,
                    y: y as f32,
                    vx: o.wind as f32 * 0.06 * self.mode.res().0 as f32 + (r.float() as f32 - 0.5) * 0.3,
                    vy: -(0.35 + 0.65 * r.float() as f32) * speed,
                    life: 1.0,
                    fade: 0.012 + 0.03 * r.float() as f32,
                });
            }
        }
        for s in &mut self.sparks {
            s.x += s.vx + (self.rng.float() as f32 - 0.5) * 0.7 * self.mode.res().0 as f32;
            s.vx += o.wind as f32 * 0.004;
            s.y += s.vy;
            s.vy *= 0.985;
            s.life -= s.fade;
        }
        self.sparks.retain(|s| s.life > 0.0 && s.y >= 0.0 && s.x >= 0.0 && s.x < w as f32);
    }

    fn at(&self, x: usize, y: usize) -> f32 {
        self.heat[y * self.w + x]
    }

    fn draw(&self, o: &Options, c: &mut Canvas, depth: Depth) {
        let pal = PALETTES[o.palette].1;
        let colored = depth != Depth::None;
        match self.mode {
            Mode::Pixels => {
                let mut p = Pixels::new(self.cols, self.rows);
                for y in 0..self.h {
                    for x in 0..self.w {
                        let v = self.at(x, y);
                        if v > EMPTY {
                            p.put(x as i64, y as i64, shade(pal, v));
                        }
                    }
                }
                if o.campfire {
                    draw_logs(&mut p, self.cols, self.rows, pal, self.tick, self.lit);
                }
                for s in &self.sparks {
                    p.put(s.x as i64, s.y as i64, scale(shade(pal, 0.75 + 0.25 * s.life), (0.5 + 0.6 * s.life) as f64));
                }
                p.show(c);
                return;
            }
            Mode::Chars => {
                for row in 0..self.rows {
                    for x in 0..self.cols {
                        let v = (self.at(x, 2 * row) + self.at(x, 2 * row + 1)) / 2.0;
                        let i = (v * (RAMP.len() - 1) as f32 * 1.1 + bayer(x, row) - 0.5).round().clamp(0.0, (RAMP.len() - 1) as f32) as usize;
                        if i > 0 {
                            c.set(x as i64, row as i64, RAMP[i], colored.then(|| shade(pal, v.max(0.12))), false);
                        }
                    }
                }
                for s in &self.sparks {
                    let (x, row) = (s.x as i64, (s.y / 2.0) as i64);
                    if c.get(x as usize, row as usize).is_some_and(|cell| cell.ch == ' ') {
                        let ch = if s.life > 0.6 { '*' } else if s.life > 0.3 { '+' } else { '.' };
                        c.set(x, row, ch, colored.then(|| shade(pal, 0.7 + 0.3 * s.life)), false);
                    }
                }
            }
            Mode::Braille => {
                let mut bits = vec![0u8; self.cols * self.rows];
                let mut hot = vec![0f32; self.cols * self.rows];
                // Dot bits for (dx, dy) in a braille cell.
                const DOT: [[u8; 2]; 4] = [[0x01, 0x08], [0x02, 0x10], [0x04, 0x20], [0x40, 0x80]];
                for y in 0..self.h {
                    for x in 0..self.w {
                        let v = self.at(x, y);
                        // Dense where hot, thinning out (dithered) where it cools.
                        if v > EMPTY && v * 1.7 > bayer(x, y) {
                            let i = (y / 4) * self.cols + x / 2;
                            bits[i] |= DOT[y % 4][x % 2];
                            hot[i] = hot[i].max(v);
                        }
                    }
                }
                for s in &self.sparks {
                    let (x, y) = (s.x as usize, s.y as usize);
                    if x < self.w && y < self.h {
                        let i = (y / 4) * self.cols + x / 2;
                        bits[i] |= DOT[y % 4][x % 2];
                        hot[i] = hot[i].max(0.7 + 0.3 * s.life);
                    }
                }
                for (i, (&b, &v)) in bits.iter().zip(&hot).enumerate() {
                    if b != 0 {
                        let ch = char::from_u32(0x2800 + b as u32).unwrap_or(' ');
                        c.set((i % self.cols) as i64, (i / self.cols) as i64, ch, colored.then(|| shade(pal, v.max(0.15))), false);
                    }
                }
            }
        }
        if o.campfire {
            // Over characters, a cell with half a log would hide the other half:
            // whole cells instead.
            let mut p = Pixels::new(self.cols, self.rows);
            draw_logs(&mut p, self.cols, self.rows, pal, self.tick, self.lit);
            for row in 0..self.rows as i64 {
                for x in 0..self.cols as i64 {
                    let cell = match (p.get(x, 2 * row), p.get(x, 2 * row + 1)) {
                        (Some(t), Some(b)) if t != b => Cell { ch: '▀', fg: Some(t), bg: Some(b), bold: false },
                        (Some(t), _) => Cell { ch: ' ', fg: None, bg: Some(t), bold: false },
                        (None, Some(b)) => Cell { ch: ' ', fg: None, bg: Some(b), bold: false },
                        (None, None) => continue,
                    };
                    if colored {
                        c.put(x, row, cell);
                    } else {
                        c.set(x, row, '#', None, false);
                    }
                }
            }
        }
    }
}

/// Half the width of the pile of logs, in cells.
fn logs_half(cols: usize) -> f64 {
    (cols as f64 * 0.2).clamp(5.0, 26.0).min(cols as f64 / 2.0)
}

/// Two logs leaning on each other and one lying in front, with glowing cracks.
fn draw_logs(p: &mut Pixels, cols: usize, rows: usize, pal: &[(f32, Rgb)], tick: u64, lit: bool) {
    let (cx, half, base) = (cols as f64 / 2.0, logs_half(cols), (rows * 2) as f64);
    let bark = Rgb(92, 58, 34);
    // Two leaning like a little tent, their tops crossing above the middle,
    // and one lying across the front.
    let top = base - (rows as f64 * 0.6).clamp(4.0, 10.0);
    let logs = [
        (cx - half, base - 1.5, cx + half * 0.25, top, 1.3),
        (cx + half, base - 1.5, cx - half * 0.25, top, 1.3),
        (cx - half * 0.8, base - 1.5, cx + half * 0.8, base - 1.5, 1.5),
    ];
    for (k, &(x0, y0, x1, y1, r)) in logs.iter().enumerate() {
        let n = ((x1 - x0).abs().max((y1 - y0).abs()) * 2.0).ceil() as usize;
        for i in 0..=n {
            let t = i as f64 / n as f64;
            let (x, y) = (x0 + (x1 - x0) * t, y0 + (y1 - y0) * t);
            p.disc(x, y, r, scale(bark, 0.7), 1.0);
        }
        // A lighter top edge makes the logs look round.
        for i in 0..=n {
            let t = i as f64 / n as f64;
            p.put((x0 + (x1 - x0) * t) as i64, (y0 + (y1 - y0) * t - r * 0.6) as i64, bark);
        }
        // The cut end, with its rings.
        p.disc(x0, y0, r + 0.3, Rgb(190, 150, 100), 1.0);
        p.put(x0 as i64, y0 as i64, Rgb(130, 90, 50));
        // Glowing cracks along the log, pulsing.
        if lit {
            for j in 1..n / 6 {
                let t = j as f64 * 6.0 / n as f64;
                let phase = (tick as f64 * 0.15 + j as f64 * 1.7 + k as f64 * 2.3).sin() as f32;
                let (x, y) = (x0 + (x1 - x0) * t, y0 + (y1 - y0) * t + 0.4);
                p.put(x as i64, y as i64, shade(pal, 0.55 + 0.2 * phase));
            }
        }
    }
}

fn main() {
    let mut cli = Cli::new("rfire");
    let t = Lang::pick(cli.lang, &EN, &PT, &ES);
    let mut o = Options { mode: Mode::Pixels, palette: 0, intensity: 6, wind: 0, campfire: false, sparks: true, delay: 35, frames: None };
    let mut color = None;
    while let Some(a) = cli.next() {
        match a {
            Arg::Pos(p) => cli.unknown(&p),
            Arg::Opt(opt) => match opt.as_str() {
                "-h" | "--help" => cli.help(t.help),
                "-V" | "--version" => cli.version(),
                "-m" | "--mode" => {
                    let v = cli.value(&opt).to_ascii_lowercase();
                    o.mode = MODES.iter().find(|(n, _)| *n == v).map(|m| m.1).unwrap_or_else(|| cli.fail(&format!("{} '{opt}': {v}", common(cli.lang).invalid_value)));
                }
                "-p" | "--palette" => {
                    let v = cli.value(&opt).to_ascii_lowercase();
                    o.palette = PALETTES.iter().position(|(n, _)| *n == v).unwrap_or_else(|| cli.fail(&format!("{} '{opt}': {v}", common(cli.lang).invalid_value)));
                }
                "-i" | "--intensity" => o.intensity = cli.number(&opt, 1..=10),
                "-w" | "--wind" => o.wind = cli.number(&opt, -5..=5),
                "-c" | "--campfire" => o.campfire = true,
                "-S" | "--no-sparks" => o.sparks = false,
                "-d" | "--delay" => o.delay = cli.number(&opt, 10..=1000),
                "-f" | "--frames" => o.frames = Some(cli.number(&opt, 1..=u64::MAX)),
                "-C" | "--no-color" => color = Some(false),
                _ => cli.unknown(&opt),
            },
        }
    }
    if !std::io::stdout().is_terminal() {
        die("rfire", common(cli.lang).not_a_terminal);
    }
    let depth = term::depth(color);
    // Pixels need colours: without them, the characters of aafire.
    if depth == Depth::None && o.mode == Mode::Pixels {
        o.mode = Mode::Chars;
    }
    if let Err(e) = run(&mut o, depth) {
        die("rfire", &e.to_string());
    }
}

fn run(o: &mut Options, depth: Depth) -> std::io::Result<()> {
    // Escape codes are needed even without colours (cursor moves).
    term::enable_ansi();
    let _screen = Screen::enter()?;
    let mut out = std::io::stdout().lock();
    let (cols, rows) = term::size();
    let mut c = Canvas::new(cols, rows);
    let mut fire = Fire::new(o.mode, cols, rows, Rng::new());
    let mut frames = 0u64;
    loop {
        let start = Instant::now();
        let frame = Duration::from_millis(o.delay);
        fire.step(o);
        c.clear();
        fire.draw(o, &mut c, depth);
        c.flush(&mut out, depth)?;
        frames += 1;
        if o.frames.is_some_and(|n| frames >= n) {
            return Ok(());
        }
        match tui::poll(frame.saturating_sub(start.elapsed()))? {
            Input::Quit => return Ok(()),
            Input::Key(' ') => fire.lit = !fire.lit,
            Input::Key('+') | Input::Key('=') | Input::Up => o.intensity = (o.intensity + 1).min(10),
            Input::Key('-') | Input::Key('_') | Input::Down => o.intensity = o.intensity.saturating_sub(1).max(1),
            Input::Left => o.wind = (o.wind - 1).max(-5),
            Input::Right => o.wind = (o.wind + 1).min(5),
            Input::Key('p') => o.palette = (o.palette + 1) % PALETTES.len(),
            Input::Key('s') => o.sparks = !o.sparks,
            Input::Key('c') => o.campfire = !o.campfire,
            Input::Key('m') => {
                let i = MODES.iter().position(|m| m.1 == o.mode).unwrap_or(0);
                o.mode = MODES[(i + 1) % MODES.len()].1;
                if depth == Depth::None && o.mode == Mode::Pixels {
                    o.mode = Mode::Chars;
                }
                fire = Fire::new(o.mode, cols.max(c.w), rows.max(c.h), Rng::new());
            }
            Input::Resize => {
                let (cols, rows) = term::size();
                c.resize(cols, rows);
                fire = Fire::new(o.mode, cols, rows, Rng::new());
            }
            _ => {}
        }
        let left = frame.saturating_sub(start.elapsed());
        if !left.is_zero() {
            std::thread::sleep(left);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn options(mode: Mode) -> Options {
        Options { mode, palette: 0, intensity: 6, wind: 0, campfire: false, sparks: true, delay: 35, frames: None }
    }

    #[test]
    fn palettes_run_from_dark_to_bright() {
        let lum = |c: Rgb| c.0 as u32 + c.1 as u32 + c.2 as u32;
        for (name, stops) in PALETTES {
            assert_eq!(stops[0].0, 0.0, "{name}");
            assert_eq!(stops[stops.len() - 1].0, 1.0, "{name}");
            for i in 1..=20 {
                let (a, b) = (shade(stops, (i - 1) as f32 / 20.0), shade(stops, i as f32 / 20.0));
                assert!(lum(b) >= lum(a), "{name} at {i}");
            }
            assert_eq!(shade(stops, 5.0), stops[stops.len() - 1].1);
        }
    }

    #[test]
    fn flames_rise_and_die_out() {
        let mut o = options(Mode::Pixels);
        let mut f = Fire::new(Mode::Pixels, 80, 24, Rng::seeded(3));
        for _ in 0..200 {
            f.step(&o);
        }
        assert!(f.heat.iter().all(|v| (0.0..=1.3).contains(v)));
        // Flames over the bottom third, but not up to the top.
        let row = |f: &Fire, y: usize| (0..f.w).map(|x| f.at(x, y)).sum::<f32>() / f.w as f32;
        assert!(row(&f, f.h - 1) > 0.5, "{}", row(&f, f.h - 1));
        assert!(row(&f, f.h * 2 / 3) > 0.05);
        assert!(row(&f, 0) < 0.01);
        assert!(!f.sparks.is_empty());
        // Higher flames with more intensity.
        let mut g = Fire::new(Mode::Pixels, 80, 24, Rng::seeded(3));
        o.intensity = 10;
        for _ in 0..200 {
            g.step(&o);
        }
        assert!(row(&g, f.h / 3) > row(&f, f.h / 3));
        // Put out: everything goes dark.
        g.lit = false;
        for _ in 0..200 {
            g.step(&o);
        }
        assert!(g.heat.iter().all(|&v| v < EMPTY), "{:?}", g.heat.iter().cloned().fold(0.0, f32::max));
        assert!(g.sparks.is_empty());
    }

    #[test]
    fn every_mode_draws_at_any_size() {
        for &(_, mode) in MODES {
            for (cols, rows) in [(1, 1), (2, 3), (7, 2), (80, 24), (200, 60)] {
                for campfire in [false, true] {
                    let mut o = options(mode);
                    o.campfire = campfire;
                    o.wind = 5;
                    let mut f = Fire::new(mode, cols, rows, Rng::seeded(cols as u64));
                    let mut c = Canvas::new(cols, rows);
                    for _ in 0..60 {
                        f.step(&o);
                    }
                    for depth in [Depth::True, Depth::None] {
                        c.clear();
                        f.draw(&o, &mut c, depth);
                    }
                    if cols >= 80 {
                        let filled = (0..cols).filter(|&x| c.get(x, rows - 3).is_some_and(|cell| cell != Cell { ch: ' ', fg: None, bg: None, bold: false })).count();
                        assert!(filled > cols / 4, "{mode:?} {campfire}: {filled}");
                    }
                }
            }
        }
    }

    /// `cargo test --bin rfire snapshot -- --ignored --nocapture`
    /// SNAP_MODE (pixels, chars, braille), SNAP_PAL, SNAP_CAMP=1, SNAP_W, SNAP_H, SNAP_WIND.
    #[test]
    #[ignore]
    fn snapshot() {
        let env = |k: &str| std::env::var(k).ok();
        let mode = env("SNAP_MODE").and_then(|m| MODES.iter().find(|(n, _)| *n == m).map(|m| m.1)).unwrap_or(Mode::Pixels);
        let mut o = options(mode);
        o.palette = env("SNAP_PAL").and_then(|p| PALETTES.iter().position(|(n, _)| *n == p)).unwrap_or(0);
        o.campfire = env("SNAP_CAMP").is_some();
        o.wind = env("SNAP_WIND").and_then(|v| v.parse().ok()).unwrap_or(0);
        let w = env("SNAP_W").and_then(|v| v.parse().ok()).unwrap_or(80);
        let h = env("SNAP_H").and_then(|v| v.parse().ok()).unwrap_or(24);
        let mut f = Fire::new(mode, w, h, Rng::seeded(7));
        for _ in 0..150 {
            f.step(&o);
        }
        let mut c = Canvas::new(w, h);
        f.draw(&o, &mut c, Depth::True);
        print!("{}", c.to_ansi(Depth::True));
    }
}

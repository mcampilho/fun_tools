//! rasciiquarium: an aquarium in the terminal (like asciiquarium), with original
//! art. q, Esc or Ctrl+C to quit, p to pause.

use fun::cli::{die, Arg, Cli};
use fun::i18n::{common, Lang};
use fun::rng::Rng;
use fun::term::{self, Rgb};
use fun::tui::{self, Canvas, Input, Screen};
use std::io::IsTerminal;
use std::time::{Duration, Instant};

struct Texts {
    help: &'static str,
}

const EN: Texts = Texts {
    help: "\
Usage: rasciiquarium [options]

An aquarium: fish, bubbles, seaweed, a castle and the odd visitor.
Keys: q/Esc/Ctrl+C quit, p pause, r restart.

  -s, --speed <n>      Speed, 1 (slow) to 10 (fast) (default 5)
  -n, --fish <n>       How many fish, 1 (few) to 10 (many) (default 5)
      --lang <code>    Language: en, pt, es (also: RTOOLS_LANG)
  -h, --help           Show this help
  -V, --version        Show the version
",
};

const PT: Texts = Texts {
    help: "\
Utilização: rasciiquarium [opções]

Um aquário: peixes, bolhas, algas, um castelo e um ou outro visitante.
Teclas: q/Esc/Ctrl+C sair, p pausa, r recomeçar.

  -s, --speed <n>      Velocidade, de 1 (lenta) a 10 (rápida) (por omissão 5)
  -n, --fish <n>       Quantos peixes, de 1 (poucos) a 10 (muitos) (por omissão 5)
      --lang <código>  Idioma: en, pt, es (também: RTOOLS_LANG)
  -h, --help           Mostrar esta ajuda
  -V, --version        Mostrar a versão
",
};

const ES: Texts = Texts {
    help: "\
Uso: rasciiquarium [opciones]

Un acuario: peces, burbujas, algas, un castillo y algún visitante.
Teclas: q/Esc/Ctrl+C salir, p pausa, r reiniciar.

  -s, --speed <n>      Velocidad, de 1 (lenta) a 10 (rápida) (por defecto 5)
  -n, --fish <n>       Cuántos peces, de 1 (pocos) a 10 (muchos) (por defecto 5)
      --lang <código>  Idioma: en, pt, es (también: RTOOLS_LANG)
  -h, --help           Mostrar esta ayuda
  -V, --version        Mostrar la versión
",
};

// All art faces right; it is mirrored for creatures swimming left.
// In fish, `o` and `°` are eyes (drawn white).
const FISH: &[&[&str]] = &[
    &["><>"],
    &["><((°>"],
    &[" ,/", "><  o>", " `\\"],
    &["   __", "\\/ /  \\", " )=  o >", "/\\ \\__/"],
    &["   _____", "\\ /  ~~ \\", " |  ~~  o>", "/ \\_____/"],
    &["  _.-.", ">(_  o)>", "   `-'"],
];

const SHARK: &[&str] = &[
    "         |\\",
    "  ___.---' `-----.__",
    " <___  '     ___    o`>",
    "     `--..__.'   `-'",
];

const WHALE: &[&str] = &[
    "      .-'\"\"'-.",
    " \\  .'        `.",
    "  >(   o        )",
    " /  `.__..--.__.'",
];
const SPOUT: &[&[&str]] = &[&["   :"], &["  .:.", "   :"], &[" .:.:.", "  .:.", "   :"], &["  . .", " .   .", "  . ."]];

const SHIP: &[&str] = &["      |\\", "      | \\", "      |__\\", "  ____|_____", "  \\ o  o  o/"];

const JELLY: [&[&str]; 2] = [&[" .--.", "(    )", " )||(", " (||)"], &[" .--.", "(    )", " (||)", " )||("]];

const CASTLE: &[&str] = &[
    "            T~~",
    "            |",
    "           /^\\",
    "          /   \\",
    "  _   _  /_____\\  _   _",
    " | |_| |_| |_| |_| |_| |",
    " |   _    _____    _   |",
    " |  | |  |  _  |  | |  |",
    " |__|_|__|_|_|_|__|_|__|",
];

const WATER: Rgb = Rgb(70, 140, 230);
const WATER_LIGHT: Rgb = Rgb(140, 200, 255);
const SAND: Rgb = Rgb(200, 175, 120);
const WEED: Rgb = Rgb(40, 170, 70);
const STONE: Rgb = Rgb(160, 160, 170);
const WHITE: Rgb = Rgb(240, 240, 240);
const FISH_COLORS: &[Rgb] = &[
    Rgb(255, 120, 60), Rgb(255, 210, 60), Rgb(90, 220, 120), Rgb(240, 90, 160),
    Rgb(120, 170, 255), Rgb(200, 120, 255), Rgb(80, 220, 220), Rgb(255, 90, 90),
];

fn mirror(art: &[&str]) -> Vec<String> {
    let w = art.iter().map(|l| l.chars().count()).max().unwrap_or(0);
    art.iter()
        .map(|l| {
            let mut s: Vec<char> = l.chars().collect();
            s.resize(w, ' ');
            s.iter()
                .rev()
                .map(|&c| match c {
                    '(' => ')', ')' => '(', '<' => '>', '>' => '<', '/' => '\\', '\\' => '/',
                    '[' => ']', ']' => '[', '{' => '}', '}' => '{', '`' => '\'', '\'' => '`',
                    c => c,
                })
                .collect::<String>()
                .trim_end()
                .to_string()
        })
        .collect()
}

#[derive(Clone, Copy, PartialEq)]
enum Kind {
    Fish,
    Shark,
    Whale,
    Ship,
    Jelly,
}

struct Thing {
    kind: Kind,
    x: f64,
    y: f64,
    dx: f64,
    dy: f64,
    art: Vec<String>,
    color: Rgb,
    age: u64,
}

impl Thing {
    fn width(&self) -> i64 {
        self.art.iter().map(|l| l.chars().count()).max().unwrap_or(0) as i64
    }
}

struct Bubble {
    x: i64,
    y: f64,
}

struct Tank {
    w: i64,
    h: i64,
    rng: Rng,
    things: Vec<Thing>,
    bubbles: Vec<Bubble>,
    weeds: Vec<(i64, i64)>,
    tick: u64,
    fish_level: f64,
}

/// Rows of sky above the water (room for the ship), then four rows of waves.
const SKY: i64 = 5;
const SURFACE: i64 = SKY + 4;

impl Tank {
    fn new(w: usize, h: usize, fish_level: f64, rng: Rng) -> Tank {
        let mut t = Tank { w: w as i64, h: h as i64, rng, things: Vec::new(), bubbles: Vec::new(), weeds: Vec::new(), tick: 0, fish_level };
        let n_weeds = (t.w / 7).max(2);
        // Not in front of the castle (bottom right).
        let castle_x = t.w - CASTLE.iter().map(|l| l.len()).max().unwrap_or(0) as i64 - 3;
        for _ in 0..n_weeds {
            let x = t.rng.range(1, castle_x.max(2));
            let hgt = t.rng.range(3, (t.h / 4).clamp(4, 9));
            t.weeds.push((x, hgt));
        }
        for _ in 0..t.target_fish() {
            t.add_fish(true);
        }
        t
    }

    fn target_fish(&self) -> usize {
        ((self.w * (self.h - SURFACE)) as f64 / 400.0 * self.fish_level).clamp(2.0, 80.0) as usize
    }

    fn add_fish(&mut self, anywhere: bool) {
        let art = *self.rng.pick(FISH);
        let right = self.rng.chance(0.5);
        let lines: Vec<String> = if right { art.iter().map(|s| s.to_string()).collect() } else { mirror(art) };
        let fw = lines.iter().map(|l| l.chars().count()).max().unwrap_or(1) as i64;
        let lo = SURFACE + 1;
        let hi = (self.h - lines.len() as i64 - 2).max(lo + 1);
        let y = self.rng.range(lo, hi) as f64;
        let speed = 0.25 + self.rng.float() * 0.9;
        let x = if anywhere {
            self.rng.range(0, self.w.max(1)) as f64
        } else if right {
            -(fw as f64)
        } else {
            self.w as f64
        };
        let color = *self.rng.pick(FISH_COLORS);
        self.things.push(Thing { kind: Kind::Fish, x, y, dx: if right { speed } else { -speed }, dy: 0.0, art: lines, color, age: 0 });
    }

    fn add_visitor(&mut self) {
        let right = self.rng.chance(0.5);
        let (kind, art, color, y, speed): (Kind, &[&str], Rgb, f64, f64) = match self.rng.below(4) {
            0 => (Kind::Shark, SHARK, Rgb(150, 160, 180), self.rng.range(SURFACE + 2, (self.h - 8).max(SURFACE + 3)) as f64, 1.1),
            1 => (Kind::Whale, WHALE, Rgb(90, 120, 200), (SKY - 1) as f64, 0.45),
            2 => (Kind::Ship, SHIP, Rgb(210, 170, 110), (SKY - SHIP.len() as i64 + 1) as f64, 0.5),
            _ => (Kind::Jelly, JELLY[0], Rgb(240, 130, 210), (self.h - 6) as f64, 0.15),
        };
        let lines: Vec<String> = if right || kind == Kind::Jelly { art.iter().map(|s| s.to_string()).collect() } else { mirror(art) };
        let w = lines.iter().map(|l| l.chars().count()).max().unwrap_or(1) as f64;
        let (x, dx, dy) = if kind == Kind::Jelly {
            (self.rng.range(2, (self.w - 8).max(3)) as f64, 0.0, -speed)
        } else if right {
            (-w, speed, 0.0)
        } else {
            (self.w as f64, -speed, 0.0)
        };
        self.things.push(Thing { kind, x, y, dx, dy, art: lines, color, age: 0 });
    }

    fn step(&mut self, factor: f64) {
        self.tick += 1;
        let (w, h) = (self.w as f64, self.h as f64);
        for t in &mut self.things {
            t.x += t.dx * factor;
            t.y += t.dy * factor;
            t.age += 1;
            if t.kind == Kind::Jelly {
                t.art = JELLY[(t.age / 6 % 2) as usize].iter().map(|s| s.to_string()).collect();
            }
        }
        let mut new_bubbles = Vec::new();
        for t in &self.things {
            if t.kind == Kind::Fish && self.rng.chance(0.012 * factor) {
                let head = if t.dx > 0.0 { t.x as i64 + t.width() } else { t.x as i64 - 1 };
                new_bubbles.push(Bubble { x: head, y: t.y });
            }
        }
        self.bubbles.extend(new_bubbles);
        for b in &mut self.bubbles {
            b.y -= 0.4 * factor;
        }
        self.bubbles.retain(|b| b.y >= SURFACE as f64);
        self.things.retain(|t| {
            let tw = t.art.iter().map(|l| l.chars().count()).max().unwrap_or(0) as f64;
            t.x > -tw - 1.0 && t.x < w + 1.0 && t.y > -10.0 && t.y < h
        });
        // Sharks eat the small fish they touch.
        let sharks: Vec<(f64, f64, f64)> = self
            .things
            .iter()
            .filter(|t| t.kind == Kind::Shark)
            .map(|t| (if t.dx > 0.0 { t.x + t.width() as f64 - 3.0 } else { t.x + 1.0 }, t.y + 2.0, t.dx))
            .collect();
        self.things.retain(|t| {
            t.kind != Kind::Fish
                || !sharks.iter().any(|&(mx, my, _)| (t.y - my).abs() < 1.5 && mx >= t.x - 1.0 && mx <= t.x + t.width() as f64)
        });
        let fish = self.things.iter().filter(|t| t.kind == Kind::Fish).count();
        if fish < self.target_fish() && self.rng.chance(0.08 * factor) {
            self.add_fish(false);
        }
        let visitors = self.things.iter().filter(|t| t.kind != Kind::Fish).count();
        if visitors == 0 && self.rng.chance(0.004 * factor) {
            self.add_visitor();
        }
    }

    fn draw(&self, c: &mut Canvas) {
        c.clear();
        // Water surface: rolling waves.
        let shift = (self.tick / 4) as i64;
        let rows = ["~-~^~~-~~^~-~~", "^~~-~^~~~-~^~~", "~~-~~~^~-~~~~^", "-~~^~~~~-~~^~~"];
        for (y, pat) in rows.iter().enumerate() {
            let pat: Vec<char> = pat.chars().collect();
            for x in 0..self.w {
                let i = ((x + shift * if y % 2 == 0 { 1 } else { -1 }).rem_euclid(pat.len() as i64)) as usize;
                let col = if y == 0 { WATER_LIGHT } else { WATER };
                c.set(x, SKY + y as i64, pat[i], Some(col), false);
            }
        }
        // Sand.
        let sand: Vec<char> = ".,:;'.,`.;:,.".chars().collect();
        for x in 0..self.w {
            c.set(x, self.h - 1, sand[(x as usize * 7) % sand.len()], Some(SAND), false);
        }
        // Castle, bottom right.
        let cw = CASTLE.iter().map(|l| l.len()).max().unwrap_or(0) as i64;
        if self.w > cw + 10 && self.h > CASTLE.len() as i64 + SURFACE + 3 {
            let (x0, y0) = (self.w - cw - 2, self.h - 1 - CASTLE.len() as i64);
            for (i, l) in CASTLE.iter().enumerate() {
                let col = if i == 0 { Rgb(230, 80, 80) } else { STONE };
                c.text(x0, y0 + i as i64, l, Some(col), true);
            }
        }
        // Seaweed: stalks of ( and ) that sway.
        for (k, &(x, hgt)) in self.weeds.iter().enumerate() {
            for j in 0..hgt {
                let sway = ((self.tick / 8) as i64 + j + k as i64) % 2 == 0;
                c.set(x + if sway { 1 } else { 0 }, self.h - 2 - j, if sway { ')' } else { '(' }, Some(WEED), false);
            }
        }
        // Creatures.
        for t in &self.things {
            let (x0, y0) = (t.x as i64, t.y as i64);
            for (j, line) in t.art.iter().enumerate() {
                // Inside the outline the creature is opaque: what is behind does not show through.
                let first = line.chars().position(|ch| ch != ' ').unwrap_or(0);
                let last = line.chars().count().saturating_sub(1);
                for (i, ch) in line.chars().enumerate() {
                    if ch == ' ' {
                        if i > first && i < last {
                            c.set(x0 + i as i64, y0 + j as i64, ' ', None, false);
                        }
                        continue;
                    }
                    let col = if t.kind == Kind::Fish && (ch == 'o' || ch == '°') { WHITE } else { t.color };
                    c.set(x0 + i as i64, y0 + j as i64, ch, Some(col), false);
                }
            }
            if t.kind == Kind::Whale {
                // A spout now and then, above the head.
                let phase = (t.age / 5) as usize % (SPOUT.len() + 4);
                if let Some(sp) = SPOUT.get(phase) {
                    let head = if t.dx > 0.0 { x0 + 14 } else { x0 + 3 };
                    for (j, l) in sp.iter().rev().enumerate() {
                        c.text(head - 3, y0 - 1 - j as i64, l, Some(WATER_LIGHT), true);
                    }
                }
            }
        }
        for b in &self.bubbles {
            let ch = if b.y < (SURFACE + 3) as f64 { 'O' } else if b.y < self.h as f64 / 2.0 { 'o' } else { '.' };
            c.set(b.x, b.y as i64, ch, Some(WATER_LIGHT), false);
        }
    }
}

fn main() {
    let mut cli = Cli::new("rasciiquarium");
    let t = Lang::pick(cli.lang, &EN, &PT, &ES);
    let (mut speed, mut fish) = (5u32, 5u32);
    while let Some(a) = cli.next() {
        match a {
            Arg::Pos(p) => cli.unknown(&p),
            Arg::Opt(opt) => match opt.as_str() {
                "-h" | "--help" => cli.help(t.help),
                "-V" | "--version" => cli.version(),
                "-s" | "--speed" => speed = cli.number(&opt, 1..=10),
                "-n" | "--fish" => fish = cli.number(&opt, 1..=10),
                _ => cli.unknown(&opt),
            },
        }
    }
    if !std::io::stdout().is_terminal() {
        die("rasciiquarium", common(cli.lang).not_a_terminal);
    }
    if let Err(e) = run(speed, fish) {
        die("rasciiquarium", &e.to_string());
    }
}

fn run(speed: u32, fish: u32) -> std::io::Result<()> {
    let depth = term::depth(Some(true));
    let _screen = Screen::enter()?;
    let mut out = std::io::stdout().lock();
    let (w, h) = term::size();
    let mut c = Canvas::new(w, h);
    let level = fish as f64 / 5.0;
    let mut tank = Tank::new(w, h, level, Rng::new());
    let factor = 0.3 + speed as f64 * 0.14;
    let frame = Duration::from_millis(60);
    let mut paused = false;
    loop {
        let start = Instant::now();
        if !paused {
            tank.step(factor);
        }
        tank.draw(&mut c);
        c.flush(&mut out, depth)?;
        match tui::poll(frame.saturating_sub(start.elapsed()))? {
            Input::Quit => return Ok(()),
            Input::Key('p') | Input::Key(' ') => paused = !paused,
            Input::Key('r') => tank = Tank::new(c.w, c.h, level, Rng::new()),
            Input::Resize => {
                let (w, h) = term::size();
                c.resize(w, h);
                tank = Tank::new(w, h, level, Rng::new());
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

    #[test]
    fn mirrors_and_runs() {
        assert_eq!(mirror(&["><((°>"]), ["<°))><"]);
        assert_eq!(mirror(&[" ,/", "><  o>"]), ["   \\,", "<o  ><"]);
        let mut t = Tank::new(100, 30, 1.0, Rng::seeded(9));
        let mut c = Canvas::new(100, 30);
        for _ in 0..3000 {
            t.step(1.0);
            t.draw(&mut c);
        }
        assert!(t.things.iter().any(|x| x.kind == Kind::Fish));
    }

    /// `cargo test --bin rasciiquarium snapshot -- --ignored --nocapture` prints a frame.
    #[test]
    #[ignore]
    fn snapshot() {
        let mut t = Tank::new(110, 32, 1.6, Rng::seeded(4));
        t.add_visitor();
        let mut c = Canvas::new(110, 32);
        for _ in 0..60 {
            t.step(1.0);
        }
        t.draw(&mut c);
        for y in 0..c.h {
            let row: String = (0..c.w).map(|x| c.get(x, y).map_or(' ', |cell| cell.ch)).collect();
            println!("{}", row.trim_end());
        }
    }
}

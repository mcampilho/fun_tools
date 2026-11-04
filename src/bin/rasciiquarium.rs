//! rasciiquarium: an aquarium in the terminal (like asciiquarium), with original
//! art. q, Esc or Ctrl+C to quit, p to pause.

use fun::cli::{die, Arg, Cli};
use fun::i18n::{common, Lang};
use fun::pix::{self, mix, Pixels};
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
  -u, --unicode        Unicode look: colour pixel art made of half blocks and
                       symbols (needs a font with them; best in 24-bit colour)
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
  -u, --unicode        Aspeto Unicode: pixel art a cores feita de meios blocos e
                       símbolos (precisa de uma fonte que os tenha; melhor a 24 bits)
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
  -u, --unicode        Aspecto Unicode: pixel art en color hecho de medios bloques y
                       símbolos (necesita una fuente que los tenga; mejor a 24 bits)
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

// The Unicode look: pixel art, one character per pixel, two pixel rows per
// terminal row. Letters are colours (see `pix::palette`); `.` is transparent.
const FISH_PX: &[&[&str]] = &[
    &[
        "B...AAA.",
        "BBAAAAKA",
        "BBACCCAA",
        "B...AA..",
    ],
    &[
        "....BBBB....",
        "B..AAAAAAA..",
        "BBAAAAAAWKA.",
        "BBAACCCCAAAA",
        "B..ACCCCAA..",
        ".....BB.....",
    ],
    &[
        "B.....BBBB......",
        "BBAAAAAAAAAAAWK.",
        "BBACCCCCCCCCAAAA",
        "B......BB.......",
    ],
    &[
        "......B....",
        ".....BA....",
        "....ABAA...",
        "B..AABAAA..",
        "BBAAABAAWK.",
        "BBAAABAAAAA",
        "B..ACBCCA..",
        "....CBCC...",
        ".....BC....",
        "......B....",
    ],
    &[
        "..B.B.B...",
        ".BAAAAAAB.",
        "BAAAAAAWKA",
        "BBAAAAAAAA",
        "BBACCCCCAA",
        "BACCCCCCAB",
        ".BCCCCCCB.",
        "..B.B.B...",
    ],
];

const SHARK_PX: &[&str] = &[
    "...............B................",
    "..............BB................",
    ".............BBB................",
    "B...........BBBB................",
    "BB.......AAAAAAAAAAAAAAA........",
    ".BB...AAAAAAAAAAAAAAAAAAAAAA....",
    "..BBAAAAAAAAAAAAAAAAAABABAKAAA..",
    "..BBAAAAAAAAAAAAAAAAAABABAAAAAAA",
    ".BB...CCCCCCCCCCCCCCCCCCCCWCWCC.",
    "BB.......CCCCBBBCCCCCCCCCCCCC...",
    "B...........BBB.................",
    ".............B..................",
];
/// Pixel row of the shark's mouth (where it eats).
const SHARK_PX_MOUTH: i64 = 8;

const WHALE_PX: &[&str] = &[
    "BB................................",
    ".BB.......AAAAAAAAAAAA............",
    "..BB...AAAAAAAAAAAAAAAAAA.........",
    "..BBBAAAAAAAAAAAAAAAAAAAAAA.......",
    ".BB.AAAAAAAAAAAAAAAAAAAAAAAAA.....",
    "BB..AAAAAAAAAAAAAAAAAAAAAAAAAAA...",
    "....AAAAAAAAAAAAAAAAAAAAAAAWKAAA..",
    ".....AAAAAAAAAAAAAAAAAAAAAAAAAAAA.",
    "......CCCCCCCCCCCCCCCCCCCCCCCCCCCA",
    ".......DCDCDCDCDCDCDCDCDCDCDCCCC..",
    "..........CCCCCCCCCCCCCCCCCCC.....",
    "...............BBB................",
    "................BB................",
];
/// Column of the blowhole in the right-facing whale.
const WHALE_PX_HOLE: i64 = 24;

const SHIP_PX: &[&str] = &[
    "............RRR............",
    "............RR.............",
    "............M..............",
    "......WWWWW.M.WWWWWW.......",
    ".....WWWWWL.M.WWWWWWL......",
    "....WWWWWWL.M.WWWWWWWL.....",
    "...WWWWWWWL.M.WWWWWWWWL....",
    "..LLLLLLLLL.M.LLLLLLLLLL...",
    "............M..............",
    "NNNNNNNNNNNNNNNNNNNNNNNNNNN",
    "NNNNNNNNNNNNNNNNNNNNNNNNNN.",
    ".MMYMMMMYMMMMYMMMMYMMMMMM..",
    "..MMMMMMMMMMMMMMMMMMMMMM...",
];

const JELLY_PX: [&[&str]; 2] = [
    &[
        "...CCCC...",
        "..CAAAAC..",
        ".CAAAAAAC.",
        ".AAAAAAAA.",
        "AAAAAAAAAA",
        "BABABABABA",
        ".B..B..B..",
        "..B..B..B.",
        ".B..B..B..",
        "..B..B..B.",
    ],
    &[
        "...CCCC...",
        "..CAAAAC..",
        ".CAAAAAAC.",
        ".AAAAAAAA.",
        "AAAAAAAAAA",
        "BABABABABA",
        "..B..B..B.",
        ".B..B..B..",
        "..B..B..B.",
        ".B..B..B..",
    ],
];

const CASTLE_PX: &[&str] = &[
    "..............R..............",
    "..............RR.............",
    "..............RRR............",
    "..............F..............",
    ".............QQQ.............",
    "............QRRRQ............",
    "...........QRRRRRQ...........",
    "..........QRRRRRRRQ..........",
    "..........LLLLLLLLL..........",
    "L.L.L.....LLLLKLLLL.....L.L.L",
    "LLLLL.....LLLKKKLLL.....LLLLL",
    "LLKLL.....LLLKKKLLL.....LLKLL",
    "LLKLLL.L.LLLLLLLLLLL.L.LLLKLL",
    "LLLLLLLLLLLLLLLLLLLLLLLLLLLLL",
    "LLLLLELLLLLLLLLLLLLLLLLELLLLL",
    "LLLLLLLLLLLLKKKKKLLLLLLLLLLLL",
    "LELLLLLLLLLKKKKKKKLLLLLLLLLEL",
    "LLLLLLLLLLLKKKKKKKLLLLLLLLLLL",
    "LLLLLLELLLLKKKKKKKLLLLLLELLLL",
    "LLLLLLLLLLLKKKKKKKLLLLLLLLLLL",
];

/// The pictures of one look, and where things go (in rows for ASCII, in pixel
/// rows for Unicode).
struct Art {
    pixels: bool,
    fish: &'static [&'static [&'static str]],
    shark: &'static [&'static str],
    whale: &'static [&'static str],
    ship: &'static [&'static str],
    jelly: [&'static [&'static str]; 2],
    castle: &'static [&'static str],
    /// Units per terminal row: 1 or 2.
    unit: i64,
    /// Top of the water, and the first row below the waves.
    sky: i64,
    surface: i64,
    sand: i64,
    shark_mouth: i64,
}

const ASCII_ART: Art = Art {
    pixels: false,
    fish: FISH,
    shark: SHARK,
    whale: WHALE,
    ship: SHIP,
    jelly: JELLY,
    castle: CASTLE,
    unit: 1,
    sky: SKY,
    surface: SURFACE,
    sand: 1,
    shark_mouth: 2,
};

const PIXEL_ART: Art = Art {
    pixels: true,
    fish: FISH_PX,
    shark: SHARK_PX,
    whale: WHALE_PX,
    ship: SHIP_PX,
    jelly: JELLY_PX,
    castle: CASTLE_PX,
    unit: 2,
    sky: 12,
    surface: 16,
    sand: 4,
    shark_mouth: SHARK_PX_MOUTH,
};

impl Art {
    fn flip(&self, art: &[&str]) -> Vec<String> {
        if self.pixels { pix::mirror(art) } else { mirror(art) }
    }
}

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
    art: &'static Art,
    w: i64,
    /// In the art's units: terminal rows, or pixel rows.
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

fn art_width(art: &[&str]) -> i64 {
    art.iter().map(|l| l.chars().count()).max().unwrap_or(0) as i64
}

impl Tank {
    fn new(art: &'static Art, w: usize, h: usize, fish_level: f64, rng: Rng) -> Tank {
        let (w, h) = (w as i64, h as i64 * art.unit);
        let mut t = Tank { art, w, h, rng, things: Vec::new(), bubbles: Vec::new(), weeds: Vec::new(), tick: 0, fish_level };
        let n_weeds = (t.w / 7).max(2);
        // Not in front of the castle (bottom right).
        let castle_x = t.w - art_width(art.castle) - 3;
        for _ in 0..n_weeds {
            let x = t.rng.range(1, castle_x.max(2));
            let hgt = t.rng.range(3, (t.h / art.unit / 4).clamp(4, 9)) * art.unit;
            t.weeds.push((x, hgt));
        }
        for _ in 0..t.target_fish() {
            t.add_fish(true);
        }
        t
    }

    fn target_fish(&self) -> usize {
        let rows = (self.h - self.art.surface) / self.art.unit;
        ((self.w * rows) as f64 / 400.0 * self.fish_level).clamp(2.0, 80.0) as usize
    }

    fn add_fish(&mut self, anywhere: bool) {
        let art = *self.rng.pick(self.art.fish);
        let right = self.rng.chance(0.5);
        let lines: Vec<String> = if right { art.iter().map(|s| s.to_string()).collect() } else { self.art.flip(art) };
        let fw = lines.iter().map(|l| l.chars().count()).max().unwrap_or(1) as i64;
        let lo = self.art.surface + self.art.unit;
        let hi = (self.h - lines.len() as i64 - 2 * self.art.unit).max(lo + 1);
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
        let which = self.rng.below(4);
        self.add_visitor_kind(which);
    }

    fn add_visitor_kind(&mut self, which: usize) {
        let right = self.rng.chance(0.5);
        let a = self.art;
        let u = a.unit;
        let (kind, art, color, y, speed): (Kind, &[&str], Rgb, f64, f64) = match which {
            0 => (Kind::Shark, a.shark, Rgb(150, 160, 180), self.rng.range(a.surface + 2 * u, (self.h - 8 * u).max(a.surface + 3 * u)) as f64, 1.1),
            // Whale: its back above the waves; ship: its hull in the water.
            1 => (Kind::Whale, a.whale, Rgb(90, 120, 200), (a.sky - if a.pixels { 5 } else { 1 }) as f64, 0.45),
            2 => (Kind::Ship, a.ship, Rgb(210, 170, 110), (a.sky - a.ship.len() as i64 + if a.pixels { 3 } else { 1 }) as f64, 0.5),
            _ => (Kind::Jelly, a.jelly[0], Rgb(240, 130, 210), (self.h - 6 * u) as f64, 0.15 * u as f64),
        };
        let lines: Vec<String> = if right || kind == Kind::Jelly { art.iter().map(|s| s.to_string()).collect() } else { a.flip(art) };
        let w = lines.iter().map(|l| l.chars().count()).max().unwrap_or(1) as f64;
        let (x, dx, dy) = if kind == Kind::Jelly {
            (self.rng.range(2, (self.w - 12).max(3)) as f64, 0.0, -speed)
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
                t.art = self.art.jelly[(t.age / 6 % 2) as usize].iter().map(|s| s.to_string()).collect();
            }
        }
        let u = self.art.unit as f64;
        let mut new_bubbles = Vec::new();
        for t in &self.things {
            if t.kind == Kind::Fish && self.rng.chance(0.012 * factor) {
                let head = if t.dx > 0.0 { t.x as i64 + t.width() } else { t.x as i64 - 1 };
                new_bubbles.push(Bubble { x: head, y: t.y + (t.art.len() as f64 / 2.0).floor() * (u - 1.0) });
            }
        }
        self.bubbles.extend(new_bubbles);
        for b in &mut self.bubbles {
            b.y -= 0.4 * factor * u;
        }
        let surface = self.art.surface as f64;
        self.bubbles.retain(|b| b.y >= surface);
        self.things.retain(|t| {
            let tw = t.art.iter().map(|l| l.chars().count()).max().unwrap_or(0) as f64;
            t.x > -tw - 1.0 && t.x < w + 1.0 && t.y > -10.0 * u && t.y < h
        });
        // Sharks eat the small fish they touch (the mouth must be inside the fish).
        let mouth = self.art.shark_mouth as f64;
        let sharks: Vec<(f64, f64)> = self
            .things
            .iter()
            .filter(|t| t.kind == Kind::Shark)
            .map(|t| (if t.dx > 0.0 { t.x + t.width() as f64 - 3.0 } else { t.x + 2.0 }, t.y + mouth))
            .collect();
        self.things.retain(|t| {
            let (top, bottom) = (t.y, t.y + t.art.len() as f64 - 1.0);
            t.kind != Kind::Fish
                || !sharks.iter().any(|&(mx, my)| my >= top - 0.5 * u && my <= bottom + 0.5 * u && mx >= t.x - 1.0 && mx <= t.x + t.width() as f64)
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

    fn draw(&self, c: &mut Canvas, p: &mut Pixels) {
        c.clear();
        if self.art.pixels {
            self.draw_pixels(c, p);
        } else {
            self.draw_ascii(c);
        }
    }

    fn draw_pixels(&self, c: &mut Canvas, p: &mut Pixels) {
        p.clear();
        let (w, h) = (self.w, self.h);
        let a = self.art;
        let t = self.tick as f64;
        // Night sky and the moon.
        for y in 0..a.sky + 3 {
            let col = mix(Rgb(10, 14, 40), Rgb(50, 78, 135), y as f64 / (a.sky + 3) as f64);
            for x in 0..w {
                p.put(x, y, col);
            }
        }
        let moon = (w as f64 - 12.5, 4.5);
        // A crescent: the moon's disc without a second, offset disc.
        for y in 0..9 {
            for x in (moon.0 as i64 - 5)..(moon.0 as i64 + 5) {
                let (px, py) = (x as f64 + 0.5, y as f64 + 0.5);
                let lit = (px - moon.0).hypot(py - moon.1) <= 3.6;
                let shade = (px - moon.0 - 1.6).hypot(py - moon.1 + 0.7) <= 3.1;
                if lit && !shade {
                    p.put(x, y, Rgb(245, 235, 200));
                }
            }
        }
        // Water: deeper is darker; light ripples near the top; waves on the surface.
        for x in 0..w {
            let xf = x as f64;
            let wave = (xf * 0.35 + t * 0.12).sin() * 0.9 + (xf * 0.13 - t * 0.07).sin() * 0.8;
            let top = a.sky + wave.round() as i64;
            for y in top..h {
                let d = ((y - a.sky) as f64 / (h - a.sky) as f64).max(0.0);
                let mut col = mix(Rgb(45, 130, 200), Rgb(6, 30, 75), d.powf(0.8));
                let ray = ((xf * 0.09 + y as f64 * 0.045 - t * 0.02).sin() * (xf * 0.031 - y as f64 * 0.02 + t * 0.011).sin()).max(0.0);
                col = mix(col, Rgb(150, 215, 255), ray.powi(3) * 0.35 * (1.0 - d));
                if y <= top + 1 {
                    col = mix(col, Rgb(200, 235, 255), if y == top { 0.6 } else { 0.25 });
                }
                p.put(x, y, col);
            }
        }
        // Sand with a wavy top, and a few pebbles.
        for x in 0..w {
            let top = h - a.sand + ((x as f64 * 0.21).sin() * 0.8 + (x as f64 * 0.07).cos() * 0.7).round() as i64;
            for y in top..h {
                let n = hash(x, y);
                let col = match n % 23 {
                    0 => Rgb(140, 128, 115),
                    1 | 2 => Rgb(225, 200, 145),
                    k if k % 2 == 0 => Rgb(200, 175, 120),
                    _ => Rgb(182, 155, 102),
                };
                p.put(x, y, col);
            }
        }
        // Castle, bottom right.
        let cw = art_width(a.castle);
        let ch = a.castle.len() as i64;
        if w > cw + 10 && h > ch + a.surface + 6 {
            p.sprite(w - cw - 3, h - a.sand + 1 - ch, a.castle, |k| pix::palette(STONE, k));
        }
        // Seaweed: stalks that sway more towards the tip, with leaves.
        for (k, &(x, hgt)) in self.weeds.iter().enumerate() {
            let bottom = h - a.sand + 1;
            for j in 0..hgt {
                let f = j as f64 / hgt as f64;
                let sway = (j as f64 * 0.4 + t * 0.07 + k as f64 * 1.7).sin() * f * 1.8;
                let xx = x + sway.round() as i64;
                let col = mix(Rgb(20, 105, 45), Rgb(90, 200, 100), f);
                p.put(xx, bottom - j, col);
                if j % 4 == 2 {
                    p.put(xx + if (j / 4 + k as i64) % 2 == 0 { 1 } else { -1 }, bottom - j, mix(col, Rgb(140, 230, 120), 0.3));
                }
            }
        }
        // Creatures.
        for th in &self.things {
            let (x0, y0) = (th.x as i64, th.y as i64);
            p.sprite(x0, y0, &th.art, |k| pix::palette(th.color, k));
            if th.kind == Kind::Whale {
                let phase = (th.age / 3) as i64 % 18;
                if phase < 9 {
                    let hole = if th.dx > 0.0 { x0 + WHALE_PX_HOLE } else { x0 + th.width() - 1 - WHALE_PX_HOLE };
                    let top = y0 + 1;
                    let hgt = phase.min(5);
                    for j in 1..=hgt {
                        p.put(hole, top - j, Rgb(170, 220, 255));
                    }
                    let spray = (phase - 3).max(0) as f64;
                    for s in [-1.0, 1.0] {
                        for k in 0..3 {
                            let sx = hole as f64 + s * (1.0 + k as f64 * 0.8 + spray * 0.5);
                            let sy = (top - hgt) as f64 + k as f64 * 0.9 + spray * 0.4;
                            p.blend(sx.round() as i64, sy.round() as i64, Rgb(200, 235, 255), 0.8);
                        }
                    }
                }
            }
        }
        p.show(c);
        // Characters on top: stars and bubbles.
        for row in 0..(a.sky / 2 - 1) {
            for x in 0..w {
                let n = hash(x, row * 1000);
                if n % 41 == 0 && (moon.0 - x as f64).abs() > 5.0 {
                    let twinkle = (n / 41 + self.tick / 10) % 7 == 0;
                    let ch = if n % 3 == 0 { '✦' } else if twinkle { '∙' } else { '·' };
                    p.glyph(c, x, row, ch, Rgb(235, 235, 200));
                }
            }
        }
        for b in &self.bubbles {
            let row = b.y as i64 / 2;
            let ch = if b.y < h as f64 * 0.45 { '○' } else { '◦' };
            p.glyph(c, b.x, row, ch, Rgb(225, 245, 255));
        }
    }

    fn draw_ascii(&self, c: &mut Canvas) {
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

/// A fixed pseudo-random number per position (sand grains, stars).
fn hash(x: i64, y: i64) -> u64 {
    let mut n = (x as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15) ^ (y as u64).wrapping_mul(0xC2B2_AE3D_27D4_EB4F);
    n ^= n >> 29;
    n = n.wrapping_mul(0xBF58_476D_1CE4_E5B9);
    n ^ (n >> 32)
}

fn main() {
    let mut cli = Cli::new("rasciiquarium");
    let t = Lang::pick(cli.lang, &EN, &PT, &ES);
    let (mut speed, mut fish, mut unicode) = (5u32, 5u32, false);
    while let Some(a) = cli.next() {
        match a {
            Arg::Pos(p) => cli.unknown(&p),
            Arg::Opt(opt) => match opt.as_str() {
                "-h" | "--help" => cli.help(t.help),
                "-V" | "--version" => cli.version(),
                "-s" | "--speed" => speed = cli.number(&opt, 1..=10),
                "-n" | "--fish" => fish = cli.number(&opt, 1..=10),
                "-u" | "--unicode" => unicode = true,
                _ => cli.unknown(&opt),
            },
        }
    }
    if !std::io::stdout().is_terminal() {
        die("rasciiquarium", common(cli.lang).not_a_terminal);
    }
    let art = if unicode { &PIXEL_ART } else { &ASCII_ART };
    if let Err(e) = run(art, speed, fish) {
        die("rasciiquarium", &e.to_string());
    }
}

fn run(art: &'static Art, speed: u32, fish: u32) -> std::io::Result<()> {
    let depth = term::depth(Some(true));
    let _screen = Screen::enter()?;
    let mut out = std::io::stdout().lock();
    let (w, h) = term::size();
    let mut c = Canvas::new(w, h);
    let mut p = Pixels::new(w, h);
    let level = fish as f64 / 5.0;
    let mut tank = Tank::new(art, w, h, level, Rng::new());
    let factor = 0.3 + speed as f64 * 0.14;
    let frame = Duration::from_millis(60);
    let mut paused = false;
    loop {
        let start = Instant::now();
        if !paused {
            tank.step(factor);
        }
        tank.draw(&mut c, &mut p);
        c.flush(&mut out, depth)?;
        match tui::poll(frame.saturating_sub(start.elapsed()))? {
            Input::Quit => return Ok(()),
            Input::Key('p') | Input::Key(' ') => paused = !paused,
            Input::Key('r') => tank = Tank::new(art, c.w, c.h, level, Rng::new()),
            Input::Resize => {
                let (w, h) = term::size();
                c.resize(w, h);
                p = Pixels::new(w, h);
                tank = Tank::new(art, w, h, level, Rng::new());
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
        for art in [&ASCII_ART, &PIXEL_ART] {
            let mut t = Tank::new(art, 100, 30, 1.0, Rng::seeded(9));
            let mut c = Canvas::new(100, 30);
            let mut p = Pixels::new(100, 30);
            for _ in 0..3000 {
                t.step(1.0);
                t.draw(&mut c, &mut p);
            }
            assert!(t.things.iter().any(|x| x.kind == Kind::Fish));
        }
    }

    #[test]
    fn pixel_sprites_are_rectangles() {
        let a = &PIXEL_ART;
        let all = a.fish.iter().copied().chain([a.shark, a.whale, a.ship, a.jelly[0], a.jelly[1], a.castle]);
        for (k, s) in all.enumerate() {
            let w = s[0].chars().count();
            assert!(s.iter().all(|r| r.chars().count() == w), "sprite {k}");
        }
    }

    /// Prints a frame with colours:
    /// `SNAP=unicode SNAP_SEED=4 cargo test --bin rasciiquarium snapshot -- --ignored --nocapture`.
    #[test]
    #[ignore]
    fn snapshot() {
        let env = |k: &str, d: u64| std::env::var(k).ok().and_then(|v| v.parse().ok()).unwrap_or(d);
        let art = if std::env::var("SNAP").as_deref() == Ok("unicode") { &PIXEL_ART } else { &ASCII_ART };
        let (w, h) = (env("SNAP_W", 110) as usize, env("SNAP_H", 32) as usize);
        let mut t = Tank::new(art, w, h, 1.6, Rng::seeded(env("SNAP_SEED", 4)));
        // SNAP_KINDS=0123: shark, whale, ship, jelly; each one placed at SNAP_X.
        let kinds = std::env::var("SNAP_KINDS").unwrap_or_default();
        let mut c = Canvas::new(w, h);
        let mut p = Pixels::new(w, h);
        for _ in 0..env("SNAP_STEPS", 60) {
            t.step(1.0);
        }
        t.things.retain(|x| x.kind == Kind::Fish);
        for (i, k) in kinds.chars().enumerate() {
            t.add_visitor_kind(k.to_digit(10).unwrap_or(0) as usize);
            let v = t.things.last_mut().unwrap();
            v.x = (5 + i * 38) as f64;
            v.age = env("SNAP_AGE", 10);
        }
        t.draw(&mut c, &mut p);
        print!("{}", c.to_ansi(term::Depth::True));
    }
}

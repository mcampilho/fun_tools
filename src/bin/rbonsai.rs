//! rbonsai: grows a random bonsai tree in the terminal (like cbonsai), either
//! printed at once or growing live.

use fun::cli::{Arg, Cli};
use fun::i18n::{common, Lang};
use fun::pix::{self, mix};
use fun::rng::Rng;
use fun::term::{self, Depth, Rgb};
use std::io::{IsTerminal, Write};
use std::time::Duration;

struct Texts {
    help: &'static str,
}

const EN: Texts = Texts {
    help: "\
Usage: rbonsai [options]

Grows a random bonsai tree.

  -l, --live           Watch it grow (terminal only)
  -t, --time <s>       Seconds between growth steps in --live (default 0.03)
  -s, --seed <n>       Seed, to grow the same tree again (shown with -v)
  -L, --life <n>       Trunk length (default: from the height)
  -M, --multiplier <n> Branchiness, 1 to 10 (default 5)
  -b, --base <n>       Pot: 0 none, 1 large, 2 small (default 1)
  -W, --width <n>      Width (default: the terminal's, or 80)
  -H, --height <n>     Height (default: the terminal's, or 24)
  -u, --unicode        Unicode look: a colour pixel tree made of half blocks (needs
                       a font with them; best in 24-bit colour)
  -f, --foliage <name> With -u: green, autumn, blossom or random (default green)
  -v, --verbose        Print the seed at the end
      --color          Always use colours
      --no-color       Never use colours (also: NO_COLOR)
      --lang <code>    Language: en, pt, es (also: RTOOLS_LANG)
  -h, --help           Show this help
  -V, --version        Show the version
",
};

const PT: Texts = Texts {
    help: "\
Utilização: rbonsai [opções]

Faz crescer um bonsai ao acaso.

  -l, --live           Ver a árvore a crescer (só num terminal)
  -t, --time <s>       Segundos entre cada passo do --live (por omissão 0.03)
  -s, --seed <n>       Semente, para voltar a obter a mesma árvore (mostrada com -v)
  -L, --life <n>       Comprimento do tronco (por omissão: conforme a altura)
  -M, --multiplier <n> Quantidade de ramos, de 1 a 10 (por omissão 5)
  -b, --base <n>       Vaso: 0 nenhum, 1 grande, 2 pequeno (por omissão 1)
  -W, --width <n>      Largura (por omissão: a do terminal, ou 80)
  -H, --height <n>     Altura (por omissão: a do terminal, ou 24)
  -u, --unicode        Aspeto Unicode: uma árvore de pixels a cores feita de meios
                       blocos (precisa de uma fonte que os tenha; melhor a 24 bits)
  -f, --foliage <nome> Com -u: green (verde), autumn (outono), blossom (em flor) ou
                       random (ao acaso) (por omissão green)
  -v, --verbose        Mostrar a semente no fim
      --color          Usar sempre cores
      --no-color       Nunca usar cores (também: NO_COLOR)
      --lang <código>  Idioma: en, pt, es (também: RTOOLS_LANG)
  -h, --help           Mostrar esta ajuda
  -V, --version        Mostrar a versão
",
};

const ES: Texts = Texts {
    help: "\
Uso: rbonsai [opciones]

Hace crecer un bonsái al azar.

  -l, --live           Ver cómo crece (solo en un terminal)
  -t, --time <s>       Segundos entre cada paso de --live (por defecto 0.03)
  -s, --seed <n>       Semilla, para obtener el mismo árbol (se muestra con -v)
  -L, --life <n>       Longitud del tronco (por defecto: según la altura)
  -M, --multiplier <n> Cantidad de ramas, de 1 a 10 (por defecto 5)
  -b, --base <n>       Maceta: 0 ninguna, 1 grande, 2 pequeña (por defecto 1)
  -W, --width <n>      Anchura (por defecto: la del terminal, u 80)
  -H, --height <n>     Altura (por defecto: la del terminal, o 24)
  -u, --unicode        Aspecto Unicode: un árbol de píxeles en color hecho de medios
                       bloques (necesita una fuente que los tenga; mejor a 24 bits)
  -f, --foliage <nombre> Con -u: green (verde), autumn (otoño), blossom (en flor) o
                       random (al azar) (por defecto green)
  -v, --verbose        Mostrar la semilla al final
      --color          Usar siempre colores
      --no-color       No usar nunca colores (también: NO_COLOR)
      --lang <código>  Idioma: en, pt, es (también: RTOOLS_LANG)
  -h, --help           Mostrar esta ayuda
  -V, --version        Mostrar la versión
",
};

#[derive(Clone, Copy, PartialEq, Eq)]
enum Kind {
    Trunk,
    Branch,
    Leaf,
    LeafDark,
    Pot,
    Soil,
}

impl Kind {
    fn color(self) -> Rgb {
        match self {
            Kind::Trunk => Rgb(135, 85, 40),
            Kind::Branch => Rgb(175, 120, 60),
            Kind::Leaf => Rgb(110, 200, 70),
            Kind::LeafDark => Rgb(40, 150, 60),
            Kind::Pot => Rgb(200, 110, 80),
            Kind::Soil => Rgb(150, 110, 70),
        }
    }

    fn wood(self) -> bool {
        matches!(self, Kind::Trunk | Kind::Branch)
    }
}

const POTS: [&[&str]; 2] = [
    &[
        "(.,;~^~;.,.;~^~;.,.;~^)",
        " \\                   /",
        "  \\_________________/",
        "   `-'           `-'",
    ],
    &["(~.,;~.,~;.)", " \\_______/"],
];

type Cell = (char, Kind);

struct Tree {
    w: i64,
    h: i64,
    cells: Vec<Option<Cell>>,
    /// Every placement in growth order; `true` ends a growth step (for --live).
    log: Vec<(i64, i64, Cell, bool)>,
    rng: Rng,
    mult: f64,
    budget: usize,
}

impl Tree {
    fn set(&mut self, x: i64, y: i64, c: Cell, step_end: bool) {
        if x < 0 || y < 0 || x >= self.w || y >= self.h {
            return;
        }
        let i = (y * self.w + x) as usize;
        // Leaves never cover wood, and nothing covers the pot.
        match self.cells[i] {
            Some((_, k)) if k == Kind::Pot || k == Kind::Soil => return,
            Some((_, k)) if k.wood() && !c.1.wood() => return,
            _ => {}
        }
        self.cells[i] = Some(c);
        self.log.push((x, y, c, step_end));
    }

    fn trunk(&mut self, mut x: i64, mut y: i64, life: i64) {
        let mut drift = 0;
        for i in 0..life {
            if y <= 1 {
                break;
            }
            if i > 1 && self.rng.chance(0.35) {
                drift = self.rng.range(-1, 2);
            }
            let dx = if i < 2 { 0 } else { drift };
            let dy = if i > 2 && dx != 0 && self.rng.chance(0.2) { 0 } else { -1 };
            x = (x + dx).clamp(2, self.w - 3);
            y += dy;
            let ch = match (dx, dy) {
                (0, _) => '|',
                (-1, -1) => '\\',
                (1, -1) => '/',
                _ => '~',
            };
            // A thicker trunk near the ground.
            if i < life / 3 {
                let side = if dx < 0 { '\\' } else { '/' };
                self.set(x + 1, y, (if dx == 0 { '|' } else { side }, Kind::Trunk), false);
            }
            self.set(x, y, (ch, Kind::Trunk), true);
            let rem = life - i;
            if i > life / 5 && self.rng.chance(0.08 * self.mult) {
                let dir = if self.rng.chance(0.5) { 1 } else { -1 };
                let len = rem / 2 + self.rng.range(3, 8);
                self.branch(x, y, dir, len);
            }
        }
        // Crown: a few branches and leaves at the top.
        let crown = 2 + (self.mult / 2.0) as i64;
        for k in 0..crown {
            let dir = if k % 2 == 0 { 1 } else { -1 };
            let len = self.rng.range(3, 9);
            self.branch(x, y, dir, len);
        }
        self.leaves(x, y - 1, 3);
    }

    fn branch(&mut self, mut x: i64, mut y: i64, dir: i64, life: i64) {
        if self.budget == 0 {
            return;
        }
        self.budget -= 1;
        for i in 0..life {
            let dy = if self.rng.chance(0.35) { -1 } else { 0 };
            let dx = if dy == 0 || self.rng.chance(0.8) { dir } else { 0 };
            if x + dx < 1 || x + dx > self.w - 2 || y + dy < 1 {
                break;
            }
            x += dx;
            y += dy;
            let ch = match (dx, dy) {
                (0, _) => '|',
                (1, -1) => '/',
                (-1, -1) => '\\',
                _ => '~',
            };
            self.set(x, y, (ch, Kind::Branch), true);
            let rem = life - i;
            if rem > 5 && self.rng.chance(0.05 * self.mult) {
                let d = if self.rng.chance(0.8) { dir } else { -dir };
                self.branch(x, y, d, rem / 2);
            }
            if rem > 2 && self.rng.chance(0.12) {
                self.leaves(x, y - 1, 1);
            }
        }
        let r = self.rng.range(1, 3);
        self.leaves(x, y, r);
    }

    fn leaves(&mut self, cx: i64, cy: i64, r: i64) {
        const CHARS: &[char] = &['&', '*', '@', '%', '&', '*'];
        let (rx, ry) = (r * 2 + 1, r);
        for dy in -ry..=ry {
            for dx in -rx..=rx {
                let d = (dx as f64 / rx as f64).powi(2) + (dy as f64 / (ry as f64 + 0.5)).powi(2);
                if d <= 1.0 && self.rng.chance(0.8 - d * 0.3) {
                    let ch = *self.rng.pick(CHARS);
                    let kind = if self.rng.chance(0.5) { Kind::Leaf } else { Kind::LeafDark };
                    self.set(cx + dx, cy + dy, (ch, kind), false);
                }
            }
        }
        if let Some(last) = self.log.last_mut() {
            last.3 = true;
        }
    }
}

fn grow(w: i64, h: i64, seed: u64, life: Option<i64>, mult: f64, base: usize) -> Tree {
    let mut t = Tree {
        w,
        h,
        cells: vec![None; (w * h) as usize],
        log: Vec::new(),
        rng: Rng::seeded(seed),
        mult,
        budget: 60,
    };
    let pot: &[&str] = if base == 0 { &[] } else { POTS[base - 1] };
    let pot_w = pot.iter().map(|l| l.chars().count()).max().unwrap_or(0) as i64;
    let soil_y = h - pot.len() as i64;
    let px = (w - pot_w) / 2;
    for (j, line) in pot.iter().enumerate() {
        for (i, ch) in line.chars().enumerate() {
            if ch != ' ' {
                let kind = if j == 0 { Kind::Soil } else { Kind::Pot };
                t.set(px + i as i64, soil_y + j as i64, (ch, kind), false);
            }
        }
    }
    let life = life.unwrap_or(((soil_y as f64) * 0.55).round() as i64).max(3);
    t.trunk(w / 2, soil_y, life);
    t
}

fn paint(c: Option<Cell>, depth: Depth) -> String {
    match c {
        None => " ".into(),
        Some((ch, _)) if depth == Depth::None => ch.to_string(),
        Some((ch, k)) => {
            let bold = if k.wood() { "\x1b[1m" } else { "" };
            format!("{bold}{}{ch}{}", term::fg(depth, k.color()), term::RESET)
        }
    }
}

/// The whole tree as text, without empty rows above it or spaces at the line ends.
fn render(t: &Tree, depth: Depth) -> String {
    let mut out = String::new();
    let mut started = false;
    for y in 0..t.h {
        let row = &t.cells[(y * t.w) as usize..((y + 1) * t.w) as usize];
        let end = row.iter().rposition(Option::is_some).map_or(0, |p| p + 1);
        if end == 0 && !started {
            continue;
        }
        started = true;
        for c in &row[..end] {
            out.push_str(&paint(*c, depth));
        }
        out.push('\n');
    }
    out
}

/// Replays the growth in place: reserves the rows, then draws each placement.
fn live(t: &Tree, depth: Depth, step: Duration) {
    let mut out = std::io::stdout().lock();
    let top = (0..t.h).find(|&y| t.cells[(y * t.w) as usize..((y + 1) * t.w) as usize].iter().any(Option::is_some));
    let top = top.unwrap_or(0);
    let rows = t.h - top;
    // Make room, go back up and remember the corner.
    let _ = write!(out, "{}\x1b[{rows}A\r\x1b7", "\n".repeat(rows as usize));
    let mut frame = String::new();
    for &(x, y, c, end) in &t.log {
        frame.push_str("\x1b8");
        if y > top {
            frame.push_str(&format!("\x1b[{}B", y - top));
        }
        if x > 0 {
            frame.push_str(&format!("\x1b[{x}C"));
        }
        frame.push_str(&paint(Some(c), depth));
        if end {
            let _ = out.write_all(frame.as_bytes()).and_then(|_| out.flush());
            frame.clear();
            std::thread::sleep(step);
        }
    }
    let _ = write!(out, "{frame}\x1b8\x1b[{rows}B\r");
    let _ = out.flush();
}

// ---------------------------------------------------------------------------
// The Unicode look: a pixel tree, two square pixels per cell (half blocks).

/// Leaf colours, darkest first.
const FOLIAGE: [(&str, [Rgb; 4]); 3] = [
    ("green", [Rgb(28, 88, 38), Rgb(48, 128, 48), Rgb(84, 168, 60), Rgb(145, 205, 92)]),
    ("autumn", [Rgb(125, 38, 22), Rgb(185, 68, 26), Rgb(228, 118, 36), Rgb(250, 186, 72)]),
    ("blossom", [Rgb(178, 72, 112), Rgb(225, 125, 160), Rgb(248, 175, 200), Rgb(255, 225, 235)]),
];
const POT_COLORS: [Rgb; 4] = [Rgb(170, 85, 55), Rgb(45, 75, 125), Rgb(115, 72, 45), Rgb(105, 155, 135)];
const BARK_DARK: Rgb = Rgb(70, 42, 24);
const BARK_LIGHT: Rgb = Rgb(165, 115, 70);

struct PxTree {
    w: i64,
    h: i64,
    px: Vec<Option<Rgb>>,
    /// Every pixel in growth order; `true` ends a growth step (for --live).
    log: Vec<(i64, i64, Rgb, bool)>,
    rng: Rng,
    mult: f64,
    budget: usize,
    leaf: [Rgb; 4],
    /// The side of the trunk's last branch (they alternate).
    trunk_side: f64,
}

impl PxTree {
    fn put(&mut self, x: i64, y: i64, c: Rgb) {
        if x < 0 || y < 0 || x >= self.w || y >= self.h {
            return;
        }
        let i = (y * self.w + x) as usize;
        if self.px[i] != Some(c) {
            self.px[i] = Some(c);
            self.log.push((x, y, c, false));
        }
    }

    fn end_step(&mut self) {
        if let Some(last) = self.log.last_mut() {
            last.3 = true;
        }
    }

    /// A round piece of wood, lit from the left.
    fn stamp(&mut self, cx: f64, cy: f64, r: f64) {
        let r = r.max(0.6);
        let (x0, x1) = ((cx - r).floor() as i64, (cx + r).ceil() as i64);
        let (y0, y1) = ((cy - r).floor() as i64, (cy + r).ceil() as i64);
        for y in y0..=y1 {
            for x in x0..=x1 {
                let (dx, dy) = (x as f64 + 0.5 - cx, y as f64 + 0.5 - cy);
                if dx.hypot(dy) <= r {
                    let t = 0.75 - dx / r * 0.45 + if (x * 7 + y * 3) % 11 == 0 { -0.25 } else { 0.0 };
                    self.put(x, y, mix(BARK_DARK, BARK_LIGHT, t));
                }
            }
        }
    }

    /// A trunk or branch from (x, y) towards `angle`, getting thinner, with side
    /// branches and a pad of leaves at the end.
    fn limb(&mut self, mut x: f64, mut y: f64, mut angle: f64, len: f64, thick: f64, depth: u32) {
        if self.budget == 0 {
            return;
        }
        self.budget -= 1;
        let n = len.max(2.0) as usize;
        let side = if angle.cos() >= 0.0 { 1.0 } else { -1.0 };
        let mut last_fork = 0;
        let phase = self.rng.float() * 6.0;
        for i in 0..n {
            let f = i as f64 / n as f64;
            if depth == 0 {
                // The trunk: an S-curve upwards.
                angle = -std::f64::consts::FRAC_PI_2 + (f * 4.2 + phase).sin() * 0.55 + (self.rng.float() - 0.5) * 0.15;
            } else {
                // Branches wander and bend out towards the horizontal.
                angle += (self.rng.float() - 0.5) * 0.3;
                let flat = if side > 0.0 { -0.15 } else { std::f64::consts::PI + 0.15 };
                angle += (flat - angle) * 0.04;
            }
            x += angle.cos();
            y += angle.sin();
            if x < 1.0 || x > (self.w - 2) as f64 || y < 1.0 {
                break;
            }
            let r = thick * (1.0 - f * 0.6);
            self.stamp(x, y, r);
            self.end_step();
            // The trunk forks at regular steps up its top part, branches now and then.
            let forks = if depth == 0 {
                f > 0.22 && f < 0.9 && (i - last_fork) as f64 > n as f64 / (2.0 + self.mult * 0.6) && self.rng.chance(0.5)
            } else {
                depth < 3 && i - last_fork > 4 && f > 0.25 && self.rng.chance(0.03 * self.mult)
            };
            if forks {
                self.trunk_side = -self.trunk_side;
                last_fork = i;
                let dir = if depth == 0 { self.trunk_side } else if self.rng.chance(0.75) { side } else { -side };
                let a = -std::f64::consts::FRAC_PI_2 + dir * (1.05 + self.rng.float() * 0.35);
                let sub = if depth == 0 { len * (0.6 + self.rng.float() * 0.35) * (1.15 - f * 0.55) } else { (len * (1.0 - f)).max(5.0) * 0.6 };
                self.limb(x, y, a, sub, (r * 0.6).max(0.7), depth + 1);
            }
        }
        let size = (2.0 + thick * 1.2 + len * 0.06).min(9.0);
        self.pad(x, y - size * 0.3, size * 1.6, size * 0.5);
        if depth == 0 {
            // The crown.
            for dir in [-1.0, 1.0] {
                let a = -std::f64::consts::FRAC_PI_2 + dir * (0.7 + self.rng.float() * 0.5);
                self.limb(x, y, a, len * 0.25 + 4.0, (thick * 0.4).max(0.8), 2);
            }
        }
    }

    /// A flattened cloud of leaves: lighter on top, darker underneath.
    fn pad(&mut self, cx: f64, cy: f64, rx: f64, ry: f64) {
        let n = (rx * ry * 0.9) as usize + 6;
        for _ in 0..n {
            let a = self.rng.float() * std::f64::consts::TAU;
            let d = self.rng.float().sqrt();
            let (x, y) = (cx + a.cos() * d * rx, cy + a.sin() * d * ry);
            let up = ((cy + ry - y) / (2.0 * ry)).clamp(0.0, 1.0);
            let k = ((up * 3.2 + self.rng.float() * 1.1) as usize).min(3);
            let r = 0.8 + self.rng.float() * 1.3;
            let col = self.leaf[k];
            let (x0, x1) = ((x - r).floor() as i64, (x + r).ceil() as i64);
            for py in (y - r).floor() as i64..=(y + r).ceil() as i64 {
                for px in x0..=x1 {
                    if (px as f64 + 0.5 - x).hypot(py as f64 + 0.5 - y) <= r {
                        self.put(px, py, col);
                    }
                }
            }
        }
        self.end_step();
    }
}

/// Draws the pot (if any) and returns the pixel row of the soil's top.
fn px_pot(t: &mut PxTree, base: usize, color: Rgb) -> i64 {
    let (w, h) = (t.w, t.h);
    if base == 0 {
        return h;
    }
    let big = base == 1;
    let pw = if big { (w / 2).clamp(18, 64) } else { (w / 4).clamp(12, 28) };
    let ph = if big { 8 } else { 10 };
    let x0 = (w - pw) / 2;
    let top = h - ph;
    let light = mix(color, Rgb(255, 255, 255), 0.35);
    let dark = pix::scale(color, 0.6);
    for j in 0..ph {
        // A tray with feet, or a round bowl.
        let inset = if big { if j >= ph - 2 { 2 } else { 0 } } else { (j * j / 12).min(pw / 3) };
        for i in inset..pw - inset {
            if big && j >= ph - 2 && i > 4 && i < pw - 5 {
                continue; // between the feet
            }
            let f = i as f64 / pw as f64;
            let mut c = mix(light, dark, f * 0.9);
            if j == 0 || j == 1 {
                c = if j == 0 { mix(light, Rgb(255, 255, 255), 0.2) } else { dark };
            }
            t.put(x0 + i, top + j, c);
        }
    }
    // Soil and moss in the rim.
    for i in 1..pw - 1 {
        let n = (i * 7 + 3) % 9;
        let c = match n {
            0 | 4 => Rgb(80, 130, 60),
            1 => Rgb(110, 160, 75),
            _ => Rgb(72, 50, 34),
        };
        t.put(x0 + i, top - 1, c);
        if n == 0 {
            t.put(x0 + i, top - 2, Rgb(95, 150, 65));
        }
    }
    t.end_step();
    top - 1
}

fn grow_px(w: i64, rows: i64, seed: u64, life: Option<i64>, mult: f64, base: usize, foliage: usize) -> PxTree {
    let h = rows * 2;
    let mut rng = Rng::seeded(seed);
    let leaf_kind = if foliage < FOLIAGE.len() { foliage } else { rng.below(FOLIAGE.len()) };
    let pot = POT_COLORS[rng.below(POT_COLORS.len())];
    let mut t = PxTree { w, h, px: vec![None; (w * h) as usize], log: Vec::new(), rng, mult, budget: 70, leaf: FOLIAGE[leaf_kind].1, trunk_side: 1.0 };
    let soil = px_pot(&mut t, base, pot);
    let len = life.map(|l| l as f64 * 2.0).unwrap_or(soil as f64 * 0.6).max(8.0);
    let thick = (w as f64 / 28.0).clamp(2.2, 4.0) * (len / 30.0).clamp(0.7, 1.25);
    // Roots spread over the soil.
    let cx = w as f64 / 2.0;
    for k in 0..(thick * 2.0) as i64 {
        let off = k as f64 * 0.8;
        t.stamp(cx - off - 1.0, soil as f64 - 0.5, 0.8);
        t.stamp(cx + off + 1.0, soil as f64 - 0.5, 0.8);
    }
    t.limb(cx, soil as f64, -std::f64::consts::FRAC_PI_2, len, thick, 0);
    t
}

/// One cell of pixel art: its two pixels as half blocks.
fn px_cell(top: Option<Rgb>, bottom: Option<Rgb>, depth: Depth) -> String {
    if depth == Depth::None {
        return match (top, bottom) {
            (None, None) => " ",
            (Some(_), None) => "▀",
            (None, Some(_)) => "▄",
            _ => "█",
        }
        .into();
    }
    match (top, bottom) {
        (None, None) => " ".into(),
        (Some(t), None) => format!("{}▀{}", term::fg(depth, t), term::RESET),
        (None, Some(b)) => format!("{}▄{}", term::fg(depth, b), term::RESET),
        (Some(t), Some(b)) if t == b => format!("{} {}", term::color(depth, t, true), term::RESET),
        (Some(t), Some(b)) => format!("{}{}▀{}", term::fg(depth, t), term::color(depth, b, true), term::RESET),
    }
}

impl PxTree {
    fn get(&self, x: i64, y: i64) -> Option<Rgb> {
        (y < self.h).then(|| self.px[(y * self.w + x) as usize]).flatten()
    }

    /// The first cell row with something in it.
    fn top_row(&self) -> i64 {
        (0..self.h).find(|&y| (0..self.w).any(|x| self.px[(y * self.w + x) as usize].is_some())).unwrap_or(0) / 2
    }

    fn render(&self, depth: Depth) -> String {
        let mut out = String::new();
        for row in self.top_row()..(self.h + 1) / 2 {
            let end = (0..self.w).rev().find(|&x| self.get(x, 2 * row).is_some() || self.get(x, 2 * row + 1).is_some()).map_or(0, |p| p + 1);
            for x in 0..end {
                out.push_str(&px_cell(self.get(x, 2 * row), self.get(x, 2 * row + 1), depth));
            }
            out.push('\n');
        }
        out
    }

    /// Replays the growth in place, one step at a time.
    fn live(&self, depth: Depth, step: Duration) {
        let mut out = std::io::stdout().lock();
        let top = self.top_row();
        let rows = (self.h + 1) / 2 - top;
        let _ = write!(out, "{}\x1b[{rows}A\r\x1b7", "\n".repeat(rows as usize));
        let mut shown = PxTree { w: self.w, h: self.h, px: vec![None; self.px.len()], log: Vec::new(), rng: Rng::seeded(0), mult: 0.0, budget: 0, leaf: self.leaf, trunk_side: 1.0 };
        let mut dirty: Vec<(i64, i64)> = Vec::new();
        let mut frame = String::new();
        for (n, &(x, y, c, end)) in self.log.iter().enumerate() {
            shown.px[(y * self.w + x) as usize] = Some(c);
            if !dirty.contains(&(x, y / 2)) {
                dirty.push((x, y / 2));
            }
            if end || n + 1 == self.log.len() {
                for &(cx, row) in &dirty {
                    frame.push_str("\x1b8");
                    if row > top {
                        frame.push_str(&format!("\x1b[{}B", row - top));
                    }
                    if cx > 0 {
                        frame.push_str(&format!("\x1b[{cx}C"));
                    }
                    frame.push_str(&px_cell(shown.get(cx, 2 * row), shown.get(cx, 2 * row + 1), depth));
                }
                dirty.clear();
                let _ = out.write_all(frame.as_bytes()).and_then(|_| out.flush());
                frame.clear();
                std::thread::sleep(step);
            }
        }
        let _ = write!(out, "\x1b8\x1b[{rows}B\r");
        let _ = out.flush();
    }
}

fn main() {
    let mut cli = Cli::new("rbonsai");
    let t = Lang::pick(cli.lang, &EN, &PT, &ES);
    let (mut live_mode, mut step, mut seed, mut life) = (false, 0.03f64, None::<u64>, None::<i64>);
    let (mut mult, mut base, mut width, mut height) = (5.0f64, 1usize, None::<i64>, None::<i64>);
    let (mut verbose, mut color, mut unicode, mut foliage) = (false, None::<bool>, false, 0usize);
    while let Some(a) = cli.next() {
        match a {
            Arg::Pos(p) => cli.unknown(&p),
            Arg::Opt(opt) => match opt.as_str() {
                "-h" | "--help" => cli.help(t.help),
                "-V" | "--version" => cli.version(),
                "-l" | "--live" => live_mode = true,
                "-t" | "--time" => step = cli.number(&opt, 0.0..=10.0),
                "-s" | "--seed" => seed = Some(cli.parse(&opt)),
                "-L" | "--life" => life = Some(cli.number(&opt, 3..=1000)),
                "-M" | "--multiplier" => mult = cli.number(&opt, 1.0..=10.0),
                "-b" | "--base" => base = cli.number(&opt, 0..=2),
                "-W" | "--width" => width = Some(cli.number(&opt, 20..=1000)),
                "-H" | "--height" => height = Some(cli.number(&opt, 8..=1000)),
                "-v" | "--verbose" => verbose = true,
                "-u" | "--unicode" => unicode = true,
                "-f" | "--foliage" => {
                    let v = cli.value(&opt);
                    foliage = match v.as_str() {
                        "random" => FOLIAGE.len(),
                        _ => FOLIAGE.iter().position(|(n, _)| *n == v).unwrap_or_else(|| {
                            cli.fail(&format!("{} '{opt}': {v}", common(cli.lang).invalid_value))
                        }),
                    };
                }
                "--color" => color = Some(true),
                "--no-color" => color = Some(false),
                _ => cli.unknown(&opt),
            },
        }
    }
    let tty = std::io::stdout().is_terminal();
    let (tw, th) = if tty { term::size() } else { (80, 24) };
    let w = width.unwrap_or(tw as i64);
    // Leave a line for the prompt.
    let h = height.unwrap_or((th as i64 - 1).max(8));
    let seed = seed.unwrap_or_else(|| Rng::new().next_u64() % 1_000_000);
    let depth = term::depth(color);
    if unicode {
        let tree = grow_px(w, h, seed, life, mult, base, foliage);
        if live_mode && tty {
            // Moving the cursor needs escape codes even without colours (NO_COLOR).
            term::enable_ansi();
            tree.live(depth, Duration::from_secs_f64(step / 4.0));
        } else {
            let _ = std::io::stdout().write_all(tree.render(depth).as_bytes());
        }
        if verbose {
            println!("seed: {seed}");
        }
        return;
    }
    let tree = grow(w, h, seed, life, mult, base);
    if live_mode && tty {
            // Moving the cursor needs escape codes even without colours (NO_COLOR).
            term::enable_ansi();
        live(&tree, depth, Duration::from_secs_f64(step));
    } else {
        let _ = std::io::stdout().write_all(render(&tree, depth).as_bytes());
    }
    if verbose {
        println!("seed: {seed}");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn repeatable_and_inside() {
        let a = render(&grow(80, 24, 42, None, 5.0, 1), Depth::None);
        let b = render(&grow(80, 24, 42, None, 5.0, 1), Depth::None);
        assert_eq!(a, b);
        assert!(a.lines().count() <= 24);
        assert!(a.lines().all(|l| l.chars().count() <= 80));
        assert!(a.contains("\\___"));
        for f in 0..=FOLIAGE.len() {
            let t = grow_px(80, 24, 7, None, 5.0, 1, f);
            assert!(t.render(Depth::True).lines().count() <= 24);
            assert_eq!(t.render(Depth::None), grow_px(80, 24, 7, None, 5.0, 1, f).render(Depth::None));
        }
    }

    /// `SNAP_SEED=3 cargo test --bin rbonsai snapshot -- --ignored --nocapture`
    #[test]
    #[ignore]
    fn snapshot() {
        let env = |k: &str, d: u64| std::env::var(k).ok().and_then(|v| v.parse().ok()).unwrap_or(d);
        let t = grow_px(env("SNAP_W", 100) as i64, env("SNAP_H", 30) as i64, env("SNAP_SEED", 3), None, env("SNAP_M", 5) as f64, env("SNAP_BASE", 1) as usize, env("SNAP_F", 0) as usize);
        print!("{}", t.render(Depth::True));
    }

    #[test]
    fn trunk_touches_the_soil() {
        for seed in 0..20 {
            let t = grow(60, 20, seed, None, 5.0, 1);
            let soil_y = 20 - POTS[0].len() as i64;
            let above = &t.cells[((soil_y - 1) * 60) as usize..(soil_y * 60) as usize];
            assert!(above.iter().any(|c| matches!(c, Some((_, Kind::Trunk)))), "seed {seed}");
        }
    }
}

//! rbonsai: grows a random bonsai tree in the terminal (like cbonsai), either
//! printed at once or growing live.

use fun::cli::{Arg, Cli};
use fun::i18n::Lang;
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

fn main() {
    let mut cli = Cli::new("rbonsai");
    let t = Lang::pick(cli.lang, &EN, &PT, &ES);
    let (mut live_mode, mut step, mut seed, mut life) = (false, 0.03f64, None::<u64>, None::<i64>);
    let (mut mult, mut base, mut width, mut height) = (5.0f64, 1usize, None::<i64>, None::<i64>);
    let (mut verbose, mut color) = (false, None::<bool>);
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
    let tree = grow(w, h, seed, life, mult, base);
    if live_mode && tty {
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

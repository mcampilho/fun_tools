//! rblocks: a falling-blocks game in the terminal. Its default mode is evil
//! (like bastet): each new piece is the one that fits worst.

use fun::cli::{die, Arg, Cli};
use fun::i18n::{common, Lang};
use fun::pix::{mix, scale, Pixels};
use fun::rng::Rng;
use fun::scores;
use fun::term::{self, Depth, Rgb};
use fun::tui::{self, Canvas, Input, Screen};
use std::io::IsTerminal;
use std::time::{Duration, Instant};

struct Texts {
    help: &'static str,
    score: &'static str,
    lines: &'static str,
    level: &'static str,
    next: &'static str,
    best: &'static str,
    mode: [&'static str; 3],
    paused: &'static str,
    over: &'static str,
    again: &'static str,
    small: &'static str,
    keys: &'static [&'static str],
    table: &'static str,
}

const EN: Texts = Texts {
    help: "\
Usage: rblocks [options]

Falling blocks: fill whole rows to clear them. In the evil mode (the default,
like bastet) every new piece is the one that suits you least.

Keys: ← → or a d move, ↑ or w rotate, z rotate back, ↓ or s drop faster,
space drop at once, p pause, q or Esc quit.

  -m, --mode <name>    evil (default), random or nice (the most helpful piece)
  -L, --level <n>      Starting level, 0 to 15 (default 0)
  -G, --no-ghost       Do not show where the piece will land
  -S, --scores         Show the high scores and exit
      --lang <code>    Language: en, pt, es (also: RTOOLS_LANG)
  -h, --help           Show this help
  -V, --version        Show the version
",
    score: "Score",
    lines: "Lines",
    level: "Level",
    next: "Next",
    best: "Best",
    mode: ["evil", "random", "nice"],
    paused: "Paused: p to go on",
    over: "Game over!",
    again: "n new game, q quit",
    small: "The terminal is too small: at least 44 × 23.",
    keys: &["← → move", "↑ rotate  z back", "↓ faster", "space drop", "p pause  q quit"],
    table: "High scores",
};

const PT: Texts = Texts {
    help: "\
Utilização: rblocks [opções]

Blocos a cair: complete linhas inteiras para as limpar. No modo maldoso (o modo
por omissão, como no bastet) cada peça nova é a que menos lhe convém.

Teclas: ← → ou a d mover, ↑ ou w rodar, z rodar ao contrário, ↓ ou s descer
mais depressa, espaço largar, p pausa, q ou Esc sair.

  -m, --mode <nome>    evil (maldoso, por omissão), random (ao acaso) ou nice
                       (a peça que mais ajuda)
  -L, --level <n>      Nível inicial, de 0 a 15 (por omissão 0)
  -G, --no-ghost       Não mostrar onde a peça vai cair
  -S, --scores         Mostrar os recordes e sair
      --lang <código>  Idioma: en, pt, es (também: RTOOLS_LANG)
  -h, --help           Mostrar esta ajuda
  -V, --version        Mostrar a versão
",
    score: "Pontos",
    lines: "Linhas",
    level: "Nível",
    next: "Próxima",
    best: "Recorde",
    mode: ["maldoso", "ao acaso", "simpático"],
    paused: "Pausa: p para continuar",
    over: "Fim do jogo!",
    again: "n novo jogo, q sair",
    small: "O terminal é pequeno demais: pelo menos 44 × 23.",
    keys: &["← → mover", "↑ rodar  z voltar", "↓ mais depressa", "espaço largar", "p pausa  q sair"],
    table: "Recordes",
};

const ES: Texts = Texts {
    help: "\
Uso: rblocks [opciones]

Bloques que caen: complete filas enteras para borrarlas. En el modo malvado (el
modo por defecto, como en bastet) cada pieza nueva es la que menos le conviene.

Teclas: ← → o a d mover, ↑ o w girar, z girar al revés, ↓ o s bajar más rápido,
espacio soltar, p pausa, q o Esc salir.

  -m, --mode <nombre>  evil (malvado, por defecto), random (al azar) o nice
                       (la pieza que más ayuda)
  -L, --level <n>      Nivel inicial, de 0 a 15 (por defecto 0)
  -G, --no-ghost       No mostrar dónde caerá la pieza
  -S, --scores         Mostrar los récords y salir
      --lang <código>  Idioma: en, pt, es (también: RTOOLS_LANG)
  -h, --help           Mostrar esta ayuda
  -V, --version        Mostrar la versión
",
    score: "Puntos",
    lines: "Líneas",
    level: "Nivel",
    next: "Siguiente",
    best: "Récord",
    mode: ["malvado", "al azar", "amable"],
    paused: "Pausa: p para seguir",
    over: "¡Fin del juego!",
    again: "n juego nuevo, q salir",
    small: "El terminal es demasiado pequeño: al menos 44 × 23.",
    keys: &["← → mover", "↑ girar  z volver", "↓ más rápido", "espacio soltar", "p pausa  q salir"],
    table: "Récords",
};

const W: i64 = 10;
const H: i64 = 20;
const MODES: [&str; 3] = ["evil", "random", "nice"];

/// The seven pieces of four blocks, as cells in their box, with their colours.
const PIECES: [(&[(i64, i64)], i64, Rgb); 7] = [
    (&[(0, 1), (1, 1), (2, 1), (3, 1)], 4, Rgb(80, 190, 215)),   // I
    (&[(0, 0), (1, 0), (0, 1), (1, 1)], 2, Rgb(230, 190, 70)),   // O
    (&[(1, 0), (0, 1), (1, 1), (2, 1)], 3, Rgb(170, 105, 215)),  // T
    (&[(1, 0), (2, 0), (0, 1), (1, 1)], 3, Rgb(105, 195, 105)),  // S
    (&[(0, 0), (1, 0), (1, 1), (2, 1)], 3, Rgb(220, 85, 90)),    // Z
    (&[(2, 0), (0, 1), (1, 1), (2, 1)], 3, Rgb(235, 140, 60)),   // L
    (&[(0, 0), (0, 1), (1, 1), (2, 1)], 3, Rgb(85, 125, 225)),   // J
];

fn cells(kind: usize, rot: usize) -> [(i64, i64); 4] {
    let (base, n, _) = PIECES[kind];
    let mut out = [(0, 0); 4];
    for (i, &(x, y)) in base.iter().enumerate() {
        let (mut x, mut y) = (x, y);
        for _ in 0..rot % 4 {
            (x, y) = (n - 1 - y, x);
        }
        out[i] = (x, y);
    }
    out
}

#[derive(Clone, Copy)]
struct Piece {
    kind: usize,
    rot: usize,
    x: i64,
    y: i64,
}

impl Piece {
    fn spawn(kind: usize) -> Piece {
        Piece { kind, rot: 0, x: (W - PIECES[kind].1) / 2, y: -1 }
    }

    fn blocks(&self) -> impl Iterator<Item = (i64, i64)> + '_ {
        cells(self.kind, self.rot).into_iter().map(|(x, y)| (self.x + x, self.y + y))
    }
}

#[derive(Clone)]
struct Board {
    cells: Vec<Option<usize>>,
}

impl Board {
    fn new() -> Board {
        Board { cells: vec![None; (W * H) as usize] }
    }

    fn get(&self, x: i64, y: i64) -> Option<usize> {
        if y < 0 { None } else { self.cells[(y * W + x) as usize] }
    }

    fn fits(&self, p: &Piece) -> bool {
        p.blocks().all(|(x, y)| (0..W).contains(&x) && y < H && (y < 0 || self.get(x, y).is_none()))
    }

    /// Fixes the piece in place and clears full rows; returns how many, or None
    /// if the piece is above the top (game over).
    fn lock(&mut self, p: &Piece) -> Option<u32> {
        let mut out = false;
        for (x, y) in p.blocks() {
            if y < 0 {
                out = true;
            } else {
                self.cells[(y * W + x) as usize] = Some(p.kind);
            }
        }
        let mut cleared = 0;
        let mut y = H - 1;
        while y >= 0 {
            if (0..W).all(|x| self.get(x, y).is_some()) {
                self.cells.drain((y * W) as usize..((y + 1) * W) as usize);
                self.cells.splice(0..0, std::iter::repeat_n(None, W as usize));
                cleared += 1;
            } else {
                y -= 1;
            }
        }
        if out { None } else { Some(cleared) }
    }

    fn drop_y(&self, p: &Piece) -> Piece {
        let mut q = *p;
        while self.fits(&Piece { y: q.y + 1, ..q }) {
            q.y += 1;
        }
        q
    }

    /// How good the board is for the player (higher is better).
    fn rate(&self, cleared: u32) -> f64 {
        let mut heights = [0i64; W as usize];
        let mut holes = 0;
        for x in 0..W {
            let mut seen = false;
            for y in 0..H {
                if self.get(x, y).is_some() {
                    if !seen {
                        heights[x as usize] = H - y;
                        seen = true;
                    }
                } else if seen {
                    holes += 1;
                }
            }
        }
        let total: i64 = heights.iter().sum();
        let bumps: i64 = heights.windows(2).map(|w| (w[0] - w[1]).abs()).sum();
        let top = *heights.iter().max().unwrap_or(&0);
        -0.51 * total as f64 + 0.76 * cleared as f64 - 0.36 * holes as f64 - 0.18 * bumps as f64 - if top > H - 4 { 20.0 } else { 0.0 }
    }

    /// The best rating the player can reach with this piece.
    fn best_with(&self, kind: usize) -> f64 {
        let mut best = f64::NEG_INFINITY;
        for rot in 0..4 {
            for x in -2..W {
                let p = Piece { kind, rot, x, y: -1 };
                if !self.fits(&p) {
                    continue;
                }
                let landed = self.drop_y(&p);
                let mut b = self.clone();
                match b.lock(&landed) {
                    Some(c) => best = best.max(b.rate(c)),
                    None => best = best.max(-1000.0),
                }
            }
        }
        best
    }
}

/// Chooses the pieces: evil (the worst for the player), random (shuffled sets of
/// seven) or nice (the best).
struct Chooser {
    mode: usize,
    bag: Vec<usize>,
    rng: Rng,
    last: Vec<usize>,
}

impl Chooser {
    fn next(&mut self, board: &Board) -> usize {
        let k = if self.mode == 1 {
            if self.bag.is_empty() {
                self.bag = (0..7).collect();
                for i in (1..7).rev() {
                    let j = self.rng.below(i + 1);
                    self.bag.swap(i, j);
                }
            }
            self.bag.pop().unwrap_or(0)
        } else {
            let mut rated: Vec<(f64, usize)> = (0..7).map(|k| (board.best_with(k), k)).collect();
            if self.mode == 0 {
                rated.sort_by(|a, b| a.0.total_cmp(&b.0));
            } else {
                rated.sort_by(|a, b| b.0.total_cmp(&a.0));
            }
            // Ties at random; and the same piece at most three times in a row.
            let first = rated[0].0;
            let ties: Vec<usize> = rated.iter().filter(|r| (r.0 - first).abs() < 1e-9).map(|r| r.1).collect();
            let mut k = *self.rng.pick(&ties);
            if self.last.len() >= 3 && self.last.iter().rev().take(3).all(|&l| l == k) {
                k = rated.iter().map(|r| r.1).find(|&o| o != k).unwrap_or(k);
            }
            k
        };
        self.last.push(k);
        if self.last.len() > 3 {
            self.last.remove(0);
        }
        k
    }
}

struct Game {
    board: Board,
    piece: Piece,
    next: Option<usize>,
    chooser: Chooser,
    score: u64,
    lines: u32,
    start_level: u32,
    over: bool,
}

impl Game {
    fn new(mode: usize, start_level: u32, rng: Rng) -> Game {
        let board = Board::new();
        let mut chooser = Chooser { mode, bag: Vec::new(), rng, last: Vec::new() };
        let first = chooser.next(&board);
        let next = (mode == 1).then(|| chooser.next(&board));
        Game { board, piece: Piece::spawn(first), next, chooser, score: 0, lines: 0, start_level, over: false }
    }

    fn level(&self) -> u32 {
        self.start_level + self.lines / 10
    }

    fn gravity(&self) -> Duration {
        Duration::from_millis((800.0 * 0.85f64.powi(self.level() as i32)).max(40.0) as u64)
    }

    fn shift(&mut self, dx: i64, dy: i64) -> bool {
        let p = Piece { x: self.piece.x + dx, y: self.piece.y + dy, ..self.piece };
        let ok = self.board.fits(&p);
        if ok {
            self.piece = p;
        }
        ok
    }

    fn rotate(&mut self, turns: usize) {
        let rot = (self.piece.rot + turns) % 4;
        // Nudges when the turned piece hits a wall or a block.
        for (dx, dy) in [(0, 0), (-1, 0), (1, 0), (-2, 0), (2, 0), (0, -1)] {
            let p = Piece { rot, x: self.piece.x + dx, y: self.piece.y + dy, ..self.piece };
            if self.board.fits(&p) {
                self.piece = p;
                return;
            }
        }
    }

    fn lock(&mut self) {
        match self.board.lock(&self.piece) {
            None => self.over = true,
            Some(n) => {
                self.score += [0, 100, 300, 500, 800][n as usize] * (self.level() as u64 + 1);
                self.lines += n;
                let kind = match self.next {
                    Some(k) => {
                        self.next = Some(self.chooser.next(&self.board));
                        k
                    }
                    None => self.chooser.next(&self.board),
                };
                self.piece = Piece::spawn(kind);
                if !self.board.fits(&self.piece) {
                    self.over = true;
                }
            }
        }
    }

    /// One step down; locks the piece if it cannot go on.
    fn fall(&mut self) {
        if !self.shift(0, 1) {
            self.lock();
        }
    }

    fn hard_drop(&mut self) {
        let landed = self.board.drop_y(&self.piece);
        self.score += 2 * (landed.y - self.piece.y) as u64;
        self.piece = landed;
        self.lock();
    }
}

const BG: [Rgb; 2] = [Rgb(24, 24, 32), Rgb(28, 28, 38)];

/// A block as 2 × 2 pixels with a bevel: light top left, dark bottom right.
fn block(p: &mut Pixels, x: i64, y: i64, c: Rgb) {
    p.put(x, y, mix(c, Rgb(255, 255, 255), 0.35));
    p.put(x + 1, y, c);
    p.put(x, y + 1, c);
    p.put(x + 1, y + 1, scale(c, 0.62));
}

struct View {
    canvas: Canvas,
    pixels: Pixels,
}

impl View {
    fn draw(&mut self, g: &Game, t: &Texts, ghost: bool, best: u64, paused: bool) {
        let c = &mut self.canvas;
        let p = &mut self.pixels;
        c.clear();
        p.clear();
        let (cols, rows) = (c.w as i64, c.h as i64);
        if cols < 44 || rows < 23 {
            c.text(0, 0, t.small, None, false);
            return;
        }
        // The well: 10 × 20 blocks, two columns by one row each.
        let (bx, by) = ((cols - 44) / 2 + 1, (rows - 22) / 2 + 1);
        for y in 0..H {
            for x in 0..W {
                let (px, py) = ((bx + 2 * x), 2 * (by + y));
                let col = match g.board.get(x, y) {
                    Some(k) => {
                        block(p, px, py, PIECES[k].2);
                        continue;
                    }
                    None => BG[((x + y) % 2) as usize],
                };
                for (dx, dy) in [(0, 0), (1, 0), (0, 1), (1, 1)] {
                    p.put(px + dx, py + dy, col);
                }
            }
        }
        if !g.over {
            if ghost {
                let landed = g.board.drop_y(&g.piece);
                for (x, y) in landed.blocks().filter(|&(_, y)| y >= 0) {
                    let col = mix(BG[0], PIECES[g.piece.kind].2, 0.35);
                    for (dx, dy) in [(0, 0), (1, 0), (0, 1), (1, 1)] {
                        p.put(bx + 2 * x + dx, 2 * (by + y) + dy, col);
                    }
                }
            }
            for (x, y) in g.piece.blocks().filter(|&(_, y)| y >= 0) {
                block(p, bx + 2 * x, 2 * (by + y), PIECES[g.piece.kind].2);
            }
        }
        // The next piece, when it is known.
        let sx = bx + 2 * W + 3;
        if let Some(k) = g.next {
            for (x, y) in cells(k, 0) {
                block(p, sx + 2 * x, 2 * (by + 2 + y), PIECES[k].2);
            }
        }
        p.show(c);
        // Frame around the well.
        let frame = Some(Rgb(150, 150, 165));
        for y in 0..H {
            c.text(bx - 1, by + y, "│", frame, false);
            c.text(bx + 2 * W, by + y, "│", frame, false);
        }
        c.text(bx - 1, by + H, &format!("└{}┘", "─".repeat(2 * W as usize)), frame, false);
        // Side panel.
        let label = Some(Rgb(150, 150, 165));
        let value = Some(Rgb(240, 240, 240));
        c.text(sx, by, t.next, label, false);
        if g.next.is_none() {
            c.text(sx + 2, by + 2, "?", value, false);
        }
        let rows_info = [
            (t.score, g.score.to_string()),
            (t.lines, g.lines.to_string()),
            (t.level, g.level().to_string()),
            (t.best, best.max(g.score).to_string()),
        ];
        for (i, (k, v)) in rows_info.iter().enumerate() {
            c.text(sx, by + 6 + 2 * i as i64, k, label, false);
            c.text(sx + 10, by + 6 + 2 * i as i64, v, value, false);
        }
        c.text(sx, by + 14, &format!("[{}]", t.mode[g.chooser.mode]), Some(Rgb(220, 120, 120)), false);
        for (i, k) in t.keys.iter().enumerate() {
            c.text(sx, by + 16 + i as i64, k, Some(Rgb(110, 110, 125)), false);
        }
        let msg = if g.over { Some((t.over, Rgb(240, 90, 90))) } else if paused { Some((t.paused, Rgb(240, 210, 80))) } else { None };
        if let Some((m, col)) = msg {
            let mx = bx + W - fun::text::width(m) as i64 / 2;
            c.text(mx.max(0), by + 8, &format!(" {m} "), Some(col), false);
            if g.over {
                let mx = bx + W - fun::text::width(t.again) as i64 / 2;
                c.text(mx.max(0), by + 10, &format!(" {} ", t.again), value, false);
            }
        }
    }
}

fn print_scores(t: &Texts, mode: usize) {
    let v = scores::load("rblocks", MODES[mode]);
    println!("{} ({})", t.table, t.mode[mode]);
    for (i, e) in v.iter().enumerate() {
        println!("{:>3}. {:>8}  {:<16} {}", i + 1, e.score, e.name, e.date);
    }
}

fn main() {
    let mut cli = Cli::new("rblocks");
    let t = Lang::pick(cli.lang, &EN, &PT, &ES);
    let (mut mode, mut level, mut ghost, mut show) = (0usize, 0u32, true, false);
    while let Some(a) = cli.next() {
        match a {
            Arg::Pos(p) => cli.unknown(&p),
            Arg::Opt(opt) => match opt.as_str() {
                "-h" | "--help" => cli.help(t.help),
                "-V" | "--version" => cli.version(),
                "-m" | "--mode" => {
                    let v = cli.value(&opt).to_ascii_lowercase();
                    mode = MODES.iter().position(|m| *m == v).unwrap_or_else(|| cli.fail(&format!("{} '{opt}': {v}", common(cli.lang).invalid_value)));
                }
                "-L" | "--level" => level = cli.number(&opt, 0..=15),
                "-G" | "--no-ghost" => ghost = false,
                "-S" | "--scores" => show = true,
                _ => cli.unknown(&opt),
            },
        }
    }
    if show {
        print_scores(t, mode);
        return;
    }
    if !std::io::stdout().is_terminal() {
        die("rblocks", common(cli.lang).not_a_terminal);
    }
    let result = run(t, mode, level, ghost);
    match result {
        Ok(score) => {
            if scores::add("rblocks", MODES[mode], score).is_some() || score > 0 {
                print_scores(t, mode);
            }
        }
        Err(e) => die("rblocks", &e.to_string()),
    }
}

/// Plays until the player quits; returns the last game's score (the others are
/// saved as they end).
fn run(t: &Texts, mode: usize, level: u32, ghost: bool) -> std::io::Result<u64> {
    let depth: Depth = term::depth(Some(true));
    let _screen = Screen::enter()?;
    let mut out = std::io::stdout().lock();
    let (w, h) = term::size();
    let mut view = View { canvas: Canvas::new(w, h), pixels: Pixels::new(w, h) };
    let mut game = Game::new(mode, level, Rng::new());
    let mut best = scores::load("rblocks", MODES[mode]).first().map_or(0, |e| e.score);
    let mut paused = false;
    let mut fall_at = Instant::now() + game.gravity();
    let mut saved = false;
    loop {
        view.draw(&game, t, ghost, best, paused);
        view.canvas.flush(&mut out, depth)?;
        if game.over && !saved {
            saved = true;
            scores::add("rblocks", MODES[mode], game.score);
            best = best.max(game.score);
        }
        let wait = if game.over || paused { Duration::from_millis(250) } else { fall_at.saturating_duration_since(Instant::now()) };
        let input = tui::poll(wait)?;
        match input {
            Input::Quit => return Ok(if saved { 0 } else { game.score }),
            Input::Resize => {
                let (w, h) = term::size();
                view = View { canvas: Canvas::new(w, h), pixels: Pixels::new(w, h) };
            }
            Input::Key('n') if game.over => {
                game = Game::new(mode, level, Rng::new());
                saved = false;
                fall_at = Instant::now() + game.gravity();
            }
            Input::Key('p') | Input::Key('P') if !game.over => paused = !paused,
            _ if game.over || paused => {}
            Input::Left | Input::Key('a') | Input::Key('h') => {
                game.shift(-1, 0);
            }
            Input::Right | Input::Key('d') | Input::Key('l') => {
                game.shift(1, 0);
            }
            Input::Up | Input::Key('w') | Input::Key('x') | Input::Key('k') => game.rotate(1),
            Input::Key('z') | Input::Key('y') => game.rotate(3),
            Input::Down | Input::Key('s') | Input::Key('j') => {
                if game.shift(0, 1) {
                    game.score += 1;
                }
                fall_at = Instant::now() + game.gravity();
            }
            Input::Key(' ') => {
                game.hard_drop();
                fall_at = Instant::now() + game.gravity();
            }
            _ => {}
        }
        if !game.over && !paused && Instant::now() >= fall_at {
            game.fall();
            fall_at = Instant::now() + game.gravity();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rotations_and_rows() {
        // Four turns bring every piece back.
        for k in 0..7 {
            assert_eq!(cells(k, 0), cells(k, 4));
        }
        let mut b = Board::new();
        for x in 0..W {
            if x != 4 {
                b.cells[((H - 1) * W + x) as usize] = Some(0);
            }
        }
        // A vertical I in the gap clears the bottom row.
        let p = Piece { kind: 0, rot: 1, x: 2, y: H - 4 };
        assert!(b.fits(&p));
        assert_eq!(b.lock(&p), Some(1));
        assert_eq!((0..W).filter(|&x| b.get(x, H - 1).is_some()).count(), 1);
    }

    #[test]
    fn evil_gives_the_worst_and_nice_the_best() {
        // A well one block wide: the I fits it best.
        let mut b = Board::new();
        for y in H - 4..H {
            for x in 1..W {
                b.cells[(y * W + x) as usize] = Some(1);
            }
        }
        let nice = Chooser { mode: 2, bag: Vec::new(), rng: Rng::seeded(1), last: Vec::new() }.next(&b);
        let evil = Chooser { mode: 0, bag: Vec::new(), rng: Rng::seeded(1), last: Vec::new() }.next(&b);
        assert_eq!(nice, 0);
        assert_ne!(evil, 0);
        // A whole game, with the computer playing badly, ends.
        let mut g = Game::new(0, 0, Rng::seeded(3));
        for _ in 0..2000 {
            if g.over {
                break;
            }
            g.hard_drop();
        }
        assert!(g.over);
    }

    /// `cargo test --bin rblocks snapshot -- --ignored --nocapture`
    #[test]
    #[ignore]
    fn snapshot() {
        let mut g = Game::new(1, 3, Rng::seeded(8));
        let mut r = Rng::seeded(2);
        for _ in 0..14 {
            for _ in 0..r.below(4) {
                g.rotate(1);
            }
            let dx = r.range(-4, 5);
            for _ in 0..dx.abs() {
                g.shift(dx.signum(), 0);
            }
            g.hard_drop();
        }
        g.shift(0, 3);
        let mut v = View { canvas: Canvas::new(60, 24), pixels: Pixels::new(60, 24) };
        v.draw(&g, &PT, true, 12000, false);
        print!("{}", v.canvas.to_ansi(Depth::True));
    }
}

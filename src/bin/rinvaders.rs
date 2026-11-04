//! rinvaders: shoot down the rows of invaders before they land (in the style of
//! ninvaders), with original pixel art.

use fun::cli::{die, Arg, Cli};
use fun::i18n::{common, Lang};
use fun::pix::{mix, Pixels};
use fun::rng::Rng;
use fun::scores;
use fun::term::{self, Rgb};
use fun::tui::{self, Canvas, Input, Screen};
use std::io::IsTerminal;
use std::time::{Duration, Instant};

struct Texts {
    help: &'static str,
    score: &'static str,
    best: &'static str,
    lives: &'static str,
    level: &'static str,
    over: &'static str,
    again: &'static str,
    paused: &'static str,
    small: &'static str,
    keys: &'static str,
    table: &'static str,
}

const EN: Texts = Texts {
    help: "\
Usage: rinvaders [options]

Rows of invaders come down: shoot them before they land. The shields protect
you until they are shot to pieces. A saucer flies by now and then.

Keys: ← → or a d move, space or w shoot, p pause, q or Esc quit.

  -L, --level <n>      Starting level, 1 to 10 (default 1)
  -S, --scores         Show the high scores and exit
      --lang <code>    Language: en, pt, es (also: RTOOLS_LANG)
  -h, --help           Show this help
  -V, --version        Show the version
",
    score: "Score",
    best: "Best",
    lives: "Lives",
    level: "Level",
    over: "Game over!",
    again: "n new game, q quit",
    paused: "Paused: p to go on",
    small: "The terminal is too small: at least 60 × 24.",
    keys: "← → move  space shoot  p pause  q quit",
    table: "High scores",
};

const PT: Texts = Texts {
    help: "\
Utilização: rinvaders [opções]

Filas de invasores descem: abata-os antes que aterrem. Os escudos protegem-no
até ficarem desfeitos. De vez em quando passa um disco voador.

Teclas: ← → ou a d mover, espaço ou w disparar, p pausa, q ou Esc sair.

  -L, --level <n>      Nível inicial, de 1 a 10 (por omissão 1)
  -S, --scores         Mostrar os recordes e sair
      --lang <código>  Idioma: en, pt, es (também: RTOOLS_LANG)
  -h, --help           Mostrar esta ajuda
  -V, --version        Mostrar a versão
",
    score: "Pontos",
    best: "Recorde",
    lives: "Vidas",
    level: "Nível",
    over: "Fim do jogo!",
    again: "n novo jogo, q sair",
    paused: "Pausa: p para continuar",
    small: "O terminal é pequeno demais: pelo menos 60 × 24.",
    keys: "← → mover  espaço disparar  p pausa  q sair",
    table: "Recordes",
};

const ES: Texts = Texts {
    help: "\
Uso: rinvaders [opciones]

Filas de invasores bajan: derríbelos antes de que aterricen. Los escudos le
protegen hasta quedar deshechos. De vez en cuando pasa un platillo volante.

Teclas: ← → o a d mover, espacio o w disparar, p pausa, q o Esc salir.

  -L, --level <n>      Nivel inicial, de 1 a 10 (por defecto 1)
  -S, --scores         Mostrar los récords y salir
      --lang <código>  Idioma: en, pt, es (también: RTOOLS_LANG)
  -h, --help           Mostrar esta ayuda
  -V, --version        Mostrar la versión
",
    score: "Puntos",
    best: "Récord",
    lives: "Vidas",
    level: "Nivel",
    over: "¡Fin del juego!",
    again: "n juego nuevo, q salir",
    paused: "Pausa: p para seguir",
    small: "El terminal es demasiado pequeño: al menos 60 × 24.",
    keys: "← → mover  espacio disparar  p pausa  q salir",
    table: "Récords",
};

// Original pixel art, two frames each: `A` body, `K` eyes, `Y` lights.
const INVADERS: [[&[&str]; 2]; 3] = [
    [
        &["...AAA...", ".AAAAAAA.", "AAKAAAKAA", "AAAAAAAAA", ".A.A.A.A.", "A.A...A.A"],
        &["...AAA...", ".AAAAAAA.", "AAKAAAKAA", "AAAAAAAAA", ".A.A.A.A.", ".A.A.A.A."],
    ],
    [
        &["A.......A", ".A.AAA.A.", ".AAAAAAA.", "AAKAAAKAA", "AAAAAAAAA", ".AA...AA."],
        &["...A.A...", "..AAAAA..", ".AAAAAAA.", "AAKAAAKAA", "AAAAAAAAA", "A.......A"],
    ],
    [
        &["..AAAAA..", ".AAAAAAA.", "AAKKAKKAA", "AAAAAAAAA", "A.AA.AA.A", ".A..A..A."],
        &["..AAAAA..", ".AAAAAAA.", "AAKKAKKAA", "AAAAAAAAA", ".AA.A.AA.", "A..A.A..A"],
    ],
];
const INVADER_COLORS: [Rgb; 3] = [Rgb(230, 100, 220), Rgb(90, 200, 240), Rgb(120, 225, 110)];
const POINTS: [u64; 3] = [30, 20, 10];
/// Which kind each row of the formation is, for 2 to 5 rows.
fn row_kinds(rows: usize) -> &'static [usize] {
    match rows {
        0..=2 => &[0, 2],
        3 => &[0, 1, 2],
        4 => &[0, 1, 2, 2],
        _ => &[0, 1, 1, 2, 2],
    }
}

const SAUCER: &[&str] = &[".....AAAAA.....", "...AAAAAAAAA...", ".AAYAAYAAYAAYA.", "AAAAAAAAAAAAAAA", "..AA.......AA.."];
const SHIP: &[&str] = &[".....W.....", "....AWA....", "...AAAAA...", ".AAAAAAAAA.", "AAAAAAAAAAA", "AA.......AA"];
const BOOM: &[&str] = &["A..A.A..A", ".A.....A.", "..A.A.A..", "AA.....AA", "..A.A.A..", ".A.....A.", "A..A.A..A"];
const SHIELD: &[&str] = &[
    "....AAAAAAAA....",
    "..AAAAAAAAAAAA..",
    ".AAAAAAAAAAAAAA.",
    "AAAAAAAAAAAAAAAA",
    "AAAAAAAAAAAAAAAA",
    "AAAAAAAAAAAAAAAA",
    "AAAAA......AAAAA",
    "AAAA........AAAA",
];

fn color(base: Rgb) -> impl Fn(char) -> Option<Rgb> {
    move |ch| match ch {
        'A' => Some(base),
        'K' => Some(Rgb(20, 20, 30)),
        'Y' => Some(Rgb(255, 230, 90)),
        'W' => Some(Rgb(250, 250, 250)),
        _ => None,
    }
}

fn hits(art: &[&str], ax: i64, ay: i64, x: i64, y: i64) -> bool {
    let (i, j) = (x - ax, y - ay);
    j >= 0 && (j as usize) < art.len() && i >= 0 && art[j as usize].as_bytes().get(i as usize).is_some_and(|&b| b != b'.')
}

struct Shot {
    x: i64,
    y: f64,
}

struct Game {
    w: i64,
    h: i64,
    rng: Rng,
    alive: Vec<bool>,
    cols: usize,
    kinds: &'static [usize],
    fx: i64,
    fy: i64,
    dir: i64,
    frame: usize,
    next_step: u32,
    ship: i64,
    shot: Option<Shot>,
    bombs: Vec<Shot>,
    shields: Vec<Vec<bool>>,
    shield_y: i64,
    saucer: Option<(f64, f64)>,
    booms: Vec<(i64, i64, u32, Rgb)>,
    score: u64,
    lives: u32,
    level: u32,
    dead_for: u32,
    over: bool,
    tick: u32,
    banner: u32,
    extra_given: bool,
}

const GAP_X: i64 = 12;
const GAP_Y: i64 = 8;

impl Game {
    fn new(w: usize, rows: usize, level: u32, rng: Rng) -> Game {
        let (w, h) = (w as i64, rows as i64 * 2);
        let cols = ((w - 20) / GAP_X).clamp(5, 11) as usize;
        let mut g = Game {
            w, h, rng, alive: Vec::new(), cols, kinds: row_kinds(((h - 28) / GAP_Y).max(2) as usize), fx: 0, fy: 0, dir: 1, frame: 0, next_step: 0, ship: w / 2 - 5, shot: None,
            bombs: Vec::new(), shields: Vec::new(), shield_y: h - 18, saucer: None, booms: Vec::new(), score: 0, lives: 3,
            level, dead_for: 0, over: false, tick: 0, banner: 40, extra_given: false,
        };
        g.start_level();
        g
    }

    fn start_level(&mut self) {
        self.alive = vec![true; self.cols * self.kinds.len()];
        self.fx = (self.w - self.cols as i64 * GAP_X) / 2;
        let lowest = self.shield_y - self.kinds.len() as i64 * GAP_Y - 8;
        // Below the saucer's flight path, lower in later levels.
        self.fy = (9 + (self.level as i64 - 1).min(6) * 2).min(lowest).max(7);
        self.dir = 1;
        self.bombs.clear();
        self.shot = None;
        self.banner = 40;
        // Four shields, whole again.
        let n = 4;
        self.shields = (0..n).map(|_| SHIELD.iter().flat_map(|r| r.bytes().map(|b| b == b'A')).collect()).collect();
    }

    fn shield_x(&self, i: usize) -> i64 {
        let span = self.w / 4;
        span * i as i64 + (span - 16) / 2
    }

    fn invader_at(&self, i: usize) -> (i64, i64, usize) {
        let (r, c) = (i / self.cols, i % self.cols);
        (self.fx + c as i64 * GAP_X, self.fy + r as i64 * GAP_Y, self.kinds[r])
    }

    fn left(&self) -> usize {
        self.alive.iter().filter(|&&a| a).count()
    }

    /// Frames between two steps of the formation: faster with fewer invaders.
    fn step_every(&self) -> u32 {
        let frac = self.left() as f64 / self.alive.len() as f64;
        ((2.0 + 22.0 * frac) / (1.0 + self.level as f64 * 0.12)).max(1.0) as u32
    }

    /// Takes a bite out of a shield where a shot hits it; true if it hit.
    fn shield_hit(&mut self, x: i64, y: i64) -> bool {
        for i in 0..self.shields.len() {
            let (sx, sy) = (self.shield_x(i), self.shield_y);
            let (dx, dy) = (x - sx, y - sy);
            if (0..16).contains(&dx) && (0..8).contains(&dy) && self.shields[i][(dy * 16 + dx) as usize] {
                for (bx, by) in [(0, 0), (-1, 0), (1, 0), (0, -1), (0, 1), (-1, 1), (1, -1)] {
                    let (px, py) = (dx + bx, dy + by);
                    if (0..16).contains(&px) && (0..8).contains(&py) && (bx == 0 || self.rng.chance(0.6)) {
                        self.shields[i][(py * 16 + px) as usize] = false;
                    }
                }
                return true;
            }
        }
        false
    }

    fn step(&mut self) {
        self.tick += 1;
        self.banner = self.banner.saturating_sub(1);
        self.booms.retain_mut(|b| {
            b.2 -= 1;
            b.2 > 0
        });
        if self.over {
            return;
        }
        if self.dead_for > 0 {
            self.dead_for -= 1;
            if self.dead_for == 0 && self.lives == 0 {
                self.over = true;
            }
            return;
        }
        // The formation steps sideways, and down at the edges.
        if self.next_step == 0 {
            self.next_step = self.step_every();
            self.frame ^= 1;
            let (mut lo, mut hi) = (i64::MAX, i64::MIN);
            for i in (0..self.alive.len()).filter(|&i| self.alive[i]) {
                let (x, _, _) = self.invader_at(i);
                lo = lo.min(x);
                hi = hi.max(x + 9);
            }
            if (self.dir > 0 && hi + 2 >= self.w - 1) || (self.dir < 0 && lo - 2 <= 0) {
                self.dir = -self.dir;
                self.fy += 3;
            } else {
                self.fx += 2 * self.dir;
            }
        }
        self.next_step -= 1;
        // Invaders crush the shields they reach, and win if they reach the ship.
        let ship_y = self.h - 8;
        for i in (0..self.alive.len()).filter(|&i| self.alive[i]) {
            let (x, y, _) = self.invader_at(i);
            if y + 6 >= ship_y {
                self.lives = 0;
                self.dead_for = 30;
                self.booms.push((self.ship + 1, ship_y - 1, 30, Rgb(250, 120, 60)));
                return;
            }
            for s in 0..self.shields.len() {
                let (sx, sy) = (self.shield_x(s), self.shield_y);
                for dy in 0..6 {
                    for dx in 0..9 {
                        let (px, py) = (x + dx - sx, y + dy - sy);
                        if (0..16).contains(&px) && (0..8).contains(&py) {
                            self.shields[s][(py * 16 + px) as usize] = false;
                        }
                    }
                }
            }
        }
        // The ship's shot.
        if let Some(mut s) = self.shot.take() {
            s.y -= 2.0;
            let (x, y) = (s.x, s.y as i64);
            let mut gone = y < 2 || self.shield_hit(x, y) || self.shield_hit(x, y + 1);
            if !gone {
                for i in (0..self.alive.len()).filter(|&i| self.alive[i]) {
                    let (ix, iy, k) = self.invader_at(i);
                    let art = INVADERS[k][self.frame];
                    if hits(art, ix, iy, x, y) || hits(art, ix, iy, x, y + 1) {
                        self.alive[i] = false;
                        self.score += POINTS[k] * self.level as u64;
                        self.booms.push((ix, iy - 1, 8, INVADER_COLORS[k]));
                        gone = true;
                        break;
                    }
                }
            }
            if !gone {
                if let Some((ux, uy)) = self.saucer {
                    if hits(SAUCER, ux as i64, uy as i64, x, y) {
                        let bonus = 50 * self.rng.range(1, 7) as u64;
                        self.score += bonus;
                        self.booms.push((ux as i64 + 3, uy as i64 - 1, 14, Rgb(240, 80, 80)));
                        self.saucer = None;
                        gone = true;
                    }
                }
            }
            if !gone {
                self.shot = Some(s);
            }
        }
        // Bombs: from the lowest invader of a column.
        let chance = 0.02 + 0.008 * self.level as f64;
        if self.bombs.len() < 3 + self.level as usize / 2 && self.rng.chance(chance) {
            let c = self.rng.below(self.cols);
            if let Some(i) = (0..self.kinds.len()).rev().map(|r| r * self.cols + c).find(|&i| self.alive[i]) {
                let (x, y, _) = self.invader_at(i);
                self.bombs.push(Shot { x: x + 4, y: (y + 6) as f64 });
            }
        }
        let speed = 0.7 + 0.05 * self.level as f64;
        let mut keep = Vec::new();
        for mut b in std::mem::take(&mut self.bombs) {
            b.y += speed;
            let (x, y) = (b.x, b.y as i64);
            if y >= self.h - 1 || self.shield_hit(x, y + 2) {
                continue;
            }
            if hits(SHIP, self.ship, ship_y, x, y + 2) {
                self.lives = self.lives.saturating_sub(1);
                self.dead_for = 40;
                self.booms.push((self.ship + 1, ship_y - 1, 40, Rgb(250, 200, 80)));
                self.bombs.clear();
                self.shot = None;
                return;
            }
            keep.push(b);
        }
        self.bombs = keep;
        // The saucer.
        match self.saucer {
            Some((x, y)) if x > -16.0 && x < self.w as f64 + 1.0 => {
                // Half a pixel per frame; y tells the way (1 right, 2 left).
                let dir = if y > 1.5 { -0.5 } else { 0.5 };
                self.saucer = Some((x + dir, y));
            }
            Some(_) => self.saucer = None,
            None if self.rng.chance(0.002) => {
                // y = 1 flies right, y = 2 flies left.
                self.saucer = Some(if self.rng.chance(0.5) { (-15.0, 1.0) } else { (self.w as f64, 2.0) });
            }
            None => {}
        }
        if !self.extra_given && self.score >= 1500 {
            self.extra_given = true;
            self.lives += 1;
        }
        if self.left() == 0 {
            self.level += 1;
            self.start_level();
        }
    }

    fn fire(&mut self) {
        if self.shot.is_none() && self.dead_for == 0 && !self.over {
            self.shot = Some(Shot { x: self.ship + 5, y: (self.h - 9) as f64 });
        }
    }

    fn steer(&mut self, dx: i64) {
        if self.dead_for == 0 && !self.over {
            self.ship = (self.ship + dx).clamp(0, self.w - 11);
        }
    }

    fn draw(&self, c: &mut Canvas, p: &mut Pixels, t: &Texts, best: u64, paused: bool) {
        c.clear();
        p.clear();
        if self.w < 60 || self.h < 48 {
            c.text(0, 0, t.small, None, false);
            return;
        }
        // A few faint stars.
        for i in 0..(self.w / 6) {
            let (x, y) = ((i * 37 + 11) % self.w, (i * 53 + 7) % (self.h - 12) + 2);
            p.put(x, y, Rgb(55, 55, 75));
        }
        for i in (0..self.alive.len()).filter(|&i| self.alive[i]) {
            let (x, y, k) = self.invader_at(i);
            p.sprite(x, y, INVADERS[k][self.frame], color(INVADER_COLORS[k]));
        }
        for (s, sh) in self.shields.iter().enumerate() {
            let sx = self.shield_x(s);
            for (j, &on) in sh.iter().enumerate() {
                if on {
                    let (dx, dy) = (j as i64 % 16, j as i64 / 16);
                    p.put(sx + dx, self.shield_y + dy, mix(Rgb(70, 200, 90), Rgb(30, 120, 50), dy as f64 / 8.0));
                }
            }
        }
        if let Some((x, y)) = self.saucer {
            let lights = if self.tick / 4 % 2 == 0 { Rgb(255, 230, 90) } else { Rgb(255, 120, 60) };
            p.sprite(x as i64, y as i64, SAUCER, |ch| if ch == 'Y' { Some(lights) } else { color(Rgb(230, 70, 80))(ch) });
        }
        let ship_y = self.h - 8;
        if self.dead_for == 0 && !self.over {
            p.sprite(self.ship, ship_y, SHIP, color(Rgb(110, 220, 255)));
        }
        if let Some(s) = &self.shot {
            p.put(s.x, s.y as i64, Rgb(255, 255, 255));
            p.put(s.x, s.y as i64 + 1, Rgb(200, 220, 255));
        }
        for b in &self.bombs {
            let wig = if (b.y as i64 / 2) % 2 == 0 { 0 } else { 1 };
            for k in 0..3 {
                p.put(b.x + if k == 1 { wig } else { 0 }, b.y as i64 + k, Rgb(255, 200, 70));
            }
        }
        for &(x, y, life, col) in &self.booms {
            if life % 4 != 1 {
                p.sprite(x, y, BOOM, color(col));
            }
        }
        // The ground.
        for x in 0..self.w {
            p.put(x, self.h - 1, Rgb(70, 200, 90));
        }
        p.show(c);
        // The scores on the top row.
        let top = format!(
            "{} {:<7} {} {:<7} {} {}  {} {}",
            t.score, self.score, t.best, best.max(self.score), t.lives, "♥".repeat(self.lives as usize), t.level, self.level
        );
        c.text(1, 0, &top, Some(Rgb(230, 230, 230)), false);
        let rows = self.h / 2;
        let msg = if self.over {
            Some((t.over, Rgb(240, 90, 90)))
        } else if paused {
            Some((t.paused, Rgb(240, 210, 80)))
        } else if self.banner > 0 {
            None
        } else {
            None
        };
        if self.banner > 0 && !self.over {
            let s = format!(" {} {} ", t.level, self.level);
            c.text((self.w - s.chars().count() as i64) / 2, rows / 2, &s, Some(Rgb(250, 250, 250)), false);
        }
        if let Some((m, col)) = msg {
            let x = (self.w - fun::text::width(m) as i64) / 2;
            c.text(x, rows / 2, &format!(" {m} "), Some(col), false);
            if self.over {
                let x = (self.w - fun::text::width(t.again) as i64) / 2;
                c.text(x, rows / 2 + 2, &format!(" {} ", t.again), Some(Rgb(240, 240, 240)), false);
            }
        }
        if self.tick < 120 {
            let x = (self.w - fun::text::width(t.keys) as i64) / 2;
            c.text(x, rows - 1, t.keys, Some(Rgb(130, 130, 145)), false);
        }
    }
}

fn print_scores(t: &Texts) {
    println!("{}", t.table);
    for (i, e) in scores::load("rinvaders", "normal").iter().enumerate() {
        println!("{:>3}. {:>8}  {:<16} {}", i + 1, e.score, e.name, e.date);
    }
}

fn main() {
    let mut cli = Cli::new("rinvaders");
    let t = Lang::pick(cli.lang, &EN, &PT, &ES);
    let (mut level, mut show) = (1u32, false);
    while let Some(a) = cli.next() {
        match a {
            Arg::Pos(p) => cli.unknown(&p),
            Arg::Opt(opt) => match opt.as_str() {
                "-h" | "--help" => cli.help(t.help),
                "-V" | "--version" => cli.version(),
                "-L" | "--level" => level = cli.number(&opt, 1..=10),
                "-S" | "--scores" => show = true,
                _ => cli.unknown(&opt),
            },
        }
    }
    if show {
        print_scores(t);
        return;
    }
    if !std::io::stdout().is_terminal() {
        die("rinvaders", common(cli.lang).not_a_terminal);
    }
    match run(t, level) {
        Ok(played) => {
            if played {
                print_scores(t);
            }
        }
        Err(e) => die("rinvaders", &e.to_string()),
    }
}

/// Plays until the player quits; true if any game scored.
fn run(t: &Texts, level: u32) -> std::io::Result<bool> {
    let depth = term::depth(Some(true));
    let _screen = Screen::enter()?;
    let mut out = std::io::stdout().lock();
    let (w, h) = term::size();
    let mut c = Canvas::new(w, h);
    let mut p = Pixels::new(w, h);
    let mut g = Game::new(w, h, level, Rng::new());
    let mut best = scores::load("rinvaders", "normal").first().map_or(0, |e| e.score);
    let (mut paused, mut saved, mut played) = (false, false, false);
    let frame = Duration::from_millis(30);
    loop {
        let start = Instant::now();
        if !paused {
            g.step();
        }
        if g.over && !saved {
            saved = true;
            played |= g.score > 0;
            scores::add("rinvaders", "normal", g.score);
            best = best.max(g.score);
        }
        g.draw(&mut c, &mut p, t, best, paused);
        c.flush(&mut out, depth)?;
        // Every key that came in during the frame.
        loop {
            let left = frame.saturating_sub(start.elapsed());
            match tui::poll(left)? {
                Input::None => break,
                Input::Quit => {
                    if !saved && g.score > 0 {
                        scores::add("rinvaders", "normal", g.score);
                        played = true;
                    }
                    return Ok(played);
                }
                Input::Resize => {
                    let (w, h) = term::size();
                    c = Canvas::new(w, h);
                    p = Pixels::new(w, h);
                    g = Game::new(w, h, level, Rng::new());
                    saved = false;
                }
                Input::Key('n') if g.over => {
                    g = Game::new(c.w, c.h, level, Rng::new());
                    saved = false;
                }
                Input::Key('p') | Input::Key('P') if !g.over => paused = !paused,
                _ if paused => {}
                Input::Left | Input::Key('a') | Input::Key('h') => g.steer(-2),
                Input::Right | Input::Key('d') | Input::Key('l') => g.steer(2),
                Input::Key(' ') | Input::Up | Input::Key('w') | Input::Key('k') => g.fire(),
                _ => {}
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn art_and_play() {
        for s in INVADERS.iter().flatten().chain([&SAUCER, &SHIP, &BOOM, &SHIELD]) {
            let w = s[0].len();
            assert!(s.iter().all(|r| r.len() == w));
        }
        // A player that only shoots and stands still still kills some invaders.
        let mut g = Game::new(80, 24, 1, Rng::seeded(4));
        for _ in 0..3000 {
            g.fire();
            g.step();
        }
        assert!(g.score > 0, "score {} lives {} left {} over {} ship {} fx {} fy {}", g.score, g.lives, g.left(), g.over, g.ship, g.fx, g.fy);
        let mut c = Canvas::new(80, 24);
        let mut p = Pixels::new(80, 24);
        g.draw(&mut c, &mut p, &EN, 0, false);
    }

    /// `cargo test --bin rinvaders snapshot -- --ignored --nocapture`
    #[test]
    #[ignore]
    fn snapshot() {
        let mut g = Game::new(100, 30, 1, Rng::seeded(6));
        for i in 0..220 {
            if i % 20 == 0 {
                g.steer(if i < 100 { -6 } else { 8 });
            }
            g.fire();
            g.step();
        }
        g.saucer = Some((30.0, 1.0));
        let mut c = Canvas::new(100, 30);
        let mut p = Pixels::new(100, 30);
        g.draw(&mut c, &mut p, &PT, 4200, false);
        print!("{}", c.to_ansi(term::Depth::True));
    }
}

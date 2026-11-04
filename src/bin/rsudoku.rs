//! rsudoku: sudoku in the terminal (in the style of nudoku): puzzles made on the
//! spot, always with one solution, in four difficulties; or printed for paper.

use fun::cli::{die, Arg, Cli};
use fun::i18n::{common, Lang};
use fun::rng::Rng;
use fun::term::{self, Rgb};
use fun::tui::{self, Canvas, Input, Screen};
use std::io::IsTerminal;
use std::time::{Duration, Instant};

struct Texts {
    help: &'static str,
    levels: [&'static str; 4],
    time: &'static str,
    level: &'static str,
    hints: &'static str,
    keys: &'static [&'static str],
    solved: &'static str,
    shown: &'static str,
    wrong: &'static str,
    right: &'static str,
    small: &'static str,
    puzzle: &'static str,
    solution: &'static str,
}

const EN: Texts = Texts {
    help: "\
Usage: rsudoku [options]

Sudoku: fill the grid so that every row, column and 3 × 3 box has the digits
1 to 9 once. Each puzzle is made on the spot and has a single solution.

Keys: arrows or h j k l move, 1-9 write, 0 space or Backspace erase, c check,
? hint, u undo, n new puzzle, S show the solution, q or Esc quit.

  -d, --difficulty <name>  easy, normal (default), hard or expert
  -s, --seed <n>       Seed, to get the same puzzle again
  -p, --print <n>      Print n puzzles as text instead of playing (1 to 100)
      --solution       With -p: print the solutions too
      --lang <code>    Language: en, pt, es (also: RTOOLS_LANG)
  -h, --help           Show this help
  -V, --version        Show the version
",
    levels: ["easy", "normal", "hard", "expert"],
    time: "Time",
    level: "Level",
    hints: "Hints",
    keys: &["arrows  move", "1-9     write", "0       erase", "c       check", "?       hint", "u       undo", "n       new", "S       solve", "q       quit"],
    solved: "Solved! n for a new puzzle",
    shown: "That is the solution: n for a new puzzle",
    wrong: "Wrong digits in red",
    right: "No mistakes so far",
    small: "The terminal is too small: at least 56 × 21.",
    puzzle: "Puzzle",
    solution: "Solution",
};

const PT: Texts = Texts {
    help: "\
Utilização: rsudoku [opções]

Sudoku: preencha a grelha de modo que cada linha, coluna e quadrado 3 × 3 tenha
os algarismos de 1 a 9 uma vez. Cada puzzle é feito na hora e tem uma só solução.

Teclas: setas ou h j k l mover, 1-9 escrever, 0 espaço ou Backspace apagar,
c verificar, ? ajuda, u desfazer, n novo puzzle, S mostrar a solução, q ou Esc sair.

  -d, --difficulty <nome>  easy (fácil), normal (por omissão), hard (difícil) ou
                       expert (perito)
  -s, --seed <n>       Semente, para voltar a ter o mesmo puzzle
  -p, --print <n>      Imprimir n puzzles em texto em vez de jogar (de 1 a 100)
      --solution       Com -p: imprimir também as soluções
      --lang <código>  Idioma: en, pt, es (também: RTOOLS_LANG)
  -h, --help           Mostrar esta ajuda
  -V, --version        Mostrar a versão
",
    levels: ["fácil", "normal", "difícil", "perito"],
    time: "Tempo",
    level: "Nível",
    hints: "Ajudas",
    keys: &["setas   mover", "1-9     escrever", "0       apagar", "c       verificar", "?       ajuda", "u       desfazer", "n       novo", "S       solução", "q       sair"],
    solved: "Resolvido! n para um novo puzzle",
    shown: "É esta a solução: n para um novo puzzle",
    wrong: "Algarismos errados a vermelho",
    right: "Sem erros até agora",
    small: "O terminal é pequeno demais: pelo menos 56 × 21.",
    puzzle: "Puzzle",
    solution: "Solução",
};

const ES: Texts = Texts {
    help: "\
Uso: rsudoku [opciones]

Sudoku: rellene la cuadrícula para que cada fila, columna y caja 3 × 3 tenga los
dígitos del 1 al 9 una vez. Cada puzle se crea al momento y tiene una sola solución.

Teclas: flechas o h j k l mover, 1-9 escribir, 0 espacio o Retroceso borrar,
c comprobar, ? pista, u deshacer, n puzle nuevo, S mostrar la solución, q o Esc salir.

  -d, --difficulty <nombre>  easy (fácil), normal (por defecto), hard (difícil) o
                       expert (experto)
  -s, --seed <n>       Semilla, para obtener el mismo puzle
  -p, --print <n>      Imprimir n puzles en texto en vez de jugar (de 1 a 100)
      --solution       Con -p: imprimir también las soluciones
      --lang <código>  Idioma: en, pt, es (también: RTOOLS_LANG)
  -h, --help           Mostrar esta ayuda
  -V, --version        Mostrar la versión
",
    levels: ["fácil", "normal", "difícil", "experto"],
    time: "Tiempo",
    level: "Nivel",
    hints: "Pistas",
    keys: &["flechas mover", "1-9     escribir", "0       borrar", "c       comprobar", "?       pista", "u       deshacer", "n       nuevo", "S       solución", "q       salir"],
    solved: "¡Resuelto! n para un puzle nuevo",
    shown: "Esta es la solución: n para un puzle nuevo",
    wrong: "Dígitos erróneos en rojo",
    right: "Sin errores por ahora",
    small: "El terminal es demasiado pequeño: al menos 56 × 21.",
    puzzle: "Puzle",
    solution: "Solución",
};

const LEVEL_NAMES: [&str; 4] = ["easy", "normal", "hard", "expert"];
/// How many digits each difficulty tries to leave.
const CLUES: [usize; 4] = [40, 32, 27, 23];

type Grid = [u8; 81];

fn box_of(i: usize) -> usize {
    (i / 27) * 3 + (i % 9) / 3
}

/// The digits (as bits 1-9) that cell `i` may still take.
fn options(g: &Grid, i: usize) -> u16 {
    let (r, c, b) = (i / 9, i % 9, box_of(i));
    let mut used = 0u16;
    for j in 0..81 {
        if g[j] != 0 && (j / 9 == r || j % 9 == c || box_of(j) == b) {
            used |= 1 << g[j];
        }
    }
    !used & 0b11_1111_1110
}

/// Counts solutions, up to `limit`; with `fill`, leaves the first one in `g`.
fn solve(g: &mut Grid, limit: usize, rng: Option<&mut Rng>, fill: bool) -> usize {
    // The empty cell with the fewest choices first.
    let mut best: Option<(usize, u16)> = None;
    for i in 0..81 {
        if g[i] == 0 {
            let o = options(g, i);
            if o == 0 {
                return 0;
            }
            if best.is_none_or(|(_, b)| o.count_ones() < b.count_ones()) {
                best = Some((i, o));
            }
        }
    }
    let Some((i, o)) = best else { return 1 };
    let mut digits: Vec<u8> = (1..=9).filter(|d| o & (1 << d) != 0).collect();
    let mut rng = rng;
    if let Some(r) = rng.as_deref_mut() {
        for k in (1..digits.len()).rev() {
            let j = r.below(k + 1);
            digits.swap(k, j);
        }
    }
    let mut count = 0;
    for d in digits {
        g[i] = d;
        count += solve(g, limit - count, rng.as_deref_mut(), fill);
        if count >= limit || (fill && count > 0) {
            if !fill {
                g[i] = 0;
            }
            return count;
        }
    }
    g[i] = 0;
    count
}

/// A new puzzle and its solution.
fn make(level: usize, rng: &mut Rng) -> (Grid, Grid) {
    let mut full = [0u8; 81];
    solve(&mut full, 1, Some(rng), true);
    let mut puzzle = full;
    let mut order: Vec<usize> = (0..81).collect();
    for k in (1..81).rev() {
        let j = rng.below(k + 1);
        order.swap(k, j);
    }
    let mut clues = 81;
    for i in order {
        if clues <= CLUES[level] {
            break;
        }
        let keep = puzzle[i];
        puzzle[i] = 0;
        let mut test = puzzle;
        if solve(&mut test, 2, None, false) != 1 {
            puzzle[i] = keep;
        } else {
            clues -= 1;
        }
    }
    (puzzle, full)
}

fn to_text(g: &Grid) -> String {
    let mut s = String::new();
    for r in 0..9 {
        if r % 3 == 0 {
            s.push_str(if r == 0 { "┏━━━━━━━┳━━━━━━━┳━━━━━━━┓\n" } else { "┣━━━━━━━╋━━━━━━━╋━━━━━━━┫\n" });
        }
        for c in 0..9 {
            if c % 3 == 0 {
                s.push_str("┃ ");
            }
            let d = g[r * 9 + c];
            s.push(if d == 0 { '·' } else { (b'0' + d) as char });
            s.push(' ');
        }
        s.push_str("┃\n");
    }
    s.push_str("┗━━━━━━━┻━━━━━━━┻━━━━━━━┛\n");
    s
}

struct Game {
    puzzle: Grid,
    solution: Grid,
    board: Grid,
    cursor: usize,
    undo: Vec<(usize, u8)>,
    level: usize,
    start: Instant,
    end: Option<Duration>,
    checked: bool,
    revealed: bool,
    hints: u32,
}

impl Game {
    fn new(level: usize, rng: &mut Rng) -> Game {
        let (puzzle, solution) = make(level, rng);
        Game { puzzle, solution, board: puzzle, cursor: 40, undo: Vec::new(), level, start: Instant::now(), end: None, checked: false, revealed: false, hints: 0 }
    }

    fn done(&self) -> bool {
        self.end.is_some()
    }

    fn write(&mut self, d: u8) {
        let i = self.cursor;
        if self.done() || self.puzzle[i] != 0 || self.board[i] == d {
            return;
        }
        self.undo.push((i, self.board[i]));
        self.board[i] = d;
        self.checked = false;
        if self.board == self.solution {
            self.end = Some(self.start.elapsed());
        }
    }

    fn conflict(&self, i: usize) -> bool {
        let d = self.board[i];
        d != 0 && (0..81).any(|j| j != i && self.board[j] == d && (j / 9 == i / 9 || j % 9 == i % 9 || box_of(j) == box_of(i)))
    }

    fn handle(&mut self, input: Input) {
        let (r, c) = (self.cursor / 9, self.cursor % 9);
        match input {
            Input::Left | Input::Key('h') | Input::Key('a') => self.cursor = r * 9 + (c + 8) % 9,
            Input::Right | Input::Key('l') | Input::Key('d') => self.cursor = r * 9 + (c + 1) % 9,
            Input::Up | Input::Key('k') | Input::Key('w') => self.cursor = ((r + 8) % 9) * 9 + c,
            Input::Down | Input::Key('j') | Input::Key('s') => self.cursor = ((r + 1) % 9) * 9 + c,
            Input::Key(k @ '1'..='9') => self.write(k as u8 - b'0'),
            Input::Key('0') | Input::Key(' ') | Input::Key('.') | Input::Erase => self.write(0),
            Input::Key('c') => self.checked = true,
            Input::Key('?') => {
                if !self.done() && self.board[self.cursor] != self.solution[self.cursor] {
                    self.hints += 1;
                    let d = self.solution[self.cursor];
                    self.write(d);
                }
            }
            Input::Key('u') => {
                if let Some((i, d)) = self.undo.pop().filter(|_| !self.done()) {
                    self.board[i] = d;
                    self.cursor = i;
                }
            }
            Input::Key('S') => {
                if !self.done() {
                    self.board = self.solution;
                    self.revealed = true;
                    self.end = Some(self.start.elapsed());
                }
            }
            _ => {}
        }
    }

    fn draw(&self, c: &mut Canvas, t: &Texts) {
        c.clear();
        let (cols, rows) = (c.w as i64, c.h as i64);
        if cols < 56 || rows < 21 {
            c.text(0, 0, t.small, None, false);
            return;
        }
        let (x0, y0) = ((cols - 56) / 2, (rows - 19) / 2);
        let line = Some(Rgb(120, 120, 140));
        let heavy = Some(Rgb(200, 200, 215));
        // Lines: heavy around the boxes, light between the cells.
        for gy in 0..=18 {
            let mut s = String::new();
            for gx in 0..=36 {
                let (vl, hl) = (gx % 4 == 0, gy % 2 == 0);
                let (vh, hh) = (gx % 12 == 0, gy % 6 == 0);
                let ch = match (vl, hl) {
                    (true, true) => {
                        let (top, bot, lef, rig) = (gy > 0, gy < 18, gx > 0, gx < 36);
                        match (top, bot, lef, rig) {
                            (false, true, false, true) => '┏',
                            (false, true, true, false) => '┓',
                            (true, false, false, true) => '┗',
                            (true, false, true, false) => '┛',
                            (false, true, true, true) => if vh { '┳' } else { '┯' },
                            (true, false, true, true) => if vh { '┻' } else { '┷' },
                            (true, true, false, true) => if hh { '┣' } else { '┠' },
                            (true, true, true, false) => if hh { '┫' } else { '┨' },
                            _ => match (vh, hh) {
                                (true, true) => '╋',
                                (true, false) => '╂',
                                (false, true) => '┿',
                                _ => '┼',
                            },
                        }
                    }
                    (true, false) => if vh { '┃' } else { '│' },
                    (false, true) => if hh { '━' } else { '─' },
                    _ => ' ',
                };
                s.push(ch);
            }
            for (gx, ch) in s.chars().enumerate() {
                if ch != ' ' {
                    let strong = "┏┓┗┛┳┻┣┫╋┃━".contains(ch);
                    c.set(x0 + gx as i64, y0 + gy, ch, if strong { heavy } else { line }, false);
                }
            }
        }
        // The cells.
        let cur = self.cursor;
        let cur_digit = self.board[cur];
        for i in 0..81 {
            let (r, col) = ((i / 9) as i64, (i % 9) as i64);
            let (cx, cy) = (x0 + col * 4 + 1, y0 + r * 2 + 1);
            let d = self.board[i];
            let same_area = i / 9 == cur / 9 || i % 9 == cur % 9 || box_of(i) == box_of(cur);
            let bg = if i == cur {
                Some(Rgb(70, 90, 150))
            } else if d != 0 && d == cur_digit {
                Some(Rgb(55, 60, 85))
            } else if same_area {
                Some(Rgb(34, 36, 50))
            } else {
                None
            };
            let given = self.puzzle[i] != 0;
            let wrong = d != 0 && !given && (self.conflict(i) || (self.checked && d != self.solution[i]));
            let fg = if given {
                Rgb(240, 240, 245)
            } else if wrong {
                Rgb(250, 90, 90)
            } else if self.revealed {
                Rgb(240, 200, 90)
            } else {
                Rgb(100, 200, 250)
            };
            for k in 0..3 {
                let ch = if k == 1 && d != 0 { (b'0' + d) as char } else { ' ' };
                c.put(cx + k, cy, fun::tui::Cell { ch, fg: Some(fg), bg, bold: given });
            }
        }
        // Side panel.
        let sx = x0 + 40;
        let label = Some(Rgb(150, 150, 165));
        let value = Some(Rgb(240, 240, 240));
        let secs = self.end.unwrap_or_else(|| self.start.elapsed()).as_secs();
        c.text(sx, y0, t.level, label, false);
        c.text(sx + 8, y0, t.levels[self.level], value, false);
        c.text(sx, y0 + 1, t.time, label, false);
        c.text(sx + 8, y0 + 1, &format!("{}:{:02}", secs / 60, secs % 60), value, false);
        c.text(sx, y0 + 2, t.hints, label, false);
        c.text(sx + 8, y0 + 2, &self.hints.to_string(), value, false);
        for (k, s) in t.keys.iter().enumerate() {
            c.text(sx, y0 + 5 + k as i64, s, Some(Rgb(120, 120, 135)), false);
        }
        let status = if self.revealed {
            Some((t.shown, Rgb(240, 200, 90)))
        } else if self.done() {
            Some((t.solved, Rgb(110, 230, 120)))
        } else if self.checked {
            let any = (0..81).any(|i| self.board[i] != 0 && self.board[i] != self.solution[i]);
            Some(if any { (t.wrong, Rgb(250, 90, 90)) } else { (t.right, Rgb(110, 230, 120)) })
        } else {
            None
        };
        if let Some((s, col)) = status {
            c.text(x0, y0 + 19, s, Some(col), false);
        }
    }
}

fn main() {
    let mut cli = Cli::new("rsudoku");
    let t = Lang::pick(cli.lang, &EN, &PT, &ES);
    let (mut level, mut seed, mut print, mut solution) = (1usize, None::<u64>, None::<usize>, false);
    while let Some(a) = cli.next() {
        match a {
            Arg::Pos(p) => cli.unknown(&p),
            Arg::Opt(opt) => match opt.as_str() {
                "-h" | "--help" => cli.help(t.help),
                "-V" | "--version" => cli.version(),
                "-d" | "--difficulty" => {
                    let v = cli.value(&opt).to_ascii_lowercase();
                    level = LEVEL_NAMES.iter().position(|n| *n == v).unwrap_or_else(|| cli.fail(&format!("{} '{opt}': {v}", common(cli.lang).invalid_value)));
                }
                "-s" | "--seed" => seed = Some(cli.parse(&opt)),
                "-p" | "--print" => print = Some(cli.number(&opt, 1..=100)),
                "--solution" => solution = true,
                _ => cli.unknown(&opt),
            },
        }
    }
    let mut rng = seed.map_or_else(Rng::new, Rng::seeded);
    if let Some(n) = print {
        for k in 0..n {
            let (p, s) = make(level, &mut rng);
            let clues = p.iter().filter(|&&d| d != 0).count();
            println!("{} {} ({}, {clues})", t.puzzle, k + 1, t.levels[level]);
            print!("{}", to_text(&p));
            if solution {
                println!("{} {}", t.solution, k + 1);
                print!("{}", to_text(&s));
            }
            if k + 1 < n {
                println!();
            }
        }
        return;
    }
    if !std::io::stdout().is_terminal() {
        die("rsudoku", common(cli.lang).not_a_terminal);
    }
    if let Err(e) = run(t, level, &mut rng) {
        die("rsudoku", &e.to_string());
    }
}

fn run(t: &Texts, level: usize, rng: &mut Rng) -> std::io::Result<()> {
    let depth = term::depth(Some(true));
    let _screen = Screen::enter()?;
    let mut out = std::io::stdout().lock();
    let (w, h) = term::size();
    let mut c = Canvas::new(w, h);
    let mut g = Game::new(level, rng);
    loop {
        g.draw(&mut c, t);
        c.flush(&mut out, depth)?;
        match tui::poll(Duration::from_millis(500))? {
            Input::Quit => return Ok(()),
            Input::Resize => {
                let (w, h) = term::size();
                c = Canvas::new(w, h);
            }
            Input::Key('n') => g = Game::new(level, rng),
            i => g.handle(i),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn valid(g: &Grid) -> bool {
        (0..81).all(|i| g[i] != 0 && (0..81).all(|j| j == i || g[j] != g[i] || (j / 9 != i / 9 && j % 9 != i % 9 && box_of(j) != box_of(i))))
    }

    #[test]
    fn puzzles_have_one_solution() {
        let mut rng = Rng::seeded(7);
        for level in 0..4 {
            let (p, s) = make(level, &mut rng);
            assert!(valid(&s));
            assert!((0..81).all(|i| p[i] == 0 || p[i] == s[i]));
            let mut test = p;
            assert_eq!(solve(&mut test, 2, None, false), 1);
            let clues = p.iter().filter(|&&d| d != 0).count();
            assert!(clues <= CLUES[level] + 6, "level {level}: {clues}");
        }
    }

    #[test]
    fn playing() {
        let mut g = Game::new(0, &mut Rng::seeded(1));
        let empty: Vec<usize> = (0..81).filter(|&i| g.puzzle[i] == 0).collect();
        for &i in &empty {
            g.cursor = i;
            let d = g.solution[i];
            g.handle(Input::Key((b'0' + d) as char));
        }
        assert!(g.done() && !g.revealed);
        assert!(to_text(&g.board).starts_with("┏━━━━━━━┳"));
    }

    /// `cargo test --bin rsudoku snapshot -- --ignored --nocapture`
    #[test]
    #[ignore]
    fn snapshot() {
        let mut g = Game::new(1, &mut Rng::seeded(3));
        let empty: Vec<usize> = (0..81).filter(|&i| g.puzzle[i] == 0).collect();
        for &i in empty.iter().take(12) {
            g.cursor = i;
            let d = if i % 5 == 0 { g.solution[i] % 9 + 1 } else { g.solution[i] };
            g.handle(Input::Key((b'0' + d) as char));
        }
        g.cursor = empty[14];
        g.checked = true;
        let mut c = Canvas::new(70, 24);
        g.draw(&mut c, &PT);
        print!("{}", c.to_ansi(term::Depth::True));
    }
}

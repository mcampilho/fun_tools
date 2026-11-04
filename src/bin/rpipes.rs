//! rpipes: pipes grow across the terminal, turning at random (like pipes.sh).
//! q, Esc or Ctrl+C to quit.

use fun::cli::{die, Arg, Cli};
use fun::i18n::{common, Lang};
use fun::rng::Rng;
use fun::term::{self, Depth, Rgb};
use fun::tui::{self, Canvas, Input, Screen};
use std::io::IsTerminal;
use std::time::{Duration, Instant};

struct Texts {
    help: &'static str,
    kinds: &'static str,
}

const EN: Texts = Texts {
    help: "\
Usage: rpipes [options]

Pipes grow across the terminal, turning at random.
Keys: q/Esc/Ctrl+C quit, space or c clear, 1 to 9 speed.

  -p, --pipes <n>      Number of pipes, 1 to 100 (default 1)
  -t, --type <n>       Pipe style, 0 to 8 (see -l); can be given more than once:
                       each pipe takes one of them. Or c followed by 6 characters:
                       vertical, horizontal and the corners ┌ ┐ └ ┘
  -l, --list           List the pipe styles
  -f, --fps <n>        Steps per second, 10 to 500 (default 75)
  -s, --straight <n>   Straightness, 1 (twisty) to 20 (straight) (default 13)
  -r, --reset <n>      Clear the screen after this many pieces, 0 = never (default 2000)
  -R, --random         Start the pipes at random places (default: the middle)
  -K, --keep           Keep the colour and style when a pipe crosses the edge
  -B, --no-bold        No bold
  -C, --no-color       No colours
      --lang <code>    Language: en, pt, es (also: RTOOLS_LANG)
  -h, --help           Show this help
  -V, --version        Show the version
",
    kinds: "Pipe styles:",
};

const PT: Texts = Texts {
    help: "\
Utilização: rpipes [opções]

Canos crescem pelo terminal e viram ao acaso.
Teclas: q/Esc/Ctrl+C sair, espaço ou c limpar, 1 a 9 velocidade.

  -p, --pipes <n>      Número de canos, de 1 a 100 (por omissão 1)
  -t, --type <n>       Estilo dos canos, de 0 a 8 (ver -l); pode repetir-se: cada cano
                       fica com um deles. Ou c seguido de 6 caracteres: vertical,
                       horizontal e os cantos ┌ ┐ └ ┘
  -l, --list           Mostrar os estilos de canos
  -f, --fps <n>        Passos por segundo, de 10 a 500 (por omissão 75)
  -s, --straight <n>   Quão direitos, de 1 (às voltas) a 20 (direitos) (por omissão 13)
  -r, --reset <n>      Limpar o ecrã ao fim deste número de peças, 0 = nunca
                       (por omissão 2000)
  -R, --random         Começar em sítios ao acaso (por omissão: ao meio)
  -K, --keep           Manter a cor e o estilo quando um cano passa a borda
  -B, --no-bold        Sem negrito
  -C, --no-color       Sem cores
      --lang <código>  Idioma: en, pt, es (também: RTOOLS_LANG)
  -h, --help           Mostrar esta ajuda
  -V, --version        Mostrar a versão
",
    kinds: "Estilos de canos:",
};

const ES: Texts = Texts {
    help: "\
Uso: rpipes [opciones]

Tuberías que crecen por el terminal y giran al azar.
Teclas: q/Esc/Ctrl+C salir, espacio o c limpiar, 1 a 9 velocidad.

  -p, --pipes <n>      Número de tuberías, de 1 a 100 (por defecto 1)
  -t, --type <n>       Estilo, de 0 a 8 (ver -l); puede repetirse: cada tubería toma
                       uno de ellos. O c seguido de 6 caracteres: vertical,
                       horizontal y las esquinas ┌ ┐ └ ┘
  -l, --list           Mostrar los estilos
  -f, --fps <n>        Pasos por segundo, de 10 a 500 (por defecto 75)
  -s, --straight <n>   Cuán rectas, de 1 (con curvas) a 20 (rectas) (por defecto 13)
  -r, --reset <n>      Limpiar la pantalla tras este número de piezas, 0 = nunca
                       (por defecto 2000)
  -R, --random         Empezar en sitios al azar (por defecto: en el centro)
  -K, --keep           Mantener el color y el estilo al cruzar el borde
  -B, --no-bold        Sin negrita
  -C, --no-color       Sin colores
      --lang <código>  Idioma: en, pt, es (también: RTOOLS_LANG)
  -h, --help           Mostrar esta ayuda
  -V, --version        Mostrar la versión
",
    kinds: "Estilos de tuberías:",
};

/// Pipe styles: vertical, horizontal, then the corners ┌ ┐ └ ┘.
const STYLES: &[(&str, &str)] = &[
    ("heavy", "┃━┏┓┗┛"),
    ("light", "│─┌┐└┘"),
    ("rounded", "│─╭╮╰╯"),
    ("double", "║═╔╗╚╝"),
    ("dashed", "╏╍┏┓┗┛"),
    ("mixed", "║─╓╖╙╜"),
    ("blocks", "█▀████"),
    ("dots", "••••••"),
    ("ascii", "|-++++"),
];

const COLORS: &[Rgb] = &[
    Rgb(230, 70, 70), Rgb(90, 210, 90), Rgb(240, 200, 60), Rgb(80, 140, 250),
    Rgb(210, 100, 220), Rgb(70, 210, 220), Rgb(235, 235, 235), Rgb(250, 140, 50),
];

/// Directions: up, right, down, left.
const DX: [i64; 4] = [0, 1, 0, -1];
const DY: [i64; 4] = [-1, 0, 1, 0];

/// The piece that joins a pipe coming in going `from` and leaving going `to`.
fn piece(style: &[char], from: usize, to: usize) -> char {
    let [v, h, tl, tr, bl, br] = [style[0], style[1], style[2], style[3], style[4], style[5]];
    match (from, to) {
        (0, 0) | (2, 2) => v,
        (1, 1) | (3, 3) => h,
        (0, 1) | (3, 2) => tl, // ┌: up then right, or left then down
        (0, 3) | (1, 2) => tr, // ┐
        (2, 1) | (3, 0) => bl, // └
        (2, 3) | (1, 0) => br, // ┘
        _ => v,
    }
}

struct Pipe {
    x: i64,
    y: i64,
    dir: usize,
    color: Rgb,
    style: usize,
}

struct Options {
    pipes: usize,
    styles: Vec<Vec<char>>,
    fps: u32,
    straight: u32,
    reset: usize,
    random: bool,
    keep: bool,
    bold: bool,
    color: bool,
}

struct Garden {
    pipes: Vec<Pipe>,
    rng: Rng,
    w: i64,
    h: i64,
    pieces: usize,
}

impl Garden {
    fn new(o: &Options, w: usize, h: usize, mut rng: Rng) -> Garden {
        let (w, h) = (w as i64, h as i64);
        let pipes = (0..o.pipes)
            .map(|_| {
                let (x, y) = if o.random { (rng.range(0, w.max(1)), rng.range(0, h.max(1))) } else { (w / 2, h / 2) };
                Pipe { x, y, dir: rng.below(4), color: *rng.pick(COLORS), style: rng.below(o.styles.len()) }
            })
            .collect();
        Garden { pipes, rng, w, h, pieces: 0 }
    }

    /// Moves every pipe one cell and draws the new pieces.
    fn step(&mut self, o: &Options, c: &mut Canvas) {
        for p in &mut self.pipes {
            // Turn left or right now and then, never back.
            let mut to = p.dir;
            if self.rng.below(o.straight as usize + 1) == 0 {
                to = (p.dir + if self.rng.chance(0.5) { 1 } else { 3 }) % 4;
            }
            let ch = piece(&o.styles[p.style], p.dir, to);
            let fg = o.color.then_some(p.color);
            c.set(p.x, p.y, ch, fg, o.bold);
            p.dir = to;
            p.x += DX[to];
            p.y += DY[to];
            // Across the edge: on to the other side, maybe as a new pipe.
            if p.x < 0 || p.y < 0 || p.x >= self.w || p.y >= self.h {
                p.x = p.x.rem_euclid(self.w.max(1));
                p.y = p.y.rem_euclid(self.h.max(1));
                if !o.keep {
                    p.color = *self.rng.pick(COLORS);
                    p.style = self.rng.below(o.styles.len());
                }
            }
            self.pieces += 1;
        }
        if o.reset > 0 && self.pieces >= o.reset {
            c.clear();
            self.pieces = 0;
        }
    }
}

fn main() {
    let mut cli = Cli::new("rpipes");
    let t = Lang::pick(cli.lang, &EN, &PT, &ES);
    let mut o = Options { pipes: 1, styles: Vec::new(), fps: 75, straight: 13, reset: 2000, random: false, keep: false, bold: true, color: true };
    while let Some(a) = cli.next() {
        match a {
            Arg::Pos(p) => cli.unknown(&p),
            Arg::Opt(opt) => match opt.as_str() {
                "-h" | "--help" => cli.help(t.help),
                "-V" | "--version" => cli.version(),
                "-l" | "--list" => {
                    println!("{}", t.kinds);
                    for (i, (name, s)) in STYLES.iter().enumerate() {
                        let s: Vec<char> = s.chars().collect();
                        // A little loop of the style: ┌──┐ / └──┘
                        let sample = format!("{}{}{}{} {}{}{}{}", s[2], s[1], s[1], s[3], s[4], s[1], s[1], s[5]);
                        println!("  {i}  {sample}  {}  {name}", s[0]);
                    }
                    return;
                }
                "-p" | "--pipes" => o.pipes = cli.number(&opt, 1..=100),
                "-t" | "--type" => {
                    let v = cli.value(&opt);
                    if let Some(custom) = v.strip_prefix('c').filter(|s| s.chars().count() == 6) {
                        o.styles.push(custom.chars().collect());
                    } else {
                        match v.parse::<usize>().ok().and_then(|i| STYLES.get(i)) {
                            Some((_, s)) => o.styles.push(s.chars().collect()),
                            None => cli.fail(&format!("{} '{opt}': {v}", common(cli.lang).invalid_value)),
                        }
                    }
                }
                "-f" | "--fps" => o.fps = cli.number(&opt, 10..=500),
                "-s" | "--straight" => o.straight = cli.number(&opt, 1..=20),
                "-r" | "--reset" => o.reset = cli.number(&opt, 0..=1_000_000),
                "-R" | "--random" => o.random = true,
                "-K" | "--keep" => o.keep = true,
                "-B" | "--no-bold" => o.bold = false,
                "-C" | "--no-color" => o.color = false,
                _ => cli.unknown(&opt),
            },
        }
    }
    if o.styles.is_empty() {
        o.styles.push(STYLES[0].1.chars().collect());
    }
    if !std::io::stdout().is_terminal() {
        die("rpipes", common(cli.lang).not_a_terminal);
    }
    let depth = if o.color { term::depth(Some(true)) } else { Depth::None };
    if let Err(e) = run(&mut o, depth) {
        die("rpipes", &e.to_string());
    }
}

fn run(o: &mut Options, depth: Depth) -> std::io::Result<()> {
    let _screen = Screen::enter()?;
    let mut out = std::io::stdout().lock();
    let (w, h) = term::size();
    let mut c = Canvas::new(w, h);
    let mut g = Garden::new(o, w, h, Rng::new());
    loop {
        let start = Instant::now();
        let frame = Duration::from_micros(1_000_000 / o.fps as u64);
        g.step(o, &mut c);
        c.flush(&mut out, depth)?;
        match tui::poll(frame.saturating_sub(start.elapsed()))? {
            Input::Quit => return Ok(()),
            Input::Key(' ') | Input::Key('c') => c.clear(),
            Input::Key(k @ '1'..='9') => o.fps = 10 + (k as u32 - '0' as u32) * 25,
            Input::Resize => {
                let (w, h) = term::size();
                c.resize(w, h);
                g = Garden::new(o, w, h, Rng::new());
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
    fn pieces_join() {
        let s: Vec<char> = STYLES[1].1.chars().collect();
        assert_eq!(piece(&s, 0, 1), '┌'); // going up, then right
        assert_eq!(piece(&s, 1, 2), '┐'); // going right, then down
        assert_eq!(piece(&s, 2, 3), '┘'); // going down, then left
        assert_eq!(piece(&s, 3, 0), '└'); // going left, then up
        assert_eq!(piece(&s, 1, 1), '─');
        assert!(STYLES.iter().all(|(_, s)| s.chars().count() >= 6));
        let o = Options { pipes: 5, styles: vec![s], fps: 75, straight: 3, reset: 500, random: true, keep: false, bold: true, color: true };
        let mut c = Canvas::new(40, 12);
        let mut g = Garden::new(&o, 40, 12, Rng::seeded(1));
        for _ in 0..2000 {
            g.step(&o, &mut c);
        }
        assert!(g.pipes.iter().all(|p| p.x >= 0 && p.x < 40 && p.y >= 0 && p.y < 12));
    }

    /// `cargo test --bin rpipes snapshot -- --ignored --nocapture`
    #[test]
    #[ignore]
    fn snapshot() {
        let styles = (0..STYLES.len() - 1).map(|i| STYLES[i].1.chars().collect()).collect();
        let o = Options { pipes: 6, styles, fps: 75, straight: 10, reset: 0, random: true, keep: false, bold: true, color: true };
        let mut c = Canvas::new(100, 28);
        let mut g = Garden::new(&o, 100, 28, Rng::seeded(5));
        for _ in 0..260 {
            g.step(&o, &mut c);
        }
        print!("{}", c.to_ansi(Depth::True));
    }
}

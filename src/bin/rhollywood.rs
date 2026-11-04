//! rhollywood: fill the terminal with panes of busy-looking "hacker" activity
//! (like hollywood). Everything shown is made up. q, Esc or Ctrl+C to quit.

use fun::cli::{die, Arg, Cli};
use fun::i18n::{common, Lang};
use fun::rng::Rng;
use fun::term::{self, Rgb};
use fun::tui::{self, Canvas, Input, Screen};
use std::collections::VecDeque;
use std::io::IsTerminal;
use std::time::{Duration, Instant};

struct Texts {
    help: &'static str,
}

const EN: Texts = Texts {
    help: "\
Usage: rhollywood [options]

Fills the terminal with panes of busy-looking technical activity, as in the
films. Everything shown is made up. Keys: q/Esc/Ctrl+C quit, space shuffle.

  -p, --panes <n>      Number of panes, 1 to 9 (default: from the terminal size)
  -i, --interval <s>   Seconds between pane changes, 0 = never (default 10)
  -s, --speed <n>      Speed, 1 (slow) to 10 (fast) (default 5)
      --lang <code>    Language: en, pt, es (also: RTOOLS_LANG)
  -h, --help           Show this help
  -V, --version        Show the version
",
};

const PT: Texts = Texts {
    help: "\
Utilização: rhollywood [opções]

Enche o terminal de painéis com atividade técnica muito ocupada, como nos
filmes. É tudo inventado. Teclas: q/Esc/Ctrl+C sair, espaço baralhar.

  -p, --panes <n>      Número de painéis, de 1 a 9 (por omissão: conforme o tamanho)
  -i, --interval <s>   Segundos entre mudanças de painel, 0 = nunca (por omissão 10)
  -s, --speed <n>      Velocidade, de 1 (lenta) a 10 (rápida) (por omissão 5)
      --lang <código>  Idioma: en, pt, es (também: RTOOLS_LANG)
  -h, --help           Mostrar esta ajuda
  -V, --version        Mostrar a versão
",
};

const ES: Texts = Texts {
    help: "\
Uso: rhollywood [opciones]

Llena el terminal de paneles con actividad técnica muy ajetreada, como en las
películas. Todo es inventado. Teclas: q/Esc/Ctrl+C salir, espacio barajar.

  -p, --panes <n>      Número de paneles, de 1 a 9 (por defecto: según el tamaño)
  -i, --interval <s>   Segundos entre cambios de panel, 0 = nunca (por defecto 10)
  -s, --speed <n>      Velocidad, de 1 (lenta) a 10 (rápida) (por defecto 5)
      --lang <código>  Idioma: en, pt, es (también: RTOOLS_LANG)
  -h, --help           Mostrar esta ayuda
  -V, --version        Mostrar la versión
",
};

const GREEN: Rgb = Rgb(60, 220, 90);
const DIM: Rgb = Rgb(30, 120, 50);
const CYAN: Rgb = Rgb(80, 210, 230);
const YELLOW: Rgb = Rgb(240, 210, 80);
const RED: Rgb = Rgb(240, 80, 80);
const WHITE: Rgb = Rgb(225, 225, 225);
const GREY: Rgb = Rgb(130, 130, 130);
const MAGENTA: Rgb = Rgb(210, 110, 230);

type Line = Vec<(String, Rgb)>;

#[derive(Clone, Copy, PartialEq)]
enum Kind {
    Hex,
    Code,
    Log,
    Net,
    Procs,
    Meters,
    Progress,
    Scope,
    Rain,
}

const KINDS: &[Kind] = &[
    Kind::Hex, Kind::Code, Kind::Log, Kind::Net, Kind::Procs, Kind::Meters, Kind::Progress, Kind::Scope, Kind::Rain,
];

impl Kind {
    fn title(self) -> &'static str {
        match self {
            Kind::Hex => "memdump",
            Kind::Code => "build",
            Kind::Log => "syslog",
            Kind::Net => "netstat",
            Kind::Procs => "top",
            Kind::Meters => "cpu",
            Kind::Progress => "decrypt",
            Kind::Scope => "signal",
            Kind::Rain => "stream",
        }
    }
}

struct Pane {
    kind: Kind,
    lines: VecDeque<Line>,
    procs: Vec<(String, u32, f64, f64)>,
    meters: Vec<f64>,
    tasks: Vec<(String, f64, f64)>,
    phase: f64,
    rain: Vec<(f64, f64)>,
    addr: u64,
    indent: usize,
}

const WORDS: &[&str] = &[
    "kernel", "socket", "buffer", "cipher", "packet", "thread", "daemon", "vector", "matrix", "quantum",
    "proxy", "tunnel", "shard", "cache", "token", "hash", "payload", "node", "cluster", "firewall",
    "handshake", "entropy", "sector", "uplink", "relay", "mainframe", "protocol", "checksum", "mirror", "core",
];
const VERBS: &[&str] = &[
    "allocating", "decrypting", "routing", "compiling", "indexing", "rebuilding", "syncing", "scanning",
    "tracing", "flushing", "mapping", "validating", "bypassing", "rotating", "spawning", "patching",
];
const PROCS: &[&str] = &[
    "sshd", "nginx", "postgres", "redis", "kworker", "systemd", "dockerd", "cryptd", "netmon", "watchdog",
    "rustc", "cargo", "node", "python3", "java", "syslogd", "cron", "tor", "openvpn", "kswapd",
];
const TYPES: &[&str] = &["u8", "u32", "i64", "f64", "usize", "Vec<u8>", "&str", "String", "Option<Key>", "Packet"];

fn ip(rng: &mut Rng) -> String {
    format!("{}.{}.{}.{}", rng.range(10, 223), rng.range(0, 255), rng.range(0, 255), rng.range(1, 254))
}

impl Pane {
    fn new(kind: Kind, rng: &mut Rng) -> Pane {
        let procs = (0..24)
            .map(|_| {
                (rng.pick(PROCS).to_string(), rng.range(100, 32000) as u32, rng.float() * 30.0, rng.float() * 8.0)
            })
            .collect();
        let tasks = (0..6).map(|_| (Pane::task_name(rng), rng.float() * 100.0, 0.5 + rng.float() * 3.0)).collect();
        Pane {
            kind,
            lines: VecDeque::new(),
            procs,
            meters: (0..8).map(|_| rng.float() * 100.0).collect(),
            tasks,
            phase: 0.0,
            rain: (0..200).map(|_| (rng.float() * -30.0, 0.3 + rng.float())).collect(),
            addr: rng.next_u64() & 0x7fff_ffff_f000,
            indent: 0,
        }
    }

    fn task_name(rng: &mut Rng) -> String {
        format!("{} {}", rng.pick(VERBS), rng.pick(WORDS))
    }

    fn line(&mut self, rng: &mut Rng) -> Option<Line> {
        Some(match self.kind {
            Kind::Hex => {
                let bytes: Vec<u8> = (0..16).map(|_| rng.below(256) as u8).collect();
                let hex: Vec<String> = bytes.iter().map(|b| format!("{b:02x}")).collect();
                let asc: String = bytes.iter().map(|&b| if (32..127).contains(&b) { b as char } else { '.' }).collect();
                self.addr += 16;
                vec![
                    (format!("{:012x}  ", self.addr), CYAN),
                    (format!("{}  {}", hex[..8].join(" "), hex[8..].join(" ")), GREEN),
                    (format!("  |{asc}|"), DIM),
                ]
            }
            Kind::Code => {
                let pad = "    ".repeat(self.indent);
                let w = rng.pick(WORDS);
                let (text, color, next) = match rng.below(9) {
                    0 if self.indent < 3 => (format!("fn {}_{}(x: {}) -> {} {{", rng.pick(VERBS).trim_end_matches("ing"), w, rng.pick(TYPES), rng.pick(TYPES)), CYAN, self.indent + 1),
                    1 if self.indent < 3 => (format!("for {} in {}s.iter() {{", &w[..1], w), MAGENTA, self.indent + 1),
                    2 if self.indent < 3 => (format!("if {}.len() > 0x{:x} {{", w, rng.range(16, 4096)), MAGENTA, self.indent + 1),
                    3 | 4 if self.indent > 0 => ("}".to_string(), WHITE, self.indent - 1),
                    5 => (format!("// {} {} ({} ms)", rng.pick(VERBS), w, rng.range(1, 999)), GREY, self.indent),
                    6 => (format!("let {} = {}::new({});", w, rng.pick(TYPES), rng.range(0, 65535)), WHITE, self.indent),
                    7 => (format!("{}.{}(&mut {})?;", w, rng.pick(VERBS).trim_end_matches("ing"), rng.pick(WORDS)), GREEN, self.indent),
                    _ => (format!("   Compiling {} v0.{}.{}", w, rng.range(1, 30), rng.range(0, 9)), YELLOW, self.indent),
                };
                let out = vec![(format!("{pad}{text}"), color)];
                self.indent = next;
                out
            }
            Kind::Log => {
                let (lvl, col) = *rng.pick(&[("INFO ", GREEN), ("INFO ", GREEN), ("DEBUG", GREY), ("WARN ", YELLOW), ("ERROR", RED)]);
                let t = rng.range(0, 86400);
                vec![
                    (format!("[{:02}:{:02}:{:02}] ", t / 3600, t / 60 % 60, t % 60), DIM),
                    (format!("{lvl} "), col),
                    (format!("{}[{}]: {} {} {}", rng.pick(PROCS), rng.range(100, 9999), rng.pick(VERBS), rng.pick(WORDS), rng.pick(WORDS)), WHITE),
                ]
            }
            Kind::Net => {
                let st = *rng.pick(&[("ESTABLISHED", GREEN), ("SYN_SENT", YELLOW), ("TIME_WAIT", GREY), ("LISTEN", CYAN), ("CLOSE_WAIT", RED)]);
                vec![
                    (format!("tcp  {:>21} ", format!("{}:{}", ip(rng), rng.range(1024, 65535))), WHITE),
                    (format!("-> {:>21} ", format!("{}:{}", ip(rng), *rng.pick(&[22, 80, 443, 8080, 5432, 6379, 1337]))), CYAN),
                    (st.0.to_string(), st.1),
                ]
            }
            _ => return None,
        })
    }

    fn step(&mut self, rng: &mut Rng, h: usize) {
        match self.kind {
            Kind::Procs => {
                for p in &mut self.procs {
                    p.2 = (p.2 + (rng.float() - 0.5) * 12.0).clamp(0.0, 99.9);
                    p.3 = (p.3 + (rng.float() - 0.5)).clamp(0.1, 25.0);
                }
                self.procs.sort_by(|a, b| b.2.total_cmp(&a.2));
            }
            Kind::Meters => {
                for m in &mut self.meters {
                    *m = (*m + (rng.float() - 0.5) * 30.0).clamp(1.0, 100.0);
                }
            }
            Kind::Progress => {
                for t in &mut self.tasks {
                    t.1 += t.2 * rng.float() * 2.0;
                    if t.1 >= 100.0 {
                        *t = (Pane::task_name(rng), 0.0, 0.5 + rng.float() * 3.0);
                    }
                }
            }
            Kind::Scope => self.phase += 0.25,
            Kind::Rain => {
                for d in &mut self.rain {
                    d.0 += d.1;
                    if d.0 > h as f64 + 10.0 {
                        *d = (rng.float() * -10.0, 0.3 + rng.float());
                    }
                }
            }
            _ => {
                for _ in 0..rng.range(1, 3) {
                    if let Some(l) = self.line(rng) {
                        self.lines.push_back(l);
                    }
                }
                while self.lines.len() > h.max(1) {
                    self.lines.pop_front();
                }
            }
        }
    }

    fn draw(&self, c: &mut Canvas, x0: i64, y0: i64, w: usize, h: usize, rng: &mut Rng) {
        // Border with the title.
        let (x1, y1) = (x0 + w as i64 - 1, y0 + h as i64 - 1);
        for x in x0..=x1 {
            c.set(x, y0, '─', Some(DIM), false);
            c.set(x, y1, '─', Some(DIM), false);
        }
        for y in y0..=y1 {
            c.set(x0, y, '│', Some(DIM), false);
            c.set(x1, y, '│', Some(DIM), false);
        }
        c.set(x0, y0, '┌', Some(DIM), false);
        c.set(x1, y0, '┐', Some(DIM), false);
        c.set(x0, y1, '└', Some(DIM), false);
        c.set(x1, y1, '┘', Some(DIM), false);
        c.text(x0 + 2, y0, &format!(" {} ", self.kind.title()), Some(GREEN), false);
        let (ix, iy, iw, ih) = (x0 + 1, y0 + 1, w.saturating_sub(2), h.saturating_sub(2));
        let put = |c: &mut Canvas, row: usize, parts: &[(String, Rgb)]| {
            let mut x = 0usize;
            for (s, col) in parts {
                for ch in s.chars() {
                    if x >= iw {
                        return;
                    }
                    c.set(ix + x as i64, iy + row as i64, ch, Some(*col), false);
                    x += 1;
                }
            }
        };
        match self.kind {
            Kind::Procs => {
                put(c, 0, &[(format!("{:>6} {:<10} {:>5} {:>5}", "PID", "COMMAND", "%CPU", "%MEM"), YELLOW)]);
                for (i, p) in self.procs.iter().take(ih.saturating_sub(1)).enumerate() {
                    let col = if p.2 > 70.0 { RED } else if p.2 > 30.0 { YELLOW } else { GREEN };
                    put(c, i + 1, &[(format!("{:>6} {:<10} ", p.1, p.0), WHITE), (format!("{:>5.1} {:>5.1}", p.2, p.3), col)]);
                }
            }
            Kind::Meters => {
                for (i, m) in self.meters.iter().enumerate().take(ih) {
                    let bar_w = iw.saturating_sub(12);
                    let n = (*m / 100.0 * bar_w as f64) as usize;
                    let col = if *m > 80.0 { RED } else if *m > 50.0 { YELLOW } else { GREEN };
                    put(c, i, &[(format!("cpu{i} ["), WHITE), ("|".repeat(n), col), (" ".repeat(bar_w - n), col), (format!("]{:>4.0}%", m), WHITE)]);
                }
            }
            Kind::Progress => {
                for (i, t) in self.tasks.iter().enumerate() {
                    if 2 * i + 1 >= ih {
                        break;
                    }
                    let bar_w = iw.saturating_sub(8);
                    let n = (t.1 / 100.0 * bar_w as f64) as usize;
                    put(c, 2 * i, &[(t.0.clone(), WHITE)]);
                    put(c, 2 * i + 1, &[("[".into(), DIM), ("#".repeat(n), GREEN), (".".repeat(bar_w - n), DIM), (format!("]{:>4.0}%", t.1), CYAN)]);
                }
            }
            Kind::Scope => {
                let mid = ih as f64 / 2.0;
                for x in 0..iw {
                    let v = (self.phase + x as f64 * 0.25).sin() * 0.6 + (self.phase * 1.7 + x as f64 * 0.07).sin() * 0.3;
                    let y = (mid - v * mid * 0.9).clamp(0.0, ih as f64 - 1.0) as i64;
                    c.set(ix + x as i64, iy + y, '•', Some(CYAN), false);
                    if (x as i64 + self.phase as i64) % 8 == 0 {
                        c.set(ix + x as i64, iy + mid as i64, '·', Some(DIM), false);
                    }
                }
            }
            Kind::Rain => {
                for (x, d) in self.rain.iter().enumerate().take(iw) {
                    for k in 0..6 {
                        let y = d.0 as i64 - k;
                        if y >= 0 && (y as usize) < ih {
                            let ch = char::from(b'0' + rng.below(10) as u8);
                            c.set(ix + x as i64, iy + y, ch, Some(if k == 0 { WHITE } else { DIM }), false);
                        }
                    }
                }
            }
            _ => {
                let skip = self.lines.len().saturating_sub(ih);
                for (i, l) in self.lines.iter().skip(skip).enumerate() {
                    put(c, i, l);
                }
            }
        }
    }
}

/// Columns × rows of panes for the terminal size (or for `n` panes).
fn grid(w: usize, h: usize, n: Option<usize>) -> (usize, usize) {
    match n {
        Some(n) => {
            let cols = (n as f64).sqrt().ceil() as usize;
            (cols, n.div_ceil(cols))
        }
        None => ((w / 55).clamp(1, 3), (h / 14).clamp(1, 3)),
    }
}

fn main() {
    let mut cli = Cli::new("rhollywood");
    let t = Lang::pick(cli.lang, &EN, &PT, &ES);
    let (mut panes, mut interval, mut speed) = (None::<usize>, 10.0f64, 5u32);
    while let Some(a) = cli.next() {
        match a {
            Arg::Pos(p) => cli.unknown(&p),
            Arg::Opt(opt) => match opt.as_str() {
                "-h" | "--help" => cli.help(t.help),
                "-V" | "--version" => cli.version(),
                "-p" | "--panes" => panes = Some(cli.number(&opt, 1..=9)),
                "-i" | "--interval" => interval = cli.number(&opt, 0.0..=3600.0),
                "-s" | "--speed" => speed = cli.number(&opt, 1..=10),
                _ => cli.unknown(&opt),
            },
        }
    }
    if !std::io::stdout().is_terminal() {
        die("rhollywood", common(cli.lang).not_a_terminal);
    }
    if let Err(e) = run(panes, interval, speed) {
        die("rhollywood", &e.to_string());
    }
}

fn shuffle(n: usize, rng: &mut Rng) -> Vec<Pane> {
    let mut kinds = KINDS.to_vec();
    for i in (1..kinds.len()).rev() {
        kinds.swap(i, rng.below(i + 1));
    }
    (0..n).map(|i| Pane::new(kinds[i % kinds.len()], rng)).collect()
}

fn run(panes: Option<usize>, interval: f64, speed: u32) -> std::io::Result<()> {
    let depth = term::depth(Some(true));
    let _screen = Screen::enter()?;
    let mut out = std::io::stdout().lock();
    let mut rng = Rng::new();
    let (w, h) = term::size();
    let mut c = Canvas::new(w, h);
    let (mut cols, mut rows) = grid(w, h, panes);
    let mut list = shuffle(cols * rows, &mut rng);
    let frame = Duration::from_millis(30 + 20 * (11 - speed as u64));
    let mut last_change = Instant::now();
    loop {
        let start = Instant::now();
        if interval > 0.0 && last_change.elapsed().as_secs_f64() >= interval {
            // One pane turns into something else, as when hollywood splits a window.
            let i = rng.below(list.len());
            let used: Vec<Kind> = list.iter().map(|p| p.kind).collect();
            let free: Vec<Kind> = KINDS.iter().copied().filter(|k| !used.contains(k)).collect();
            let k = if free.is_empty() { *rng.pick(KINDS) } else { *rng.pick(&free) };
            list[i] = Pane::new(k, &mut rng);
            last_change = Instant::now();
        }
        c.clear();
        let (pw, ph) = (c.w / cols, c.h / rows);
        for (i, p) in list.iter_mut().enumerate() {
            let (cx, cy) = (i % cols, i / cols);
            // The last column and row take what is left.
            let wpx = if cx + 1 == cols { c.w - pw * cx } else { pw };
            let hpx = if cy + 1 == rows { c.h - ph * cy } else { ph };
            p.step(&mut rng, hpx.saturating_sub(2));
            p.draw(&mut c, (pw * cx) as i64, (ph * cy) as i64, wpx, hpx, &mut rng);
        }
        c.flush(&mut out, depth)?;
        match tui::poll(frame.saturating_sub(start.elapsed()))? {
            Input::Quit => return Ok(()),
            Input::Key(' ') => list = shuffle(cols * rows, &mut rng),
            Input::Resize => {
                let (w, h) = term::size();
                c.resize(w, h);
                (cols, rows) = grid(w, h, panes);
                list = shuffle(cols * rows, &mut rng);
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
    fn every_pane_draws() {
        let mut rng = Rng::seeded(2);
        let mut c = Canvas::new(120, 40);
        for &k in KINDS {
            let mut p = Pane::new(k, &mut rng);
            for _ in 0..50 {
                p.step(&mut rng, 18);
                p.draw(&mut c, 0, 0, 60, 20, &mut rng);
            }
        }
        assert_eq!(grid(120, 40, Some(5)), (3, 2));
        assert_eq!(grid(170, 50, None), (3, 3));
    }
}

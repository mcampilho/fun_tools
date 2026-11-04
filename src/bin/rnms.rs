//! rnms: "no more secrets" (like nms): text appears scrambled, then decrypts
//! itself character by character, as in the films.

use crossterm::terminal;
use fun::cli::{die, Arg, Cli};
use fun::i18n::{common, Lang};
use fun::rng::Rng;
use fun::term::{self, Depth, Rgb};
use fun::text::width;
use fun::tui::{self, Input};
use std::io::{IsTerminal, Write};
use std::time::{Duration, Instant};

struct Texts {
    help: &'static str,
    demo: &'static str,
}

const EN: Texts = Texts {
    help: "\
Usage: rnms [options] [text...]
       command | rnms [options]

Shows the text scrambled, then decrypts it character by character.
Without -a it waits for a key before decrypting.

  -a, --auto           Decrypt without waiting for a key
  -s, --spaces         Scramble the spaces too
  -f, --color <name>   Colour of the decrypted text: white, yellow, black, magenta,
                       blue, green, red, cyan (default blue)
  -c, --clear          Clear the screen first
  -t, --time <s>       Seconds of scrambling before decrypting (default 2)
  -u, --unicode        Scramble with block and geometric symbols too
      --lang <code>    Language: en, pt, es (also: RTOOLS_LANG)
  -h, --help           Show this help
  -V, --version        Show the version
",
    demo: "ACCESS GRANTED\n\nWelcome back, operator.\nAll systems are online and every secret is safe.\n\n(Give rnms your own text: echo hello | rnms)",
};

const PT: Texts = Texts {
    help: "\
Utilização: rnms [opções] [texto...]
            comando | rnms [opções]

Mostra o texto baralhado e depois desencripta-o carácter a carácter.
Sem -a espera por uma tecla antes de desencriptar.

  -a, --auto           Desencriptar sem esperar por uma tecla
  -s, --spaces         Baralhar também os espaços
  -f, --color <nome>   Cor do texto desencriptado: white, yellow, black, magenta,
                       blue, green, red, cyan (por omissão blue)
  -c, --clear          Limpar o ecrã primeiro
  -t, --time <s>       Segundos a baralhar antes de desencriptar (por omissão 2)
  -u, --unicode        Baralhar também com símbolos de blocos e geométricos
      --lang <código>  Idioma: en, pt, es (também: RTOOLS_LANG)
  -h, --help           Mostrar esta ajuda
  -V, --version        Mostrar a versão
",
    demo: "ACESSO AUTORIZADO\n\nBem-vindo de volta, operador.\nTodos os sistemas estão ligados e os segredos estão a salvo.\n\n(Dê ao rnms o seu próprio texto: echo olá | rnms)",
};

const ES: Texts = Texts {
    help: "\
Uso: rnms [opciones] [texto...]
     comando | rnms [opciones]

Muestra el texto revuelto y luego lo descifra carácter a carácter.
Sin -a espera una tecla antes de descifrar.

  -a, --auto           Descifrar sin esperar una tecla
  -s, --spaces         Revolver también los espacios
  -f, --color <nombre> Color del texto descifrado: white, yellow, black, magenta,
                       blue, green, red, cyan (por defecto blue)
  -c, --clear          Limpiar la pantalla primero
  -t, --time <s>       Segundos revolviendo antes de descifrar (por defecto 2)
  -u, --unicode        Revolver también con símbolos de bloques y geométricos
      --lang <código>  Idioma: en, pt, es (también: RTOOLS_LANG)
  -h, --help           Mostrar esta ayuda
  -V, --version        Mostrar la versión
",
    demo: "ACCESO CONCEDIDO\n\nBienvenido de nuevo, operador.\nTodos los sistemas están en línea y los secretos están a salvo.\n\n(Dé a rnms su propio texto: echo hola | rnms)",
};

const NAMED: &[(&str, Rgb)] = &[
    ("white", Rgb(240, 240, 240)),
    ("yellow", Rgb(245, 215, 60)),
    ("black", Rgb(20, 20, 20)),
    ("magenta", Rgb(220, 80, 220)),
    ("blue", Rgb(70, 130, 255)),
    ("green", Rgb(60, 220, 90)),
    ("red", Rgb(235, 60, 60)),
    ("cyan", Rgb(60, 215, 225)),
];

const SYMBOLS: &str = "░▒▓█▀▄▌▐■□▪▫▲△▼▽◆◇○●◐◑◒◓◢◣◤◥";

/// One character of the text: what it really is, what it shows meanwhile, and
/// when it is revealed (seconds after decrypting starts).
struct Glyph {
    real: char,
    shown: char,
    reveal: f64,
}

struct Secret {
    lines: Vec<Vec<Glyph>>,
    pool: Vec<char>,
    rng: Rng,
}

impl Secret {
    fn new(text: &str, spaces: bool, unicode: bool, mut rng: Rng) -> Secret {
        let mut pool: Vec<char> = (33u8..127).map(char::from).collect();
        if unicode {
            pool.extend(SYMBOLS.chars());
        }
        let lines = text
            .lines()
            .map(|l| {
                l.chars()
                    .map(|c| {
                        // Wide characters (CJK, emoji) stay as they are: a narrow mask would move the rest.
                        let masked = (spaces || !c.is_whitespace()) && width(&c.to_string()) == 1;
                        Glyph { real: c, shown: if masked { *rng.pick(&pool) } else { c }, reveal: if masked { rng.float().powf(0.7) * 4.0 } else { 0.0 } }
                    })
                    .collect()
            })
            .collect();
        Secret { lines, pool, rng }
    }

    /// Changes some masks; after `t` seconds of decrypting, the due ones show their real character.
    fn tick(&mut self, decrypting: Option<f64>) {
        for g in self.lines.iter_mut().flatten() {
            if g.shown == g.real && (decrypting.is_some() || g.reveal == 0.0) {
                continue;
            }
            match decrypting {
                Some(t) if t >= g.reveal => g.shown = g.real,
                _ => {
                    if self.rng.chance(0.15) {
                        g.shown = *self.rng.pick(&self.pool);
                    }
                }
            }
        }
    }

    fn done(&self) -> bool {
        self.lines.iter().flatten().all(|g| g.shown == g.real)
    }

    /// The text as it is now: revealed characters in `color` and bold, the others plain.
    fn frame(&self, depth: Depth, color: Rgb, decrypting: bool) -> String {
        let mut s = String::new();
        for (i, line) in self.lines.iter().enumerate() {
            if i > 0 {
                s.push_str("\r\n");
            }
            s.push_str("\x1b[2K");
            for g in line {
                let revealed = decrypting && g.shown == g.real && !g.real.is_whitespace();
                if revealed && depth != Depth::None {
                    s.push_str(&format!("\x1b[1m{}{}{}", term::fg(depth, color), g.shown, term::RESET));
                } else {
                    s.push(g.shown);
                }
            }
        }
        s
    }
}

/// Restores the terminal (raw mode, cursor) when dropped.
struct Raw;

impl Drop for Raw {
    fn drop(&mut self) {
        let _ = terminal::disable_raw_mode();
        let mut out = std::io::stdout();
        let _ = write!(out, "\x1b[0m\x1b[?25h");
        let _ = out.flush();
    }
}

fn main() {
    let mut cli = Cli::new("rnms");
    let t = Lang::pick(cli.lang, &EN, &PT, &ES);
    let (mut auto, mut spaces, mut color, mut clear, mut time, mut unicode) = (false, false, NAMED[4].1, false, 2.0f64, false);
    let mut words = Vec::new();
    while let Some(a) = cli.next() {
        match a {
            Arg::Pos(p) => words.push(p),
            Arg::Opt(opt) => match opt.as_str() {
                "-h" | "--help" => cli.help(t.help),
                "-V" | "--version" => cli.version(),
                "-a" | "--auto" => auto = true,
                "-s" | "--spaces" => spaces = true,
                "-c" | "--clear" => clear = true,
                "-u" | "--unicode" => unicode = true,
                "-t" | "--time" => time = cli.number(&opt, 0.0..=60.0),
                "-f" | "--color" => {
                    let v = cli.value(&opt).to_ascii_lowercase();
                    color = match NAMED.iter().find(|(n, _)| *n == v) {
                        Some(&(_, c)) => c,
                        None => cli.fail(&format!("{} '{opt}': {v}", common(cli.lang).invalid_value)),
                    };
                }
                _ => cli.unknown(&opt),
            },
        }
    }
    let text = if !words.is_empty() {
        words.join(" ")
    } else {
        fun::input::piped_stdin().filter(|s| !s.trim().is_empty()).unwrap_or_else(|| t.demo.to_string())
    };
    let text = fun::text::expand_tabs(&fun::input::clean(&text), 8);
    if !std::io::stdout().is_terminal() {
        // Not a terminal: nothing to animate, the secret is just told.
        print!("{text}");
        if !text.ends_with('\n') {
            println!();
        }
        return;
    }
    let depth = term::depth(None);
    // The animation needs escape codes even without colours (NO_COLOR).
    term::enable_ansi();
    if let Err(e) = run(&text, auto, spaces, unicode, color, clear, time, depth) {
        die("rnms", &e.to_string());
    }
}

#[allow(clippy::too_many_arguments)]
fn run(text: &str, auto: bool, spaces: bool, unicode: bool, color: Rgb, clear: bool, time: f64, depth: Depth) -> std::io::Result<()> {
    let (cols, rows) = term::size();
    // Lines wider than the terminal would wrap and break the redrawing: cut them,
    // and keep to one screen.
    let mut lines: Vec<String> = text.lines().map(|l| l.chars().take(cols.saturating_sub(1)).collect()).collect();
    lines.truncate(rows.saturating_sub(1).max(1));
    let mut secret = Secret::new(&lines.join("\n"), spaces, unicode, Rng::new());
    let n = secret.lines.len().max(1);

    let mut out = std::io::stdout().lock();
    if clear {
        write!(out, "\x1b[2J\x1b[H")?;
    }
    // Make room, go back up and remember where the text starts.
    write!(out, "\x1b[?25l{}\x1b[{n}A\r\x1b7", "\n".repeat(n))?;
    terminal::enable_raw_mode()?;
    let _raw = Raw;
    let step = Duration::from_millis(35);
    let draw = |out: &mut std::io::StdoutLock, s: &Secret, dec: bool| -> std::io::Result<()> {
        write!(out, "\x1b8{}", s.frame(depth, color, dec))?;
        out.flush()
    };
    // Scrambling.
    let start = Instant::now();
    while start.elapsed().as_secs_f64() < time {
        secret.tick(None);
        draw(&mut out, &secret, false)?;
        if let Input::Quit = tui::poll(step)? {
            return finish(&mut out, n);
        }
    }
    // Wait for a key (the masks keep changing).
    if !auto {
        loop {
            secret.tick(None);
            draw(&mut out, &secret, false)?;
            match tui::poll(step)? {
                Input::Quit => return finish(&mut out, n),
                Input::Key(_) => break,
                _ => {}
            }
        }
    }
    // Decrypting.
    let start = Instant::now();
    while !secret.done() {
        secret.tick(Some(start.elapsed().as_secs_f64()));
        draw(&mut out, &secret, true)?;
        if let Input::Quit = tui::poll(step)? {
            break;
        }
    }
    for g in secret.lines.iter_mut().flatten() {
        g.shown = g.real;
    }
    draw(&mut out, &secret, true)?;
    finish(&mut out, n)
}

/// Leaves the cursor below the text.
fn finish(out: &mut std::io::StdoutLock, n: usize) -> std::io::Result<()> {
    write!(out, "\x1b8\x1b[{n}B\r")?;
    out.flush()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scrambles_then_reveals() {
        let mut s = Secret::new("Olá mundo\n日本 ok", false, true, Rng::seeded(2));
        assert!(s.lines[0][3].shown == ' ' && s.lines[1][0].shown == '日');
        for i in 0..200 {
            s.tick(Some(i as f64 * 0.05));
        }
        assert!(s.done());
        let plain = s.frame(Depth::None, NAMED[4].1, true);
        assert_eq!(plain.replace("\x1b[2K", ""), "Olá mundo\r\n日本 ok");
    }
}

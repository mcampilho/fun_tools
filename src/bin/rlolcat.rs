//! rlolcat: rainbow-coloured text (like lolcat), line by line as it arrives.

use fun::cli::{Arg, Cli};
use fun::i18n::{common, Lang};
use fun::term::{self, Depth, Rgb};
use fun::text::{skip_escape, width};
use std::io::{self, BufWriter, IsTerminal, Write};
use std::time::Duration;

struct Texts {
    help: &'static str,
}

const EN: Texts = Texts {
    help: "\
Usage: rlolcat [options] [file...]

Rainbow-coloured text. Reads the files (\"-\" or none: stdin).
Without a terminal the text goes out unchanged, unless --force.

  -p, --spread <f>     Rainbow spread: characters per colour step (default 3.0)
  -F, --freq <f>       Rainbow frequency (default 0.1)
  -S, --seed <n>       Starting colour (default: random)
  -i, --invert         Colour the background instead of the text
  -a, --animate        Animate each line (terminal only)
  -d, --duration <n>   Animation frames per line (default 12)
  -s, --speed <f>      Animation frames per second (default 20)
  -f, --force          Colour even when the output is not a terminal
      --lang <code>    Language: en, pt, es (also: RTOOLS_LANG)
  -h, --help           Show this help
  -V, --version        Show the version
",
};

const PT: Texts = Texts {
    help: "\
Utilização: rlolcat [opções] [ficheiro...]

Texto com as cores do arco-íris. Lê os ficheiros (\"-\" ou nenhum: stdin).
Sem terminal, o texto sai sem alterações, a não ser com --force.

  -p, --spread <f>     Espalhamento: caracteres por passo de cor (por omissão 3.0)
  -F, --freq <f>       Frequência do arco-íris (por omissão 0.1)
  -S, --seed <n>       Cor inicial (por omissão: ao acaso)
  -i, --invert         Colorir o fundo em vez do texto
  -a, --animate        Animar cada linha (só num terminal)
  -d, --duration <n>   Imagens da animação por linha (por omissão 12)
  -s, --speed <f>      Imagens da animação por segundo (por omissão 20)
  -f, --force          Colorir mesmo quando a saída não é um terminal
      --lang <código>  Idioma: en, pt, es (também: RTOOLS_LANG)
  -h, --help           Mostrar esta ajuda
  -V, --version        Mostrar a versão
",
};

const ES: Texts = Texts {
    help: "\
Uso: rlolcat [opciones] [fichero...]

Texto con los colores del arco iris. Lee los ficheros (\"-\" o ninguno: stdin).
Sin terminal, el texto sale sin cambios, salvo con --force.

  -p, --spread <f>     Dispersión: caracteres por paso de color (por defecto 3.0)
  -F, --freq <f>       Frecuencia del arco iris (por defecto 0.1)
  -S, --seed <n>       Color inicial (por defecto: al azar)
  -i, --invert         Colorear el fondo en vez del texto
  -a, --animate        Animar cada línea (solo en un terminal)
  -d, --duration <n>   Imágenes de la animación por línea (por defecto 12)
  -s, --speed <f>      Imágenes de la animación por segundo (por defecto 20)
  -f, --force          Colorear aunque la salida no sea un terminal
      --lang <código>  Idioma: en, pt, es (también: RTOOLS_LANG)
  -h, --help           Mostrar esta ayuda
  -V, --version        Mostrar la versión
",
};

struct Opts {
    spread: f64,
    freq: f64,
    invert: bool,
    animate: bool,
    duration: u32,
    speed: f64,
}

/// One line in rainbow colours, starting at colour position `pos`.
/// Escape codes already in the text are dropped (they would fight the rainbow).
fn paint(line: &str, pos: f64, o: &Opts, depth: Depth, out: &mut String) {
    let mut chars = line.chars().peekable();
    let mut col = 0usize;
    let mut last = String::new();
    while let Some(c) = chars.next() {
        if c == '\x1b' {
            skip_escape(&mut chars);
            continue;
        }
        let rgb = term::rainbow(o.freq, pos + col as f64 / o.spread);
        let code = if o.invert {
            format!("{}{}", term::fg(depth, Rgb(0, 0, 0)), term::color(depth, rgb, true))
        } else {
            term::fg(depth, rgb)
        };
        // Spaces need no colour unless the background is coloured.
        if (o.invert || c != ' ') && code != last {
            out.push_str(&code);
            last = code;
        }
        out.push(c);
        col += width(c.encode_utf8(&mut [0; 4]));
    }
    if !last.is_empty() {
        out.push_str(term::RESET);
    }
}

fn main() {
    let mut cli = Cli::new("rlolcat");
    let t = Lang::pick(cli.lang, &EN, &PT, &ES);
    let mut o = Opts { spread: 3.0, freq: 0.1, invert: false, animate: false, duration: 12, speed: 20.0 };
    let mut seed: Option<f64> = None;
    let mut force = false;
    let mut files = Vec::new();
    while let Some(a) = cli.next() {
        match a {
            Arg::Pos(p) => files.push(p),
            Arg::Opt(opt) => match opt.as_str() {
                "-h" | "--help" => cli.help(t.help),
                "-V" | "--version" => cli.version(),
                "-p" | "--spread" => o.spread = cli.number(&opt, 0.1..=1000.0),
                "-F" | "--freq" => o.freq = cli.number(&opt, 0.0..=100.0),
                "-S" | "--seed" => seed = Some(cli.parse::<u32>(&opt) as f64),
                "-i" | "--invert" => o.invert = true,
                "-a" | "--animate" => o.animate = true,
                "-d" | "--duration" => o.duration = cli.number(&opt, 1..=1000),
                "-s" | "--speed" => o.speed = cli.number(&opt, 0.1..=1000.0),
                "-f" | "--force" => force = true,
                _ => cli.unknown(&opt),
            },
        }
    }
    if files.is_empty() {
        files.push("-".into());
    }
    let depth = term::depth(force.then_some(true));
    let tty = io::stdout().is_terminal();
    let animate = o.animate && tty && depth != Depth::None;
    let mut pos = seed.unwrap_or_else(|| fun::rng::Rng::new().below(256) as f64);

    let stdout = io::stdout();
    let mut out = BufWriter::new(stdout.lock());
    let mut buf = Vec::new();
    let mut s = String::new();
    if animate {
        let _ = write!(out, "\x1b[?25l");
    }
    let mut status = 0;
    'files: for f in &files {
        let mut reader = match fun::input::open(f) {
            Ok(r) => r,
            Err(e) => {
                eprintln!("rlolcat: {} '{f}': {e}", common(cli.lang).cannot_read);
                status = 1;
                continue;
            }
        };
        let mut first = true;
        loop {
            let line = match fun::input::read_line(&mut *reader, &mut buf, first) {
                Ok(Some(l)) => l,
                Ok(None) => break,
                Err(e) => {
                    eprintln!("rlolcat: {} '{f}': {e}", common(cli.lang).cannot_read);
                    status = 1;
                    break;
                }
            };
            first = false;
            let res = if depth == Depth::None {
                writeln!(out, "{line}")
            } else if animate {
                let mut r = Ok(());
                for frame in 0..o.duration {
                    s.clear();
                    paint(&line, pos + frame as f64 * o.spread / 4.0, &o, depth, &mut s);
                    r = write!(out, "\r{s}").and_then(|_| out.flush());
                    std::thread::sleep(Duration::from_secs_f64(1.0 / o.speed));
                }
                r.and_then(|_| writeln!(out))
            } else {
                s.clear();
                paint(&line, pos, &o, depth, &mut s);
                writeln!(out, "{s}")
            };
            pos += 1.0;
            // Show each line as soon as it is ready when writing to a terminal.
            let res = res.and_then(|_| if tty { out.flush() } else { Ok(()) });
            if res.is_err() {
                // Closed pipe (e.g. `| head`): stop quietly.
                break 'files;
            }
        }
    }
    if animate {
        let _ = write!(out, "\x1b[?25h");
    }
    // A failed flush means the reader went away: nothing to report.
    let _ = out.flush();
    drop(out);
    std::process::exit(status);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn paints_and_drops_old_colours() {
        let o = Opts { spread: 3.0, freq: 0.1, invert: false, animate: false, duration: 1, speed: 1.0 };
        let mut s = String::new();
        paint("\x1b[31mab c", 0.0, &o, Depth::Ansi256, &mut s);
        assert_eq!(fun::text::strip_ansi(&s), "ab c");
        assert!(!s.contains("[31m"));
        assert!(s.ends_with(term::RESET));
    }
}

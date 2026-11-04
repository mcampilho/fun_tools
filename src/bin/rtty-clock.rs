//! rtty-clock: a big digital clock (like tty-clock). q, Esc or Ctrl+C to quit.

use chrono::{Datelike, Local, Timelike, Utc};
use fun::cli::{die, Arg, Cli};
use fun::i18n::{common, Lang};
use fun::pixfont;
use fun::term::{self, Rgb};
use fun::tui::{self, Canvas, Input, Screen};
use std::io::IsTerminal;
use std::time::Duration;

struct Texts {
    help: &'static str,
    days: [&'static str; 7],
    months: [&'static str; 12],
    /// Date layout: {wd} weekday, {d} day, {m} month name, {y} year.
    date: &'static str,
    unknown_color: &'static str,
}

const EN: Texts = Texts {
    help: "\
Usage: rtty-clock [options]

A big digital clock. Keys while it runs: q/Esc quit, s seconds, t 12/24 hours,
d date, r bounce, x box, b blink, 1 to 8 colour.

  -s, --seconds        Show the seconds
  -t, --twelve         12-hour clock
  -u, --utc            UTC instead of local time
  -D, --no-date        Hide the date
  -f, --format <fmt>   Date format, strftime style (e.g. \"%Y-%m-%d\")
  -C, --color <name>   green, red, blue, yellow, cyan, magenta, white, orange
                       (default green)
  -x, --box            Draw a box around the clock
  -B, --blink          Blink the colons
  -r, --rebound        Bounce around the screen
  -S, --screensaver    Quit on any key
  -m, --mini           Small digits (also used when the terminal is too small)
      --lang <code>    Language: en, pt, es (also: RTOOLS_LANG)
  -h, --help           Show this help
  -V, --version        Show the version
",
    days: ["Monday", "Tuesday", "Wednesday", "Thursday", "Friday", "Saturday", "Sunday"],
    months: [
        "January", "February", "March", "April", "May", "June", "July", "August", "September", "October",
        "November", "December",
    ],
    date: "{wd}, {m} {d}, {y}",
    unknown_color: "unknown colour",
};

const PT: Texts = Texts {
    help: "\
Utilização: rtty-clock [opções]

Um relógio digital grande. Teclas enquanto corre: q/Esc sair, s segundos, t 12/24 horas,
d data, r saltar, x caixa, b piscar, 1 a 8 cor.

  -s, --seconds        Mostrar os segundos
  -t, --twelve         Relógio de 12 horas
  -u, --utc            UTC em vez da hora local
  -D, --no-date        Esconder a data
  -f, --format <fmt>   Formato da data, ao estilo strftime (ex.: \"%Y-%m-%d\")
  -C, --color <nome>   green, red, blue, yellow, cyan, magenta, white, orange
                       (por omissão green)
  -x, --box            Desenhar uma caixa à volta do relógio
  -B, --blink          Fazer piscar os dois pontos
  -r, --rebound        Andar aos saltos pelo ecrã
  -S, --screensaver    Sair com qualquer tecla
  -m, --mini           Algarismos pequenos (também quando o terminal é pequeno)
      --lang <código>  Idioma: en, pt, es (também: RTOOLS_LANG)
  -h, --help           Mostrar esta ajuda
  -V, --version        Mostrar a versão
",
    days: ["segunda-feira", "terça-feira", "quarta-feira", "quinta-feira", "sexta-feira", "sábado", "domingo"],
    months: [
        "janeiro", "fevereiro", "março", "abril", "maio", "junho", "julho", "agosto", "setembro", "outubro",
        "novembro", "dezembro",
    ],
    date: "{wd}, {d} de {m} de {y}",
    unknown_color: "cor desconhecida",
};

const ES: Texts = Texts {
    help: "\
Uso: rtty-clock [opciones]

Un reloj digital grande. Teclas mientras funciona: q/Esc salir, s segundos, t 12/24 horas,
d fecha, r rebotar, x caja, b parpadeo, 1 a 8 color.

  -s, --seconds        Mostrar los segundos
  -t, --twelve         Reloj de 12 horas
  -u, --utc            UTC en vez de la hora local
  -D, --no-date        Ocultar la fecha
  -f, --format <fmt>   Formato de la fecha, al estilo strftime (p. ej. \"%Y-%m-%d\")
  -C, --color <nombre> green, red, blue, yellow, cyan, magenta, white, orange
                       (por defecto green)
  -x, --box            Dibujar una caja alrededor del reloj
  -B, --blink          Hacer parpadear los dos puntos
  -r, --rebound        Rebotar por la pantalla
  -S, --screensaver    Salir con cualquier tecla
  -m, --mini           Cifras pequeñas (también cuando el terminal es pequeño)
      --lang <código>  Idioma: en, pt, es (también: RTOOLS_LANG)
  -h, --help           Mostrar esta ayuda
  -V, --version        Mostrar la versión
",
    days: ["lunes", "martes", "miércoles", "jueves", "viernes", "sábado", "domingo"],
    months: [
        "enero", "febrero", "marzo", "abril", "mayo", "junio", "julio", "agosto", "septiembre", "octubre",
        "noviembre", "diciembre",
    ],
    date: "{wd}, {d} de {m} de {y}",
    unknown_color: "color desconocido",
};

const COLORS: &[(&str, Rgb)] = &[
    ("green", Rgb(40, 200, 80)),
    ("red", Rgb(225, 60, 70)),
    ("blue", Rgb(60, 120, 255)),
    ("yellow", Rgb(240, 200, 40)),
    ("cyan", Rgb(40, 200, 210)),
    ("magenta", Rgb(200, 60, 200)),
    ("white", Rgb(230, 230, 230)),
    ("orange", Rgb(255, 140, 30)),
];

struct Opts {
    seconds: bool,
    twelve: bool,
    utc: bool,
    date: bool,
    format: Option<String>,
    color: Rgb,
    boxed: bool,
    blink: bool,
    rebound: bool,
    screensaver: bool,
    mini: bool,
}

struct Now {
    h: u32,
    m: u32,
    s: u32,
    pm: bool,
    date: String,
}

fn now(o: &Opts, t: &Texts) -> Now {
    // Local and UTC times have different types; take the parts we need from either.
    macro_rules! parts {
        ($dt:expr) => {{
            let dt = $dt;
            let date = match &o.format {
                Some(f) => dt.format(f).to_string(),
                None => t
                    .date
                    .replace("{wd}", t.days[dt.weekday().num_days_from_monday() as usize])
                    .replace("{d}", &dt.day().to_string())
                    .replace("{m}", t.months[dt.month0() as usize])
                    .replace("{y}", &dt.year().to_string()),
            };
            (dt.hour(), dt.minute(), dt.second(), date)
        }};
    }
    let (h, m, s, date) = if o.utc { parts!(Utc::now()) } else { parts!(Local::now()) };
    let pm = h >= 12;
    let h = if o.twelve { (h + 11) % 12 + 1 } else { h };
    Now { h, m, s, pm, date }
}

/// The time as pixels (rows of on/off).
fn time_pixels(n: &Now, o: &Opts, colon_on: bool) -> Vec<Vec<bool>> {
    let mut text = format!("{:02}:{:02}", n.h, n.m);
    if o.seconds {
        text.push_str(&format!(":{:02}", n.s));
    }
    // Same-width digits: pixfont's proportional bitmap would make "1" narrow and
    // the clock would jump around; draw each character in a fixed 5-pixel box.
    let mut rows = vec![Vec::new(); 7];
    for (i, c) in text.chars().enumerate() {
        let g = pixfont::glyph(c);
        let (from, to) = if c == ':' { (2, 3) } else { (0, 5) };
        for (y, row) in rows.iter_mut().enumerate() {
            if i > 0 {
                row.push(false);
            }
            for x in from..to {
                row.push(g[y + 1][x] && (c != ':' || colon_on));
            }
        }
    }
    rows
}

fn draw(c: &mut Canvas, n: &Now, o: &Opts, pos: (i64, i64), colon_on: bool) -> (i64, i64) {
    let px = time_pixels(n, o, colon_on);
    let pw = px[0].len() as i64;
    // Big digits: two columns by one row per pixel; mini: half blocks.
    let big = !o.mini && (pw * 2 + 4) as usize <= c.w && 7 + 5 <= c.h;
    let (w, h) = if big { (pw * 2, 7) } else { (pw, 4) };
    let suffix = if o.twelve { if n.pm { " PM" } else { " AM" } } else { "" };
    let date = if o.date { format!("{}{suffix}", n.date) } else { suffix.trim().to_string() };
    let total_w = w.max(date.chars().count() as i64) + 4;
    let total_h = h + if date.is_empty() { 0 } else { 2 } + 2;
    let (x0, y0) = pos;
    c.clear();
    if o.boxed {
        let (r, b) = (x0 + total_w - 1, y0 + total_h - 1);
        for x in x0..=r {
            c.set(x, y0, '─', Some(o.color), false);
            c.set(x, b, '─', Some(o.color), false);
        }
        for y in y0..=b {
            c.set(x0, y, '│', Some(o.color), false);
            c.set(r, y, '│', Some(o.color), false);
        }
        c.set(x0, y0, '┌', Some(o.color), false);
        c.set(r, y0, '┐', Some(o.color), false);
        c.set(x0, b, '└', Some(o.color), false);
        c.set(r, b, '┘', Some(o.color), false);
    }
    let dx = x0 + (total_w - w) / 2;
    let dy = y0 + 1;
    if big {
        for (y, row) in px.iter().enumerate() {
            for (x, &on) in row.iter().enumerate() {
                if on {
                    c.paint(dx + 2 * x as i64, dy + y as i64, o.color);
                    c.paint(dx + 2 * x as i64 + 1, dy + y as i64, o.color);
                }
            }
        }
    } else {
        for r in 0..4 {
            for x in 0..pw as usize {
                let top = px.get(2 * r).is_some_and(|row| row[x]);
                let bot = px.get(2 * r + 1).is_some_and(|row| row[x]);
                let ch = match (top, bot) {
                    (true, true) => '█',
                    (true, false) => '▀',
                    (false, true) => '▄',
                    _ => continue,
                };
                c.set(dx + x as i64, dy + r as i64, ch, Some(o.color), false);
            }
        }
    }
    if !date.is_empty() {
        let dw = date.chars().count() as i64;
        c.text(x0 + (total_w - dw) / 2, dy + h + 1, &date, Some(o.color), false);
    }
    (total_w, total_h)
}

fn main() {
    let mut cli = Cli::new("rtty-clock");
    let t = Lang::pick(cli.lang, &EN, &PT, &ES);
    let mut o = Opts {
        seconds: false,
        twelve: false,
        utc: false,
        date: true,
        format: None,
        color: COLORS[0].1,
        boxed: false,
        blink: false,
        rebound: false,
        screensaver: false,
        mini: false,
    };
    while let Some(a) = cli.next() {
        match a {
            Arg::Pos(p) => cli.unknown(&p),
            Arg::Opt(opt) => match opt.as_str() {
                "-h" | "--help" => cli.help(t.help),
                "-V" | "--version" => cli.version(),
                "-s" | "--seconds" => o.seconds = true,
                "-t" | "--twelve" => o.twelve = true,
                "-u" | "--utc" => o.utc = true,
                "-D" | "--no-date" => o.date = false,
                "-f" | "--format" => o.format = Some(cli.value(&opt)),
                "-C" | "--color" => {
                    let v = cli.value(&opt).to_ascii_lowercase();
                    o.color = COLORS
                        .iter()
                        .find(|(n, _)| *n == v)
                        .map(|(_, c)| *c)
                        .unwrap_or_else(|| cli.fail(&format!("{} '{v}'", t.unknown_color)));
                }
                "-x" | "--box" => o.boxed = true,
                "-B" | "--blink" => o.blink = true,
                "-r" | "--rebound" => o.rebound = true,
                "-S" | "--screensaver" => o.screensaver = true,
                "-m" | "--mini" => o.mini = true,
                // tty-clock's -c (centre) is what rtty-clock always does.
                "-c" | "--center" => {}
                _ => cli.unknown(&opt),
            },
        }
    }
    if let Some(f) = &o.format {
        // A bad strftime pattern would panic while formatting: check it once now.
        use std::fmt::Write;
        let mut s = String::new();
        if write!(s, "{}", Local::now().format(f)).is_err() {
            cli.fail(&format!("{} '-f': {f}", common(cli.lang).invalid_value));
        }
    }
    if !std::io::stdout().is_terminal() {
        die("rtty-clock", common(cli.lang).not_a_terminal);
    }
    if let Err(e) = run(&mut o, t) {
        die("rtty-clock", &e.to_string());
    }
}

fn run(o: &mut Opts, t: &Texts) -> std::io::Result<()> {
    let depth = term::depth(Some(true));
    let _screen = Screen::enter()?;
    let mut out = std::io::stdout().lock();
    let (w, h) = term::size();
    let mut c = Canvas::new(w, h);
    let (mut x, mut y, mut vx, mut vy) = (0i64, 0i64, 1i64, 1i64);
    let mut size = (0i64, 0i64);
    let mut tick = 0u64;
    loop {
        let n = now(o, t);
        let colon_on = !o.blink || n.s % 2 == 0;
        if o.rebound {
            x += vx;
            y += vy;
            if x <= 0 || x + size.0 >= c.w as i64 {
                vx = -vx;
            }
            if y <= 0 || y + size.1 >= c.h as i64 {
                vy = -vy;
            }
            x = x.clamp(0, (c.w as i64 - size.0).max(0));
            y = y.clamp(0, (c.h as i64 - size.1).max(0));
        } else {
            x = (c.w as i64 - size.0).max(0) / 2;
            y = (c.h as i64 - size.1).max(0) / 2;
        }
        size = draw(&mut c, &n, o, (x, y), colon_on);
        if tick == 0 && !o.rebound {
            // First frame: now that the size is known, centre it.
            x = (c.w as i64 - size.0).max(0) / 2;
            y = (c.h as i64 - size.1).max(0) / 2;
            size = draw(&mut c, &n, o, (x, y), colon_on);
        }
        c.flush(&mut out, depth)?;
        tick += 1;
        let wait = if o.rebound { Duration::from_millis(120) } else { Duration::from_millis(200) };
        match tui::poll(wait)? {
            Input::Quit => return Ok(()),
            Input::Key(_) if o.screensaver => return Ok(()),
            Input::Resize => {
                let (w, h) = term::size();
                c.resize(w, h);
            }
            Input::Key(k) => match k {
                's' => o.seconds = !o.seconds,
                't' => o.twelve = !o.twelve,
                'd' => o.date = !o.date,
                'r' => o.rebound = !o.rebound,
                'x' => o.boxed = !o.boxed,
                'b' => o.blink = !o.blink,
                'm' => o.mini = !o.mini,
                '1'..='8' => o.color = COLORS[k as usize - '1' as usize].1,
                _ => {}
            },
            Input::None => {}
            // Arrows, Enter...: any key quits the screensaver.
            _ if o.screensaver => return Ok(()),
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn digits_have_fixed_width() {
        let o = Opts {
            seconds: true, twelve: false, utc: false, date: true, format: None, color: COLORS[0].1,
            boxed: true, blink: false, rebound: false, screensaver: false, mini: false,
        };
        let a = time_pixels(&Now { h: 11, m: 11, s: 11, pm: false, date: String::new() }, &o, true);
        let b = time_pixels(&Now { h: 20, m: 48, s: 59, pm: true, date: String::new() }, &o, true);
        assert_eq!(a[0].len(), b[0].len());
        let mut c = Canvas::new(100, 30);
        let (w, h) = draw(&mut c, &Now { h: 9, m: 5, s: 0, pm: false, date: "x".into() }, &o, (0, 0), true);
        assert!(w > 20 && h > 7);
    }
}

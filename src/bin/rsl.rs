//! rsl: a steam train crosses the terminal (like sl). q, Esc or Ctrl+C to stop it.

use fun::cli::{die, Arg, Cli};
use fun::i18n::{common, Lang};
use fun::term::{self, Depth, Rgb};
use fun::tui::{self, Canvas, Input, Screen};
use std::io::IsTerminal;
use std::time::{Duration, Instant};

struct Texts {
    help: &'static str,
}

const EN: Texts = Texts {
    help: "\
Usage: rsl [options]

A steam train crosses the terminal. Press q, Esc or Ctrl+C to stop it.

  -l, --little         A little train
  -F, --fly            The train takes off
  -n, --cars <n>       Number of carriages, 0 to 9 (default 2)
  -s, --speed <n>      Speed, 1 (slow) to 10 (fast) (default 5)
      --lang <code>    Language: en, pt, es (also: RTOOLS_LANG)
  -h, --help           Show this help
  -V, --version        Show the version
",
};

const PT: Texts = Texts {
    help: "\
Utilização: rsl [opções]

Um comboio a vapor atravessa o terminal. Prima q, Esc ou Ctrl+C para o parar.

  -l, --little         Um comboio pequeno
  -F, --fly            O comboio levanta voo
  -n, --cars <n>       Número de carruagens, de 0 a 9 (por omissão 2)
  -s, --speed <n>      Velocidade, de 1 (lenta) a 10 (rápida) (por omissão 5)
      --lang <código>  Idioma: en, pt, es (também: RTOOLS_LANG)
  -h, --help           Mostrar esta ajuda
  -V, --version        Mostrar a versão
",
};

const ES: Texts = Texts {
    help: "\
Uso: rsl [opciones]

Un tren de vapor cruza el terminal. Pulse q, Esc o Ctrl+C para pararlo.

  -l, --little         Un tren pequeño
  -F, --fly            El tren despega
  -n, --cars <n>       Número de vagones, de 0 a 9 (por defecto 2)
  -s, --speed <n>      Velocidad, de 1 (lenta) a 10 (rápida) (por defecto 5)
      --lang <código>  Idioma: en, pt, es (también: RTOOLS_LANG)
  -h, --help           Mostrar esta ayuda
  -V, --version        Mostrar la versión
",
};

// `%` is a wheel spoke (it turns as the train moves).
const LOCO: &[&str] = &[
    "     __                       _______  ",
    "    |__|_____________________|  [ ]  | ",
    "    _||__________________ ___|_______| ",
    "   /  ___   ___   ___  | |  |  ___   | ",
    "  <  |___| |___| |___| | |  | |___|  | ",
    "   \\_________________________________|_",
    "     (%)   (%)   (%)        (%)   (%)  ",
];
/// Where the smoke comes out (column, row above the art).
const LOCO_CHIMNEY: i64 = 5;

const CAR: &[&str] = &[
    "                      ",
    "  __________________  ",
    " |  _   _   _   _   | ",
    " | |_| |_| |_| |_|  | ",
    " |                  | ",
    "=|__________________|=",
    "   (%)          (%)   ",
];

const LITTLE_LOCO: &[&str] = &[
    "   _   __________ ",
    "  | |_|   [ ]    |",
    " <  ___________ _|",
    "  \\_____________|_",
    "   (%) (%)  (%)   ",
];
const LITTLE_CHIMNEY: i64 = 3;

const LITTLE_CAR: &[&str] = &[
    "            ",
    " _________  ",
    "| [ ] [ ] | ",
    "|_________|=",
    "  (%) (%)   ",
];

const SMOKE: &[&str] = &["(   )", "(  )", "( )", "()", "o", "."];
const SPOKES: [char; 4] = ['-', '\\', '|', '/'];

const LOCO_COLOR: Rgb = Rgb(235, 90, 60);
const CAR_COLOR: Rgb = Rgb(90, 165, 235);
const WHEEL_COLOR: Rgb = Rgb(210, 210, 210);
const SMOKE_COLOR: Rgb = Rgb(170, 170, 170);

struct Puff {
    x: f64,
    y: f64,
    age: usize,
}

/// Rows of the whole train (locomotive + carriages) and the colour of each column.
fn build(little: bool, cars: usize) -> (Vec<String>, Vec<Rgb>, i64) {
    let (loco, car, chimney) = if little { (LITTLE_LOCO, LITTLE_CAR, LITTLE_CHIMNEY) } else { (LOCO, CAR, LOCO_CHIMNEY) };
    let mut rows: Vec<String> = loco.iter().map(|s| s.to_string()).collect();
    let mut colors = vec![LOCO_COLOR; loco[0].chars().count()];
    for _ in 0..cars {
        for (r, line) in rows.iter_mut().zip(car) {
            r.push_str(line);
        }
        colors.extend(std::iter::repeat_n(CAR_COLOR, car[0].chars().count()));
    }
    (rows, colors, chimney)
}

fn main() {
    let mut cli = Cli::new("rsl");
    let t = Lang::pick(cli.lang, &EN, &PT, &ES);
    let (mut little, mut fly, mut cars, mut speed) = (false, false, 2usize, 5u32);
    while let Some(a) = cli.next() {
        match a {
            Arg::Pos(p) => cli.unknown(&p),
            Arg::Opt(opt) => match opt.as_str() {
                "-h" | "--help" => cli.help(t.help),
                "-V" | "--version" => cli.version(),
                "-l" | "--little" => little = true,
                "-F" | "--fly" => fly = true,
                "-n" | "--cars" => cars = cli.number(&opt, 0..=9),
                "-s" | "--speed" => speed = cli.number(&opt, 1..=10),
                _ => cli.unknown(&opt),
            },
        }
    }
    if !std::io::stdout().is_terminal() {
        die("rsl", common(cli.lang).not_a_terminal);
    }
    let depth = term::depth(Some(true));
    if let Err(e) = run(little, fly, cars, speed, depth) {
        die("rsl", &e.to_string());
    }
}

fn run(little: bool, fly: bool, cars: usize, speed: u32, depth: Depth) -> std::io::Result<()> {
    let (rows, colors, chimney) = build(little, cars);
    let train_w = rows[0].chars().count() as i64;
    let train_h = rows.len() as i64;
    let _screen = Screen::enter()?;
    let mut out = std::io::stdout().lock();
    let (mut w, mut h) = term::size();
    let mut canvas = Canvas::new(w, h);
    let frame = Duration::from_millis(15 + 10 * (11 - speed as u64));
    let mut puffs: Vec<Puff> = Vec::new();
    let mut x = w as i64;
    let mut tick = 0usize;
    while x > -train_w - 12 {
        let start = Instant::now();
        let base_y = (h as i64 - train_h) / 2 + 2;
        let y = if fly { base_y - (w as i64 - x) / 4 } else { base_y };
        if fly && y + train_h < 0 {
            break;
        }

        canvas.clear();
        // Smoke first, so the train passes in front of it.
        if tick % 3 == 0 {
            puffs.push(Puff { x: (x + chimney) as f64, y: (y - 1) as f64, age: 0 });
        }
        for p in &mut puffs {
            p.age += 1;
            p.y -= 0.3;
            p.x += 0.35;
        }
        puffs.retain(|p| p.age / 4 < SMOKE.len());
        for p in &puffs {
            let s = SMOKE[p.age / 4];
            canvas.text(p.x as i64 - s.len() as i64 / 2, p.y as i64, s, Some(SMOKE_COLOR), true);
        }
        let spoke = SPOKES[tick % SPOKES.len()];
        for (r, line) in rows.iter().enumerate() {
            let wheels = r + 1 == rows.len();
            for (i, ch) in line.chars().enumerate() {
                if ch == ' ' {
                    continue;
                }
                let (ch, color) = match ch {
                    '%' => (spoke, WHEEL_COLOR),
                    _ if wheels => (ch, WHEEL_COLOR),
                    _ => (ch, colors[i]),
                };
                canvas.set(x + i as i64, y + r as i64, ch, Some(color), false);
            }
        }
        canvas.flush(&mut out, depth)?;

        match tui::poll(frame.saturating_sub(start.elapsed()))? {
            Input::Quit => return Ok(()),
            Input::Resize => {
                (w, h) = term::size();
                canvas.resize(w, h);
            }
            _ => {}
        }
        let left = frame.saturating_sub(start.elapsed());
        if !left.is_zero() {
            std::thread::sleep(left);
        }
        x -= 1;
        tick += 1;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rows_line_up() {
        for little in [false, true] {
            let (rows, colors, _) = build(little, 3);
            let w = rows[0].chars().count();
            assert!(rows.iter().all(|r| r.chars().count() == w), "little={little}");
            assert_eq!(colors.len(), w);
        }
    }
}

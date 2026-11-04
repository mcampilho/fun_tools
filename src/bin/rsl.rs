//! rsl: a steam train crosses the terminal (like sl). q, Esc or Ctrl+C to stop it.

use fun::cli::{die, Arg, Cli};
use fun::i18n::{common, Lang};
use fun::pix::{self, mix, Pixels};
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
  -u, --unicode        Unicode look: colour pixel art made of half blocks and
                       symbols (needs a font with them; best in 24-bit colour)
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
  -u, --unicode        Aspeto Unicode: pixel art a cores feita de meios blocos e
                       símbolos (precisa de uma fonte que os tenha; melhor a 24 bits)
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
  -u, --unicode        Aspecto Unicode: pixel art en color hecho de medios bloques y
                       símbolos (necesita una fuente que los tenga; mejor a 24 bits)
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

// The Unicode look: pixel art (see `pix::palette`), facing left like the ASCII
// train. The wheels, rods and smoke are drawn on top by the program.
const LOCO_PX: &[&str] = &[
    "....KKKKK...................KKKKKKKKKKKKKKK..",
    "....FKKKF...................QQQQQQQQQQQQQQQ..",
    ".....KKK..........GGG........AAAAAAAAAAAAA...",
    ".....KKK.........GGGGG.......AHHHHAAAHHHHA...",
    "..EEEEEEEEEEEEEEEEEEEEEEEEEEEAHHHHAAAHHHHA...",
    ".FFFFFFFFGFFFFFFFFFGFFFFFFFFFAHHHHAAAHHHHA...",
    "YFFFFFFFFGFFFFFFFFFGFFFFFFFFFAAAAAAAAAAAAA...",
    "YFFFFFFFFGFFFFFFFFFGFFFFFFFFFAAAAAAAAAAAAA...",
    ".FFFFFFFFGFFFFFFFFFGFFFFFFFFFABBBBBBBBBBBA...",
    "..KKKKKKKKKKKKKKKKKKKKKKKKKKKAAAAAAAAAAAAA...",
    ".QQQQQQQQQQQQQQQQQQQQQQQQQQQQQQQQQQQQQQQQQQ..",
    ".KKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKK..",
    "EEF.FFFFF....................................",
    "EEF.FFFFF....................................",
    ".EF.FFFFF....................................",
    "..E..........................................",
];
const TENDER_PX: &[&str] = &[
    "..KFKFKKFKFK....",
    ".KKKKKKKKKKKKK..",
    "AAAAAAAAAAAAAAA.",
    "ABBBBBBBBBBBBBA.",
    "ABGGGGGGGGGGGBA.",
    "ABBBBBBBBBBBBBA.",
    "AAAAAAAAAAAAAAA.",
    "QQQQQQQQQQQQQQQ.",
    "KKKKKKKKKKKKKKKK",
];
const CAR_PX: &[&str] = &[
    "...KKKKKKKKKKKKKKKKKKKKKKKKKK...",
    "..KKKKKKKKKKKKKKKKKKKKKKKKKKKK..",
    ".AAAAAAAAAAAAAAAAAAAAAAAAAAAAAA.",
    ".ACCCCCCCCCCCCCCCCCCCCCCCCCCCCA.",
    ".AAHHHAAHHHAAHHHAABBAAHHHAAHHHA.",
    ".AAHKHAAHHHAAHHHAABBAAHHKAAHKHA.",
    ".AAKKKAAHHHAAKHHAABBAAHKKAAKKKA.",
    ".AAAAAAAAAAAAAAAAABBAAAAAAAAAAA.",
    ".GGGGGGGGGGGGGGGGGGGGGGGGGGGGGG.",
    ".BBBBBBBBBBBBBBBBBBBBBBBBBBBBBB.",
    "KKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKK",
];
const LITTLE_LOCO_PX: &[&str] = &[
    "..KKK.............KKKKKKK.",
    "...F...GG.........QQQQQQQ.",
    ".EEEEEEEEEEEEEEEEEAAAAAAA.",
    "YFFFFGFFFFFFGFFFFFAHHAHHA.",
    "YFFFFGFFFFFFGFFFFFAHHAHHA.",
    ".KKKKKKKKKKKKKKKKKAAAAAAA.",
    "QQQQQQQQQQQQQQQQQQQQQQQQQ.",
    "EF........................",
    "E.........................",
];
const LITTLE_CAR_PX: &[&str] = &[
    "..KKKKKKKKKKKK..",
    ".AAAAAAAAAAAAAA.",
    ".AHHAHKAHHAHKAA.",
    ".AHHAKKAHHAKKAA.",
    ".GGGGGGGGGGGGGG.",
    "KKKKKKKKKKKKKKKK",
];

/// One vehicle of the pixel train: its picture, how far below the top of the
/// train it starts, its wheels (centre x, centre y, radius) and whether the
/// biggest ones are joined by a coupling rod.
struct Vehicle {
    body: &'static [&'static str],
    top: i64,
    wheels: &'static [(f64, f64, f64)],
    rod: bool,
}

/// Pixel rows from the top of the train to the rail.
const RAIL_PX: i64 = 18;
const LITTLE_RAIL_PX: i64 = 12;

const LOCO_V: Vehicle = Vehicle {
    body: LOCO_PX,
    top: 0,
    wheels: &[(6.0, 16.0, 2.0), (14.5, 14.5, 3.5), (22.5, 14.5, 3.5), (30.5, 14.5, 3.5), (39.0, 16.0, 2.0)],
    rod: true,
};
const TENDER_V: Vehicle = Vehicle { body: TENDER_PX, top: 3, wheels: &[(4.0, 16.0, 2.0), (11.0, 16.0, 2.0)], rod: false };
const CAR_V: Vehicle = Vehicle {
    body: CAR_PX,
    top: 1,
    wheels: &[(6.0, 16.0, 2.0), (10.5, 16.0, 2.0), (21.5, 16.0, 2.0), (26.0, 16.0, 2.0)],
    rod: false,
};
const LITTLE_LOCO_V: Vehicle = Vehicle {
    body: LITTLE_LOCO_PX,
    top: 0,
    wheels: &[(3.5, 10.5, 1.5), (9.5, 9.5, 2.5), (15.5, 9.5, 2.5), (22.0, 10.5, 1.5)],
    rod: true,
};
const LITTLE_CAR_V: Vehicle = Vehicle { body: LITTLE_CAR_PX, top: 1, wheels: &[(4.0, 10.5, 1.5), (12.0, 10.5, 1.5)], rod: false };

const CAR_COLORS: &[Rgb] = &[Rgb(45, 110, 185), Rgb(40, 125, 75), Rgb(150, 50, 60)];

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

/// The pixel train: each vehicle with its colour and its x from the train's front.
fn build_px(little: bool, cars: usize) -> Vec<(&'static Vehicle, Rgb, i64)> {
    let mut v = Vec::new();
    let mut x = 0;
    let mut add = |veh: &'static Vehicle, col: Rgb| {
        v.push((veh, col, x));
        x += veh.body[0].chars().count() as i64;
    };
    if little {
        add(&LITTLE_LOCO_V, LOCO_COLOR);
    } else {
        add(&LOCO_V, LOCO_COLOR);
        add(&TENDER_V, LOCO_COLOR);
    }
    for i in 0..cars {
        add(if little { &LITTLE_CAR_V } else { &CAR_V }, CAR_COLORS[i % CAR_COLORS.len()]);
    }
    v
}

/// The scenery behind the train: sky, clouds, two rows of hills, grass and the track.
fn scenery(p: &mut Pixels, rail: i64, tick: usize) {
    let (w, h) = (p.w as i64, p.h as i64);
    let horizon = rail - 3;
    for y in 0..h {
        for x in 0..w {
            let col = if y < horizon {
                mix(Rgb(70, 135, 210), Rgb(195, 222, 245), y as f64 / horizon as f64)
            } else {
                mix(Rgb(95, 165, 70), Rgb(50, 110, 45), (y - horizon) as f64 / (h - horizon).max(1) as f64)
            };
            p.put(x, y, col);
        }
    }
    // Clouds drift slowly.
    for i in 0..(w / 30 + 2) {
        let span = w + 40;
        let cx = ((i * 71 + 13) as f64 - tick as f64 * 0.05).rem_euclid(span as f64) - 20.0;
        let cy = 3.0 + ((i * 37) % (horizon / 3).max(1)) as f64;
        for (dx, dy, r) in [(0.0, 0.0, 2.6), (3.0, -1.0, 3.2), (6.5, 0.0, 2.6), (3.0, 1.0, 2.6)] {
            p.disc(cx + dx, cy + dy, r, Rgb(250, 250, 252), 0.9);
        }
    }
    for x in 0..w {
        let xf = x as f64;
        let far = horizon - (6.0 + (xf * 0.031).sin() * 3.0 + (xf * 0.083 + 1.0).sin() * 1.5).round() as i64;
        let near = horizon - (2.5 + (xf * 0.055 + 2.0).sin() * 2.0 + (xf * 0.13).sin()).round() as i64;
        for y in far..horizon {
            p.put(x, y, Rgb(125, 165, 150));
        }
        for y in near..horizon {
            p.put(x, y, Rgb(85, 145, 80));
        }
        // Track: rail, then sleepers on ballast.
        p.put(x, rail, Rgb(190, 190, 200));
        p.put(x, rail + 1, if x % 4 == 0 { Rgb(95, 60, 35) } else { Rgb(130, 120, 110) });
        p.put(x, rail + 2, Rgb(110, 100, 92));
    }
}

/// A wheel turned by `angle`: tyre, spokes and hub.
fn wheel(p: &mut Pixels, cx: f64, cy: f64, r: f64, angle: f64) {
    // A little larger than r: small discs of pixels look like crosses otherwise.
    p.disc(cx, cy, r + 0.35, Rgb(30, 30, 36), 1.0);
    p.disc(cx, cy, r - 0.6, Rgb(200, 40, 40), 1.0);
    let n = if r >= 3.0 { 4 } else { 2 };
    for k in 0..n {
        let a = angle + k as f64 * std::f64::consts::TAU / n as f64;
        p.line(cx, cy, cx + a.cos() * (r - 0.6), cy + a.sin() * (r - 0.6), Rgb(110, 20, 25));
    }
    p.put(cx.floor() as i64, cy.floor() as i64, Rgb(215, 170, 60));
}

fn main() {
    let mut cli = Cli::new("rsl");
    let t = Lang::pick(cli.lang, &EN, &PT, &ES);
    let (mut little, mut fly, mut cars, mut speed, mut unicode) = (false, false, 2usize, 5u32, false);
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
                "-u" | "--unicode" => unicode = true,
                _ => cli.unknown(&opt),
            },
        }
    }
    if !std::io::stdout().is_terminal() {
        die("rsl", common(cli.lang).not_a_terminal);
    }
    let depth = term::depth(Some(true));
    let result = if unicode { run_px(little, fly, cars, speed, depth) } else { run(little, fly, cars, speed, depth) };
    if let Err(e) = result {
        die("rsl", &e.to_string());
    }
}

struct PxPuff {
    x: f64,
    y: f64,
    age: f64,
}

/// The scene with the front of the train at `x` (it moves left as `tick` grows).
struct PxScene {
    train: Vec<(&'static Vehicle, Rgb, i64)>,
    rail_h: i64,
    little: bool,
    fly: bool,
    puffs: Vec<PxPuff>,
}

impl PxScene {
    fn new(little: bool, fly: bool, cars: usize) -> PxScene {
        PxScene { train: build_px(little, cars), rail_h: if little { LITTLE_RAIL_PX } else { RAIL_PX }, little, fly, puffs: Vec::new() }
    }

    fn width(&self) -> i64 {
        self.train.last().map_or(0, |(v, _, x)| x + v.body[0].chars().count() as i64)
    }

    /// The rail's pixel row and the top of the train for a buffer `h` pixels high.
    fn rows(&self, h: i64, w: i64, x: i64) -> (i64, i64) {
        let rail = (h * 2 / 3 + 4).clamp(self.rail_h + 6, (h - 3).max(self.rail_h + 6));
        let top = rail - self.rail_h;
        (rail, if self.fly { top - (w - x) / 3 } else { top })
    }

    /// Moves the smoke and draws one frame.
    fn frame(&mut self, p: &mut Pixels, x: i64, tick: usize) {
        let (w, h) = (p.w as i64, p.h as i64);
        let (rail, y) = self.rows(h, w, x);
        scenery(p, rail, tick);
        // Smoke from the chimney: puffs that grow, drift back and fade into the sky.
        let chimney = if self.little { 3 } else { 6 };
        if tick % 3 == 0 {
            self.puffs.push(PxPuff { x: (x + chimney) as f64 + 0.5, y: (y - 1) as f64, age: 0.0 });
        }
        for pf in &mut self.puffs {
            pf.age += 1.0;
            pf.y -= 0.55 - pf.age * 0.008 + (pf.age * 0.5).sin() * 0.15;
            pf.x += 0.6;
        }
        self.puffs.retain(|pf| pf.age < 40.0);
        for pf in &self.puffs {
            let f = pf.age / 40.0;
            p.disc(pf.x, pf.y, 1.2 + pf.age * 0.16, mix(Rgb(245, 245, 245), Rgb(150, 150, 155), f), 0.95 * (1.0 - f));
        }
        // The wheels roll: one turn per 2πr pixels.
        let travelled = (w - x) as f64;
        for &(veh, col, dx) in &self.train {
            let (vx, vy) = (x + dx, y + veh.top);
            p.sprite(vx, vy, veh.body, |k| pix::palette(col, k));
            let mut pins = Vec::new();
            for &(cx, cy, r) in veh.wheels {
                let a = -travelled / r;
                let (wx, wy) = ((x + dx) as f64 + cx, y as f64 + cy);
                wheel(p, wx, wy, r, a);
                if r > 2.0 {
                    pins.push((wx + a.cos() * r * 0.55, wy + a.sin() * r * 0.55));
                }
            }
            if veh.rod && pins.len() >= 2 {
                let (first, last) = (pins[0], pins[pins.len() - 1]);
                let steel = Rgb(205, 205, 215);
                p.line(first.0, first.1, last.0, last.1, steel);
                // Piston rod into the cylinder at the front.
                let cyl = (vx as f64 + if self.little { 2.0 } else { 6.0 }, vy as f64 + if self.little { 7.5 } else { 13.0 });
                p.line(cyl.0, cyl.1, first.0, first.1, steel);
            }
        }
    }
}

fn run_px(little: bool, fly: bool, cars: usize, speed: u32, depth: Depth) -> std::io::Result<()> {
    let mut scene = PxScene::new(little, fly, cars);
    let train_w = scene.width();
    let _screen = Screen::enter()?;
    let mut out = std::io::stdout().lock();
    let (mut w, mut h) = term::size();
    let mut canvas = Canvas::new(w, h);
    let mut p = Pixels::new(w, h);
    let frame = Duration::from_millis(15 + 10 * (11 - speed as u64));
    let mut x = w as i64;
    let mut tick = 0usize;
    while x > -train_w - 30 {
        let start = Instant::now();
        if fly && scene.rows(p.h as i64, w as i64, x).1 + scene.rail_h + 30 < 0 {
            break;
        }
        scene.frame(&mut p, x, tick);
        p.show(&mut canvas);
        canvas.flush(&mut out, depth)?;
        match tui::poll(frame.saturating_sub(start.elapsed()))? {
            Input::Quit => return Ok(()),
            Input::Resize => {
                (w, h) = term::size();
                canvas.resize(w, h);
                p = Pixels::new(w, h);
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
        for s in [LOCO_PX, TENDER_PX, CAR_PX, LITTLE_LOCO_PX, LITTLE_CAR_PX] {
            let w = s[0].chars().count();
            assert!(s.iter().all(|r| r.chars().count() == w), "{}", s[0]);
        }
        let mut scene = PxScene::new(false, true, 3);
        let mut p = Pixels::new(80, 24);
        for i in 0..300 {
            scene.frame(&mut p, 80 - i, i as usize);
        }
    }

    /// Prints a frame with colours:
    /// `SNAP_X=20 cargo test --bin rsl snapshot -- --ignored --nocapture`.
    #[test]
    #[ignore]
    fn snapshot() {
        let env = |k: &str, d: i64| std::env::var(k).ok().and_then(|v| v.parse().ok()).unwrap_or(d);
        let (w, h) = (env("SNAP_W", 120) as usize, env("SNAP_H", 30) as usize);
        let mut scene = PxScene::new(env("SNAP_LITTLE", 0) == 1, env("SNAP_FLY", 0) == 1, env("SNAP_CARS", 2) as usize);
        let mut p = Pixels::new(w, h);
        let mut c = Canvas::new(w, h);
        let x_end = env("SNAP_X", 10);
        let mut x = w as i64;
        let mut tick = 0;
        while x >= x_end {
            scene.frame(&mut p, x, tick);
            x -= 1;
            tick += 1;
        }
        p.show(&mut c);
        print!("{}", c.to_ansi(Depth::True));
    }
}

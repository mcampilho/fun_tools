//! Colours: whether to use them, how many the terminal shows, and the escape
//! codes for a colour at that depth.

use std::io::IsTerminal;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Depth {
    None,
    Ansi16,
    Ansi256,
    True,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Rgb(pub u8, pub u8, pub u8);

/// Colour depth for stdout. `forced`: `Some(true)` for `--color`, `Some(false)`
/// for `--no-color`; otherwise colours only go to a terminal and not with NO_COLOR.
/// RTOOLS_COLORS=16|256|truecolor overrides the detected depth.
pub fn depth(forced: Option<bool>) -> Depth {
    let wanted = forced.unwrap_or_else(|| {
        std::env::var_os("NO_COLOR").is_none_or(|v| v.is_empty()) && std::io::stdout().is_terminal()
    });
    if !wanted || !enable_ansi() {
        return Depth::None;
    }
    match std::env::var("RTOOLS_COLORS").unwrap_or_default().to_ascii_lowercase().as_str() {
        "16" => return Depth::Ansi16,
        "256" => return Depth::Ansi256,
        "truecolor" | "24bit" => return Depth::True,
        _ => {}
    }
    let env = |k: &str| std::env::var(k).unwrap_or_default().to_ascii_lowercase();
    if matches!(env("COLORTERM").as_str(), "truecolor" | "24bit") || cfg!(windows) {
        // The Windows 10/11 console and Windows Terminal show 24-bit colour.
        return Depth::True;
    }
    match env("TERM").as_str() {
        t if t.contains("256") || t.contains("kitty") || t.contains("alacritty") => Depth::Ansi256,
        "dumb" => Depth::None,
        _ => Depth::Ansi16,
    }
}

/// Turns on escape-code processing for stdout (Windows console); true if it works.
#[cfg(windows)]
pub fn enable_ansi() -> bool {
    use windows_sys::Win32::System::Console::{
        GetConsoleMode, GetStdHandle, SetConsoleMode, ENABLE_VIRTUAL_TERMINAL_PROCESSING,
        STD_OUTPUT_HANDLE,
    };
    // SAFETY: plain Win32 console calls on our own stdout handle.
    unsafe {
        let h = GetStdHandle(STD_OUTPUT_HANDLE);
        let mut mode = 0;
        if GetConsoleMode(h, &mut mode) == 0 {
            // Not a console: a pipe, a file, or a terminal such as mintty.
            return true;
        }
        mode & ENABLE_VIRTUAL_TERMINAL_PROCESSING != 0
            || SetConsoleMode(h, mode | ENABLE_VIRTUAL_TERMINAL_PROCESSING) != 0
    }
}

#[cfg(not(windows))]
pub fn enable_ansi() -> bool {
    true
}

pub const RESET: &str = "\x1b[0m";

/// Escape code for a foreground (`bg == false`) or background colour.
pub fn color(depth: Depth, c: Rgb, bg: bool) -> String {
    match depth {
        Depth::None => String::new(),
        Depth::True => format!("\x1b[{};2;{};{};{}m", if bg { 48 } else { 38 }, c.0, c.1, c.2),
        Depth::Ansi256 => format!("\x1b[{};5;{}m", if bg { 48 } else { 38 }, to256(c)),
        Depth::Ansi16 => {
            let n = to16(c);
            let base = match (bg, n >= 8) {
                (false, false) => 30,
                (false, true) => 90 - 8,
                (true, false) => 40,
                (true, true) => 100 - 8,
            };
            format!("\x1b[{}m", base + n)
        }
    }
}

pub fn fg(depth: Depth, c: Rgb) -> String {
    color(depth, c, false)
}

/// Nearest xterm-256 colour (6×6×6 cube or grey ramp).
pub fn to256(Rgb(r, g, b): Rgb) -> u8 {
    let level = |v: u8| -> u8 {
        if v < 48 { 0 } else if v < 115 { 1 } else { ((v as u16 - 35) / 40) as u8 }
    };
    let (lr, lg, lb) = (level(r), level(g), level(b));
    let cube_val = |l: u8| if l == 0 { 0 } else { 55 + 40 * l as i32 };
    let cube = 16 + 36 * lr + 6 * lg + lb;
    let dist = |a: (i32, i32, i32)| {
        (a.0 - r as i32).pow(2) + (a.1 - g as i32).pow(2) + (a.2 - b as i32).pow(2)
    };
    let cube_d = dist((cube_val(lr), cube_val(lg), cube_val(lb)));
    let avg = (r as i32 + g as i32 + b as i32) / 3;
    let gi = ((avg - 8).max(0) / 10).min(23);
    let gv = 8 + 10 * gi;
    if dist((gv, gv, gv)) < cube_d { 232 + gi as u8 } else { cube }
}

/// Nearest of the 16 basic colours (Windows console's default palette).
pub fn to16(Rgb(r, g, b): Rgb) -> u8 {
    const BASIC: [(i32, i32, i32); 16] = [
        (12, 12, 12), (197, 15, 31), (19, 161, 14), (193, 156, 0),
        (0, 55, 218), (136, 23, 152), (58, 150, 221), (204, 204, 204),
        (118, 118, 118), (231, 72, 86), (22, 198, 12), (249, 241, 165),
        (59, 120, 255), (180, 0, 158), (97, 214, 214), (242, 242, 242),
    ];
    (0..16u8)
        .min_by_key(|&i| {
            let (br, bg, bb) = BASIC[i as usize];
            (r as i32 - br).pow(2) * 3 + (g as i32 - bg).pow(2) * 4 + (b as i32 - bb).pow(2) * 2
        })
        .unwrap_or(7)
}

/// Rainbow colour at position `x` (lolcat's formula: three phase-shifted sines).
pub fn rainbow(freq: f64, x: f64) -> Rgb {
    use std::f64::consts::PI;
    let c = |phase: f64| ((freq * x + phase).sin() * 127.0 + 128.0) as u8;
    Rgb(c(0.0), c(2.0 * PI / 3.0), c(4.0 * PI / 3.0))
}

/// Terminal size (columns, rows), with 80×24 when unknown.
pub fn size() -> (usize, usize) {
    crossterm::terminal::size()
        .ok()
        .filter(|&(w, h)| w > 0 && h > 0)
        .map_or((80, 24), |(w, h)| (w as usize, h as usize))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn conversions() {
        assert_eq!(to256(Rgb(255, 0, 0)), 196);
        assert_eq!(to256(Rgb(0, 0, 0)), 16);
        assert_eq!(to256(Rgb(128, 128, 128)), 244);
        assert_eq!(to16(Rgb(250, 250, 250)), 15);
        assert_eq!(color(Depth::Ansi16, Rgb(197, 15, 31), false), "\x1b[31m");
        assert_eq!(color(Depth::True, Rgb(1, 2, 3), true), "\x1b[48;2;1;2;3m");
        assert_eq!(color(Depth::None, Rgb(1, 2, 3), true), "");
    }
}

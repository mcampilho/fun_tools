//! Pixel art for the full-screen tools' Unicode look: a buffer of square pixels,
//! shown two per terminal cell with half blocks (`▀ ▄`) and coloured backgrounds.

use crate::term::{self, Depth, Rgb};
use crate::tui::{Canvas, Cell};

pub struct Pixels {
    pub w: usize,
    /// Pixel rows: twice the terminal rows.
    pub h: usize,
    px: Vec<Option<Rgb>>,
}

impl Pixels {
    /// A buffer for a terminal of `cols` × `rows` cells.
    pub fn new(cols: usize, rows: usize) -> Pixels {
        Pixels { w: cols, h: rows * 2, px: vec![None; cols * rows * 2] }
    }

    pub fn clear(&mut self) {
        self.px.fill(None);
    }

    pub fn get(&self, x: i64, y: i64) -> Option<Rgb> {
        self.index(x, y).and_then(|i| self.px[i])
    }

    fn index(&self, x: i64, y: i64) -> Option<usize> {
        (x >= 0 && y >= 0 && (x as usize) < self.w && (y as usize) < self.h).then(|| y as usize * self.w + x as usize)
    }

    pub fn put(&mut self, x: i64, y: i64, c: Rgb) {
        if let Some(i) = self.index(x, y) {
            self.px[i] = Some(c);
        }
    }

    /// Paints `c` over what is there with opacity `a` (0 to 1).
    pub fn blend(&mut self, x: i64, y: i64, c: Rgb, a: f64) {
        if let Some(i) = self.index(x, y) {
            self.px[i] = match self.px[i] {
                Some(under) => Some(mix(under, c, a)),
                None if a >= 0.5 => Some(c),
                None => None,
            };
        }
    }

    /// A filled circle, with soft edges when `a` < 1.
    pub fn disc(&mut self, cx: f64, cy: f64, r: f64, c: Rgb, a: f64) {
        let (x0, x1) = ((cx - r).floor() as i64, (cx + r).ceil() as i64);
        let (y0, y1) = ((cy - r).floor() as i64, (cy + r).ceil() as i64);
        for y in y0..=y1 {
            for x in x0..=x1 {
                let d = ((x as f64 + 0.5 - cx).powi(2) + (y as f64 + 0.5 - cy).powi(2)).sqrt();
                if d <= r {
                    if a >= 1.0 {
                        self.put(x, y, c);
                    } else {
                        self.blend(x, y, c, a * (1.0 - (d / r).powi(3)).clamp(0.0, 1.0));
                    }
                }
            }
        }
    }

    pub fn line(&mut self, x0: f64, y0: f64, x1: f64, y1: f64, c: Rgb) {
        let n = (x1 - x0).abs().max((y1 - y0).abs()).ceil().max(1.0) as usize;
        for i in 0..=n {
            let t = i as f64 / n as f64;
            self.put((x0 + (x1 - x0) * t).floor() as i64, (y0 + (y1 - y0) * t).floor() as i64, c);
        }
    }

    /// Draws a sprite: one string per pixel row, one character per pixel, coloured
    /// by `pal` (`None` = transparent).
    pub fn sprite<S: AsRef<str>>(&mut self, x: i64, y: i64, rows: &[S], pal: impl Fn(char) -> Option<Rgb>) {
        for (j, row) in rows.iter().enumerate() {
            for (i, ch) in row.as_ref().chars().enumerate() {
                if let Some(c) = pal(ch) {
                    self.put(x + i as i64, y + j as i64, c);
                }
            }
        }
    }

    /// Copies the pixels into the canvas. A cell whose two pixels share a colour
    /// is a space on a coloured background: block glyphs leave thin dark lines in
    /// terminals whose font is shorter than the line.
    pub fn show(&self, c: &mut Canvas) {
        for row in 0..self.h / 2 {
            for x in 0..self.w {
                let (top, bottom) = (self.px[2 * row * self.w + x], self.px[(2 * row + 1) * self.w + x]);
                let cell = match (top, bottom) {
                    (None, None) => continue,
                    (Some(t), None) => Cell { ch: '▀', fg: Some(t), bg: None, bold: false },
                    (None, Some(b)) => Cell { ch: '▄', fg: Some(b), bg: None, bold: false },
                    (Some(t), Some(b)) if t == b => Cell { ch: ' ', fg: None, bg: Some(t), bold: false },
                    (Some(t), Some(b)) => Cell { ch: '▀', fg: Some(t), bg: Some(b), bold: false },
                };
                c.put(x as i64, row as i64, cell);
            }
        }
    }

    /// The pixels as lines of text with colour codes, for printing (not full
    /// screen); without colours, the shapes are kept with block characters.
    /// Empty space at the line ends is left out.
    pub fn to_lines(&self, depth: Depth) -> Vec<String> {
        let mut lines = Vec::new();
        for row in 0..self.h.div_ceil(2) as i64 {
            let px = |x: i64| (self.get(x, 2 * row), self.get(x, 2 * row + 1));
            let end = (0..self.w as i64).rev().find(|&x| px(x) != (None, None)).map_or(0, |x| x + 1);
            let mut s = String::new();
            for x in 0..end {
                let cell = match px(x) {
                    (None, None) => " ".to_string(),
                    _ if depth == Depth::None => match px(x) {
                        (Some(_), None) => "▀",
                        (None, Some(_)) => "▄",
                        _ => "█",
                    }
                    .to_string(),
                    (Some(t), None) => format!("{}▀{}", term::fg(depth, t), term::RESET),
                    (None, Some(b)) => format!("{}▄{}", term::fg(depth, b), term::RESET),
                    (Some(t), Some(b)) if t == b => format!("{} {}", term::color(depth, t, true), term::RESET),
                    (Some(t), Some(b)) => format!("{}{}▀{}", term::fg(depth, t), term::color(depth, b, true), term::RESET),
                };
                s.push_str(&cell);
            }
            lines.push(s);
        }
        lines
    }

    /// A character on top of the pixels of cell (`x`, `row`), on their colour.
    pub fn glyph(&self, c: &mut Canvas, x: i64, row: i64, ch: char, fg: Rgb) {
        let bg = match (self.get(x, 2 * row), self.get(x, 2 * row + 1)) {
            (Some(t), Some(b)) => Some(mix(t, b, 0.5)),
            (t, b) => t.or(b),
        };
        c.put(x, row, Cell { ch, fg: Some(fg), bg, bold: false });
    }
}

/// From `a` (t = 0) to `b` (t = 1).
pub fn mix(a: Rgb, b: Rgb, t: f64) -> Rgb {
    let t = t.clamp(0.0, 1.0);
    let m = |x: u8, y: u8| (x as f64 + (y as f64 - x as f64) * t).round() as u8;
    Rgb(m(a.0, b.0), m(a.1, b.1), m(a.2, b.2))
}

/// Darker (f < 1) or brighter (f > 1).
pub fn scale(c: Rgb, f: f64) -> Rgb {
    let s = |x: u8| (x as f64 * f).round().clamp(0.0, 255.0) as u8;
    Rgb(s(c.0), s(c.1), s(c.2))
}

/// The colours of the sprites' letters. `A`–`D` come from the sprite's own
/// colour (base, dark, light, darker); the others are fixed. `.` and space are
/// transparent.
pub fn palette(base: Rgb, ch: char) -> Option<Rgb> {
    Some(match ch {
        'A' => base,
        'B' => scale(base, 0.6),
        'C' => mix(base, Rgb(255, 255, 255), 0.45),
        'D' => scale(base, 0.35),
        'W' => Rgb(245, 245, 245),
        'K' => Rgb(22, 22, 30),
        'F' => Rgb(70, 70, 82),
        'E' => Rgb(120, 120, 132),
        'L' => Rgb(185, 185, 195),
        'R' => Rgb(205, 45, 45),
        'Q' => Rgb(130, 25, 30),
        'Y' => Rgb(250, 215, 80),
        'H' => Rgb(255, 235, 160),
        'O' => Rgb(245, 140, 40),
        'G' => Rgb(215, 170, 60),
        'N' => Rgb(140, 90, 50),
        'M' => Rgb(90, 55, 30),
        'U' => Rgb(150, 210, 255),
        'V' => Rgb(60, 160, 80),
        _ => return None,
    })
}

/// The sprite mirrored left to right (rows padded to the same width first).
pub fn mirror(rows: &[&str]) -> Vec<String> {
    let w = rows.iter().map(|r| r.chars().count()).max().unwrap_or(0);
    rows.iter()
        .map(|r| {
            let mut v: Vec<char> = r.chars().collect();
            v.resize(w, '.');
            v.iter().rev().collect()
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cells_from_pixels() {
        let mut p = Pixels::new(3, 1);
        let (r, g) = (Rgb(255, 0, 0), Rgb(0, 255, 0));
        p.put(0, 0, r);
        p.put(1, 1, g);
        p.put(2, 0, r);
        p.put(2, 1, r);
        let mut c = Canvas::new(3, 1);
        p.show(&mut c);
        let ch = |x| c.get(x, 0).unwrap();
        assert_eq!((ch(0).ch, ch(0).fg), ('▀', Some(r)));
        assert_eq!((ch(1).ch, ch(1).fg), ('▄', Some(g)));
        assert_eq!((ch(2).ch, ch(2).bg), (' ', Some(r)));
        assert_eq!(mirror(&["AB", "C"]), ["BA", ".C"]);
        assert_eq!(mix(Rgb(0, 0, 0), Rgb(200, 100, 50), 0.5), Rgb(100, 50, 25));
    }
}

//! Full-screen animations: alternate screen, raw mode and hidden cursor, always
//! restored (on return, on error and on panic).

use crate::term::{self, Depth, Rgb};
use crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers};
use crossterm::{cursor, execute, terminal};
use std::io::{self, Write};
use std::time::Duration;

pub struct Screen;

impl Screen {
    pub fn enter() -> io::Result<Screen> {
        let prev = std::panic::take_hook();
        std::panic::set_hook(Box::new(move |info| {
            restore();
            prev(info);
        }));
        terminal::enable_raw_mode()?;
        let screen = Screen; // from here on, Drop restores the terminal
        execute!(io::stdout(), terminal::EnterAlternateScreen, cursor::Hide, terminal::Clear(terminal::ClearType::All))?;
        Ok(screen)
    }
}

impl Drop for Screen {
    fn drop(&mut self) {
        restore();
    }
}

fn restore() {
    let mut out = io::stdout();
    let _ = execute!(out, crossterm::style::ResetColor, cursor::Show, terminal::LeaveAlternateScreen);
    let _ = terminal::disable_raw_mode();
    let _ = out.flush();
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Cell {
    pub ch: char,
    pub fg: Option<Rgb>,
    pub bg: Option<Rgb>,
    pub bold: bool,
}

const BLANK: Cell = Cell { ch: ' ', fg: None, bg: None, bold: false };

/// A screen-sized grid of characters. `flush` only writes the cells that changed
/// since the last flush, so animations do not flicker.
pub struct Canvas {
    pub w: usize,
    pub h: usize,
    cells: Vec<Cell>,
    prev: Vec<Cell>,
    full: bool,
}

impl Canvas {
    pub fn new(w: usize, h: usize) -> Canvas {
        Canvas { w, h, cells: vec![BLANK; w * h], prev: vec![BLANK; w * h], full: true }
    }

    /// New size: everything is drawn again at the next flush.
    pub fn resize(&mut self, w: usize, h: usize) {
        *self = Canvas::new(w, h);
    }

    pub fn clear(&mut self) {
        self.cells.fill(BLANK);
    }

    pub fn get(&self, x: usize, y: usize) -> Option<Cell> {
        (x < self.w && y < self.h).then(|| self.cells[y * self.w + x])
    }

    pub fn set(&mut self, x: i64, y: i64, ch: char, fg: Option<Rgb>, bold: bool) {
        if x >= 0 && y >= 0 && (x as usize) < self.w && (y as usize) < self.h {
            self.cells[y as usize * self.w + x as usize] = Cell { ch, fg, bg: None, bold };
        }
    }

    /// A space on a coloured background: fills the whole cell (block glyphs may not).
    pub fn paint(&mut self, x: i64, y: i64, bg: Rgb) {
        if x >= 0 && y >= 0 && (x as usize) < self.w && (y as usize) < self.h {
            self.cells[y as usize * self.w + x as usize] = Cell { ch: ' ', fg: None, bg: Some(bg), bold: false };
        }
    }

    /// Writes a string from (x, y); spaces are transparent when `transparent`.
    pub fn text(&mut self, x: i64, y: i64, s: &str, fg: Option<Rgb>, transparent: bool) {
        for (i, ch) in s.chars().enumerate() {
            if !(transparent && ch == ' ') {
                self.set(x + i as i64, y, ch, fg, false);
            }
        }
    }

    pub fn flush(&mut self, out: &mut impl Write, depth: Depth) -> io::Result<()> {
        let mut buf = String::with_capacity(4096);
        let mut cursor: Option<(usize, usize)> = None;
        let mut style: Option<(Option<Rgb>, Option<Rgb>, bool)> = None;
        for y in 0..self.h {
            for x in 0..self.w {
                let i = y * self.w + x;
                let c = self.cells[i];
                // The bottom-right cell is never written: some terminals scroll after it.
                if (!self.full && c == self.prev[i]) || (x + 1 == self.w && y + 1 == self.h) {
                    continue;
                }
                if cursor != Some((x, y)) {
                    buf.push_str(&format!("\x1b[{};{}H", y + 1, x + 1));
                }
                let on = depth != Depth::None;
                let st = (c.fg.filter(|_| on), c.bg.filter(|_| on), c.bold);
                if style != Some(st) {
                    buf.push_str(term::RESET);
                    if c.bold {
                        buf.push_str("\x1b[1m");
                    }
                    if let Some(rgb) = st.0 {
                        buf.push_str(&term::fg(depth, rgb));
                    }
                    if let Some(rgb) = st.1 {
                        buf.push_str(&term::color(depth, rgb, true));
                    }
                    style = Some(st);
                }
                // Without colours a painted cell must still show: use a block.
                buf.push(if c.bg.is_some() && !on { '#' } else { c.ch });
                cursor = Some((x + 1, y));
            }
        }
        self.prev.copy_from_slice(&self.cells);
        self.full = false;
        out.write_all(buf.as_bytes())?;
        out.flush()
    }
}

pub enum Input {
    None,
    /// q, Esc or Ctrl+C
    Quit,
    Resize,
    Key(char),
}

/// Waits up to `timeout` for a key or a resize.
pub fn poll(timeout: Duration) -> io::Result<Input> {
    if !event::poll(timeout)? {
        return Ok(Input::None);
    }
    Ok(match event::read()? {
        Event::Key(k) if k.kind != KeyEventKind::Release => match k.code {
            KeyCode::Esc | KeyCode::Char('q') | KeyCode::Char('Q') => Input::Quit,
            KeyCode::Char('c') if k.modifiers.contains(KeyModifiers::CONTROL) => Input::Quit,
            KeyCode::Char(c) => Input::Key(c),
            _ => Input::None,
        },
        Event::Resize(..) => Input::Resize,
        _ => Input::None,
    })
}

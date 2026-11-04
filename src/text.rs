//! Text measured in terminal columns: wide characters (CJK, emoji) take two,
//! combining marks none, and escape codes (colours) none.

use unicode_width::UnicodeWidthChar;

/// Columns a string takes on screen, ignoring ANSI escape sequences.
pub fn width(s: &str) -> usize {
    let mut w = 0;
    let mut chars = s.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '\x1b' {
            skip_escape(&mut chars);
        } else {
            w += c.width().unwrap_or(0);
        }
    }
    w
}

/// Skips the rest of an escape sequence (after ESC): CSI `[...letter` or OSC `]...BEL/ST`.
pub fn skip_escape(chars: &mut std::iter::Peekable<std::str::Chars>) {
    match chars.peek() {
        Some('[') => {
            chars.next();
            for c in chars.by_ref() {
                if ('@'..='~').contains(&c) {
                    break;
                }
            }
        }
        Some(']') => {
            chars.next();
            while let Some(c) = chars.next() {
                if c == '\x07' || (c == '\x1b' && chars.peek() == Some(&'\\')) {
                    if c == '\x1b' {
                        chars.next();
                    }
                    break;
                }
            }
        }
        Some(_) => {
            chars.next();
        }
        None => {}
    }
}

/// Removes ANSI escape sequences.
pub fn strip_ansi(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut chars = s.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '\x1b' {
            skip_escape(&mut chars);
        } else {
            out.push(c);
        }
    }
    out
}

/// Replaces tabs with spaces up to the next multiple of `tab` columns.
pub fn expand_tabs(s: &str, tab: usize) -> String {
    if !s.contains('\t') {
        return s.to_string();
    }
    let mut out = String::new();
    let mut col = 0;
    for c in s.chars() {
        if c == '\t' {
            let n = tab - col % tab;
            out.extend(std::iter::repeat_n(' ', n));
            col += n;
        } else {
            out.push(c);
            col += c.width().unwrap_or(0);
        }
    }
    out
}

/// Word-wraps one paragraph to `max` columns; words longer than a line are split.
pub fn wrap(s: &str, max: usize) -> Vec<String> {
    let max = max.max(1);
    let mut lines = Vec::new();
    let mut line = String::new();
    let mut line_w = 0;
    for word in s.split_whitespace() {
        let mut word = word.to_string();
        loop {
            let ww = width(&word);
            let need = if line.is_empty() { ww } else { line_w + 1 + ww };
            if need <= max {
                if !line.is_empty() {
                    line.push(' ');
                    line_w += 1;
                }
                line.push_str(&word);
                line_w += ww;
                break;
            }
            if !line.is_empty() {
                lines.push(std::mem::take(&mut line));
                line_w = 0;
                continue;
            }
            // A word wider than the line: cut it.
            let (head, tail) = split_at_width(&word, max);
            lines.push(head);
            word = tail;
        }
    }
    if !line.is_empty() || lines.is_empty() {
        lines.push(line);
    }
    lines
}

/// Wraps every line of `s` (empty lines are kept).
pub fn wrap_text(s: &str, max: usize) -> Vec<String> {
    s.lines().flat_map(|l| wrap(l, max)).collect()
}

fn split_at_width(s: &str, max: usize) -> (String, String) {
    let mut w = 0;
    for (i, c) in s.char_indices() {
        let cw = c.width().unwrap_or(0);
        if w + cw > max && i > 0 {
            return (s[..i].to_string(), s[i..].to_string());
        }
        w += cw;
    }
    (s.to_string(), String::new())
}

/// `s` padded with spaces to `w` columns (left, centre or right aligned).
pub fn pad(s: &str, w: usize, align: Align) -> String {
    let extra = w.saturating_sub(width(s));
    let (l, r) = match align {
        Align::Left => (0, extra),
        Align::Center => (extra / 2, extra - extra / 2),
        Align::Right => (extra, 0),
    };
    format!("{}{s}{}", " ".repeat(l), " ".repeat(r))
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Align {
    Left,
    Center,
    Right,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn widths() {
        assert_eq!(width("abc"), 3);
        assert_eq!(width("日本"), 4);
        assert_eq!(width("😀"), 2);
        assert_eq!(width("\x1b[38;5;1mhi\x1b[0m"), 2);
        assert_eq!(width("e\u{301}"), 1);
        assert_eq!(strip_ansi("\x1b[1mA\x1b]0;t\x07B"), "AB");
    }

    #[test]
    fn wraps() {
        assert_eq!(wrap("the quick brown fox", 9), ["the quick", "brown fox"]);
        assert_eq!(wrap("abcdefghij", 4), ["abcd", "efgh", "ij"]);
        assert_eq!(wrap("", 4), [""]);
        assert_eq!(wrap("日本語の", 5), ["日本", "語の"]);
        assert_eq!(expand_tabs("a\tb", 4), "a   b");
        assert_eq!(pad("ab", 5, Align::Center), " ab  ");
    }
}

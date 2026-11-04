//! Reading text from files or stdin: lossy UTF-8, without the byte-order mark
//! that Windows PowerShell puts in front of piped text, and with `\r\n` as `\n`.

use std::fs::File;
use std::io::{self, BufRead, BufReader, IsTerminal, Read};

/// Opens a file, or stdin for "-".
pub fn open(path: &str) -> io::Result<Box<dyn BufRead>> {
    if path == "-" {
        Ok(Box::new(BufReader::new(io::stdin())))
    } else {
        Ok(Box::new(BufReader::new(File::open(path)?)))
    }
}

/// All of stdin, or None when stdin is a terminal (nothing was piped in).
pub fn piped_stdin() -> Option<String> {
    if io::stdin().is_terminal() {
        return None;
    }
    let mut bytes = Vec::new();
    io::stdin().read_to_end(&mut bytes).ok()?;
    Some(clean(&String::from_utf8_lossy(&bytes)))
}

pub fn read_all(path: &str) -> io::Result<String> {
    let mut bytes = Vec::new();
    open(path)?.read_to_end(&mut bytes)?;
    Ok(clean(&String::from_utf8_lossy(&bytes)))
}

/// Removes a leading BOM and turns `\r\n` into `\n`.
pub fn clean(s: &str) -> String {
    s.strip_prefix('\u{feff}').unwrap_or(s).replace("\r\n", "\n")
}

/// Reads one line (without its line ending) as lossy UTF-8; None at the end.
/// `first` strips a BOM.
pub fn read_line(r: &mut dyn BufRead, buf: &mut Vec<u8>, first: bool) -> io::Result<Option<String>> {
    buf.clear();
    if r.read_until(b'\n', buf)? == 0 {
        return Ok(None);
    }
    while matches!(buf.last(), Some(b'\n' | b'\r')) {
        buf.pop();
    }
    let s = String::from_utf8_lossy(buf);
    let s = if first { s.strip_prefix('\u{feff}').unwrap_or(&s).to_string() } else { s.into_owned() };
    Ok(Some(s))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cleans() {
        assert_eq!(clean("\u{feff}a\r\nb"), "a\nb");
        let mut r: &[u8] = b"\xef\xbb\xbfone\r\ntwo";
        let mut buf = Vec::new();
        assert_eq!(read_line(&mut r, &mut buf, true).unwrap().as_deref(), Some("one"));
        assert_eq!(read_line(&mut r, &mut buf, false).unwrap().as_deref(), Some("two"));
        assert_eq!(read_line(&mut r, &mut buf, false).unwrap(), None);
    }
}

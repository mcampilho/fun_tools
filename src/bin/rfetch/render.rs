//! Side-by-side output (logo | info), written to stdout in one go.

use crate::logo::{Colors, Logo};
use std::io::{IsTerminal, Write};

pub struct Row {
    pub label: &'static str,
    pub value: String,
}

/// Decides whether to use colours, and turns on ANSI support on Windows.
pub fn use_color(forced: Option<bool>) -> bool {
    let wanted = forced.unwrap_or_else(|| {
        std::env::var_os("NO_COLOR").map_or(true, |v| v.is_empty()) && std::io::stdout().is_terminal()
    });
    wanted && enable_ansi()
}

#[cfg(windows)]
fn enable_ansi() -> bool {
    use windows_sys::Win32::System::Console::{
        GetConsoleMode, GetStdHandle, SetConsoleMode, ENABLE_VIRTUAL_TERMINAL_PROCESSING,
        STD_OUTPUT_HANDLE,
    };
    // SAFETY: plain Win32 console calls on our own stdout handle.
    unsafe {
        let h = GetStdHandle(STD_OUTPUT_HANDLE);
        let mut mode = 0;
        if GetConsoleMode(h, &mut mode) == 0 {
            // Not a console (pipe, file, or a terminal that already speaks ANSI, e.g. mintty).
            return true;
        }
        mode & ENABLE_VIRTUAL_TERMINAL_PROCESSING != 0
            || SetConsoleMode(h, mode | ENABLE_VIRTUAL_TERMINAL_PROCESSING) != 0
    }
}

#[cfg(not(windows))]
fn enable_ansi() -> bool {
    true
}

/// How the text is written to stdout.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Encoding {
    Utf8,
    /// When stdout is not a console: in the system's OEM code page (850, 437, ...),
    /// because the terminal reads Windows programs that way (MobaXterm), even
    /// when its hidden console is set to UTF-8.
    #[cfg_attr(not(windows), allow(dead_code))]
    OemCodePage,
}

pub fn print(logo: &Logo, title: &str, rows: &[Row], colors: Colors, encoding: Encoding) {
    let color = colors.on();
    let paint = |code: &str, s: &str| {
        if color { format!("\x1b[{code}m{s}\x1b[0m") } else { s.to_string() }
    };

    let accent = format!("1;{}", colors.sgr(logo.accent, false));
    let label_width = rows.iter().map(|r| r.label.chars().count()).max().unwrap_or(0);
    let mut info: Vec<String> = Vec::with_capacity(rows.len() + 4);
    info.push(paint(&accent, title));
    info.push("-".repeat(title.chars().count()));
    for r in rows {
        let pad = " ".repeat(label_width - r.label.chars().count());
        info.push(format!("{}{pad}  {}", paint(&accent, r.label), r.value));
    }
    if color {
        info.push(String::new());
        for range in [0..8, 8..16] {
            let swatch = range.map(|c| format!("\x1b[{}m   ", colors.sgr(c, true)));
            info.push(swatch.collect::<String>() + "\x1b[0m");
        }
    }

    // Logo lines are already coloured and exactly `logo.width` columns wide.
    let blank = " ".repeat(logo.width);
    let mut out = String::with_capacity(8192);
    out.push('\n');
    for i in 0..logo.lines.len().max(info.len()) {
        out.push_str(logo.lines.get(i).map(String::as_str).unwrap_or(&blank));
        out.push_str("   ");
        out.push_str(info.get(i).map(String::as_str).unwrap_or(""));
        out.push('\n');
    }
    let bytes = match encoding {
        Encoding::OemCodePage => to_oem_code_page(&out),
        Encoding::Utf8 => None,
    };
    // Ignore errors such as a closed pipe (`rfetch | head`) instead of panicking.
    let mut stdout = std::io::stdout().lock();
    let bytes = bytes.as_deref().unwrap_or(out.as_bytes());
    let _ = stdout.write_all(bytes).and_then(|_| stdout.flush());
}

/// The text in the system's OEM code page, or None when that is not needed:
/// stdout is a console (Rust writes UTF-16 to it), or the OEM code page is UTF-8.
/// Characters the code page lacks become `?`; the half blocks and `°` exist in
/// the usual OEM code pages (437, 850, ...).
#[cfg(windows)]
fn to_oem_code_page(s: &str) -> Option<Vec<u8>> {
    use windows_sys::Win32::Globalization::{GetOEMCP, CP_UTF8};
    use windows_sys::Win32::System::Console::{GetConsoleMode, GetStdHandle, STD_OUTPUT_HANDLE};
    // SAFETY: plain Win32 calls on our own stdout handle.
    let cp = unsafe {
        let mut mode = 0;
        if GetConsoleMode(GetStdHandle(STD_OUTPUT_HANDLE), &mut mode) != 0 {
            return None;
        }
        GetOEMCP()
    };
    if cp == CP_UTF8 { None } else { encode(s, cp) }
}

#[cfg(windows)]
fn encode(s: &str, cp: u32) -> Option<Vec<u8>> {
    use std::ptr::{null, null_mut};
    use windows_sys::Win32::Globalization::WideCharToMultiByte;
    // SAFETY: the output buffer is sized from the first call.
    unsafe {
        let wide: Vec<u16> = s.encode_utf16().collect();
        let len = i32::try_from(wide.len()).ok()?;
        let n = WideCharToMultiByte(cp, 0, wide.as_ptr(), len, null_mut(), 0, null(), null_mut());
        if n <= 0 {
            return None;
        }
        let mut buf = vec![0u8; n as usize];
        let n = WideCharToMultiByte(cp, 0, wide.as_ptr(), len, buf.as_mut_ptr(), n, null(), null_mut());
        (n > 0).then(|| {
            buf.truncate(n as usize);
            buf
        })
    }
}

#[cfg(not(windows))]
fn to_oem_code_page(_: &str) -> Option<Vec<u8>> {
    None
}

#[cfg(all(test, windows))]
mod tests {
    #[test]
    fn blocks_in_oem_code_pages() {
        for cp in [437, 850] {
            assert_eq!(super::encode("█▀▄ °C", cp).unwrap(), [0xDB, 0xDF, 0xDC, b' ', 0xF8, b'C']);
        }
    }
}

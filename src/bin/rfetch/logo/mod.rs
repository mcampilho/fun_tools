//! Logos drawn as pixel art with half blocks (`▀ ▄`) and coloured cells: two
//! pixel rows per terminal line, so pixels are square. A logo can be combined
//! with a badge: Windows + shell/environment (PowerShell, Git, MSYS2, Cygwin)
//! or Linux distribution + Windows (WSL).

mod art;

use crate::system::{Flavor, Msystem};
use art::{Art, ARTS};

pub struct Logo {
    pub lines: Vec<String>,
    /// Visible width in terminal columns.
    pub width: usize,
    /// xterm-256 colour for the labels.
    pub accent: u8,
}

/// How colours are written.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Colors {
    None,
    /// The 16 basic colours, for terminals that cannot show more (e.g. ConsoleZ).
    Ansi16,
    Ansi256,
}

impl Colors {
    pub fn on(self) -> bool {
        self != Colors::None
    }

    /// SGR parameters for a foreground (`bg == false`) or background colour.
    pub fn sgr(self, c: u8, bg: bool) -> String {
        match self {
            Colors::Ansi16 => {
                let n = nearest16(c);
                let base = match (bg, n >= 8) {
                    (false, false) => 30,
                    (false, true) => 90 - 8,
                    (true, false) => 40,
                    (true, true) => 100 - 8,
                };
                (base + n).to_string()
            }
            _ => format!("{};5;{c}", if bg { 48 } else { 38 }),
        }
    }
}

/// Badges that can go on the Windows logo (`windows-<badge>`).
const WINDOWS_BADGES: &[&str] = &[
    "powershell",
    "pwsh",
    "git",
    "cygwin",
    "msys2",
    "msys2-ucrt64",
    "msys2-mingw64",
    "msys2-mingw32",
    "msys2-clang64",
    "msys2-clangarm64",
];

/// Linux logo for each os-release ID (also matched against ID_LIKE).
const DISTROS: &[(&str, &str)] = &[
    ("ubuntu", "ubuntu"),
    ("kubuntu", "ubuntu"),
    ("xubuntu", "ubuntu"),
    ("debian", "debian"),
    ("raspbian", "debian"),
    ("linuxmint", "mint"),
    ("mint", "mint"),
    ("pop", "pop"),
    ("fedora", "fedora"),
    ("almalinux", "almalinux"),
    ("rocky", "rocky"),
    ("rhel", "redhat"),
    ("centos", "centos"),
    ("arch", "arch"),
    ("archarm", "arch"),
    ("endeavouros", "arch"),
    ("manjaro", "manjaro"),
    ("alpine", "alpine"),
    ("opensuse", "opensuse"),
    ("suse", "opensuse"),
    ("sles", "opensuse"),
    ("gentoo", "gentoo"),
    ("void", "void"),
    ("nixos", "nixos"),
];

/// Picks the logo name for this system.
pub fn auto_name(os_id: &str, id_like: &str, flavor: Flavor) -> String {
    let windows = |badge: &str| format!("windows-{badge}");
    match flavor {
        Flavor::PowerShell => return windows("powershell"),
        Flavor::Pwsh => return windows("pwsh"),
        Flavor::GitBash | Flavor::GitCmd => return windows("git"),
        Flavor::Msys2(env) => {
            return windows(match env {
                Msystem::Msys => "msys2",
                Msystem::Ucrt64 => "msys2-ucrt64",
                Msystem::Mingw64 => "msys2-mingw64",
                Msystem::Mingw32 => "msys2-mingw32",
                Msystem::Clang64 | Msystem::Clang32 => "msys2-clang64",
                Msystem::ClangArm64 => "msys2-clangarm64",
            })
        }
        Flavor::Cygwin => return windows("cygwin"),
        Flavor::Native | Flavor::Wsl => {}
    }
    if os_id.eq_ignore_ascii_case("windows") {
        return "windows".into();
    }
    let id = os_id.to_ascii_lowercase();
    let like = id_like.to_ascii_lowercase();
    let lookup = |word: &str| {
        DISTROS
            .iter()
            .find(|(k, _)| word == *k || (word.starts_with(k) && word[k.len()..].starts_with('-')))
            .map(|(_, v)| *v)
    };
    let distro = lookup(&id)
        .or_else(|| like.split_whitespace().find_map(lookup))
        .unwrap_or("linux");
    if flavor == Flavor::Wsl {
        format!("{distro}-wsl")
    } else {
        distro.to_string()
    }
}

/// Names accepted by `--logo` (the `-wsl` variants are not listed one by one).
pub fn names() -> Vec<String> {
    let mut v = vec!["windows".to_string()];
    v.extend(WINDOWS_BADGES.iter().map(|b| format!("windows-{b}")));
    v.extend(
        ARTS.iter()
            .filter(|a| a.name != "windows" && !a.name.starts_with("badge-"))
            .map(|a| a.name.to_string()),
    );
    v
}

pub fn build(name: &str, colors: Colors) -> Option<Logo> {
    let find = |n: &str| ARTS.iter().find(|a| a.name == n);
    let name = name.to_ascii_lowercase();
    let color = colors.on();

    let (pixmap, accent) = if let Some(a) = find(&name).filter(|a| !a.name.starts_with("badge-")) {
        (Pixmap::from_art(a, color), a.accent)
    } else if let Some(b) = name.strip_prefix("windows-").filter(|b| WINDOWS_BADGES.contains(b)) {
        // The badge takes the place of the bottom-right pane.
        let badge = find(&format!("badge-{b}"))?;
        let base = Pixmap::from_art(find("windows")?, color);
        (base.overlay(&Pixmap::from_art(badge, color), 16, 16), badge.accent)
    } else if let Some(d) = name.strip_suffix("-wsl") {
        let distro = find(d).filter(|a| !a.name.starts_with("badge-") && a.name != "windows")?;
        let base = Pixmap::from_art(distro, color);
        let badge = Pixmap::from_art(find("badge-winsmall")?, color);
        // y stays even, so the badge starts on a terminal line.
        let (x, y) = (base.w - badge.w * 2 / 3, (base.h - badge.h * 2 / 3) & !1);
        (base.overlay(&badge, x, y), distro.accent)
    } else {
        return None;
    };
    Some(Logo { lines: pixmap.render(colors), width: pixmap.w, accent })
}

/// Index (0..16) of the basic colour closest to an xterm-256 colour.
fn nearest16(c: u8) -> u8 {
    // The Windows console's default palette (Campbell): this mode is for Windows terminals.
    const BASIC: [(u8, u8, u8); 16] = [
        (12, 12, 12), (197, 15, 31), (19, 161, 14), (193, 156, 0),
        (0, 55, 218), (136, 23, 152), (58, 150, 221), (204, 204, 204),
        (118, 118, 118), (231, 72, 86), (22, 198, 12), (249, 241, 165),
        (59, 120, 255), (180, 0, 158), (97, 214, 214), (242, 242, 242),
    ];
    if c < 16 {
        return c;
    }
    let (r, g, b) = rgb256(c);
    (0..16u8)
        .min_by_key(|&i| {
            let (br, bg, bb) = BASIC[i as usize];
            let d = |a: u8, b: u8| (a as i32 - b as i32).pow(2);
            d(r, br) * 3 + d(g, bg) * 4 + d(b, bb) * 2
        })
        .unwrap_or(7)
}

fn rgb256(c: u8) -> (u8, u8, u8) {
    if c >= 232 {
        let v = 8 + 10 * (c - 232);
        return (v, v, v);
    }
    let i = c - 16;
    let level = |v: u8| if v == 0 { 0 } else { 55 + 40 * v };
    (level(i / 36), level(i / 6 % 6), level(i % 6))
}

#[derive(Clone)]
struct Pixmap {
    w: usize,
    h: usize,
    px: Vec<Option<u8>>,
}

impl Pixmap {
    fn from_art(a: &Art, color: bool) -> Self {
        let h = a.rows.len();
        let w = a.rows.iter().map(|r| r.chars().count()).max().unwrap_or(0);
        let mut px = vec![None; w * h];
        for (y, row) in a.rows.iter().enumerate() {
            for (x, ch) in row.chars().enumerate() {
                // Without colours, light details become holes so they stay visible.
                if ch == '.' || (!color && a.holes.contains(ch)) {
                    continue;
                }
                px[y * w + x] = a.palette.iter().find(|(c, _)| *c == ch).map(|(_, v)| *v);
            }
        }
        Pixmap { w, h, px }
    }

    fn get(&self, x: usize, y: usize) -> Option<u8> {
        if x < self.w && y < self.h { self.px[y * self.w + x] } else { None }
    }

    /// Places `badge` at (ox, oy), growing the canvas if needed and clearing the
    /// badge's box plus a 1-pixel margin, so the two never touch.
    fn overlay(&self, badge: &Pixmap, ox: usize, oy: usize) -> Pixmap {
        let (w, h) = (self.w.max(ox + badge.w), self.h.max(oy + badge.h));
        let mut px = vec![None; w * h];
        for y in 0..h {
            for x in 0..w {
                let inside = x >= ox && y >= oy && x < ox + badge.w && y < oy + badge.h;
                let margin = x + 1 >= ox && x <= ox + badge.w && y + 1 >= oy && y <= oy + badge.h;
                px[y * w + x] = if inside {
                    badge.get(x - ox, y - oy)
                } else if margin {
                    None
                } else {
                    self.get(x, y)
                };
            }
        }
        Pixmap { w, h, px }
    }

    /// One terminal cell per column and pair of pixel rows. A cell whose two
    /// pixels share a colour is a space on a coloured background: a full-block
    /// glyph leaves thin dark lines in terminals whose font is shorter than the
    /// line (the classic console, ConEmu, ConsoleZ).
    fn render(&self, colors: Colors) -> Vec<String> {
        let mut lines = Vec::with_capacity(self.h.div_ceil(2));
        for y in (0..self.h).step_by(2) {
            let mut s = String::with_capacity(self.w * 12);
            // (fg, bg) currently set; None = default colours (after a reset).
            let mut state: (Option<u8>, Option<u8>) = (None, None);
            for x in 0..self.w {
                let (top, bottom) = (self.get(x, y), self.get(x, y + 1));
                if !colors.on() {
                    s.push(match (top, bottom) {
                        (None, None) => ' ',
                        (Some(_), None) => '▀',
                        (None, Some(_)) => '▄',
                        (Some(_), Some(_)) => '█',
                    });
                    continue;
                }
                let (ch, fg, bg) = match (top, bottom) {
                    (None, None) => (' ', None, None),
                    (Some(t), None) => ('▀', Some(t), None),
                    (None, Some(b)) => ('▄', Some(b), None),
                    (Some(t), Some(b)) if t == b => (' ', None, Some(t)),
                    (Some(t), Some(b)) => ('▀', Some(t), Some(b)),
                };
                // A space only shows its background: keep whatever foreground is set.
                let mut fg = if ch == ' ' { state.0 } else { fg };
                if (fg, bg) != state {
                    // Going back to a default colour needs a reset.
                    if (state.0.is_some() && fg.is_none()) || (state.1.is_some() && bg.is_none()) {
                        s.push_str("\x1b[0m");
                        state = (None, None);
                        if ch == ' ' {
                            fg = None;
                        }
                    }
                    let mut codes = Vec::new();
                    if let Some(f) = fg.filter(|_| fg != state.0) {
                        codes.push(colors.sgr(f, false));
                    }
                    if let Some(b) = bg.filter(|_| bg != state.1) {
                        codes.push(colors.sgr(b, true));
                    }
                    if !codes.is_empty() {
                        s.push_str(&format!("\x1b[{}m", codes.join(";")));
                    }
                    state = (fg, bg);
                }
                s.push(ch);
            }
            if state != (None, None) {
                s.push_str("\x1b[0m");
            }
            lines.push(s);
        }
        lines
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_name_builds() {
        for n in names() {
            for c in [Colors::None, Colors::Ansi16, Colors::Ansi256] {
                assert!(build(&n, c).is_some(), "{n}");
            }
        }
        assert!(build("ubuntu-wsl", Colors::Ansi256).is_some());
        assert!(build("badge-git", Colors::Ansi256).is_none());
        assert!(build("windows-wsl", Colors::Ansi256).is_none());
        assert!(build("nope", Colors::Ansi256).is_none());
    }

    #[test]
    fn picks_logos() {
        assert_eq!(auto_name("windows", "", Flavor::Native), "windows");
        assert_eq!(auto_name("windows", "", Flavor::GitCmd), "windows-git");
        assert_eq!(auto_name("windows", "", Flavor::Pwsh), "windows-pwsh");
        assert_eq!(auto_name("windows", "", Flavor::Msys2(Msystem::Ucrt64)), "windows-msys2-ucrt64");
        assert_eq!(auto_name("windows", "", Flavor::Msys2(Msystem::Msys)), "windows-msys2");
        assert_eq!(auto_name("linuxmint", "ubuntu debian", Flavor::Native), "mint");
        assert_eq!(auto_name("rocky", "rhel centos fedora", Flavor::Native), "rocky");
        assert_eq!(auto_name("almalinux", "rhel centos fedora", Flavor::Native), "almalinux");
        assert_eq!(auto_name("ol", "fedora", Flavor::Native), "fedora");
        assert_eq!(auto_name("opensuse-tumbleweed", "opensuse suse", Flavor::Native), "opensuse");
        assert_eq!(auto_name("ubuntu", "debian", Flavor::Wsl), "ubuntu-wsl");
        assert_eq!(auto_name("somethingelse", "", Flavor::Native), "linux");
    }

    #[test]
    fn plain_render_has_no_escapes() {
        let l = build("windows-git", Colors::None).unwrap();
        assert!(l.lines.iter().all(|s| !s.contains('\x1b')));
        assert!(l.lines.iter().all(|s| s.chars().count() == l.width));
    }

    #[test]
    fn full_cells_use_background() {
        // The Windows panes are whole cells: background spaces, no block glyphs.
        let l = build("windows", Colors::Ansi256).unwrap();
        assert!(l.lines.iter().all(|s| !s.contains('█') && !s.contains('▀') && !s.contains('▄')));
        assert!(l.lines[0].starts_with("\x1b[48;5;39m"));
        let visible = |s: &str| {
            let mut n = 0;
            let mut esc = false;
            for ch in s.chars() {
                match (esc, ch) {
                    (false, '\x1b') => esc = true,
                    (true, 'm') => esc = false,
                    (false, _) => n += 1,
                    _ => {}
                }
            }
            n
        };
        assert!(l.lines.iter().all(|s| visible(s) == l.width));
        assert!(l.lines.iter().all(|s| !s.contains('\x1b') || s.ends_with("\x1b[0m")));
        // Every line of every logo has exactly `width` visible columns.
        for n in names() {
            let l = build(&n, Colors::Ansi256).unwrap();
            assert!(l.lines.iter().all(|s| visible(s) == l.width), "{n}");
        }
    }

    #[test]
    fn basic_colours() {
        assert_eq!(nearest16(231), 15); // white
        // The two Windows blues stay apart.
        assert_eq!(nearest16(39), 6);
        assert_eq!(nearest16(33), 12);
        assert_eq!(Colors::Ansi16.sgr(33, true), "104");
        assert_eq!(Colors::Ansi16.sgr(1, false), "31");
        assert_eq!(Colors::Ansi256.sgr(39, true), "48;5;39");
    }
}

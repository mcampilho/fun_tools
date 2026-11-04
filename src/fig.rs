//! rfiglet / rtoilet: large letters made of characters (like figlet and toilet).
//! Built-in fonts come from the pixel font; FIGlet `.flf` and TOIlet `.tlf`
//! fonts installed on the system (or given as a file) work too, with figlet's
//! fitting and smushing rules.

use crate::cli::{Arg, Cli};
use crate::i18n::Lang;
use crate::pixfont;
use crate::term::{self, Depth, Rgb};
use crate::text::{width, Align};
use std::path::PathBuf;

// ------------------------------------------------------------------ fonts

/// Built-in fonts drawn from the pixel font.
pub const BUILTIN: &[(&str, &str)] = &[
    ("pixel", "██ blocks"),
    ("tiny", "▀▄ half blocks, small"),
    ("hash", "# characters"),
    ("shade", "▓▓ shaded blocks"),
    ("drop", "blocks with a ░ drop shadow"),
];

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Mode {
    FullWidth,
    Kerning,
    Smush,
}

pub struct FlfFont {
    height: usize,
    hardblank: char,
    mode: Mode,
    rules: u32,
    /// Written right to left (e.g. Hebrew fonts).
    rtl: bool,
    glyphs: std::collections::HashMap<char, Vec<Vec<char>>>,
}

pub enum Font {
    Builtin(&'static str),
    Flf(FlfFont),
}

/// Parses a FIGlet (`flf2a`) or TOIlet (`tlf2a`) font.
pub fn parse_flf(text: &str) -> Option<FlfFont> {
    let mut lines = text.lines();
    let header = lines.next()?;
    let sig = header.get(..5)?;
    if sig != "flf2a" && sig != "tlf2a" {
        return None;
    }
    let hardblank = header[5..].chars().next()?;
    let nums: Vec<i64> = header[5 + hardblank.len_utf8()..]
        .split_whitespace()
        .map_while(|n| n.parse().ok())
        .collect();
    let height = *nums.first()? as usize;
    let old = *nums.get(3).unwrap_or(&0);
    let comments = *nums.get(4).unwrap_or(&0) as usize;
    let rtl = nums.get(5) == Some(&1);
    let (mode, rules) = match nums.get(6) {
        Some(&full) => {
            let rules = (full & 63) as u32;
            if full & 128 != 0 {
                (Mode::Smush, rules)
            } else if full & 64 != 0 {
                (Mode::Kerning, rules)
            } else {
                (Mode::FullWidth, rules)
            }
        }
        None if old < 0 => (Mode::FullWidth, 0),
        None if old == 0 => (Mode::Kerning, 0),
        None => (Mode::Smush, (old & 63) as u32),
    };
    let mut lines = lines.skip(comments);
    let mut glyphs = std::collections::HashMap::new();
    let read_glyph = |lines: &mut dyn Iterator<Item = &str>| -> Option<Vec<Vec<char>>> {
        let mut g = Vec::with_capacity(height);
        for _ in 0..height {
            let l = lines.next()?.trim_end_matches('\r');
            // The end mark is the last character, doubled on the last row.
            let end = l.chars().last()?;
            let body = l.trim_end_matches(end);
            g.push(body.chars().collect());
        }
        Some(g)
    };
    let fixed: Vec<char> = (32u8..127).map(char::from).chain(['Ä', 'Ö', 'Ü', 'ä', 'ö', 'ü', 'ß']).collect();
    for c in fixed {
        match read_glyph(&mut lines) {
            Some(g) => {
                glyphs.insert(c, g);
            }
            None => break,
        }
    }
    // Code-tagged characters: "<code> [comment]", then the rows.
    while let Some(tag) = lines.next() {
        let code = tag.split_whitespace().next().unwrap_or("");
        let n = if let Some(h) = code.strip_prefix("0x").or_else(|| code.strip_prefix("0X")) {
            i64::from_str_radix(h, 16).ok()
        } else if code.len() > 1 && code.starts_with('0') {
            i64::from_str_radix(&code[1..], 8).ok()
        } else {
            code.parse().ok()
        };
        let Some(g) = read_glyph(&mut lines) else { break };
        if let Some(c) = n.and_then(|n| u32::try_from(n).ok()).and_then(char::from_u32) {
            glyphs.insert(c, g);
        }
    }
    if glyphs.is_empty() {
        return None;
    }
    Some(FlfFont { height, hardblank, mode, rules, rtl, glyphs })
}

/// Folders searched for FIGlet/TOIlet fonts.
pub fn font_dirs(extra: &[PathBuf]) -> Vec<PathBuf> {
    let mut dirs: Vec<PathBuf> = extra.to_vec();
    for var in ["FIGLET_FONTDIR", "TOILET_FONT_PATH"] {
        if let Some(v) = std::env::var_os(var) {
            dirs.extend(std::env::split_paths(&v));
        }
    }
    if let Ok(exe) = std::env::current_exe() {
        if let Some(d) = exe.parent() {
            dirs.push(d.join("fonts"));
        }
    }
    if let Some(d) = std::env::var_os("LOCALAPPDATA").or_else(|| std::env::var_os("XDG_DATA_HOME")) {
        dirs.push(PathBuf::from(d).join("rfiglet").join("fonts"));
    }
    if let Some(h) = std::env::var_os("HOME") {
        dirs.push(PathBuf::from(h).join(".local/share/rfiglet/fonts"));
    }
    for d in ["/usr/share/figlet", "/usr/share/figlet/fonts", "/usr/local/share/figlet", "/usr/share/toilet"] {
        dirs.push(PathBuf::from(d));
    }
    dirs.retain(|d| d.is_dir());
    dirs.dedup();
    dirs
}

/// Finds a font: a built-in name, a file path, or a name in the font folders.
pub fn load(name: &str, dirs: &[PathBuf]) -> Result<Font, String> {
    if let Some((n, _)) = BUILTIN.iter().find(|(n, _)| n.eq_ignore_ascii_case(name)) {
        return Ok(Font::Builtin(n));
    }
    let mut candidates = vec![PathBuf::from(name)];
    for d in dirs {
        for ext in ["", ".flf", ".tlf"] {
            candidates.push(d.join(format!("{name}{ext}")));
        }
    }
    for p in candidates {
        if p.is_file() {
            let bytes = std::fs::read(&p).map_err(|e| format!("{}: {e}", p.display()))?;
            // Old FIGlet fonts are Latin-1; TOIlet fonts are UTF-8.
            let text = match String::from_utf8(bytes) {
                Ok(s) => s,
                Err(e) => e.into_bytes().iter().map(|&b| b as char).collect(),
            };
            return parse_flf(&text).map(Font::Flf).ok_or_else(|| p.display().to_string());
        }
    }
    Err(name.to_string())
}

/// Font names found in the folders (without extension).
pub fn installed(dirs: &[PathBuf]) -> Vec<String> {
    let mut v = Vec::new();
    for d in dirs {
        if let Ok(rd) = std::fs::read_dir(d) {
            for e in rd.flatten() {
                let p = e.path();
                if matches!(p.extension().and_then(|x| x.to_str()), Some("flf" | "tlf")) {
                    if let Some(s) = p.file_stem().and_then(|s| s.to_str()) {
                        v.push(s.to_string());
                    }
                }
            }
        }
    }
    v.sort();
    v.dedup();
    v
}

// ------------------------------------------------------------------ layout

/// Rows of a rendered piece of text (all rows the same number of chars).
pub type Block = Vec<Vec<char>>;

fn builtin_block(style: &str, text: &str) -> Block {
    let bm = pixfont::bitmap(text, 1);
    let h = bm.len();
    let w = bm.first().map_or(0, Vec::len);
    let px = |x: i64, y: i64| x >= 0 && y >= 0 && (y as usize) < h && (x as usize) < w && bm[y as usize][x as usize];
    match style {
        "tiny" => (0..h.div_ceil(2))
            .map(|r| {
                (0..w)
                    .map(|x| match (px(x as i64, 2 * r as i64), px(x as i64, 2 * r as i64 + 1)) {
                        (true, true) => '█',
                        (true, false) => '▀',
                        (false, true) => '▄',
                        _ => ' ',
                    })
                    .collect()
            })
            .collect(),
        "hash" => bm.iter().map(|r| r.iter().map(|&p| if p { '#' } else { ' ' }).collect()).collect(),
        "drop" => (0..=h as i64)
            .map(|y| {
                (0..=w as i64)
                    .flat_map(|x| {
                        let c = if px(x, y) {
                            '█'
                        } else if px(x - 1, y - 1) {
                            '░'
                        } else {
                            ' '
                        };
                        [c, c]
                    })
                    .collect()
            })
            .collect(),
        other => {
            let c = if other == "shade" { '▓' } else { '█' };
            bm.iter().map(|r| r.iter().flat_map(|&p| if p { [c, c] } else { [' ', ' '] }).collect()).collect()
        }
    }
}

/// Smushes the characters `l` (on the left) and `r` (on the right); `None` if they do not smush.
/// Universal smushing keeps the later character: the right one, or the left one in a
/// right-to-left font.
fn smush_pair(l: char, r: char, rules: u32, hb: char, rtl: bool) -> Option<char> {
    if l == ' ' {
        return Some(r);
    }
    if r == ' ' {
        return Some(l);
    }
    if rules == 0 {
        // Universal smushing: the right character wins, hardblanks give way.
        let (later, earlier) = if rtl { (l, r) } else { (r, l) };
        return Some(if later == hb { earlier } else { later });
    }
    if l == hb || r == hb {
        return (rules & 32 != 0 && l == hb && r == hb).then_some(hb);
    }
    if rules & 1 != 0 && l == r {
        return Some(l);
    }
    if rules & 2 != 0 {
        const SET: &str = "|/\\[]{}()<>";
        if l == '_' && SET.contains(r) {
            return Some(r);
        }
        if r == '_' && SET.contains(l) {
            return Some(l);
        }
    }
    if rules & 4 != 0 {
        const CLASSES: &[&str] = &["|", "/\\", "[]", "{}", "()", "<>"];
        let class = |c: char| CLASSES.iter().position(|s| s.contains(c));
        if let (Some(a), Some(b)) = (class(l), class(r)) {
            if a != b {
                return Some(if a > b { l } else { r });
            }
        }
    }
    if rules & 8 != 0 && matches!((l, r), ('[', ']') | (']', '[') | ('{', '}') | ('}', '{') | ('(', ')') | (')', '(')) {
        return Some('|');
    }
    if rules & 16 != 0 {
        match (l, r) {
            ('/', '\\') => return Some('|'),
            ('\\', '/') => return Some('Y'),
            ('>', '<') => return Some('X'),
            _ => {}
        }
    }
    None
}

/// Lays out `text` in a FIGlet font. Right-to-left fonts are laid out mirrored (each new
/// character goes to the left) and turned back at the end.
fn flf_block(f: &FlfFont, text: &str, mode: Mode) -> Block {
    let rtl = f.rtl;
    // In the mirrored layout, the spatial order of a pair is reversed.
    let pair = |a: char, b: char, rules: u32| {
        if rtl {
            smush_pair(b, a, rules, f.hardblank, true)
        } else {
            smush_pair(a, b, rules, f.hardblank, false)
        }
    };
    let mut out: Block = vec![Vec::new(); f.height];
    for c in text.chars() {
        let Some(g) = f.glyphs.get(&c).or_else(|| f.glyphs.get(&'?')) else { continue };
        let mirrored: Vec<Vec<char>>;
        let g = if rtl {
            mirrored = g.iter().map(|r| r.iter().rev().copied().collect()).collect();
            &mirrored
        } else {
            g
        };
        let gw = g.iter().map(Vec::len).max().unwrap_or(0);
        // As in figlet, the first character also moves left over its own leading
        // blank columns (as if smushed against an empty line).
        let mut amount = if mode == Mode::FullWidth { 0 } else { gw };
        if amount > 0 {
            for (row, grow) in out.iter().zip(g) {
                let trail = row.iter().rev().take_while(|&&ch| ch == ' ').count();
                let lead = grow.iter().take_while(|&&ch| ch == ' ').count();
                let mut a = trail + lead;
                if mode == Mode::Smush && trail < row.len() && lead < grow.len() {
                    let (l, r) = (row[row.len() - 1 - trail], grow[lead]);
                    if pair(l, r, f.rules).is_some() {
                        a += 1;
                    }
                }
                amount = amount.min(a);
            }
        }
        for (row, grow) in out.iter_mut().zip(g) {
            let mut grow: Vec<char> = grow.clone();
            grow.resize(gw, ' ');
            let start = row.len() as isize - amount as isize;
            for (i, &gc) in grow.iter().enumerate() {
                let p = start + i as isize;
                if p < 0 {
                    // Before the start of the line: only blank columns end up here.
                    continue;
                }
                let p = p as usize;
                if p < row.len() {
                    let lc = row[p];
                    row[p] = pair(lc, gc, if mode == Mode::Smush { f.rules } else { 0 }).unwrap_or(gc);
                } else {
                    row.push(gc);
                }
            }
        }
    }
    for row in &mut out {
        if rtl {
            row.reverse();
        }
        for c in row.iter_mut() {
            if *c == f.hardblank {
                *c = ' ';
            }
        }
    }
    out
}

pub fn render(font: &Font, text: &str, mode: Option<Mode>) -> Block {
    match font {
        Font::Builtin(style) => builtin_block(style, text),
        Font::Flf(f) => flf_block(f, text, mode.unwrap_or(f.mode)),
    }
}

fn block_width(b: &Block) -> usize {
    b.iter().map(|r| r.iter().map(|c| width(c.encode_utf8(&mut [0; 4]))).sum::<usize>()).max().unwrap_or(0)
}

/// Lays out one input line, wrapping at word boundaries to `max` columns.
pub fn layout(font: &Font, line: &str, mode: Option<Mode>, max: usize) -> Vec<Block> {
    let mut out = Vec::new();
    let mut cur = String::new();
    for word in line.split(' ') {
        let candidate = if cur.is_empty() { word.to_string() } else { format!("{cur} {word}") };
        if block_width(&render(font, &candidate, mode)) <= max || cur.is_empty() {
            cur = candidate;
        } else {
            out.push(render(font, &cur, mode));
            cur = word.to_string();
        }
        // A single word wider than the line is cut by characters.
        while block_width(&render(font, &cur, mode)) > max && cur.chars().count() > 1 {
            let mut head = String::new();
            for c in cur.chars() {
                let mut h = head.clone();
                h.push(c);
                if block_width(&render(font, &h, mode)) > max && !head.is_empty() {
                    break;
                }
                head = h;
            }
            out.push(render(font, &head, mode));
            cur = cur[head.len()..].to_string();
        }
    }
    if !cur.is_empty() || out.is_empty() {
        out.push(render(font, &cur, mode));
    }
    out
}

// ------------------------------------------------------------------ output and filters

#[derive(Clone, Copy)]
pub struct Cell {
    pub ch: char,
    pub fg: Option<Rgb>,
}

pub type Grid = Vec<Vec<Cell>>;

pub const FILTERS: &[&str] = &["crop", "rainbow", "gay", "metal", "fire", "flip", "flop", "rotate", "border"];

fn mirror(c: char, horizontal: bool) -> char {
    const H: &[(char, char)] = &[
        ('/', '\\'), ('(', ')'), ('[', ']'), ('{', '}'), ('<', '>'), ('▌', '▐'), ('▛', '▜'), ('▙', '▟'),
        ('d', 'b'), ('p', 'q'), ('`', '\''), ('┌', '┐'), ('└', '┘'), ('╭', '╮'), ('╰', '╯'),
    ];
    const V: &[(char, char)] = &[
        ('/', '\\'), ('▀', '▄'), ('^', 'v'), ('_', '‾'), ('▛', '▙'), ('▜', '▟'), ('┌', '└'), ('┐', '┘'),
        ('╭', '╰'), ('╮', '╯'), ('\'', ','), ('M', 'W'),
    ];
    let map = if horizontal { H } else { V };
    map.iter()
        .find_map(|&(a, b)| if c == a { Some(b) } else if c == b { Some(a) } else { None })
        .unwrap_or(c)
}

pub fn apply(grid: &mut Grid, filter: &str) {
    let h = grid.len();
    let w = grid.iter().map(Vec::len).max().unwrap_or(0);
    for r in grid.iter_mut() {
        r.resize(w, Cell { ch: ' ', fg: None });
    }
    match filter {
        "crop" => {
            let blank = |c: &Cell| c.ch == ' ';
            grid.retain(|r| !r.iter().all(blank));
            let left = (0..w).find(|&x| grid.iter().any(|r| !blank(&r[x]))).unwrap_or(0);
            let right = (0..w).rev().find(|&x| grid.iter().any(|r| !blank(&r[x]))).map_or(0, |x| x + 1);
            for r in grid.iter_mut() {
                *r = r[left..right.max(left)].to_vec();
            }
        }
        "rainbow" | "gay" => {
            for (y, r) in grid.iter_mut().enumerate() {
                for (x, c) in r.iter_mut().enumerate() {
                    c.fg = Some(term::rainbow(0.12, x as f64 + y as f64 * 2.0));
                }
            }
        }
        "metal" => {
            const M: [Rgb; 4] = [Rgb(60, 80, 170), Rgb(120, 150, 210), Rgb(215, 225, 240), Rgb(120, 150, 210)];
            for (y, r) in grid.iter_mut().enumerate() {
                for (x, c) in r.iter_mut().enumerate() {
                    c.fg = Some(M[(x / 2 + y) % 4]);
                }
            }
        }
        "fire" => {
            for (y, r) in grid.iter_mut().enumerate() {
                let t = if h > 1 { y as f64 / (h - 1) as f64 } else { 0.0 };
                let col = Rgb(255, (230.0 - 200.0 * t) as u8, (120.0 * (1.0 - t)) as u8 / 2);
                for c in r.iter_mut() {
                    c.fg = Some(col);
                }
            }
        }
        "flip" => {
            for r in grid.iter_mut() {
                r.reverse();
                for c in r.iter_mut() {
                    c.ch = mirror(c.ch, true);
                }
            }
        }
        "flop" => {
            grid.reverse();
            for r in grid.iter_mut() {
                for c in r.iter_mut() {
                    c.ch = mirror(c.ch, false);
                }
            }
        }
        "rotate" => {
            apply(grid, "flip");
            apply(grid, "flop");
        }
        "border" => {
            let line = |l: char, m: char, r: char| {
                let mut v = vec![Cell { ch: l, fg: None }];
                v.extend(std::iter::repeat_n(Cell { ch: m, fg: None }, w));
                v.push(Cell { ch: r, fg: None });
                v
            };
            let mut out = vec![line('┌', '─', '┐')];
            for r in grid.drain(..) {
                let mut v = vec![Cell { ch: '│', fg: None }];
                v.extend(r);
                v.push(Cell { ch: '│', fg: None });
                out.push(v);
            }
            out.push(line('└', '─', '┘'));
            *grid = out;
        }
        _ => {}
    }
}

pub fn to_string(grid: &Grid, depth: Depth) -> String {
    let mut s = String::new();
    for r in grid {
        let end = r.iter().rposition(|c| c.ch != ' ').map_or(0, |p| p + 1);
        let mut cur: Option<Rgb> = None;
        for c in &r[..end] {
            if depth != Depth::None && c.ch != ' ' && c.fg != cur {
                if let Some(rgb) = c.fg {
                    s.push_str(&term::fg(depth, rgb));
                }
                cur = c.fg;
            }
            s.push(c.ch);
        }
        if cur.is_some() {
            s.push_str(term::RESET);
        }
        s.push('\n');
    }
    s
}

// ------------------------------------------------------------------ program

struct Texts {
    help_figlet: &'static str,
    help_toilet: &'static str,
    builtin: &'static str,
    installed: &'static str,
    none_installed: &'static str,
    filters: &'static str,
    unknown_font: &'static str,
    unknown_filter: &'static str,
}

const EN: Texts = Texts {
    help_figlet: "\
Usage: rfiglet [options] [text...]

Writes the text in large letters made of characters (from the arguments, or piped in).

  -f, --font <name>    Font: built-in, installed (see --list) or a .flf/.tlf file
                       (default: \"standard\" if installed, else \"hash\")
  -d, --dir <folder>   Another folder with fonts
  -w, --width <n>      Output width (default: the terminal's, or 80)
  -c, -l, -r           Centre, left or right
  -k, --kerning        Letters touch, no smushing
  -W, --full-width     Letters keep their full width
  -s, --smush          Smush letters together (the font's rules)
  -L, --list           List the fonts
      --lang <code>    Language: en, pt, es (also: RTOOLS_LANG)
  -h, --help           Show this help
  -V, --version        Show the version
",
    help_toilet: "\
Usage: rtoilet [options] [text...]

Large coloured letters (from the arguments, or piped in).

  -f, --font <name>    Font: built-in, installed (see --list) or a .flf/.tlf file
                       (default \"pixel\")
  -d, --dir <folder>   Another folder with fonts
  -F, --filter <list>  Filters, separated by \":\": crop, rainbow (gay), metal, fire,
                       flip, flop, rotate, border
      --gay, --rainbow Same as -F rainbow
      --metal          Same as -F metal
  -w, --width <n>      Output width (default: the terminal's, or 80)
  -c, -l, -r           Centre, left or right
  -k, --kerning        Letters touch, no smushing
  -W, --full-width     Letters keep their full width
  -s, --smush          Smush letters together (the font's rules)
  -L, --list           List the fonts and filters
      --force          Colours even when the output is not a terminal
      --lang <code>    Language: en, pt, es (also: RTOOLS_LANG)
  -h, --help           Show this help
  -V, --version        Show the version
",
    builtin: "Built-in fonts:",
    installed: "Installed fonts:",
    none_installed: "(none found; put .flf/.tlf files in one of the folders below or use -d)",
    filters: "Filters:",
    unknown_font: "font not found",
    unknown_filter: "unknown filter",
};

const PT: Texts = Texts {
    help_figlet: "\
Utilização: rfiglet [opções] [texto...]

Escreve o texto em letras grandes feitas de caracteres (dos argumentos, ou recebido por pipe).

  -f, --font <nome>    Fonte: embutida, instalada (ver --list) ou um ficheiro .flf/.tlf
                       (por omissão: \"standard\" se estiver instalada, senão \"hash\")
  -d, --dir <pasta>    Outra pasta com fontes
  -w, --width <n>      Largura da saída (por omissão: a do terminal, ou 80)
  -c, -l, -r           Centrar, à esquerda ou à direita
  -k, --kerning        Letras encostadas, sem fusão
  -W, --full-width     Letras com a largura toda
  -s, --smush          Fundir as letras (regras da fonte)
  -L, --list           Mostrar as fontes
      --lang <código>  Idioma: en, pt, es (também: RTOOLS_LANG)
  -h, --help           Mostrar esta ajuda
  -V, --version        Mostrar a versão
",
    help_toilet: "\
Utilização: rtoilet [opções] [texto...]

Letras grandes e coloridas (dos argumentos, ou recebidas por pipe).

  -f, --font <nome>    Fonte: embutida, instalada (ver --list) ou um ficheiro .flf/.tlf
                       (por omissão \"pixel\")
  -d, --dir <pasta>    Outra pasta com fontes
  -F, --filter <lista> Filtros, separados por \":\": crop, rainbow (gay), metal, fire,
                       flip, flop, rotate, border
      --gay, --rainbow O mesmo que -F rainbow
      --metal          O mesmo que -F metal
  -w, --width <n>      Largura da saída (por omissão: a do terminal, ou 80)
  -c, -l, -r           Centrar, à esquerda ou à direita
  -k, --kerning        Letras encostadas, sem fusão
  -W, --full-width     Letras com a largura toda
  -s, --smush          Fundir as letras (regras da fonte)
  -L, --list           Mostrar as fontes e os filtros
      --force          Cores mesmo quando a saída não é um terminal
      --lang <código>  Idioma: en, pt, es (também: RTOOLS_LANG)
  -h, --help           Mostrar esta ajuda
  -V, --version        Mostrar a versão
",
    builtin: "Fontes embutidas:",
    installed: "Fontes instaladas:",
    none_installed: "(nenhuma encontrada; ponha ficheiros .flf/.tlf numa das pastas abaixo ou use -d)",
    filters: "Filtros:",
    unknown_font: "fonte não encontrada",
    unknown_filter: "filtro desconhecido",
};

const ES: Texts = Texts {
    help_figlet: "\
Uso: rfiglet [opciones] [texto...]

Escribe el texto en letras grandes hechas de caracteres (de los argumentos, o por tubería).

  -f, --font <nombre>  Fuente: incluida, instalada (ver --list) o un fichero .flf/.tlf
                       (por defecto: \"standard\" si está instalada, si no \"hash\")
  -d, --dir <carpeta>  Otra carpeta con fuentes
  -w, --width <n>      Anchura de la salida (por defecto: la del terminal, u 80)
  -c, -l, -r           Centrar, a la izquierda o a la derecha
  -k, --kerning        Letras juntas, sin fusión
  -W, --full-width     Letras con toda su anchura
  -s, --smush          Fundir las letras (reglas de la fuente)
  -L, --list           Mostrar las fuentes
      --lang <código>  Idioma: en, pt, es (también: RTOOLS_LANG)
  -h, --help           Mostrar esta ayuda
  -V, --version        Mostrar la versión
",
    help_toilet: "\
Uso: rtoilet [opciones] [texto...]

Letras grandes y de colores (de los argumentos, o por tubería).

  -f, --font <nombre>  Fuente: incluida, instalada (ver --list) o un fichero .flf/.tlf
                       (por defecto \"pixel\")
  -d, --dir <carpeta>  Otra carpeta con fuentes
  -F, --filter <lista> Filtros, separados por \":\": crop, rainbow (gay), metal, fire,
                       flip, flop, rotate, border
      --gay, --rainbow Lo mismo que -F rainbow
      --metal          Lo mismo que -F metal
  -w, --width <n>      Anchura de la salida (por defecto: la del terminal, u 80)
  -c, -l, -r           Centrar, a la izquierda o a la derecha
  -k, --kerning        Letras juntas, sin fusión
  -W, --full-width     Letras con toda su anchura
  -s, --smush          Fundir las letras (reglas de la fuente)
  -L, --list           Mostrar las fuentes y los filtros
      --force          Colores aunque la salida no sea un terminal
      --lang <código>  Idioma: en, pt, es (también: RTOOLS_LANG)
  -h, --help           Mostrar esta ayuda
  -V, --version        Mostrar la versión
",
    builtin: "Fuentes incluidas:",
    installed: "Fuentes instaladas:",
    none_installed: "(no se encontró ninguna; ponga ficheros .flf/.tlf en una de las carpetas de abajo o use -d)",
    filters: "Filtros:",
    unknown_font: "fuente no encontrada",
    unknown_filter: "filtro desconocido",
};

pub fn run(toilet: bool) {
    let mut cli = Cli::new(if toilet { "rtoilet" } else { "rfiglet" });
    let t = Lang::pick(cli.lang, &EN, &PT, &ES);
    let (mut font_name, mut dirs, mut width_opt) = (None::<String>, Vec::<PathBuf>::new(), None::<usize>);
    let (mut align, mut mode, mut filters, mut force, mut list) = (None::<Align>, None::<Mode>, Vec::<String>::new(), false, false);
    let mut words = Vec::new();
    while let Some(a) = cli.next() {
        match a {
            Arg::Pos(p) => words.push(p),
            Arg::Opt(opt) => match opt.as_str() {
                "-h" | "--help" => cli.help(if toilet { t.help_toilet } else { t.help_figlet }),
                "-V" | "--version" => cli.version(),
                "-f" | "--font" => font_name = Some(cli.value(&opt)),
                "-d" | "--dir" => dirs.push(PathBuf::from(cli.value(&opt))),
                "-w" | "--width" => width_opt = Some(cli.number(&opt, 1..=10_000)),
                "-t" => width_opt = None,
                "-c" | "--center" => align = Some(Align::Center),
                "-l" | "--left" => align = Some(Align::Left),
                "-r" | "--right" => align = Some(Align::Right),
                "-k" | "--kerning" => mode = Some(Mode::Kerning),
                "-W" | "--full-width" => mode = Some(Mode::FullWidth),
                "-s" | "--smush" => mode = Some(Mode::Smush),
                "-L" | "--list" => list = true,
                "-F" | "--filter" if toilet => {
                    let v = cli.value(&opt);
                    for f in v.split(':').filter(|f| !f.is_empty()) {
                        if !FILTERS.contains(&f) {
                            cli.fail(&format!("{} '{f}'", t.unknown_filter));
                        }
                        filters.push(f.to_string());
                    }
                }
                "--gay" | "--rainbow" if toilet => filters.push("rainbow".into()),
                "--metal" if toilet => filters.push("metal".into()),
                "--force" if toilet => force = true,
                _ => cli.unknown(&opt),
            },
        }
    }
    let dirs = font_dirs(&dirs);
    if list {
        print_list(t, &dirs, toilet);
        return;
    }
    let font = match &font_name {
        Some(n) => load(n, &dirs).unwrap_or_else(|n| cli.fail(&format!("{} '{n}' (--list)", t.unknown_font))),
        None if toilet => Font::Builtin("pixel"),
        None => load("standard", &dirs).unwrap_or(Font::Builtin("hash")),
    };
    // Right-to-left fonts are right-aligned unless asked otherwise, as in figlet.
    let align = align.unwrap_or(if matches!(&font, Font::Flf(f) if f.rtl) { Align::Right } else { Align::Left });
    let max = width_opt.unwrap_or_else(|| {
        use std::io::IsTerminal;
        if std::io::stdout().is_terminal() { term::size().0 } else { 80 }
    });
    // As figlet: the last column stays free (a full line would wrap in the terminal).
    let max = max.saturating_sub(1).max(1);
    let text = if !words.is_empty() {
        words.join(" ")
    } else {
        match crate::input::piped_stdin() {
            Some(s) => s,
            None => cli.help(if toilet { t.help_toilet } else { t.help_figlet }),
        }
    };

    let mut grid: Grid = Vec::new();
    for line in text.trim_end_matches('\n').lines() {
        let line = crate::text::expand_tabs(line, 8);
        for block in layout(&font, &line, mode, max) {
            let bw = block_width(&block);
            let shift = match align {
                Align::Left => 0,
                // figlet's rounding: (width - line) / 2, with width = max + 1.
                Align::Center => (max + 1).saturating_sub(bw) / 2,
                Align::Right => max.saturating_sub(bw),
            };
            for row in block {
                let mut r: Vec<Cell> = vec![Cell { ch: ' ', fg: None }; shift];
                r.extend(row.into_iter().map(|ch| Cell { ch, fg: None }));
                grid.push(r);
            }
        }
    }
    for f in &filters {
        apply(&mut grid, f);
    }
    let depth = if filters.iter().any(|f| matches!(f.as_str(), "rainbow" | "gay" | "metal" | "fire")) {
        term::depth(force.then_some(true))
    } else {
        Depth::None
    };
    use std::io::Write;
    let _ = std::io::stdout().write_all(to_string(&grid, depth).as_bytes());
}

fn print_list(t: &Texts, dirs: &[PathBuf], toilet: bool) {
    println!("{}", t.builtin);
    for (n, d) in BUILTIN {
        println!("  {n:<8} {d}");
    }
    println!("\n{}", t.installed);
    let names = installed(dirs);
    if names.is_empty() {
        println!("  {}", t.none_installed);
    } else {
        for chunk in names.chunks(6) {
            println!("  {}", chunk.iter().map(|n| format!("{n:<12}")).collect::<String>().trim_end());
        }
    }
    for d in dirs {
        println!("  [{}]", d.display());
    }
    if toilet {
        println!("\n{}\n  {}", t.filters, FILTERS.join(", "));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // A tiny FIGlet font (two characters), to test the parser and smushing.
    const FONT: &str = "flf2a$ 3 2 6 15 1 0 24463\ncomment\n";

    fn font() -> FlfFont {
        let mut s = FONT.to_string();
        for c in 32u8..127 {
            match c {
                b'A' => s.push_str(" /\\ @\n/--\\@\n    @@\n"),
                b'B' => s.push_str("|_ @\n|_)@\n    @@\n"),
                b' ' => s.push_str("$@\n$@\n$@@\n"),
                _ => s.push_str("?@\n?@\n?@@\n"),
            }
        }
        parse_flf(&s).unwrap()
    }

    #[test]
    fn parses_and_lays_out() {
        let f = font();
        assert_eq!(f.height, 3);
        assert_eq!(f.mode, Mode::Smush);
        let full = flf_block(&f, "AB", Mode::FullWidth);
        assert_eq!(full[0].iter().collect::<String>().trim_end(), " /\\ |_");
        let k = flf_block(&f, "AB", Mode::Kerning);
        assert_eq!(k[0].iter().collect::<String>().trim_end(), " /\\ |_");
        assert_eq!(k[1].iter().collect::<String>().trim_end(), "/--\\|_)");
        let s = flf_block(&f, "AB", Mode::Smush);
        // '\' and '|' smush by the hierarchy rule (the later class wins).
        assert_eq!(s[1].iter().collect::<String>().trim_end(), "/--\\_)");
    }

    #[test]
    fn builtin_fonts_and_filters() {
        for (n, _) in BUILTIN {
            let b = builtin_block(n, "Olá, Ç!");
            assert!(!b.is_empty() && b.iter().all(|r| r.len() == b[0].len()), "{n}");
        }
        let mut g: Grid = builtin_block("pixel", "ab")
            .into_iter()
            .map(|r| r.into_iter().map(|ch| Cell { ch, fg: None }).collect())
            .collect();
        for f in FILTERS {
            apply(&mut g, f);
        }
        assert!(!to_string(&g, Depth::None).is_empty());
        let wrapped = layout(&Font::Builtin("hash"), "hello world again", None, 30);
        assert!(wrapped.len() > 1 && wrapped.iter().all(|b| block_width(b) <= 30));
    }
}

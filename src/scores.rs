//! High scores for the games: the best ten per game and mode, in a small text
//! file in the user's data folder (`%LOCALAPPDATA%\rtools` on Windows,
//! `~/.local/share/rtools` elsewhere). Problems reading or writing are ignored:
//! a game never fails because of its scores.

use std::path::PathBuf;

#[derive(Clone, Debug, PartialEq)]
pub struct Entry {
    pub score: u64,
    pub name: String,
    pub date: String,
}

pub const KEEP: usize = 10;

fn folder() -> Option<PathBuf> {
    if let Some(d) = std::env::var_os("RTOOLS_DATA").filter(|v| !v.is_empty()) {
        return Some(PathBuf::from(d));
    }
    if cfg!(windows) {
        std::env::var_os("LOCALAPPDATA").map(|d| PathBuf::from(d).join("rtools"))
    } else if let Some(d) = std::env::var_os("XDG_DATA_HOME").filter(|v| !v.is_empty()) {
        Some(PathBuf::from(d).join("rtools"))
    } else {
        std::env::var_os("HOME").map(|h| PathBuf::from(h).join(".local/share/rtools"))
    }
}

fn file(game: &str, mode: &str) -> Option<PathBuf> {
    let safe: String = format!("{game}-{mode}").chars().filter(|c| c.is_ascii_alphanumeric() || *c == '-').collect();
    folder().map(|d| d.join(format!("{safe}.scores")))
}

/// The user's name, for a new score.
pub fn player() -> String {
    ["USER", "USERNAME", "LOGNAME"]
        .iter()
        .find_map(|k| std::env::var(k).ok().filter(|v| !v.trim().is_empty()))
        .unwrap_or_else(|| "player".into())
        .chars()
        .filter(|c| !c.is_control() && *c != '\t')
        .take(16)
        .collect()
}

pub fn parse(text: &str) -> Vec<Entry> {
    let mut v: Vec<Entry> = text
        .lines()
        .filter_map(|l| {
            let mut f = l.splitn(3, '\t');
            let score = f.next()?.trim().parse().ok()?;
            Some(Entry { score, name: f.next().unwrap_or("").to_string(), date: f.next().unwrap_or("").to_string() })
        })
        .collect();
    v.sort_by(|a, b| b.score.cmp(&a.score));
    v.truncate(KEEP);
    v
}

pub fn load(game: &str, mode: &str) -> Vec<Entry> {
    file(game, mode).and_then(|p| std::fs::read_to_string(p).ok()).map(|t| parse(&t)).unwrap_or_default()
}

/// Adds a score; returns its place (0 = best) when it made the table.
pub fn add(game: &str, mode: &str, score: u64) -> Option<usize> {
    if score == 0 {
        return None;
    }
    let mut v = load(game, mode);
    let e = Entry { score, name: player(), date: chrono::Local::now().format("%Y-%m-%d").to_string() };
    let place = v.iter().position(|x| score > x.score).unwrap_or(v.len());
    if place >= KEEP {
        return None;
    }
    v.insert(place, e);
    v.truncate(KEEP);
    let path = file(game, mode)?;
    let _ = std::fs::create_dir_all(path.parent()?);
    let text: String = v.iter().map(|e| format!("{}\t{}\t{}\n", e.score, e.name, e.date)).collect();
    std::fs::write(path, text).ok()?;
    Some(place)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_sorts_and_skips_junk() {
        let v = parse("10\tana\t2026-01-01\nxx\n30\trui\t2026-01-02\n");
        assert_eq!(v.iter().map(|e| e.score).collect::<Vec<_>>(), [30, 10]);
        assert_eq!(v[0].name, "rui");
    }
}

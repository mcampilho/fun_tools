//! Git user name/email read straight from the global config files
//! (no libgit2 and no `git` process, which is much faster).

use std::path::PathBuf;

#[derive(Default)]
pub struct GitInfo {
    pub name: Option<String>,
    pub email: Option<String>,
}

pub fn collect() -> GitInfo {
    let mut info = GitInfo::default();
    // Lowest priority first; later files override earlier ones.
    for path in config_files() {
        if let Ok(text) = std::fs::read_to_string(&path) {
            parse_user(&text, &mut info);
        }
    }
    info
}

fn config_files() -> Vec<PathBuf> {
    if let Some(p) = std::env::var_os("GIT_CONFIG_GLOBAL") {
        return vec![PathBuf::from(p)];
    }
    let home = std::env::var_os("HOME")
        .or_else(|| std::env::var_os("USERPROFILE"))
        .map(PathBuf::from);
    let xdg = std::env::var_os("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .or_else(|| home.as_ref().map(|h| h.join(".config")));
    let mut files = Vec::new();
    if let Some(x) = xdg {
        files.push(x.join("git").join("config"));
    }
    if let Some(h) = home {
        files.push(h.join(".gitconfig"));
    }
    files
}

fn parse_user(text: &str, info: &mut GitInfo) {
    let mut in_user = false;
    for raw in text.lines() {
        let line = raw.trim();
        if line.starts_with('[') {
            let section = line.trim_start_matches('[').split([']', ' ', '"']).next().unwrap_or("");
            in_user = section.eq_ignore_ascii_case("user") && !line.contains('"');
            continue;
        }
        if !in_user {
            continue;
        }
        let Some((key, value)) = line.split_once('=') else { continue };
        let value = clean_value(value);
        if value.is_empty() {
            continue;
        }
        match key.trim().to_ascii_lowercase().as_str() {
            "name" => info.name = Some(value),
            "email" => info.email = Some(value),
            _ => {}
        }
    }
}

/// Removes surrounding quotes and trailing `#`/`;` comments outside quotes.
fn clean_value(v: &str) -> String {
    let mut out = String::new();
    let mut quoted = false;
    for c in v.trim().chars() {
        match c {
            '"' => quoted = !quoted,
            '#' | ';' if !quoted => break,
            _ => out.push(c),
        }
    }
    out.trim().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_user_section() {
        let mut g = GitInfo::default();
        parse_user(
            "[core]\n name = no\n[user]\n\tname = \"Ana Silva\" # c\n\temail = ana@x.pt\n[user \"x\"]\n name = sub\n",
            &mut g,
        );
        assert_eq!(g.name.as_deref(), Some("Ana Silva"));
        assert_eq!(g.email.as_deref(), Some("ana@x.pt"));
    }
}

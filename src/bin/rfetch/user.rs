//! Current user, without libc or Unix-only crates.

pub struct UserInfo {
    pub username: String,
    pub full_name: Option<String>,
}

pub fn collect() -> UserInfo {
    #[cfg(unix)]
    if let Some(u) = from_passwd() {
        return u;
    }
    let username = ["USERNAME", "USER", "LOGNAME"]
        .iter()
        .find_map(|k| std::env::var(k).ok().filter(|v| !v.is_empty()))
        .unwrap_or_else(|| "user".into());
    UserInfo { username, full_name: None }
}

/// Looks up the real uid (from /proc) in /etc/passwd to get the name and GECOS.
#[cfg(unix)]
fn from_passwd() -> Option<UserInfo> {
    let status = std::fs::read_to_string("/proc/self/status").ok()?;
    let uid = status
        .lines()
        .find_map(|l| l.strip_prefix("Uid:"))?
        .split_whitespace()
        .next()?
        .to_string();
    let passwd = std::fs::read_to_string("/etc/passwd").ok()?;
    passwd.lines().find_map(|line| {
        let f: Vec<&str> = line.split(':').collect();
        if f.len() < 7 || f[2] != uid {
            return None;
        }
        let full = f[4].split(',').next().unwrap_or("").trim();
        Some(UserInfo {
            username: f[0].to_string(),
            full_name: (!full.is_empty()).then(|| full.to_string()),
        })
    })
}

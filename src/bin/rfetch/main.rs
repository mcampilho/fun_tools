mod git;
mod i18n;
mod logo;
mod net;
mod regional;
mod render;
mod system;
mod user;

use logo::Colors;
use render::Row;
use system::Terminal;
use std::process::ExitCode;
use std::sync::mpsc;
use std::time::{Duration, Instant};

/// Maximum time we wait for the network after the local data is ready.
const NET_BUDGET: Duration = Duration::from_millis(3500);

#[derive(Default)]
struct Options {
    offline: bool,
    refresh: bool,
    color: Option<bool>,
    lang: Option<String>,
    logo: Option<String>,
    help: bool,
    version: bool,
}

fn parse_args() -> Result<Options, String> {
    let mut o = Options::default();
    let mut args = std::env::args().skip(1);
    while let Some(a) = args.next() {
        match a.as_str() {
            "-o" | "--offline" => o.offline = true,
            "-r" | "--refresh" => o.refresh = true,
            "--color" => o.color = Some(true),
            "--no-color" => o.color = Some(false),
            "-l" | "--lang" => o.lang = Some(args.next().ok_or(a)?),
            "--logo" => o.logo = Some(args.next().ok_or(a)?),
            "-h" | "--help" => o.help = true,
            "-V" | "--version" => o.version = true,
            _ => {
                if let Some(v) = a.strip_prefix("--lang=") {
                    o.lang = Some(v.to_string());
                } else if let Some(v) = a.strip_prefix("--logo=") {
                    o.logo = Some(v.to_string());
                } else {
                    return Err(a);
                }
            }
        }
    }
    Ok(o)
}

fn main() -> ExitCode {
    let start = Instant::now();
    let parsed = parse_args();
    let locale = regional::locale();
    let cli_lang = parsed.as_ref().ok().and_then(|o| o.lang.as_deref());
    let t = i18n::texts(i18n::detect(cli_lang, locale.as_deref()));

    let opts = match parsed {
        Ok(o) => o,
        Err(arg) => {
            eprintln!("rfetch: {}: {arg}\n\n{}", t.unknown_option, t.help);
            return ExitCode::from(2);
        }
    };
    if opts.help {
        println!("{}", t.help);
        return ExitCode::SUCCESS;
    }
    if opts.version {
        println!("rfetch {}", env!("CARGO_PKG_VERSION"));
        return ExitCode::SUCCESS;
    }
    if opts.logo.as_deref() == Some("list") {
        println!("{}\n  {}\n\n{}", t.logos_available, logo::names().join("\n  "), t.logos_wsl);
        return ExitCode::SUCCESS;
    }
    let color = render::use_color(opts.color);
    if let Some(n) = opts.logo.as_deref().filter(|n| logo::build(n, Colors::None).is_none()) {
        eprintln!("rfetch: {}: {n} (--logo list)", t.unknown_logo);
        return ExitCode::from(2);
    }

    // Network first, in the background, so it overlaps with the local work.
    let net_rx = (!opts.offline).then(|| {
        let (tx, rx) = mpsc::channel();
        let refresh = opts.refresh;
        std::thread::spawn(move || {
            let _ = tx.send(net::collect(refresh));
        });
        rx
    });

    // Local data: system info in its own thread, the cheap parts here.
    let (sys, user, git, tz, kbd) = std::thread::scope(|s| {
        let sys = s.spawn(system::collect);
        let user = user::collect();
        let git = git::collect();
        let tz = regional::timezone();
        let kbd = regional::keyboard();
        (sys.join().expect("system info thread"), user, git, tz, kbd)
    });

    let net = net_rx
        .and_then(|rx| rx.recv_timeout(NET_BUDGET.saturating_sub(start.elapsed())).ok())
        .unwrap_or_default();

    let rows = build_rows(t, &sys, &git, tz, locale, kbd, &net);
    let forced_16 = std::env::var("RFETCH_COLORS").ok().map(|v| v.trim() == "16");
    let colors = match (color, forced_16, sys.terminal) {
        (false, _, _) => Colors::None,
        (true, Some(true), _) | (true, None, Terminal::ConsoleZ) => Colors::Ansi16,
        (true, _, _) => Colors::Ansi256,
    };
    let name = opts
        .logo
        .clone()
        .unwrap_or_else(|| logo::auto_name(&sys.os_id, &sys.os_id_like, sys.flavor));
    let logo = logo::build(&name, colors)
        .or_else(|| logo::build("linux", colors))
        .expect("built-in logo");
    let title = match &user.full_name {
        Some(full) => format!("{}@{} ({full})", user.username, sys.host),
        None => format!("{}@{}", user.username, sys.host),
    };
    let encoding = match sys.terminal {
        Terminal::MobaXterm => render::Encoding::OemCodePage,
        _ => render::Encoding::Utf8,
    };
    render::print(&logo, &title, &rows, colors, encoding);
    ExitCode::SUCCESS
}

#[allow(clippy::too_many_arguments)]
fn build_rows(
    t: &i18n::Texts,
    sys: &system::SysInfo,
    git: &git::GitInfo,
    tz: Option<String>,
    locale: Option<String>,
    kbd: Option<String>,
    net: &net::NetInfo,
) -> Vec<Row> {
    let mut rows = Vec::with_capacity(16);
    let mut add = |label, value: Option<String>| {
        if let Some(value) = value.filter(|v| !v.is_empty()) {
            rows.push(Row { label, value });
        }
    };

    add(t.os, Some(format!("{} {}", sys.os, sys.arch)));
    add(t.kernel, sys.kernel.clone());
    add(t.uptime, Some(format_uptime(t, sys.uptime_secs)));
    add(t.shell, sys.shell.clone());
    let cores = match sys.cores_physical {
        Some(p) if p != sys.cores_logical => format!("{p}C/{}T", sys.cores_logical),
        _ => format!("{}C", sys.cores_logical),
    };
    add(
        t.cpu,
        Some(match &sys.cpu {
            Some(brand) => format!("{brand} ({cores})"),
            None => cores,
        }),
    );
    add(t.memory, Some(usage(sys.mem_used, sys.mem_total)));
    add(
        t.disk,
        sys.disk.as_ref().map(|d| format!("{} {}", usage(d.used, d.total), d.mount)),
    );
    add(
        t.git,
        match (&git.name, &git.email) {
            (Some(n), Some(e)) => Some(format!("{n} <{e}>")),
            (Some(n), None) => Some(n.clone()),
            (None, Some(e)) => Some(format!("<{e}>")),
            (None, None) => None,
        },
    );
    if let Some(g) = &net.geo {
        add(t.ip, Some(g.ip.clone()));
        let country = match (&g.country, &g.country_code) {
            (Some(c), Some(cc)) => Some(format!("{c} ({cc})")),
            (c, cc) => c.clone().or_else(|| cc.clone()),
        };
        let place = [g.city.clone(), country].into_iter().flatten().collect::<Vec<_>>();
        add(t.location, Some(place.join(", ")));
    }
    if let Some(w) = &net.weather {
        add(t.weather, Some(format!("{:.1} °C, {}", w.temp_c, t.weather(w.code))));
    }
    add(t.timezone, tz);
    add(t.locale, locale);
    add(t.keyboard, kbd);
    rows
}

fn usage(used: u64, total: u64) -> String {
    const GIB: f64 = 1024.0 * 1024.0 * 1024.0;
    let pct = if total > 0 { used as f64 * 100.0 / total as f64 } else { 0.0 };
    format!("{:.1} / {:.1} GiB ({pct:.0}%)", used as f64 / GIB, total as f64 / GIB)
}

fn format_uptime(t: &i18n::Texts, secs: u64) -> String {
    let (d, h, m) = (secs / 86_400, secs % 86_400 / 3600, secs % 3600 / 60);
    let mut parts = Vec::new();
    if d > 0 {
        parts.push(format!("{d} {}", t.days));
    }
    if d > 0 || h > 0 {
        parts.push(format!("{h} {}", t.hours));
    }
    parts.push(format!("{m} {}", t.minutes));
    parts.join(" ")
}

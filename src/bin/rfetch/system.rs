//! OS, hardware and shell information (local, no network).

use std::path::{Path, PathBuf};
use sysinfo::{
    CpuRefreshKind, Disks, MemoryRefreshKind, ProcessRefreshKind, ProcessesToUpdate, RefreshKind,
    System, UpdateKind,
};

/// The environment rfetch runs in, beyond the OS itself (chooses the logo).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(not(windows), allow(dead_code))] // the Windows variants are only detected on Windows
pub enum Flavor {
    Native,
    /// Windows PowerShell 5.1
    PowerShell,
    /// PowerShell 7+
    Pwsh,
    GitBash,
    GitCmd,
    Msys2(Msystem),
    Cygwin,
    /// Linux running under WSL
    Wsl,
}

/// MSYS2 environment, from the MSYSTEM variable (each has its own icon colour).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(not(windows), allow(dead_code))]
pub enum Msystem {
    Msys,
    Ucrt64,
    Mingw64,
    Mingw32,
    Clang64,
    Clang32,
    ClangArm64,
}

impl Msystem {
    #[cfg_attr(not(windows), allow(dead_code))]
    fn from_env() -> Self {
        let v = std::env::var("MSYSTEM").unwrap_or_default().to_ascii_uppercase();
        match v.as_str() {
            "UCRT64" => Self::Ucrt64,
            "MINGW64" => Self::Mingw64,
            "MINGW32" => Self::Mingw32,
            "CLANG64" => Self::Clang64,
            "CLANG32" => Self::Clang32,
            "CLANGARM64" => Self::ClangArm64,
            _ => Self::Msys,
        }
    }
}

/// Terminals that need special output (found among the parent processes).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(not(windows), allow(dead_code))]
pub enum Terminal {
    Other,
    /// ConsoleZ copies the console's text with its 16 legacy colours.
    ConsoleZ,
    /// MobaXterm reads the output of Windows programs in the console code page.
    MobaXterm,
}

pub struct SysInfo {
    pub os: String,
    pub os_id: String,
    pub os_id_like: String,
    pub arch: String,
    pub kernel: Option<String>,
    pub host: String,
    pub uptime_secs: u64,
    pub cpu: Option<String>,
    pub cores_physical: Option<usize>,
    pub cores_logical: usize,
    pub mem_used: u64,
    pub mem_total: u64,
    pub disk: Option<DiskUsage>,
    pub shell: Option<String>,
    pub flavor: Flavor,
    pub terminal: Terminal,
}

pub struct DiskUsage {
    pub mount: String,
    pub used: u64,
    pub total: u64,
}

pub fn collect() -> SysInfo {
    // Only refresh RAM. The CPU list is NOT loaded through sysinfo: on Windows it
    // sets up PDH performance counters, which takes from 0.5 s to several seconds.
    let mut sys = System::new_with_specifics(
        RefreshKind::nothing().with_memory(MemoryRefreshKind::nothing().with_ram()),
    );

    let (os, os_id, os_id_like) = os_name();
    let cpu = cpu_brand(&mut sys)
        .map(|b| b.split_whitespace().collect::<Vec<_>>().join(" "))
        .filter(|b| !b.is_empty());
    let cores_logical = std::thread::available_parallelism().map_or(1, |n| n.get());
    let (shell, flavor, terminal) = detect_shell(&mut sys);

    SysInfo {
        os,
        os_id,
        os_id_like,
        arch: System::cpu_arch(),
        kernel: System::kernel_version(),
        host: System::host_name().unwrap_or_else(|| "localhost".into()),
        uptime_secs: System::uptime(),
        cpu,
        cores_physical: sys.physical_core_count(),
        cores_logical,
        mem_used: sys.used_memory(),
        mem_total: sys.total_memory(),
        disk: system_disk(),
        shell,
        flavor,
        terminal,
    }
}

/// CPU brand string: CPUID on x86 (instant), /proc/cpuinfo on other Linux CPUs,
/// and sysinfo only as a last resort.
fn cpu_brand(sys: &mut System) -> Option<String> {
    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    if let Some(b) = cpuid_brand() {
        return Some(b);
    }
    #[cfg(target_os = "linux")]
    if let Ok(info) = std::fs::read_to_string("/proc/cpuinfo") {
        let found = ["model name", "Model", "Hardware", "cpu model"].iter().find_map(|key| {
            info.lines().find_map(|l| {
                let (k, v) = l.split_once(':')?;
                (k.trim() == *key && !v.trim().is_empty()).then(|| v.trim().to_string())
            })
        });
        if found.is_some() {
            return found;
        }
    }
    sys.refresh_cpu_list(CpuRefreshKind::nothing());
    sys.cpus().first().map(|c| c.brand().to_string())
}

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
#[allow(unused_unsafe)]
fn cpuid_brand() -> Option<String> {
    #[cfg(target_arch = "x86")]
    use std::arch::x86::__cpuid;
    #[cfg(target_arch = "x86_64")]
    use std::arch::x86_64::__cpuid;

    // SAFETY: CPUID is available on every x86_64 CPU and on any x86 CPU Rust targets;
    // leaves 0x8000_0002..4 are only read after checking the maximum extended leaf.
    unsafe {
        if __cpuid(0x8000_0000).eax < 0x8000_0004 {
            return None;
        }
        let mut bytes = Vec::with_capacity(48);
        for leaf in 0x8000_0002u32..=0x8000_0004 {
            let r = __cpuid(leaf);
            for reg in [r.eax, r.ebx, r.ecx, r.edx] {
                bytes.extend_from_slice(&reg.to_le_bytes());
            }
        }
        let s = String::from_utf8_lossy(&bytes).trim_matches(['\0', ' ']).to_string();
        (!s.is_empty()).then_some(s)
    }
}

/// Returns (pretty name, id, id_like).
#[cfg(target_os = "linux")]
fn os_name() -> (String, String, String) {
    let text = std::fs::read_to_string("/etc/os-release")
        .or_else(|_| std::fs::read_to_string("/usr/lib/os-release"))
        .unwrap_or_default();
    let get = |key: &str| {
        text.lines()
            .find_map(|l| l.strip_prefix(key)?.strip_prefix('='))
            .map(|v| v.trim().trim_matches(['"', '\'']).to_string())
            .filter(|v| !v.is_empty())
    };
    let pretty = get("PRETTY_NAME")
        .or_else(|| get("NAME"))
        .or_else(System::long_os_version)
        .unwrap_or_else(|| "Linux".into());
    (
        pretty,
        get("ID").unwrap_or_else(System::distribution_id),
        get("ID_LIKE").unwrap_or_default(),
    )
}

#[cfg(windows)]
fn os_name() -> (String, String, String) {
    let mut name = System::long_os_version().unwrap_or_else(|| "Windows".into());
    // The registry still says "Windows 10" on Windows 11; builds >= 22000 are 11.
    let build: u32 = System::kernel_version()
        .and_then(|k| k.split('.').next_back()?.trim().parse().ok())
        .unwrap_or(0);
    if build >= 22000 && name.contains("Windows 10") {
        name = name.replacen("Windows 10", "Windows 11", 1);
    }
    (name, "windows".into(), String::new())
}

#[cfg(not(any(target_os = "linux", windows)))]
fn os_name() -> (String, String, String) {
    (
        System::long_os_version().unwrap_or_else(|| std::env::consts::OS.into()),
        System::distribution_id(),
        String::new(),
    )
}

/// Usage of the disk holding the system (C:\ on Windows, / elsewhere).
fn system_disk() -> Option<DiskUsage> {
    let target: PathBuf = if cfg!(windows) {
        let drive = std::env::var("SystemDrive").unwrap_or_else(|_| "C:".into());
        PathBuf::from(format!("{}\\", drive.trim_end_matches('\\')))
    } else {
        PathBuf::from("/")
    };
    let disks = Disks::new_with_refreshed_list();
    let disk = disks
        .list()
        .iter()
        .find(|d| same_path(d.mount_point(), &target))?;
    let total = disk.total_space();
    (total > 0).then(|| DiskUsage {
        mount: target.display().to_string(),
        used: total.saturating_sub(disk.available_space()),
        total,
    })
}

fn same_path(a: &Path, b: &Path) -> bool {
    if cfg!(windows) {
        a.to_string_lossy().eq_ignore_ascii_case(&b.to_string_lossy())
    } else {
        a == b
    }
}

const SHELLS: &[&str] = &[
    "pwsh", "powershell", "cmd", "bash", "zsh", "fish", "nu", "sh", "dash", "ksh", "mksh",
    "tcsh", "csh", "elvish", "xonsh", "ion", "oil", "osh", "yash", "murex",
];

/// A parent process: lower-case name without ".exe", and its executable path.
struct Proc {
    stem: String,
    #[cfg_attr(not(windows), allow(dead_code))]
    exe: Option<PathBuf>,
}

/// Parent processes, nearest first. Stops two levels above the first shell
/// (enough to see a launcher such as git-cmd.exe). Only these few processes
/// are refreshed, never the whole process list.
fn ancestors(sys: &mut System) -> Vec<Proc> {
    let mut chain = Vec::new();
    let Ok(mut pid) = sysinfo::get_current_pid() else { return chain };
    let kind = ProcessRefreshKind::nothing().with_exe(UpdateKind::OnlyIfNotSet);
    sys.refresh_processes_specifics(ProcessesToUpdate::Some(&[pid]), false, kind);
    let mut extra_after_shell = None::<u8>;
    for _ in 0..10 {
        let Some(parent) = sys.process(pid).and_then(|p| p.parent()) else { break };
        sys.refresh_processes_specifics(ProcessesToUpdate::Some(&[parent]), false, kind);
        let Some(p) = sys.process(parent) else { break };
        let name = p.name().to_string_lossy().to_lowercase();
        let stem = name.strip_suffix(".exe").unwrap_or(&name).trim_start_matches('-').to_string();
        let is_shell = SHELLS.contains(&stem.as_str());
        chain.push(Proc { stem, exe: p.exe().map(Path::to_path_buf) });
        match extra_after_shell {
            Some(0) => break,
            Some(n) => extra_after_shell = Some(n - 1),
            None if is_shell => extra_after_shell = Some(1),
            None => {}
        }
        pid = parent;
    }
    chain
}

fn detect_shell(sys: &mut System) -> (Option<String>, Flavor, Terminal) {
    let chain = ancestors(sys);
    let idx = chain.iter().position(|p| SHELLS.contains(&p.stem.as_str()));
    let flavor = flavor(&chain, idx);
    let terminal = chain
        .iter()
        .find_map(|p| match p.stem.as_str() {
            "console" | "consolez" if cfg!(windows) => Some(Terminal::ConsoleZ),
            "mobaxterm" if cfg!(windows) => Some(Terminal::MobaXterm),
            _ => None,
        })
        .or_else(|| idx.and_then(|i| chain[i].exe.as_deref()).and_then(terminal_of_shell))
        .unwrap_or(Terminal::Other);

    let base = idx.map(|i| pretty_shell(&chain[i].stem)).or_else(|| {
        // Fallback on Unix: the login shell.
        let shell = std::env::var("SHELL").ok()?;
        let name = Path::new(&shell).file_name()?.to_string_lossy().into_owned();
        Some(pretty_shell(&name))
    });
    let shell = base.map(|b| match flavor {
        Flavor::GitBash => format!("{b} (Git Bash)"),
        Flavor::GitCmd => "Git CMD (cmd)".to_string(),
        Flavor::Cygwin => format!("{b} (Cygwin)"),
        Flavor::Msys2(_) => match std::env::var("MSYSTEM") {
            Ok(sub) if !sub.is_empty() => format!("{b} (MSYS2 {sub})"),
            _ => format!("{b} (MSYS2)"),
        },
        _ => b,
    });
    (shell, flavor, terminal)
}

#[cfg(windows)]
fn flavor(chain: &[Proc], idx: Option<usize>) -> Flavor {
    let msystem = std::env::var_os("MSYSTEM").is_some_and(|v| !v.is_empty());
    let msys2 = Flavor::Msys2(Msystem::from_env());
    let Some(i) = idx else {
        return if msystem { msys2 } else { Flavor::Native };
    };
    let shell = &chain[i];
    let launched_by = |name: &str| chain[i + 1..].iter().any(|p| p.stem == name);
    match shell.stem.as_str() {
        "pwsh" => Flavor::Pwsh,
        "powershell" => Flavor::PowerShell,
        "cmd" if launched_by("git-cmd") => Flavor::GitCmd,
        "cmd" => Flavor::Native,
        // A Unix shell: find out which Unix layer its executable belongs to.
        _ => shell
            .exe
            .as_deref()
            .and_then(unix_layer)
            .or_else(|| launched_by("git-bash").then_some(Flavor::GitBash))
            .or_else(|| msystem.then_some(msys2))
            .unwrap_or(Flavor::Native),
    }
}

/// Looks at the folders above a shell's executable for the files that mark a
/// Git for Windows, MSYS2 or Cygwin installation.
#[cfg(windows)]
fn unix_layer(exe: &Path) -> Option<Flavor> {
    for dir in exe.ancestors().skip(1).take(4) {
        let has = |f: &str| dir.join(f).is_file();
        if has("git-bash.exe") || has("git-cmd.exe") {
            return Some(Flavor::GitBash);
        }
        if has("msys2.exe") || has("msys2_shell.cmd") {
            return Some(Flavor::Msys2(Msystem::from_env()));
        }
        if has("cygwin1.dll") || has("Cygwin.bat") {
            return Some(Flavor::Cygwin);
        }
    }
    None
}

/// MobaXterm's own shell: its Windows parent is gone (Cygwin forks), so the
/// terminal is recognised by the files next to the shell's executable.
#[cfg(windows)]
fn terminal_of_shell(exe: &Path) -> Option<Terminal> {
    let dir = exe.parent()?;
    ["motty.exe", "cygtermd.exe", "XWin_MobaX.exe"]
        .iter()
        .any(|f| dir.join(f).is_file())
        .then_some(Terminal::MobaXterm)
}

#[cfg(not(windows))]
fn terminal_of_shell(_exe: &Path) -> Option<Terminal> {
    None
}

#[cfg(not(windows))]
fn flavor(_chain: &[Proc], _idx: Option<usize>) -> Flavor {
    let wsl = std::env::var_os("WSL_DISTRO_NAME").is_some()
        || std::fs::read_to_string("/proc/sys/kernel/osrelease")
            .is_ok_and(|r| r.to_ascii_lowercase().contains("microsoft"));
    if wsl { Flavor::Wsl } else { Flavor::Native }
}

fn pretty_shell(stem: &str) -> String {
    match stem {
        "pwsh" => "PowerShell 7 (pwsh)".into(),
        "powershell" => "Windows PowerShell".into(),
        "cmd" => "Command Prompt (cmd)".into(),
        "nu" => "Nushell".into(),
        other => other.into(),
    }
}

//! rgenact: busy-looking fake activity in the terminal (like genact): builds,
//! boot logs, downloads, deployments... Nothing is really done. Ctrl+C stops it.

use fun::cli::{Arg, Cli};
use fun::i18n::{common, Lang};
use fun::rng::Rng;
use fun::term::{self, Depth, Rgb};
use std::io::Write;
use std::time::{Duration, Instant};

struct Texts {
    help: &'static str,
    modules: &'static str,
}

const EN: Texts = Texts {
    help: "\
Usage: rgenact [options]

Looks very busy: shows fake activity from one module after another, at random.
Nothing is really done. Ctrl+C stops it.

  -m, --module <name>  Only this module (can be given more than once; see -l)
  -l, --list           List the modules
  -s, --speed <x>      Speed factor, 0.1 to 20 (default 1)
      --exit-after-time <s>     Stop after this many seconds
      --exit-after-modules <n>  Stop after this many modules
      --color          Always use colours
      --no-color       Never use colours (also: NO_COLOR)
      --lang <code>    Language: en, pt, es (also: RTOOLS_LANG)
  -h, --help           Show this help
  -V, --version        Show the version
",
    modules: "Modules:",
};

const PT: Texts = Texts {
    help: "\
Utilização: rgenact [opções]

Parece muito ocupado: mostra atividade a fingir de um módulo atrás do outro, ao
acaso. Nada é feito a sério. Ctrl+C para parar.

  -m, --module <nome>  Só este módulo (pode repetir-se; ver -l)
  -l, --list           Mostrar os módulos
  -s, --speed <x>      Fator de velocidade, de 0.1 a 20 (por omissão 1)
      --exit-after-time <s>     Parar ao fim deste número de segundos
      --exit-after-modules <n>  Parar ao fim deste número de módulos
      --color          Usar sempre cores
      --no-color       Nunca usar cores (também: NO_COLOR)
      --lang <código>  Idioma: en, pt, es (também: RTOOLS_LANG)
  -h, --help           Mostrar esta ajuda
  -V, --version        Mostrar a versão
",
    modules: "Módulos:",
};

const ES: Texts = Texts {
    help: "\
Uso: rgenact [opciones]

Parece muy ocupado: muestra actividad fingida de un módulo tras otro, al azar.
No se hace nada de verdad. Ctrl+C lo para.

  -m, --module <nombre> Solo este módulo (puede repetirse; ver -l)
  -l, --list           Mostrar los módulos
  -s, --speed <x>      Factor de velocidad, de 0.1 a 20 (por defecto 1)
      --exit-after-time <s>     Parar tras este número de segundos
      --exit-after-modules <n>  Parar tras este número de módulos
      --color          Usar siempre colores
      --no-color       No usar nunca colores (también: NO_COLOR)
      --lang <código>  Idioma: en, pt, es (también: RTOOLS_LANG)
  -h, --help           Mostrar esta ayuda
  -V, --version        Mostrar la versión
",
    modules: "Módulos:",
};

const GREEN: Rgb = Rgb(80, 210, 100);
const YELLOW: Rgb = Rgb(240, 200, 60);
const RED: Rgb = Rgb(235, 80, 70);
const BLUE: Rgb = Rgb(90, 150, 250);
const CYAN: Rgb = Rgb(80, 210, 220);
const GREY: Rgb = Rgb(140, 140, 150);
const MAGENTA: Rgb = Rgb(210, 110, 220);

// Word parts for made-up names: everything shown is invented.
const PARTS: &[&str] = &[
    "async", "byte", "core", "data", "fast", "flux", "hash", "hyper", "io", "json", "lite", "log", "macro", "mem",
    "micro", "net", "num", "parse", "proto", "quick", "rand", "ring", "serde", "sync", "term", "time", "tiny", "uni",
    "url", "web", "xml", "yaml", "zip", "atom", "glyph", "pixel", "shell", "stream", "task", "vector",
];
const SUFFIXES: &[&str] = &["", "", "-core", "-derive", "-sys", "-utils", "-macros", "-rt", "-io", "-lite"];
const DIRS: &[&str] = &["drivers/net", "drivers/gpu/drm", "fs/ext4", "kernel/sched", "mm", "net/ipv4", "sound/core", "crypto", "drivers/usb/core", "arch/x86/kernel", "block", "lib"];
const SERVICES: &[&str] = &[
    "Journal Service", "Network Manager", "Load Kernel Modules", "Remount Root and Kernel File Systems", "Create Volatile Files and Directories",
    "Login Service", "D-Bus System Message Bus", "Time Synchronization", "Disk Manager", "OpenSSH Daemon", "Update UTMP about System Boot", "Permit User Sessions",
];
const KERNEL_MSGS: &[&str] = &[
    "usb {n}-{n}: new high-speed USB device number {n} using xhci_hcd",
    "EXT4-fs (nvme0n1p{n}): mounted filesystem with ordered data mode",
    "ACPI: Added _OSI(Module Device)",
    "PCI: Using configuration type 1 for base access",
    "clocksource: Switched to clocksource tsc",
    "random: crng init done",
    "e1000e 0000:00:1f.{n} eth0: NIC Link is Up 1000 Mbps Full Duplex",
    "Bluetooth: HCI device and connection manager initialized",
    "input: Power Button as /devices/LNXSYSTM:00/input/input{n}",
    "Memory: {n}K/{n}K available",
    "smpboot: CPU{n} is up",
    "NET: Registered PF_INET6 protocol family",
    "audit: initializing netlink subsys (disabled)",
    "thermal_sys: Registered thermal governor 'step_wise'",
];
const PACKAGES: &[&str] = &["libssl3", "curl", "ca-certificates", "python3-pip", "build-essential", "git", "nginx", "zlib1g-dev", "libffi-dev", "tzdata"];
const LOADING: &[&str] = &[
    "Polishing the pixels", "Counting the semicolons", "Untangling the cables", "Warming up the hamsters",
    "Aligning the bits", "Feeding the compiler", "Sorting the bytes by colour", "Asking the cache nicely",
    "Folding the proteins of the database", "Calibrating the coffee machine", "Persuading the threads to cooperate",
    "Dusting off the old backups", "Rounding up the stray pointers", "Teaching the AI to whistle",
];
const PATHS: &[&str] = &["/", "/index.html", "/api/v2/users", "/static/app.js", "/static/style.css", "/login", "/images/logo.png", "/api/v2/orders", "/health", "/feed.xml", "/search?q=rust"];
const AGENTS: &[&str] = &["Mozilla/5.0 (X11; Linux x86_64)", "Mozilla/5.0 (Windows NT 10.0; Win64; x64)", "curl/8.5.0", "ExampleBot/2.1"];

struct Ctx {
    rng: Rng,
    depth: Depth,
    speed: f64,
    deadline: Option<Instant>,
    out: std::io::Stdout,
}

impl Ctx {
    /// Waits about `ms` milliseconds (scaled by the speed). False when it is time to stop.
    fn wait(&mut self, ms: u64) -> bool {
        let _ = self.out.flush();
        std::thread::sleep(Duration::from_secs_f64(ms as f64 / 1000.0 / self.speed));
        self.deadline.is_none_or(|d| Instant::now() < d)
    }

    /// Waits a random time between `lo` and `hi` milliseconds.
    fn pause(&mut self, lo: i64, hi: i64) -> bool {
        let ms = self.rng.range(lo, hi) as u64;
        self.wait(ms)
    }

    fn c(&self, rgb: Rgb, s: &str) -> String {
        if self.depth == Depth::None { s.to_string() } else { format!("{}{s}{}", term::fg(self.depth, rgb), term::RESET) }
    }

    fn bold(&self, rgb: Rgb, s: &str) -> String {
        if self.depth == Depth::None { s.to_string() } else { format!("\x1b[1m{}{s}{}", term::fg(self.depth, rgb), term::RESET) }
    }

    fn say(&mut self, s: &str) {
        let _ = writeln!(self.out, "{s}");
    }

    fn name(&mut self) -> String {
        let a = *self.rng.pick(PARTS);
        let b = *self.rng.pick(SUFFIXES);
        if self.rng.chance(0.3) {
            let c = *self.rng.pick(PARTS);
            format!("{a}{c}{b}")
        } else {
            format!("{a}{b}")
        }
    }

    fn version(&mut self) -> String {
        format!("{}.{}.{}", self.rng.range(0, 4), self.rng.range(0, 30), self.rng.range(0, 20))
    }

    fn hex(&mut self, n: usize) -> String {
        (0..n).map(|_| char::from(b"0123456789abcdef"[self.rng.below(16)])).collect()
    }

    /// Fills `{n}` with random numbers.
    fn fill(&mut self, s: &str) -> String {
        let mut out = String::new();
        for (i, part) in s.split("{n}").enumerate() {
            if i > 0 {
                out.push_str(&self.rng.range(0, 64).to_string());
            }
            out.push_str(part);
        }
        out
    }

    /// A progress bar redrawn in place; false when it is time to stop.
    fn progress(&mut self, label: &str, total_ms: u64) -> bool {
        let width = 30;
        let steps = 40;
        let rate = self.rng.range(20, 900) as f64 / 10.0;
        for i in 0..=steps {
            let f = i as f64 / steps as f64;
            let fill = (f * width as f64) as usize;
            let bar = format!("{}{}{}", "=".repeat(fill), if fill < width { ">" } else { "" }, " ".repeat(width - fill.min(width) - if fill < width { 1 } else { 0 }));
            let eta = ((1.0 - f) * total_ms as f64 / 1000.0).ceil() as u64;
            let _ = write!(self.out, "\r{label:<28} [{}] {:>3}% {:>6.1} MB/s  ETA 0:{eta:02}", self.c(CYAN, &bar), (f * 100.0) as u32, rate);
            if !self.wait(total_ms / steps as u64) {
                self.say("");
                return false;
            }
        }
        self.say("");
        true
    }
}

type Module = fn(&mut Ctx) -> bool;

/// Name, description (en, pt, es) and the module.
const MODULES: &[(&str, [&str; 3], Module)] = &[
    ("cargo", ["Rust crates being compiled", "Crates de Rust a compilar", "Crates de Rust compilándose"], cargo),
    ("bootlog", ["A Linux machine booting", "Uma máquina Linux a arrancar", "Una máquina Linux arrancando"], bootlog),
    ("kernel", ["A kernel being built", "Um kernel a ser compilado", "Un kernel compilándose"], kernel),
    ("docker", ["A container image being built", "Uma imagem de contentor a ser construída", "Una imagen de contenedor construyéndose"], docker),
    ("download", ["Big files being downloaded", "Ficheiros grandes a descarregar", "Archivos grandes descargándose"], download),
    ("memdump", ["A memory dump", "Um despejo de memória", "Un volcado de memoria"], memdump),
    ("packages", ["System packages being installed", "Pacotes do sistema a instalar", "Paquetes del sistema instalándose"], packages),
    ("terraform", ["Cloud servers being created", "Servidores na nuvem a ser criados", "Servidores en la nube creándose"], terraform),
    ("ansible", ["Servers being configured", "Servidores a ser configurados", "Servidores configurándose"], ansible),
    ("weblog", ["A web server's access log", "O registo de acessos de um servidor web", "El registro de accesos de un servidor web"], weblog),
    ("hashcrack", ["Passwords being cracked (not really)", "Palavras-passe a ser quebradas (a fingir)", "Contraseñas descifrándose (de mentira)"], hashcrack),
    ("loading", ["Loading, loading...", "A carregar, a carregar...", "Cargando, cargando..."], loading),
];

fn cargo(c: &mut Ctx) -> bool {
    let n = c.rng.range(15, 60);
    let start = Instant::now();
    for _ in 0..n {
        let (name, v) = (c.name(), c.version());
        if c.rng.chance(0.2) {
            let s = format!("{} {name} v{v}", c.bold(GREEN, &format!("{:>12}", "Downloaded")));
            c.say(&s);
        }
        let s = format!("{} {name} v{v}", c.bold(GREEN, &format!("{:>12}", "Compiling")));
        c.say(&s);
        if c.rng.chance(0.05) {
            let s = format!("{}: unused variable: `{}`", c.bold(YELLOW, "warning"), c.rng.pick(PARTS));
            c.say(&s);
        }
        if !c.pause(60, 700) {
            return false;
        }
    }
    let s = format!("{} `release` profile [optimized] target(s) in {:.2}s", c.bold(GREEN, &format!("{:>12}", "Finished")), start.elapsed().as_secs_f64() * c.speed);
    c.say(&s);
    c.wait(800)
}

fn bootlog(c: &mut Ctx) -> bool {
    let mut t = 0.0f64;
    for _ in 0..c.rng.range(25, 60) {
        t += c.rng.float() * 0.3;
        let tpl = *c.rng.pick(KERNEL_MSGS);
        let msg = c.fill(tpl);
        let s = format!("{} {msg}", c.c(GREEN, &format!("[{t:>12.6}]")));
        c.say(&s);
        if !c.pause(10, 250) {
            return false;
        }
    }
    for _ in 0..c.rng.range(6, 14) {
        let svc = *c.rng.pick(SERVICES);
        let s = format!("[{}] Started {svc}.", c.c(GREEN, "  OK  "));
        c.say(&s);
        if !c.pause(40, 300) {
            return false;
        }
    }
    c.wait(600)
}

fn kernel(c: &mut Ctx) -> bool {
    for _ in 0..c.rng.range(30, 90) {
        let dir = *c.rng.pick(DIRS);
        let file = c.name().replace('-', "_");
        let (tag, ext) = if c.rng.chance(0.15) { ("LD [M]", "ko") } else { ("CC", "o") };
        c.say(&format!("  {tag:<7} {dir}/{file}.{ext}"));
        if !c.pause(15, 200) {
            return false;
        }
    }
    c.say("  LD      vmlinux");
    c.say("  OBJCOPY arch/x86/boot/vmlinux.bin");
    let s = format!("Kernel: arch/x86/boot/bzImage is ready  (#{})", c.rng.range(1, 40));
    c.say(&s);
    c.wait(800)
}

fn docker(c: &mut Ctx) -> bool {
    let steps = [
        "FROM debian:stable-slim".to_string(),
        "WORKDIR /app".to_string(),
        "COPY Cargo.toml Cargo.lock ./".to_string(),
        format!("RUN apt-get update && apt-get install -y {}", c.rng.pick(PACKAGES)),
        "COPY src ./src".to_string(),
        "RUN cargo build --release".to_string(),
        format!("EXPOSE {}", c.rng.range(3000, 9000)),
        format!("CMD [\"./{}\"]", c.name()),
    ];
    let n = steps.len();
    for (i, s) in steps.iter().enumerate() {
        let head = c.bold(BLUE, &format!("Step {}/{n} :", i + 1));
        c.say(&format!("{head} {s}"));
        if s.starts_with("RUN") {
            for _ in 0..c.rng.range(2, 8) {
                let l = format!(" ---> Running in {}", c.hex(12));
                c.say(&c.c(GREY, &l));
                if !c.pause(100, 600) {
                    return false;
                }
            }
        }
        let id = c.hex(12);
        c.say(&format!(" ---> {id}"));
        if !c.pause(100, 700) {
            return false;
        }
    }
    let s = format!("Successfully built {}", c.hex(12));
    c.say(&c.c(GREEN, &s));
    c.wait(800)
}

fn download(c: &mut Ctx) -> bool {
    for _ in 0..c.rng.range(2, 5) {
        let ext = *c.rng.pick(&["tar.gz", "iso", "zip", "deb", "img.xz"]);
        let file = format!("{}-{}.{ext}", c.name(), c.version());
        let file: String = file.chars().take(28).collect();
        let ms = c.rng.range(1500, 5000) as u64;
        if !c.progress(&file, ms) {
            return false;
        }
    }
    c.wait(500)
}

fn memdump(c: &mut Ctx) -> bool {
    let mut addr = 0x7ff0_0000_0000u64 + (c.rng.next_u64() % 0xffff_ffff) * 16;
    for _ in 0..c.rng.range(40, 120) {
        let bytes: Vec<u8> = (0..16).map(|_| if c.rng.chance(0.25) { 0 } else { c.rng.below(256) as u8 }).collect();
        let hex: Vec<String> = bytes.iter().map(|b| format!("{b:02x}")).collect();
        let ascii: String = bytes.iter().map(|&b| if (32..127).contains(&b) { b as char } else { '.' }).collect();
        let s = format!("{}  {}  {}", c.c(CYAN, &format!("{addr:016x}")), hex.join(" "), c.c(GREY, &format!("|{ascii}|")));
        c.say(&s);
        addr += 16;
        if !c.pause(10, 80) {
            return false;
        }
    }
    c.wait(400)
}

fn packages(c: &mut Ctx) -> bool {
    let n = c.rng.range(4, 10);
    let mut list = Vec::new();
    for _ in 0..n {
        list.push(c.name());
    }
    c.say("Reading package lists... Done");
    c.say("Building dependency tree... Done");
    c.say(&format!("The following NEW packages will be installed:\n  {}", list.join(" ")));
    for (i, p) in list.iter().enumerate() {
        let v = c.version();
        let kb = c.rng.range(20, 9000);
        c.say(&format!("Get:{} https://deb.example.org/debian stable/main amd64 {p} amd64 {v} [{kb} kB]", i + 1));
        if !c.pause(80, 500) {
            return false;
        }
    }
    for p in &list {
        c.say(&format!("Unpacking {p} ..."));
        if !c.pause(50, 300) {
            return false;
        }
        c.say(&format!("Setting up {p} ..."));
    }
    c.wait(600)
}

fn terraform(c: &mut Ctx) -> bool {
    let kinds = ["server", "volume", "network", "dns_record", "firewall", "load_balancer", "bucket"];
    let n = c.rng.range(3, 9);
    let res: Vec<String> = (0..n).map(|i| format!("cloud_{}.{}[{i}]", c.rng.pick(&kinds), c.rng.pick(PARTS))).collect();
    let s = format!("Plan: {n} to add, 0 to change, 0 to destroy.");
    c.say(&c.bold(Rgb(230, 230, 230), &s));
    for r in &res {
        c.say(&format!("{}: Creating...", c.bold(Rgb(230, 230, 230), r)));
        let secs = c.rng.range(1, 5);
        for k in 1..=secs {
            if !c.pause(300, 900) {
                return false;
            }
            c.say(&format!("{}: Still creating... [{}s elapsed]", c.bold(Rgb(230, 230, 230), r), k * 10));
        }
        let id = c.hex(8);
        c.say(&format!("{}: Creation complete after {}s [id={id}]", c.bold(Rgb(230, 230, 230), r), secs * 10 + 3));
    }
    let s = format!("Apply complete! Resources: {n} added, 0 changed, 0 destroyed.");
    c.say(&c.bold(GREEN, &s));
    c.wait(800)
}

fn ansible(c: &mut Ctx) -> bool {
    let hosts: Vec<String> = (0..c.rng.range(2, 6)).map(|i| format!("{}{:02}", c.rng.pick(&["web", "db", "cache", "worker"]), i + 1)).collect();
    let tasks = ["Gathering Facts", "install packages", "copy configuration", "create users", "enable services", "open firewall ports", "restart application"];
    for t in tasks {
        let stars = "*".repeat(60usize.saturating_sub(t.len() + 7));
        c.say(&format!("\nTASK [{t}] {stars}"));
        for h in &hosts {
            let s = if c.rng.chance(0.4) { c.c(YELLOW, &format!("changed: [{h}]")) } else { c.c(GREEN, &format!("ok: [{h}]")) };
            c.say(&s);
            if !c.pause(60, 400) {
                return false;
            }
        }
    }
    c.say("\nPLAY RECAP *********************************************");
    for h in &hosts {
        let changed = c.rng.range(0, 5);
        let s = format!("{h:<12} : {}  changed={changed}  unreachable=0  failed=0", c.c(GREEN, &format!("ok={}", 7)));
        c.say(&s);
    }
    c.wait(800)
}

fn weblog(c: &mut Ctx) -> bool {
    // Only the address ranges set aside for documentation.
    let nets = ["192.0.2", "198.51.100", "203.0.113"];
    for _ in 0..c.rng.range(30, 90) {
        let ip = format!("{}.{}", c.rng.pick(&nets), c.rng.range(1, 255));
        let (code, col) = match c.rng.below(20) {
            0 => (404, YELLOW),
            1 => (500, RED),
            2 | 3 => (304, CYAN),
            _ => (200, GREEN),
        };
        let path = *c.rng.pick(PATHS);
        let agent = *c.rng.pick(AGENTS);
        let now = chrono::Local::now().format("%d/%b/%Y:%H:%M:%S %z");
        let size = c.rng.range(200, 90000);
        let method = if c.rng.chance(0.85) { "GET" } else { "POST" };
        let s = format!("{ip} - - [{now}] \"{method} {path} HTTP/1.1\" {} {size} \"-\" \"{agent}\"", c.c(col, &code.to_string()));
        c.say(&s);
        if !c.pause(20, 400) {
            return false;
        }
    }
    c.wait(300)
}

fn hashcrack(c: &mut Ctx) -> bool {
    let n = c.rng.range(3, 7);
    let s = format!("Loaded {n} password hashes (sha256, made up)");
    c.say(&c.bold(MAGENTA, &s));
    for _ in 0..n {
        let hash = c.hex(64);
        for _ in 0..c.rng.range(5, 15) {
            let guess = c.name();
            let s = format!("{}  trying {guess:<20} {} H/s", c.c(GREY, &hash[..16]), c.rng.range(10_000, 9_000_000));
            c.say(&s);
            if !c.pause(30, 150) {
                return false;
            }
        }
        let found = c.name();
        let s = format!("{}  {}", c.c(GREY, &hash[..16]), c.bold(GREEN, &format!("FOUND: {found}")));
        c.say(&s);
    }
    c.wait(700)
}

fn loading(c: &mut Ctx) -> bool {
    for _ in 0..c.rng.range(3, 7) {
        let msg = *c.rng.pick(LOADING);
        let spin = ['⠋', '⠙', '⠹', '⠸', '⠼', '⠴', '⠦', '⠧', '⠇', '⠏'];
        for i in 0..c.rng.range(15, 40) as usize {
            let _ = write!(c.out, "\r{} {msg}...", c.c(CYAN, &spin[i % spin.len()].to_string()));
            if !c.wait(80) {
                c.say("");
                return false;
            }
        }
        let _ = writeln!(c.out, "\r{} {msg}... done", c.c(GREEN, "✔"));
    }
    c.wait(500)
}

fn main() {
    let mut cli = Cli::new("rgenact");
    let t = Lang::pick(cli.lang, &EN, &PT, &ES);
    let lang = cli.lang;
    let (mut chosen, mut speed, mut time, mut count, mut color) = (Vec::<usize>::new(), 1.0f64, None::<f64>, None::<usize>, None::<bool>);
    while let Some(a) = cli.next() {
        match a {
            Arg::Pos(p) => cli.unknown(&p),
            Arg::Opt(opt) => match opt.as_str() {
                "-h" | "--help" => cli.help(t.help),
                "-V" | "--version" => cli.version(),
                "-l" | "--list" => {
                    println!("{}", t.modules);
                    for (name, desc, _) in MODULES {
                        println!("  {name:<10} {}", lang.pick(&desc[0], &desc[1], &desc[2]));
                    }
                    return;
                }
                "-m" | "--module" => {
                    let v = cli.value(&opt).to_ascii_lowercase();
                    match MODULES.iter().position(|(n, _, _)| *n == v) {
                        Some(i) => chosen.push(i),
                        None => cli.fail(&format!("{} '{opt}': {v}", common(lang).invalid_value)),
                    }
                }
                "-s" | "--speed" => speed = cli.number(&opt, 0.1..=20.0),
                "--exit-after-time" => time = Some(cli.number(&opt, 0.1..=1e7)),
                "--exit-after-modules" => count = Some(cli.number(&opt, 1..=1_000_000)),
                "--color" => color = Some(true),
                "--no-color" => color = Some(false),
                _ => cli.unknown(&opt),
            },
        }
    }
    if chosen.is_empty() {
        chosen = (0..MODULES.len()).collect();
    }
    let mut c = Ctx {
        rng: Rng::new(),
        depth: term::depth(color),
        speed,
        deadline: time.map(|s| Instant::now() + Duration::from_secs_f64(s)),
        out: std::io::stdout(),
    };
    let mut done = 0;
    let mut last = usize::MAX;
    loop {
        // Never the same module twice in a row (when there is a choice).
        let mut i = *c.rng.pick(&chosen);
        if chosen.len() > 1 {
            while i == last {
                i = *c.rng.pick(&chosen);
            }
        }
        last = i;
        let keep_going = (MODULES[i].2)(&mut c);
        done += 1;
        if !keep_going || count.is_some_and(|n| done >= n) {
            break;
        }
    }
    let _ = c.out.flush();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_module_runs() {
        for (name, _, m) in MODULES {
            let mut c = Ctx { rng: Rng::seeded(1), depth: Depth::None, speed: 1000.0, deadline: None, out: std::io::stdout() };
            assert!(m(&mut c), "{name}");
        }
    }
}

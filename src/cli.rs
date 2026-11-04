//! A small command-line parser shared by the tools: `-x`, `--long`,
//! `--long=value`, `--` (only positional arguments after it) and `-` (stdin).
//! `--lang <code>` is handled here for every tool.

use crate::i18n::{common, Lang};
use std::str::FromStr;

pub enum Arg {
    /// An option, as written (`-W`, `--width`); an inline `=value` is kept for `value()`.
    Opt(String),
    Pos(String),
}

pub struct Cli {
    pub tool: &'static str,
    pub lang: Lang,
    args: Vec<String>,
    pos: usize,
    only_positional: bool,
    inline: Option<(String, String)>,
}

impl Cli {
    pub fn new(tool: &'static str) -> Cli {
        Cli::from_args(tool, std::env::args().skip(1).collect())
    }

    pub fn from_args(tool: &'static str, mut args: Vec<String>) -> Cli {
        // The language is needed before anything is printed, so take it out first.
        let mut lang = None;
        let mut i = 0;
        while i < args.len() && args[i] != "--" {
            if args[i] == "--lang" && i + 1 < args.len() {
                lang = Some(args[i + 1].clone());
                args.drain(i..i + 2);
            } else if let Some(v) = args[i].strip_prefix("--lang=") {
                lang = Some(v.to_string());
                args.remove(i);
            } else {
                i += 1;
            }
        }
        Cli {
            tool,
            lang: Lang::detect(lang.as_deref()),
            args,
            pos: 0,
            only_positional: false,
            inline: None,
        }
    }

    #[allow(clippy::should_implement_trait)]
    pub fn next(&mut self) -> Option<Arg> {
        if let Some((opt, v)) = self.inline.take() {
            // `--flag=value` given to an option that takes no value.
            self.fail(&format!("{} '{opt}={v}'", common(self.lang).invalid_value));
        }
        let a = self.args.get(self.pos)?.clone();
        self.pos += 1;
        if self.only_positional || a == "-" || !a.starts_with('-') {
            return Some(Arg::Pos(a));
        }
        if a == "--" {
            self.only_positional = true;
            return self.next();
        }
        if let Some((opt, v)) = a.split_once('=').filter(|_| a.starts_with("--")) {
            self.inline = Some((opt.to_string(), v.to_string()));
            return Some(Arg::Opt(opt.to_string()));
        }
        Some(Arg::Opt(a))
    }

    /// The value of the option just returned by `next()`.
    pub fn value(&mut self, opt: &str) -> String {
        if let Some((_, v)) = self.inline.take() {
            return v;
        }
        match self.args.get(self.pos) {
            Some(v) => {
                self.pos += 1;
                v.clone()
            }
            None => self.fail(&format!("{} '{opt}'", common(self.lang).missing_value)),
        }
    }

    pub fn parse<T: FromStr>(&mut self, opt: &str) -> T {
        let v = self.value(opt);
        v.trim()
            .parse()
            .unwrap_or_else(|_| self.fail(&format!("{} '{opt}': {v}", common(self.lang).invalid_value)))
    }

    /// Like `parse`, but the number must be inside `range`.
    pub fn number<T>(&mut self, opt: &str, range: std::ops::RangeInclusive<T>) -> T
    where
        T: FromStr + PartialOrd + Copy + std::fmt::Display,
    {
        let n: T = self.parse(opt);
        if !range.contains(&n) {
            self.fail(&format!(
                "{} '{opt}': {n} ({}..{})",
                common(self.lang).invalid_value,
                range.start(),
                range.end()
            ));
        }
        n
    }

    pub fn unknown(&self, opt: &str) -> ! {
        self.fail(&format!("{} '{opt}'", common(self.lang).unknown_option))
    }

    /// Prints a usage error and exits with code 2.
    pub fn fail(&self, msg: &str) -> ! {
        eprintln!("{}: {msg}\n{}", self.tool, common(self.lang).try_help);
        std::process::exit(2)
    }

    pub fn help(&self, text: &str) -> ! {
        print!("{text}");
        std::process::exit(0)
    }

    pub fn version(&self) -> ! {
        println!("{} {}", self.tool, env!("CARGO_PKG_VERSION"));
        std::process::exit(0)
    }
}

/// Prints a runtime error (not a usage error) and exits with code 1.
pub fn die(tool: &str, msg: &str) -> ! {
    eprintln!("{tool}: {msg}");
    std::process::exit(1)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cli(a: &[&str]) -> Cli {
        Cli::from_args("t", a.iter().map(|s| s.to_string()).collect())
    }

    #[test]
    fn parses() {
        let mut c = cli(&["-W", "30", "--eyes=xx", "--lang", "pt", "hello", "--", "-n"]);
        assert_eq!(c.lang, Lang::Pt);
        let mut seen = Vec::new();
        while let Some(a) = c.next() {
            match a {
                Arg::Opt(o) if o == "-W" => seen.push(format!("W{}", c.parse::<u32>("-W"))),
                Arg::Opt(o) if o == "--eyes" => seen.push(format!("e{}", c.value("--eyes"))),
                Arg::Opt(o) => seen.push(o),
                Arg::Pos(p) => seen.push(format!("p{p}")),
            }
        }
        assert_eq!(seen, ["W30", "exx", "phello", "p-n"]);
    }
}

//! rcowsay / rcowthink: an animal saying (or thinking) a message in a balloon.

use crate::cli::{Arg, Cli};
use crate::i18n::Lang;
use crate::text::{expand_tabs, width, wrap_text};

/// Animals. In the art: `$\` is the balloon trail (`\` or `o`), `$e` the two
/// eyes, `$1`/`$2` each eye, `$t` the tongue (two columns).
pub const ANIMALS: &[(&str, &str)] = &[
    (
        "cow",
        r"        $\   ^__^
         $\  ($e)\_______
            (__)\       )\/\
             $t ||----w |
                ||     ||
",
    ),
    (
        "calf",
        r"      $\   ,__,
       $\  ($e)____
           (__)    )\
            $t||--|| *
",
    ),
    (
        "sheep",
        r"     $\
      $\   .--.
          ( $1$2 )@@@@@@@@
           `--'@@@@@@@@@@
             $t@@@@@@@@@@
               ||    ||
",
    ),
    (
        "cat",
        r"   $\
    $\   /\_/\
        ( $1.$2 )
         > $t<
        /     \
       (|     |)__
        \_____/   )
",
    ),
    (
        "owl",
        r"   $\
    $\    ___
        ($1v$2)
       /(   )\
        -^-^-
",
    ),
    (
        "crab",
        r"    $\
     $\
        _~^~^~_
    \) /  $1 $2  \ (/
      '_   $t  _'
      / '-----' \
",
    ),
    (
        "ghost",
        r"   $\
    $\   .-.
        ($1 $2)
        | $t \
        \    \
         `~~~~'
",
    ),
];

struct Texts {
    help_say: &'static str,
    help_think: &'static str,
    animals: &'static str,
    unknown_animal: &'static str,
    default_say: &'static str,
    default_think: &'static str,
}

const EN: Texts = Texts {
    help_say: "\
Usage: rcowsay [options] [message...]

An animal says the message (from the arguments, or piped in).

  -f, --animal <name>  Animal (see --list); -r for a random one
  -l, --list           List the animals
  -e, --eyes <xx>      Eyes (two characters)
  -T, --tongue <xx>    Tongue (two characters)
  -W, --width <n>      Wrap the message at n columns (default 40)
  -n, --nowrap         Do not wrap; keep the message's own lines
  -b -d -g -p -s -t -w -y
                       Looks: borg, dead, greedy, paranoid, stoned, tired, wired, young
      --lang <code>    Language: en, pt, es (also: RTOOLS_LANG)
  -h, --help           Show this help
  -V, --version        Show the version
",
    help_think: "\
Usage: rcowthink [options] [message...]

An animal thinks the message (from the arguments, or piped in).

  -f, --animal <name>  Animal (see --list); -r for a random one
  -l, --list           List the animals
  -e, --eyes <xx>      Eyes (two characters)
  -T, --tongue <xx>    Tongue (two characters)
  -W, --width <n>      Wrap the message at n columns (default 40)
  -n, --nowrap         Do not wrap; keep the message's own lines
  -b -d -g -p -s -t -w -y
                       Looks: borg, dead, greedy, paranoid, stoned, tired, wired, young
      --lang <code>    Language: en, pt, es (also: RTOOLS_LANG)
  -h, --help           Show this help
  -V, --version        Show the version
",
    animals: "Animals:",
    unknown_animal: "unknown animal",
    default_say: "Moo! Rust says hello.",
    default_think: "Hmm... what should I compile next?",
};

const PT: Texts = Texts {
    help_say: "\
Utilização: rcowsay [opções] [mensagem...]

Um animal diz a mensagem (dos argumentos, ou recebida por pipe).

  -f, --animal <nome>  Animal (ver --list); -r para um ao acaso
  -l, --list           Mostrar os animais
  -e, --eyes <xx>      Olhos (dois caracteres)
  -T, --tongue <xx>    Língua (dois caracteres)
  -W, --width <n>      Quebrar a mensagem às n colunas (por omissão 40)
  -n, --nowrap         Não quebrar; manter as linhas da mensagem
  -b -d -g -p -s -t -w -y
                       Aspeto: borg, morto, ganancioso, paranoico, pedrado, cansado,
                       elétrico, jovem
      --lang <código>  Idioma: en, pt, es (também: RTOOLS_LANG)
  -h, --help           Mostrar esta ajuda
  -V, --version        Mostrar a versão
",
    help_think: "\
Utilização: rcowthink [opções] [mensagem...]

Um animal pensa a mensagem (dos argumentos, ou recebida por pipe).

  -f, --animal <nome>  Animal (ver --list); -r para um ao acaso
  -l, --list           Mostrar os animais
  -e, --eyes <xx>      Olhos (dois caracteres)
  -T, --tongue <xx>    Língua (dois caracteres)
  -W, --width <n>      Quebrar a mensagem às n colunas (por omissão 40)
  -n, --nowrap         Não quebrar; manter as linhas da mensagem
  -b -d -g -p -s -t -w -y
                       Aspeto: borg, morto, ganancioso, paranoico, pedrado, cansado,
                       elétrico, jovem
      --lang <código>  Idioma: en, pt, es (também: RTOOLS_LANG)
  -h, --help           Mostrar esta ajuda
  -V, --version        Mostrar a versão
",
    animals: "Animais:",
    unknown_animal: "animal desconhecido",
    default_say: "Muuu! O Rust diz olá.",
    default_think: "Hmm... o que compilo a seguir?",
};

const ES: Texts = Texts {
    help_say: "\
Uso: rcowsay [opciones] [mensaje...]

Un animal dice el mensaje (de los argumentos, o recibido por tubería).

  -f, --animal <nombre> Animal (ver --list); -r para uno al azar
  -l, --list            Mostrar los animales
  -e, --eyes <xx>       Ojos (dos caracteres)
  -T, --tongue <xx>     Lengua (dos caracteres)
  -W, --width <n>       Cortar el mensaje a n columnas (por defecto 40)
  -n, --nowrap          No cortar; mantener las líneas del mensaje
  -b -d -g -p -s -t -w -y
                        Aspecto: borg, muerto, codicioso, paranoico, colocado, cansado,
                        eléctrico, joven
      --lang <código>   Idioma: en, pt, es (también: RTOOLS_LANG)
  -h, --help            Mostrar esta ayuda
  -V, --version         Mostrar la versión
",
    help_think: "\
Uso: rcowthink [opciones] [mensaje...]

Un animal piensa el mensaje (de los argumentos, o recibido por tubería).

  -f, --animal <nombre> Animal (ver --list); -r para uno al azar
  -l, --list            Mostrar los animales
  -e, --eyes <xx>       Ojos (dos caracteres)
  -T, --tongue <xx>     Lengua (dos caracteres)
  -W, --width <n>       Cortar el mensaje a n columnas (por defecto 40)
  -n, --nowrap          No cortar; mantener las líneas del mensaje
  -b -d -g -p -s -t -w -y
                        Aspecto: borg, muerto, codicioso, paranoico, colocado, cansado,
                        eléctrico, joven
      --lang <código>   Idioma: en, pt, es (también: RTOOLS_LANG)
  -h, --help            Mostrar esta ayuda
  -V, --version         Mostrar la versión
",
    animals: "Animales:",
    unknown_animal: "animal desconocido",
    default_say: "¡Muuu! Rust te saluda.",
    default_think: "Hmm... ¿qué compilo ahora?",
};

fn texts(l: Lang) -> &'static Texts {
    l.pick(&EN, &PT, &ES)
}

pub struct Options {
    pub think: bool,
    pub animal: String,
    pub eyes: String,
    pub tongue: String,
    pub width: usize,
    pub wrap: bool,
}

/// Fits a user value into exactly two columns.
fn two(s: &str, fallback: &str) -> String {
    let mut out: String = s.chars().take(2).collect();
    while width(&out) < 2 {
        out.push(fallback.chars().nth(width(&out)).unwrap_or(' '));
    }
    out
}

/// The balloon around the message lines.
pub fn balloon(lines: &[String], think: bool) -> String {
    let w = lines.iter().map(|l| width(l)).max().unwrap_or(0);
    let mut out = format!(" {}\n", "_".repeat(w + 2));
    let n = lines.len();
    for (i, l) in lines.iter().enumerate() {
        let (a, b) = if think {
            ('(', ')')
        } else if n == 1 {
            ('<', '>')
        } else if i == 0 {
            ('/', '\\')
        } else if i == n - 1 {
            ('\\', '/')
        } else {
            ('|', '|')
        };
        let pad = " ".repeat(w - width(l));
        out.push_str(&format!("{a} {l}{pad} {b}\n"));
    }
    out.push_str(&format!(" {}\n", "-".repeat(w + 2)));
    out
}

/// Message in a balloon plus the animal.
pub fn render(message: &str, o: &Options) -> Option<String> {
    let art = ANIMALS.iter().find(|(n, _)| *n == o.animal)?.1;
    let message = expand_tabs(message.trim_end(), 8);
    let lines: Vec<String> = if o.wrap {
        wrap_text(&message, o.width)
    } else {
        message.lines().map(str::to_string).collect()
    };
    let lines = if lines.is_empty() { vec![String::new()] } else { lines };
    let eyes = two(&o.eyes, "oo");
    let tongue = two(&o.tongue, "  ");
    let mut e = eyes.chars();
    let (e1, e2) = (e.next().unwrap_or('o'), e.next().unwrap_or('o'));
    let body = art
        .replace("$\\", if o.think { "o" } else { "\\" })
        .replace("$e", &eyes)
        .replace("$1", &e1.to_string())
        .replace("$2", &e2.to_string())
        .replace("$t", &tongue);
    Some(balloon(&lines, o.think) + &body)
}

pub fn run(think: bool) {
    let mut cli = Cli::new(if think { "rcowthink" } else { "rcowsay" });
    let t = texts(cli.lang);
    let mut o = Options {
        think,
        animal: "cow".into(),
        eyes: "oo".into(),
        tongue: "  ".into(),
        width: 40,
        wrap: true,
    };
    let mut words = Vec::new();
    let mut random = false;
    while let Some(a) = cli.next() {
        match a {
            Arg::Pos(p) => words.push(p),
            Arg::Opt(opt) => match opt.as_str() {
                "-h" | "--help" => cli.help(if think { t.help_think } else { t.help_say }),
                "-V" | "--version" => cli.version(),
                "-l" | "--list" => {
                    let names: Vec<_> = ANIMALS.iter().map(|(n, _)| *n).collect();
                    println!("{}\n  {}", t.animals, names.join("\n  "));
                    return;
                }
                "-f" | "--animal" => o.animal = cli.value(&opt).to_ascii_lowercase(),
                "-r" | "--random" => random = true,
                "-e" | "--eyes" => o.eyes = cli.value(&opt),
                "-T" | "--tongue" => o.tongue = cli.value(&opt),
                "-W" | "--width" => o.width = cli.number(&opt, 1..=1000),
                "-n" | "--nowrap" => o.wrap = false,
                "-b" => o.eyes = "==".into(),
                "-d" => (o.eyes, o.tongue) = ("xx".into(), "U ".into()),
                "-g" => o.eyes = "$$".into(),
                "-p" => o.eyes = "@@".into(),
                "-s" => (o.eyes, o.tongue) = ("**".into(), "U ".into()),
                "-t" => o.eyes = "--".into(),
                "-w" => o.eyes = "OO".into(),
                "-y" => o.eyes = "..".into(),
                _ => cli.unknown(&opt),
            },
        }
    }
    if random {
        o.animal = crate::rng::Rng::new().pick(ANIMALS).0.to_string();
    }
    if !ANIMALS.iter().any(|(n, _)| *n == o.animal) {
        cli.fail(&format!("{} '{}'", t.unknown_animal, o.animal));
    }
    let message = if !words.is_empty() {
        words.join(" ")
    } else {
        crate::input::piped_stdin()
            .filter(|s| !s.trim().is_empty())
            .unwrap_or_else(|| (if think { t.default_think } else { t.default_say }).to_string())
    };
    if let Some(out) = render(&message, &o) {
        use std::io::Write;
        let _ = std::io::stdout().write_all(out.as_bytes());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn opts(think: bool) -> Options {
        Options { think, animal: "cow".into(), eyes: "oo".into(), tongue: "  ".into(), width: 10, wrap: true }
    }

    #[test]
    fn balloons() {
        let one = render("hi", &opts(false)).unwrap();
        assert!(one.starts_with(" ____\n< hi >\n ----\n        \\   ^__^"));
        let many = render("one two three four", &opts(false)).unwrap();
        assert!(many.starts_with(" ____________\n/ one two    \\\n\\ three four /\n"));
        let think = render("hm", &opts(true)).unwrap();
        assert!(think.contains("( hm )") && think.contains("o   ^__^"));
    }

    #[test]
    fn every_animal_renders_with_two_column_parts() {
        for (name, _) in ANIMALS {
            let o = Options { animal: name.to_string(), ..opts(false) };
            let s = render("x", &o).unwrap();
            assert!(!s.contains('$'), "{name}");
        }
        assert_eq!(two("x", "oo"), "xo");
        assert_eq!(two("abc", "oo"), "ab");
    }
}

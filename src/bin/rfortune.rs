//! rfortune: prints a random saying (like fortune). Built-in sayings in each
//! language, or fortune files (entries separated by lines holding only `%`).

use fun::cli::{die, Arg, Cli};
use fun::i18n::{common, Lang};
use fun::rng::Rng;
use std::path::{Path, PathBuf};

struct Texts {
    help: &'static str,
    nothing: &'static str,
    builtin: &'static str,
    sayings: &'static [&'static str],
}

const EN: Texts = Texts {
    help: "\
Usage: rfortune [options] [file|folder...]

Prints a random saying: a built-in one, or one from fortune files
(entries separated by lines holding only \"%\"). Folders are read file by file.
Without arguments, RFORTUNE_PATH (a list of files/folders) is used if set.

  -s, --short          Only short sayings (see -n)
  -l, --long           Only long sayings (see -n)
  -n, --length <n>     Limit between short and long, in characters (default 160)
  -m, --match <text>   Print every saying containing the text (any case)
  -c, --source         Show where the saying came from
  -e, --equal          Give every file the same chance, whatever its size
      --system         Also use the system's fortune files (Linux)
      --lang <code>    Language: en, pt, es (also: RTOOLS_LANG)
  -h, --help           Show this help
  -V, --version        Show the version
",
    nothing: "no sayings found",
    builtin: "built-in",
    sayings: &[
        "A program that compiles on the first try is not finished; it is hiding something.",
        "The borrow checker is not your enemy. It is the friend who tells you the truth.",
        "There are two hard things in computing: naming things, and naming things again after the meeting.",
        "Every \"temporary\" fix has a birthday party planned for its tenth anniversary.",
        "Today's clever code is tomorrow's archaeology.",
        "The fastest code is the code that never runs.",
        "If it works on your machine, ship your machine.",
        "A good error message is a letter to your future self. Write it kindly.",
        "Comments lie. Code lies less. Tests tell you which one is lying.",
        "Measure twice, optimise once, and only where it hurts.",
        "The bug is never where you are looking. That is why you are still looking.",
        "An hour of thinking can save a week of debugging.",
        "Make it work, make it clear, and only then make it clever.",
        "Your terminal is a garden: a little colour, a little order, and no weeds in the PATH.",
        "Rewriting it in Rust will not fix the requirements.",
        "Undefined behaviour means the compiler gets to choose the ending.",
        "Every abstraction leaks; the good ones only drip.",
        "The cloud is just someone else's computer, and it is having a bad day too.",
        "A backup you have never restored is only a hope.",
        "Read the error message. Then read it again, slowly.",
        "Premature generalisation is the root of most frameworks.",
        "Software is never finished, only released.",
        "There is no place like 127.0.0.1.",
        "The best time to write the tests was before the bug. The second best time is now.",
        "Old code has seen things. Be gentle with it.",
        "A deadline is only a suggestion until the client calls.",
        "Documentation is a love letter that you write to strangers.",
        "One more feature, said every abandoned project.",
        "If a function needs a comment to be understood, it probably needs a better name.",
        "Ctrl+S early, Ctrl+S often.",
    ],
};

const PT: Texts = Texts {
    help: "\
Utilização: rfortune [opções] [ficheiro|pasta...]

Mostra uma frase ao acaso: uma das embutidas, ou uma de ficheiros fortune
(entradas separadas por linhas só com \"%\"). As pastas são lidas ficheiro a ficheiro.
Sem argumentos, usa RFORTUNE_PATH (uma lista de ficheiros/pastas) se existir.

  -s, --short          Só frases curtas (ver -n)
  -l, --long           Só frases longas (ver -n)
  -n, --length <n>     Limite entre curta e longa, em caracteres (por omissão 160)
  -m, --match <texto>  Mostrar todas as frases que contêm o texto (sem distinguir maiúsculas)
  -c, --source         Mostrar de onde veio a frase
  -e, --equal          Dar a mesma probabilidade a todos os ficheiros, seja qual for o tamanho
      --system         Usar também os ficheiros fortune do sistema (Linux)
      --lang <código>  Idioma: en, pt, es (também: RTOOLS_LANG)
  -h, --help           Mostrar esta ajuda
  -V, --version        Mostrar a versão
",
    nothing: "nenhuma frase encontrada",
    builtin: "embutidas",
    sayings: &[
        "Um programa que compila à primeira não está acabado; está a esconder alguma coisa.",
        "O borrow checker não é teu inimigo. É o amigo que te diz a verdade.",
        "Há duas coisas difíceis em informática: dar nomes às coisas e voltar a dar-lhes nome depois da reunião.",
        "Todas as correções \"temporárias\" já têm festa marcada para o décimo aniversário.",
        "O código esperto de hoje é a arqueologia de amanhã.",
        "O código mais rápido é o que nunca chega a correr.",
        "Se funciona na tua máquina, envia a tua máquina ao cliente.",
        "Uma boa mensagem de erro é uma carta para o teu eu do futuro. Escreve-a com carinho.",
        "Os comentários mentem. O código mente menos. Os testes dizem-te qual deles está a mentir.",
        "Mede duas vezes, otimiza uma, e só onde dói.",
        "O erro nunca está onde estás a procurar. É por isso que ainda estás à procura.",
        "Uma hora a pensar pode poupar uma semana a depurar.",
        "Primeiro faz funcionar, depois torna claro, e só no fim torna esperto.",
        "O terminal é um jardim: um pouco de cor, um pouco de ordem e nenhuma erva daninha no PATH.",
        "Reescrever em Rust não corrige os requisitos.",
        "Comportamento indefinido quer dizer que o compilador escolhe o final da história.",
        "Todas as abstrações deixam passar água; as boas só pingam.",
        "A nuvem é só o computador de outra pessoa, e também está a ter um dia mau.",
        "Uma cópia de segurança que nunca restauraste é só uma esperança.",
        "Lê a mensagem de erro. Depois lê-a outra vez, devagar.",
        "A generalização prematura é a raiz da maior parte das frameworks.",
        "O software nunca está acabado, só publicado.",
        "Não há lugar como o 127.0.0.1.",
        "A melhor altura para escrever os testes era antes do erro. A segunda melhor é agora.",
        "O código antigo já viu muita coisa. Trata-o com cuidado.",
        "Um prazo é só uma sugestão até o cliente telefonar.",
        "A documentação é uma carta de amor escrita a desconhecidos.",
        "Só mais uma funcionalidade, disse cada projeto abandonado.",
        "Se uma função precisa de um comentário para se perceber, provavelmente precisa de um nome melhor.",
        "Ctrl+S cedo, Ctrl+S muitas vezes.",
    ],
};

const ES: Texts = Texts {
    help: "\
Uso: rfortune [opciones] [fichero|carpeta...]

Muestra una frase al azar: una de las incluidas, o una de ficheros fortune
(entradas separadas por líneas que solo tienen \"%\"). Las carpetas se leen fichero a fichero.
Sin argumentos, usa RFORTUNE_PATH (una lista de ficheros/carpetas) si existe.

  -s, --short          Solo frases cortas (ver -n)
  -l, --long           Solo frases largas (ver -n)
  -n, --length <n>     Límite entre corta y larga, en caracteres (por defecto 160)
  -m, --match <texto>  Mostrar todas las frases que contienen el texto (sin distinguir mayúsculas)
  -c, --source         Mostrar de dónde viene la frase
  -e, --equal          Dar la misma probabilidad a todos los ficheros, sea cual sea su tamaño
      --system         Usar también los ficheros fortune del sistema (Linux)
      --lang <código>  Idioma: en, pt, es (también: RTOOLS_LANG)
  -h, --help           Mostrar esta ayuda
  -V, --version        Mostrar la versión
",
    nothing: "no se encontraron frases",
    builtin: "incluidas",
    sayings: &[
        "Un programa que compila a la primera no está terminado; está escondiendo algo.",
        "El borrow checker no es tu enemigo. Es el amigo que te dice la verdad.",
        "Hay dos cosas difíciles en informática: poner nombres a las cosas y volver a ponérselos después de la reunión.",
        "Todo arreglo \"temporal\" ya tiene fiesta organizada para su décimo aniversario.",
        "El código ingenioso de hoy es la arqueología de mañana.",
        "El código más rápido es el que nunca llega a ejecutarse.",
        "Si funciona en tu máquina, envía tu máquina al cliente.",
        "Un buen mensaje de error es una carta a tu yo del futuro. Escríbela con cariño.",
        "Los comentarios mienten. El código miente menos. Las pruebas te dicen quién miente.",
        "Mide dos veces, optimiza una, y solo donde duele.",
        "El error nunca está donde estás buscando. Por eso sigues buscando.",
        "Una hora pensando puede ahorrar una semana depurando.",
        "Primero haz que funcione, luego que sea claro, y solo al final que sea ingenioso.",
        "El terminal es un jardín: un poco de color, un poco de orden y ninguna mala hierba en el PATH.",
        "Reescribirlo en Rust no arreglará los requisitos.",
        "Comportamiento indefinido significa que el compilador elige el final.",
        "Toda abstracción tiene fugas; las buenas solo gotean.",
        "La nube es solo el ordenador de otra persona, y también está teniendo un mal día.",
        "Una copia de seguridad que nunca has restaurado es solo una esperanza.",
        "Lee el mensaje de error. Luego léelo otra vez, despacio.",
        "La generalización prematura es la raíz de casi todos los frameworks.",
        "El software nunca se termina, solo se publica.",
        "No hay lugar como el 127.0.0.1.",
        "El mejor momento para escribir las pruebas era antes del error. El segundo mejor es ahora.",
        "El código antiguo ha visto muchas cosas. Trátalo con cuidado.",
        "Una fecha límite es solo una sugerencia hasta que llama el cliente.",
        "La documentación es una carta de amor que escribes a desconocidos.",
        "Solo una funcionalidad más, dijo cada proyecto abandonado.",
        "Si una función necesita un comentario para entenderse, probablemente necesita un nombre mejor.",
        "Ctrl+S pronto, Ctrl+S a menudo.",
    ],
};

/// Folders where Linux distributions keep fortune files.
const SYSTEM_DIRS: &[&str] = &["/usr/share/games/fortunes", "/usr/share/fortune", "/usr/share/fortunes"];

struct Source {
    name: String,
    entries: Vec<String>,
}

/// Splits a fortune file into entries (separator: a line holding only `%`).
fn parse(text: &str) -> Vec<String> {
    let mut entries = Vec::new();
    let mut cur = String::new();
    for line in text.lines() {
        if line.trim_end() == "%" {
            if !cur.trim().is_empty() {
                entries.push(cur.trim_end().to_string());
            }
            cur.clear();
        } else {
            cur.push_str(line);
            cur.push('\n');
        }
    }
    if !cur.trim().is_empty() {
        entries.push(cur.trim_end().to_string());
    }
    entries
}

/// A file, or every fortune file in a folder (index files such as `.dat` are skipped;
/// so is the `off` folder of offensive fortunes).
fn collect(path: &Path, out: &mut Vec<PathBuf>) -> std::io::Result<()> {
    if path.is_dir() {
        let mut items: Vec<_> = std::fs::read_dir(path)?.filter_map(Result::ok).map(|e| e.path()).collect();
        items.sort();
        for p in items {
            let ext = p.extension().and_then(|e| e.to_str()).unwrap_or("");
            if p.is_dir() {
                if p.file_name().is_some_and(|n| n != "off") {
                    collect(&p, out)?;
                }
            } else if !matches!(ext, "dat" | "u8" | "pdat" | "sh" | "gz") {
                out.push(p);
            }
        }
        Ok(())
    } else {
        std::fs::metadata(path)?;
        out.push(path.to_path_buf());
        Ok(())
    }
}

fn main() {
    let mut cli = Cli::new("rfortune");
    let c = common(cli.lang);
    let t = Lang::pick(cli.lang, &EN, &PT, &ES);
    let (mut short, mut long, mut limit) = (false, false, 160usize);
    let (mut pattern, mut show_source, mut equal, mut system) = (None::<String>, false, false, false);
    let mut paths: Vec<String> = Vec::new();
    while let Some(a) = cli.next() {
        match a {
            Arg::Pos(p) => paths.push(p),
            Arg::Opt(opt) => match opt.as_str() {
                "-h" | "--help" => cli.help(t.help),
                "-V" | "--version" => cli.version(),
                "-s" | "--short" => short = true,
                "-l" | "--long" => long = true,
                "-n" | "--length" => limit = cli.number(&opt, 1..=100_000),
                "-m" | "--match" => pattern = Some(cli.value(&opt).to_lowercase()),
                "-c" | "--source" => show_source = true,
                "-e" | "--equal" => equal = true,
                "--system" => system = true,
                _ => cli.unknown(&opt),
            },
        }
    }
    if paths.is_empty() {
        if let Ok(list) = std::env::var("RFORTUNE_PATH") {
            paths = std::env::split_paths(&list).map(|p| p.display().to_string()).collect();
        }
    }
    if system {
        paths.extend(SYSTEM_DIRS.iter().filter(|d| Path::new(d).is_dir()).map(|d| d.to_string()));
    }

    let mut sources = Vec::new();
    if paths.is_empty() {
        sources.push(Source { name: t.builtin.into(), entries: t.sayings.iter().map(|s| s.to_string()).collect() });
    }
    for p in &paths {
        let mut files = Vec::new();
        if let Err(e) = collect(Path::new(p), &mut files) {
            die("rfortune", &format!("{} '{p}': {e}", c.cannot_read));
        }
        for f in files {
            // Unreadable or binary files inside a folder are skipped.
            if let Ok(text) = fun::input::read_all(&f.display().to_string()) {
                let entries = parse(&text);
                if !entries.is_empty() {
                    sources.push(Source { name: f.display().to_string(), entries });
                }
            }
        }
    }

    // Filters.
    let len_ok = |e: &str| {
        let n = e.chars().count();
        (!short || n <= limit) && (!long || n > limit)
    };
    for s in &mut sources {
        s.entries.retain(|e| len_ok(e) && pattern.as_ref().is_none_or(|p| e.to_lowercase().contains(p)));
    }
    sources.retain(|s| !s.entries.is_empty());
    if sources.is_empty() {
        die("rfortune", t.nothing);
    }

    if pattern.is_some() {
        // Every match, fortune-style: separated by "%".
        let mut first = true;
        for s in &sources {
            for e in &s.entries {
                if !first {
                    println!("%");
                }
                first = false;
                if show_source {
                    println!("({})", s.name);
                }
                println!("{e}");
            }
        }
        return;
    }

    let mut rng = Rng::new();
    let (src, entry) = if equal {
        let s = rng.pick(&sources);
        (s, rng.pick(&s.entries))
    } else {
        // Every entry has the same chance.
        let total: usize = sources.iter().map(|s| s.entries.len()).sum();
        let mut k = rng.below(total);
        let s = sources
            .iter()
            .find(|s| {
                if k < s.entries.len() {
                    true
                } else {
                    k -= s.entries.len();
                    false
                }
            })
            .expect("index within total");
        (s, &s.entries[k])
    };
    if show_source {
        println!("({})\n%", src.name);
    }
    println!("{entry}");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_fortune_files() {
        let e = parse("one\n%\ntwo 50% off\nline 2\n%\n\n%\nthree\n");
        assert_eq!(e, ["one", "two 50% off\nline 2", "three"]);
    }

    #[test]
    fn same_number_of_sayings_in_every_language() {
        assert_eq!(EN.sayings.len(), PT.sayings.len());
        assert_eq!(EN.sayings.len(), ES.sayings.len());
    }
}

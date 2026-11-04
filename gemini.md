Aqui tem uma implementação completa em **Rust** inspirada no popular utilitário `lolcat` do Unix. O programa lê o fluxo de entrada (`stdin`) ou ficheiros passados como argumentos e aplica um gradiente de cores do arco-íris animado por caracteres, utilizando sequências ANSI padrão.

Para garantir que o executável seja **totalmente portátil, autónomo e sem dependências dinâmicas externas** (como o C runtime dinâmico em certas plataformas), utilizaremos a compilação estática (`musl` para Linux, MSVC estático para Windows e toolchain nativa otimizada para macOS).

---

### 1. Estrutura do Projeto Rust

Crie um novo projeto binário com o Cargo:

```bash
cargo new rlolcat --bin
cd rlolcat

```

Substitua o conteúdo de `Cargo.toml` pelas seguintes dependências essenciais:

```toml
[package]
name = "rlolcat"
version = "0.1.0"
edition = "2021"

[dependencies]
crossterm = "0.27"
atty = "0.2"

```

Substitua o código em `src/main.rs` pelo seguinte código fonte:

```rust
use std::io::{self, Read, Write};
use std::fs::File;
use std::path::PathBuf;
use crossterm::style::{Color, ResetColor, SetForegroundColor};
use crossterm::QueueableCommand;

fn print_rainbow<R: Read>(mut reader: R) -> io::Result<()> {
    let mut stdout = io::stdout();
    let mut buffer = [0; 1024];
    let mut frequency: f64 = 0.1;
    let mut seed: f64 = 0.0;

    loop {
        let n = reader.read(&mut buffer)?;
        if n == 0 {
            break;
        }

        for &byte in &buffer[..n] {
            if byte == b'\n' {
                stdout.write_all(b"\n")?;
                seed += 0.2; // Desloca o gradiente a cada nova linha
                continue;
            }

            // Cálculo do arco-íris baseado em funções senoidais para transição suave
            let red = (frequency * seed).sin() * 127.0 + 128.0;
            let green = (frequency * seed + std::f64::consts::PI * 2.0 / 3.0).sin() * 127.0 + 128.0;
            let blue = (frequency * seed + std::f64::consts::PI * 4.0 / 3.0).sin() * 127.0 + 128.0;

            stdout.queue(SetForegroundColor(Color::Rgb {
                r: red as u8,
                g: green as u8,
                b: blue as u8,
            }))?;

            stdout.write_all(&[byte])?;
            seed += 1.0;
        }
        stdout.flush()?;
    }

    stdout.queue(ResetColor)?;
    stdout.flush()?;
    Ok(())
}

fn main() {
    let args: Vec<String> = std::env::args().collect();

    if args.len() > 1 {
        for arg in &args[1..] {
            let path = PathBuf::from(arg);
            if let Ok(file) = File::open(path) {
                let _ = print_rainbow(file);
            } else {
                eprintln!("rlolcat: erro ao abrir o ficheiro: {}", arg);
            }
        }
    } else {
        let stdin = io::stdin();
        let _ = print_rainbow(stdin);
    }
}

```

---

### 2. Comandos de Compilação para Cada Plataforma

Para garantir executáveis **totalmente portáteis e sem dependências externas**, siga as instruções abaixo para cada sistema operativo alvo.

#### **A. Compilar para Linux (Estático com `musl`)**

Para gerar um binário Linux totalmente estático (que corre em qualquer distribuição sem depender de versões específicas do `glibc`), utilize o alvo `musl`.

1. Instalar o alvo musl:
```bash
rustup target add x86_64-unknown-linux-musl

```


2. Compilar de forma estática:
```bash
cargo build --release --target x86_64-unknown-linux-musl

```



*O binário autónomo ficará localizado em: `target/x86_64-unknown-linux-musl/release/rlolcat*`

---

#### **B. Compilar para Windows (Estático MSVC)**

No Windows, os executáveis gerados com a toolchain standard (`msvc`) já são bastante portáteis, mas para garantir que o binário é autónomo e não requer DLLs desnecessárias, compilamos em modo release otimizado.

1. Compilar nativamente no Windows:
```cmd
cargo build --release

```



*O executável autónomo ficará localizado em: `target\release\rlolcat.exe*`

*(Nota: Se estiver a fazer cross-compilation a partir do Linux para Windows, instale o `mingw-w64` e use o alvo `x86_64-pc-windows-gnu`).*

---

#### **C. Compilar para macOS (Intel e Apple Silicon)**

Para Mac, por defeito a compilação gera um binário compatível com a arquitetura da máquina anfitriã.

1. Compilar para macOS nativo:
```bash
cargo build --release

```



*O binário ficará localizado em: `target/release/rlolcat*`

Se pretender criar um binário universal compatível com arquiteturas Apple Silicon (M1/M2/M3) e Intel (x86_64) a partir de um Mac, pode compilar para ambos os alvos e uni-los com o comando `lipo`:

```bash
rustup target add x86_64-apple-darwin aarch64-apple-darwin
cargo build --release --target x86_64-apple-darwin
cargo build --release --target aarch64-apple-darwin

lipo -create -output rlolcat \
  target/x86_64-apple-darwin/release/rlolcat \
  target/aarch64-apple-darwin/release/rlolcat

```

Para ter múltiplos programas (binários) independentes dentro do mesmo projeto Rust, a melhor prática é utilizar a estrutura de **Multi-bin Crate** nativa do Cargo.

Em vez de ter apenas um `src/main.rs`, podemos organizar a pasta `src/` para conter um binário para o `rlolcat` e outro para o `rcowsay`.

---

### 1. Reestruturação do Projeto

Mude a estrutura de pastas do seu projeto para o seguinte formato:

```text
rlolcat/
├── Cargo.toml
└── src/
    ├── bin/
    │   ├── rlolcat.rs
    │   └── rcowsay.rs

```

1. Mova o seu código atual do `src/main.rs` para `src/bin/rlolcat.rs`.
2. Crie um novo ficheiro chamado `src/bin/rcowsay.rs`.
3. Pode apagar o ficheiro `src/main.rs` original, pois o Cargo passa a detetar automaticamente cada ficheiro dentro de `src/bin/` como um programa executável independente.

---

### 2. Implementação do `rcowsay`

Substitua o conteúdo de `src/bin/rcowsay.rs` por uma versão limpa e portátil do clássico `cowsay`:

```rust
use std::io::{self, Read};

fn print_cow(message: &str) {
    let width = message.chars().count();
    let border = "-".repeat(width + 2);

    println!(" ┌{}┐", border);
    println!(" │ {} │", message);
    println!(" └{}┘", border);
    println!("        \\   ^__^");
    println!("         \\  (oo)\\_______");
    println!("            (__)\\       )\\/\\");
    println!("                ||----w |");
    println!("                ||     ||");
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let message = if args.len() > 1 {
        args[1..].join(" ")
    } else {
        let mut buffer = String::new();
        if let Ok(_) = io::stdin().read_to_string(&mut buffer) {
            buffer.trim().to_string()
        } else {
            String::from("Olá do rcowsay!")
        }
    };

    print_cow(&message);
}

```

---

### 3. Como Compilar e Executar Cada Programa Individualmente

Com múltiplos binários no mesmo projeto, o Cargo permite escolher qual deles quer compilar ou executar utilizando a flag `--bin <nome>`.

#### **Durante o Desenvolvimento (Testar localmente):**

* Executar o rlolcat:
```bash
echo "Olá mundo" | cargo run --bin rlolcat

```


* Executar o rcowsay:
```bash
cargo run --bin rcowsay -- "O Rust é fantástico!"

```



#### **Compilar Ambas as Ferramentas para Produção (Release):**

Para compilar todos os binários otimizados de uma só vez:

```bash
cargo build --release

```

Ambos os executáveis ficarão disponíveis na pasta `target/release/` (`rlolcat` e `rcowsay`).

---

### 4. Dica Avançada: Como Juntar os Dois (Pipe estilo Unix)

Uma das maiores vantagens de manter a compatibilidade com o ecossistema Unix é que pode encadear os seus próprios programas em Rust usando o pipe (`|`):

```bash
cargo run --bin rcowsay -- "Ferramentas incríveis em Rust!" | cargo run --bin rlolcat

```


O problema com caracteres acentuados (como o "á" de "Olá" ou o "í" de "incríveis") acontece porque o Rust lê a entrada byte a byte (`u8`), e os caracteres com acento em UTF-8 ocupam **múltiplos bytes**. Quando o `rlolcat` divide o texto por cada byte isolado, os bytes secundários de um caractere UTF-8 acabam por ser tratados incorretamente ou impressos de forma isolada, corrompendo a representação visual (gerando caracteres estranhos).

Para resolver isto, devemos processar o texto iterando por **caracteres UTF-8 válidos (`char`)** em vez de bytes individuais, garantindo que cada letra acentuada seja tratada como uma unidade lógica.

Aqui tem a versão atualizada e corrigida do `src/bin/rlolcat.rs`:

```rust
use std::io::{self, Read, Write};
use std::fs::File;
use std::path::PathBuf;
use crossterm::style::{Color, ResetColor, SetForegroundColor};
use crossterm::QueueableCommand;

fn print_rainbow<R: Read>(mut reader: R) -> io::Result<()> {
    let mut stdout = io::stdout();
    let mut content = String::new();
    reader.read_to_string(&mut content)?;

    let frequency: f64 = 0.1;
    let mut seed: f64 = 0.0;

    for c in content.chars() {
        if c == '\n' {
            stdout.write_all(b"\n")?;
            seed += 0.2; // Desloca o gradiente a cada nova linha
            continue;
        }

        // Cálculo do arco-íris baseado em funções senoidais
        let red = (frequency * seed).sin() * 127.0 + 128.0;
        let green = (frequency * seed + std::f64::consts::PI * 2.0 / 3.0).sin() * 127.0 + 128.0;
        let blue = (frequency * seed + std::f64::consts::PI * 4.0 / 3.0).sin() * 127.0 + 128.0;

        stdout.queue(SetForegroundColor(Color::Rgb {
            r: red as u8,
            g: green as u8,
            b: blue as u8,
        }))?;

        // Escreve o caractere UTF-8 completo no stdout
        write!(stdout, "{}", c)?;
        seed += 1.0;
    }

    stdout.queue(ResetColor)?;
    stdout.flush()?;
    Ok(())
}

fn main() {
    let args: Vec<String> = std::env::args().collect();

    if args.len() > 1 {
        for arg in &args[1..] {
            let path = PathBuf::from(arg);
            if let Ok(file) = File::open(path) {
                let _ = print_rainbow(file);
            } else {
                eprintln!("rlolcat: erro ao abrir o ficheiro: {}", arg);
            }
        }
    } else {
        let stdin = io::stdin();
        let _ = print_rainbow(stdin);
    }
}

```

### O que mudou?

1. **Leitura UTF-8 Segura:** O conteúdo passa a ser lido inteiramente para uma `String` através de `read_to_string`, garantindo que os caracteres multibyte sejam decodificados corretamente.
2. **Iteração por Caracteres (`chars()`):** O ciclo itera sobre `content.chars()` em vez de um buffer cru de `u8`. Assim, cada caractere com acento (`á`, `é`, `ç`, etc.) é processado e colorido de forma íntegra.

Agora pode testar novamente o encadeamento com acentuação perfeita:

```bash
cargo run --bin rcowsay -- "Ferramentas incríveis em Rust!" | cargo run --bin rlolcat

```


Com certeza! Podemos adicionar o `rfortune` ao mesmo projeto, mantendo a estrutura de **multi-binário** que criámos para o `rlolcat` e o `rcowsay`.

O `fortune` clássico lê frases ou citações aleatórias de uma base de dados. Para mantermos o executável **100% autónomo e sem dependências externas de ficheiros**, vamos embutir uma coleção inicial de citações diretamente no código, permitindo também que o programa aceite ficheiros externos personalizados se desejar.

---

### 1. Implementação do `rfortune`

Crie um novo ficheiro chamado `src/bin/rfortune.rs` com o seguinte código:

```rust
use std::fs;
use std::env;
use std::time::{SystemTime, UNIX_EPOCH};

// Citações embutidas por defeito (estilo fortune clássico)
const DEFAULT_FORTUNES: &[&str] = &[
    "O código que funciona hoje é a dívida técnica de amanhã.",
    "Existem dois tipos de linguagens de programação: aquelas de que as pessoas reclamam e aquelas que ninguém usa.",
    "Em Rust confiamos; o borrow checker é o nosso melhor amigo.",
    "Simplicity is prerequisite for reliability. — Edsger W. Dijkstra",
    "A melhor maneira de prever o futuro é inventá-lo. — Alan Kay",
    "Compilação bem-sucedida à primeira vez causa suspeita imediata.",
    "Menos código significa menos bugs.",
];

fn get_random_index(max: usize) -> usize {
    if max == 0 { return 0; }
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .subsec_nanos();
    (nanos as usize) % max
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let mut fortunes: Vec<String> = DEFAULT_FORTUNES.iter().map(|s| s.to_string()).collect();

    // Se for passado um ficheiro de fortunes como argumento, lê-o (separado por '%')
    if args.len() > 1 {
        let path = &args[1];
        if let Ok(content) = fs::read_to_string(path) {
            fortunes = content
                .split('%')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect();
        } else {
            eprintln!("rfortune: não foi possível ler o ficheiro '{}'", path);
        }
    }

    if fortunes.is_empty() {
        println!("Nenhuma citação encontrada.");
        return;
    }

    let index = get_random_index(fortunes.len());
    println!("{}", fortunes[index]);
}

```

---

### 2. Como Executar e Testar

Agora tem três ferramentas poderosas no mesmo projeto Rust (`rlolcat`, `rcowsay` e `rfortune`). Pode executá-las individualmente:

* **Gerar uma citação aleatória:**
```bash
cargo run --bin rfortune

```



---

### 3. A Magia do Pipe Unix com as Três Ferramentas

Uma das grandes vantagens desta arquitetura é poder encadear os três programas através do operador pipe (`|`), criando uma experiência totalmente integrada ao estilo Unix:

```bash
cargo run --bin rfortune | cargo run --bin rcowsay | cargo run --bin rlolcat

```

### 4. Compilação Final para Produção

Para compilar todas as ferramentas otimizadas de uma só vez para a sua máquina:

```bash
cargo build --release

```

Os três executáveis autónomos (`rlolcat`, `rcowsay` e `rfortune`) ficarão prontos e disponíveis na pasta `target/release/`.


Com certeza! Podemos adicionar o `rboxes` ao nosso projeto multi-binário em Rust, inspirado no clássico utilitário Unix **`boxes`**, que desenha caixas artísticas e molduras em volta de texto.

Para manter o programa autónomo, portátil e compatível com carateres UTF-8 (incluindo acentuação em Português), criámos uma implementação flexível que suporta múltiplos estilos de caixas.

---

### 1. Implementação do `rboxes`

Crie um novo ficheiro chamado `src/bin/rboxes.rs` com o seguinte código:

```rust
use std::io::{self, Read};

// Definição de um estilo de caixa (cantos, lados e bordas horizontais)
struct BoxStyle {
    tl: &'static str,
    tr: &'static str,
    bl: &'static str,
    br: &'static str,
    h: &'static str,
    v: &'static str,
}

const STYLES: &[(&str, BoxStyle)] = &[
    ("classic", BoxStyle { tl: "+", tr: "+", bl: "+", br: "+", h: "-", v: "|" }),
    ("round", BoxStyle { tl: "╭", tr: "╮", bl: "╰", br: "╯", h: "─", v: "│" }),
    ("double", BoxStyle { tl: "╔", tr: "╗", bl: "╚", br: "╝", h: "═", v: "║" }),
    ("bold", BoxStyle { tl: "┏", tr: "┓", bl: "┗", br: "┛", h: "━", v: "┃" }),
    ("markdown", BoxStyle { tl: "```", tr: "```", bl: "```", br: "```", h: "", v: "" }),
];

fn print_boxed(text: &str, style_name: &str) {
    let lines: Vec<&str> = text.lines().collect();
    if lines.is_empty() { return; }

    // Encontra o comprimento máximo da linha (contando carateres Unicode de forma segura)
    let max_len = lines.iter().map(|l| l.chars().count()).max().unwrap_or(0);

    let style = STYLES.iter()
        .find(|s| s.0 == style_name)
        .map(|s| &s.1)
        .unwrap_or(&STYLES[1].1); // Estilo "round" por defeito

    if style_name == "markdown" {
        println!("{}", style.tl);
        for line in &lines {
            println!("{}", line);
        }
        println!("{}", style.br);
        return;
    }

    let top_bottom_h = style.h.repeat(max_len + 2);
    println!("{}{}{}", style.tl, top_bottom_h, style.tr);

    for line in lines {
        let padding = " ".repeat(max_len - line.chars().count());
        println!("{} {}{} {}", style.v, line, padding, style.v);
    }

    println!("{}{}{}", style.bl, top_bottom_h, style.br);
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let mut style_name = "round";

    // Analisa argumentos simples: ex: rboxes round
    if args.len() > 1 {
        style_name = &args[1];
    }

    let mut input = String::new();
    if let Ok(_) = io::stdin().read_to_string(&mut input) {
        let cleaned = input.trim_end_matches('\n').trim_end_matches('\r');
        if !cleaned.is_empty() {
            print_boxed(cleaned, style_name);
            return;
        }
    }

    // Mensagem de fallback se o stdin estiver vazio
    print_boxed("Olá do rboxes em Rust!", style_name);
}

```

---

### 2. Estilos Disponíveis no `rboxes`

O programa suporta vários estilos que podem ser passados como argumento:

* `round` (Cantos arredondados, por defeito)
* `classic` (Estilo ASCII tradicional `+`, `-`, `|`)
* `double` (Linha dupla `╔`, `═`, `║`)
* `bold` (Linha grossa `┏`, `━`, `┃`)
* `markdown` (Envolve em blocos de código markdown)

---

### 3. Exemplos de Execução e Integração Unix (Pipe)

Pode testar individualmente escolhendo o estilo:

```bash
echo "Ferramentas incríveis em Rust!" | cargo run --bin rboxes -- double

```

Ou, levando a sinergia ao limite, juntar as quatro ferramentas (`rfortune`, `rcowsay`, `rboxes` e `rlolcat`) numa única linha de comandos espetacular:

```bash
cargo run --bin rfortune | cargo run --bin rcowsay | cargo run --bin rboxes -- round | cargo run --bin rlolcat

```

### 4. Compilação Final

Para compilar todos os utilitários de produção em simultâneo:

```bash
cargo build --release

```

Terá agora os quatro executáveis totalmente portáteis (`rlolcat`, `rcowsay`, `rfortune` e `rboxes`) prontos na pasta `target/release/`.


Para resolver o problema dos caracteres acentuados no `rboxes` (semelhante ao que corrigimos no `rlolcat`), precisamos de garantir que o programa processa o texto iterando por **caracteres UTF-8 válidos (`chars()`)** e medindo corretamente a largura visual de cada linha com `.chars().count()`, em vez de contar bytes crus.

Aqui tem a versão atualizada e corrigida de `src/bin/rboxes.rs`:

```rust
use std::io::{self, Read};

// Definição de um estilo de caixa (cantos, lados e bordas horizontais)
struct BoxStyle {
    tl: &'static str,
    tr: &'static str,
    bl: &'static str,
    br: &'static str,
    h: &'static str,
    v: &'static str,
}

const STYLES: &[(&str, BoxStyle)] = &[
    ("classic", BoxStyle { tl: "+", tr: "+", bl: "+", br: "+", h: "-", v: "|" }),
    ("round", BoxStyle { tl: "╭", tr: "╮", bl: "╰", br: "╯", h: "─", v: "│" }),
    ("double", BoxStyle { tl: "╔", tr: "╗", bl: "╚", br: "╝", h: "═", v: "║" }),
    ("bold", BoxStyle { tl: "┏", tr: "┓", bl: "┗", br: "┛", h: "━", v: "┃" }),
    ("markdown", BoxStyle { tl: "```", tr: "```", bl: "```", br: "```", h: "", v: "" }),
];

fn print_boxed(text: &str, style_name: &str) {
    let lines: Vec<&str> = text.lines().collect();
    if lines.is_empty() { return; }

    // Encontra o comprimento máximo da linha contando corretamente os caracteres Unicode
    let max_len = lines.iter().map(|l| l.chars().count()).max().unwrap_or(0);

    let style = STYLES.iter()
        .find(|s| s.0 == style_name)
        .map(|s| &s.1)
        .unwrap_or(&STYLES[1].1); // Estilo "round" por defeito

    if style_name == "markdown" {
        println!("{}", style.tl);
        for line in &lines {
            println!("{}", line);
        }
        println!("{}", style.br);
        return;
    }

    // Cria a barra horizontal repetindo o caractere o número correto de vezes por caractere UTF-8
    let top_bottom_h: String = style.h.repeat(max_len + 2);
    println!("{}{}{}", style.tl, top_bottom_h, style.tr);

    for line in lines {
        let line_chars_count = line.chars().count();
        let padding_count = if max_len > line_chars_count { max_len - line_chars_count } else { 0 };
        let padding = " ".repeat(padding_count);
        println!("{} {}{} {}", style.v, line, padding, style.v);
    }

    println!("{}{}{}", style.bl, top_bottom_h, style.br);
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let mut style_name = "round";

    if args.len() > 1 {
        style_name = &args[1];
    }

    let mut input = String::new();
    if let Ok(_) = io::stdin().read_to_string(&mut input) {
        let cleaned = input.trim_end_matches('\n').trim_end_matches('\r');
        if !cleaned.is_empty() {
            print_boxed(cleaned, style_name);
            return;
        }
    }

    print_boxed("Olá do rboxes em Rust!", style_name);
}

```

### O que foi corrigido?

1. **Leitura UTF-8 Integrada:** O `read_to_string` assegura que os fluxos de entrada (`stdin`) que contêm acentos (como "á", "é", "ç", "ã") são interpretados de forma limpa como strings UTF-8 válidas.
2. **Alinhamento de Largura com `chars().count()`:** As linhas agora calculam o preenchimento lateral (`padding`) com base no número exato de caracteres visíveis e não no número de bytes, evitando desalinhamentos nas bordas direitas da caixa quando existem letras acentuadas.

Pode testar novamente a cadeia completa sem preocupações com a acentuação:

```bash
cargo run --bin rfortune | cargo run --bin rcowsay | cargo run --bin rboxes -- round | cargo run --bin rlolcat

```


O problema que aconteceu é que, ao utilizar o operador **`|` (pipe)** no Windows (especificamente na PowerShell ou CMD), o fluxo de entrada (`stdin`) nem sempre é lido corretamente da forma como o fizermos no `io::stdin().read_to_string()`, fazendo com que o programa ignore o pipe e caia na mensagem padrão (*fallback*) "Olá do rboxes em Rust!".

Além disso, a gestão de codificação de texto (`UTF-8` vs `CP1252`/`UTF-16`) no Windows ao canalizar dados via terminal pode por vezes descartar o *stdin*.

Para corrigir isto de forma robusta e garantir que o `rboxes` (e também os outros programas, caso queira aplicar o mesmo padrão) consiga ler corretamente tanto argumentos passados diretamente como dados vindos de um pipe no Windows ou Linux, vamos atualizar o `src/bin/rboxes.rs`.

### Versão Corrigida e Robusta de `src/bin/rboxes.rs`

Substitua o conteúdo de `src/bin/rboxes.rs` por este:

```rust
use std::io::{self, Read, IsTerminal};

struct BoxStyle {
    tl: &'static str,
    tr: &'static str,
    bl: &'static str,
    br: &'static str,
    h: &'static str,
    v: &'static str,
}

const STYLES: &[(&str, BoxStyle)] = &[
    ("classic", BoxStyle { tl: "+", tr: "+", bl: "+", br: "+", h: "-", v: "|" }),
    ("round", BoxStyle { tl: "╭", tr: "╮", bl: "╰", br: "╯", h: "─", v: "│" }),
    ("double", BoxStyle { tl: "╔", tr: "╗", bl: "╚", br: "╝", h: "═", v: "║" }),
    ("bold", BoxStyle { tl: "┏", tr: "┓", bl: "┗", br: "┛", h: "━", v: "┃" }),
    ("markdown", BoxStyle { tl: "```", tr: "```", bl: "```", br: "```", h: "", v: "" }),
];

fn print_boxed(text: &str, style_name: &str) {
    let lines: Vec<&str> = text.lines().collect();
    if lines.is_empty() { return; }

    let max_len = lines.iter().map(|l| l.chars().count()).max().unwrap_or(0);

    let style = STYLES.iter()
        .find(|s| s.0 == style_name)
        .map(|s| &s.1)
        .unwrap_or(&STYLES[1].1);

    if style_name == "markdown" {
        println!("{}", style.tl);
        for line in &lines {
            println!("{}", line);
        }
        println!("{}", style.br);
        return;
    }

    let top_bottom_h: String = style.h.repeat(max_len + 2);
    println!("{}{}{}", style.tl, top_bottom_h, style.tr);

    for line in lines {
        let line_chars_count = line.chars().count();
        let padding_count = if max_len > line_chars_count { max_len - line_chars_count } else { 0 };
        let padding = " ".repeat(padding_count);
        println!("{} {}{} {}", style.v, line, padding, style.v);
    }

    println!("{}{}{}", style.bl, top_bottom_h, style.br);
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let mut style_name = "round";
    
    // Procura por um argumento que seja o estilo (ex: double, classic, etc.)
    // Se houver argumentos além do nome do binário, o último ou o primeiro que coincida com um estilo é usado.
    for arg in &args[1..] {
        if STYLES.iter().any(|s| s.0 == arg) {
            style_name = arg;
            break;
        }
    }

    let mut input = String::new();
    
    // Verifica se há dados a chegar pelo stdin (pipe)
    // Usamos io::stdin().is_terminal() para saber se o terminal está à espera de input via pipe.
    let stdin = io::stdin();
    let has_pipe = !stdin.is_terminal();

    if has_pipe {
        if let Ok(_) = stdin.lock().read_to_string(&mut input) {
            let cleaned = input.trim_end_matches('\n').trim_end_matches('\r');
            if !cleaned.is_empty() {
                print_boxed(cleaned, style_name);
                return;
            }
        }
    }

    // Se não houver pipe, tenta usar o argumento textual se houver algum que não seja o estilo
    let text_args: Vec<String> = args[1..].iter().cloned().filter(|a| a != style_name).collect();
    if !text_args.is_empty() {
        print_boxed(&text_args.join(" "), style_name);
        return;
    }

    // Fallback se nada for fornecido
    print_boxed("Olá do rboxes em Rust!", style_name);
}

```

### O que mudou?

1. **Deteção de Pipe Inteligente (`is_terminal`):** O programa agora deteta de forma fiável se está a receber dados através de um pipe (`|`) ou se foi executado diretamente no terminal.
2. **Parsing de Argumentos Flexível:** Agora pode passar o estilo tanto antes como depois do texto (ex: `cargo run --bin rboxes -- double` ou `cargo run --bin rboxes -- "Texto" double`).

### Como testar agora:

Se estiver no Windows (PowerShell), por vezes a codificação padrão da shell interfere com acentos em pipes. Para garantir que o PowerShell passa UTF-8 corretamente para o pipe, pode correr antes:

```powershell
[Console]::OutputEncoding = [System.Text.Encoding]::UTF8

```

E depois executar:

```bash
echo "Ferramentas incríveis em Rust!" | cargo run --bin rboxes -- double

```


Excelente desafio! O **`cbonsai`** é uma das ferramentas de arte ASCII mais fascinantes do ecossistema Unix, gerando árvores de bonsai procedurais e coloridas na consola.

Para manter o nosso projeto unificado, portátil, sem dependências externas e totalmente compatível com Windows, Linux e Mac, vamos criar o **`rbonsai`** como o quinto binário do nosso projeto Rust.

---

### 1. Implementação do `rbonsai`

Crie um novo ficheiro chamado `src/bin/rbonsai.rs` com o seguinte código (utiliza a biblioteca `crossterm` que já temos no projeto para gerir cores e posicionamento no terminal de forma totalmente agnóstica à plataforma):

```rust
use std::io::{self, Write};
use std::time::SystemTime, UNIX_EPOCH;
use crossterm::{
    cursor,
    style::{Color, Print, ResetColor, SetForegroundColor},
    terminal::{Clear, ClearType},
    QueueableCommand,
};

// Gerador de números pseudo-aleatórios simples (LCG) para portabilidade sem dependências pesadas
struct SimpleRng {
    state: u64,
}

impl SimpleRng {
    fn new() -> Self {
        let seed = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos() as u64;
        Self { state: seed }
    }

    fn next_u32(&mut self) -> u32 {
        self.state ^= self.state << 13;
        self.state ^= self.state >> 7;
        self.state ^= self.state << 17;
        self.state as u32
    }

    fn gen_range(&mut self, min: u32, max: u32) -> u32 {
        if min >= max { return min; }
        min + (self.next_u32() % (max - min))
    }
}

fn draw_bonsai() -> io::Result<()> {
    let mut stdout = io::stdout();
    let mut rng = SimpleRng::new();

    stdout.queue(Clear(ClearType::All))?;
    stdout.queue(cursor::MoveTo(0, 0))?;

    // Ponto de partida do tronco
    let mut x = 40;
    let mut y = 20;

    stdout.queue(cursor::MoveTo(x, y))?;
    stdout.queue(SetForegroundColor(Color::DarkYellow))?;
    stdout.queue(Print("&"))?; // Base do tronco

    // Crescimento procedural simplificado de ramos e folhagem
    for _ in 0..45 {
        let dx = (rng.gen_range(0, 3) as i32) - 1; // -1, 0 ou 1
        let dy = -(rng.gen_range(1, 3) as i32);    // sobe 1 ou 2 linhas

        x = ((x as i32) + dx).clamp(5, 75) as u16;
        y = ((y as i32) + dy).clamp(2, 18) as u16;

        stdout.queue(cursor::MoveTo(x, y))?;
        
        if y < 8 {
            // Folhagem verde no topo
            let leaf_chars = ["&", "*", "o", "º"];
            let idx = (rng.gen_range(0, leaf_chars.len() as u32)) as usize;
            stdout.queue(SetForegroundColor(Color::Green))?;
            stdout.queue(Print(leaf_chars[idx]))?;
        } else {
            // Tronco e ramos secundários
            stdout.queue(SetForegroundColor(Color::DarkYellow))?;
            stdout.queue(Print("/"))?;
        }
    }

    // Desenhar a base / vaso do bonsai na parte inferior
    stdout.queue(cursor::MoveTo(30, 21))?;
    stdout.queue(SetForegroundColor(Color::Grey))?;
    stdout.queue(Print("_---~~~~~~~~~~~~~~~~---------___"))?;
    stdout.queue(cursor::MoveTo(28, 22))?;
    stdout.queue(Print(" \\                             /"))?;
    stdout.queue(cursor::MoveTo(27, 23))?;
    stdout.queue(Print("  \\___________________________/"))?;

    stdout.queue(ResetColor)?;
    stdout.queue(cursor::MoveTo(0, 25))?;
    stdout.flush()?;
    Ok(())
}

fn main() {
    if let Err(e) = draw_bonsai() {
        eprintln!("Erro ao desenhar o rbonsai: {}", e);
    }
}

```

---

### 2. Como Executar

Agora tem 5 ferramentas lendárias do ecossistema Unix inteiramente reescritas em Rust dentro do mesmo projeto (`rlolcat`, `rcowsay`, `rfortune`, `rboxes` e `rbonsai`):

Para testar o seu novo gerador de bonsai:

```bash
cargo run --bin rbonsai

```

### 3. A Linha de Comandos Definitiva (Pipe em Cadeia)

Pode inclusive canalizar a saída do `rbonsai` para o `rlolcat` se quiser colorir o seu bonsai dinamicamente com gradientes do arco-íris:

```bash
cargo run --bin rbonsai | cargo run --bin rlolcat

```

### 4. Compilação Final para Produção

Para compilar todas as ferramentas de uma só vez em modo otimizado e gerar os executáveis portáteis autónomos:

```bash
cargo build --release

```

Os executáveis finais estarão prontos na pasta `target/release/` para qualquer sistema operativo (`Windows`, `Linux` ou `macOS`).


Foi um pequeno erro de sintaxe na importação do módulo de tempo (`std::time`). No Rust, quando importamos múltiplos itens do mesmo sub-caminho, devemos envolvê-los entre chavetas `{}` em vez de usar uma vírgula direta.

Substitua o conteúdo de `src/bin/rbonsai.rs` pelo seguinte código corrigido:

```rust
use std::io::{self, Write};
use std::time::{SystemTime, UNIX_EPOCH};
use crossterm::{
    cursor,
    style::{Color, Print, ResetColor, SetForegroundColor},
    terminal::{Clear, ClearType},
    QueueableCommand,
};

// Gerador de números pseudo-aleatórios simples (LCG) para portabilidade sem dependências pesadas
struct SimpleRng {
    state: u64,
}

impl SimpleRng {
    fn new() -> Self {
        let seed = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos() as u64;
        Self { state: seed }
    }

    fn next_u32(&mut self) -> u32 {
        self.state ^= self.state << 13;
        self.state ^= self.state >> 7;
        self.state ^= self.state << 17;
        self.state as u32
    }

    fn gen_range(&mut self, min: u32, max: u32) -> u32 {
        if min >= max { return min; }
        min + (self.next_u32() % (max - min))
    }
}

fn draw_bonsai() -> io::Result<()> {
    let mut stdout = io::stdout();
    let mut rng = SimpleRng::new();

    stdout.queue(Clear(ClearType::All))?;
    stdout.queue(cursor::MoveTo(0, 0))?;

    // Ponto de partida do tronco
    let mut x = 40;
    let mut y = 20;

    stdout.queue(cursor::MoveTo(x, y))?;
    stdout.queue(SetForegroundColor(Color::DarkYellow))?;
    stdout.queue(Print("&"))?; // Base do tronco

    // Crescimento procedural simplificado de ramos e folhagem
    for _ in 0..45 {
        let dx = (rng.gen_range(0, 3) as i32) - 1; // -1, 0 ou 1
        let dy = -(rng.gen_range(1, 3) as i32);    // sobe 1 ou 2 linhas

        x = ((x as i32) + dx).clamp(5, 75) as u16;
        y = ((y as i32) + dy).clamp(2, 18) as u16;

        stdout.queue(cursor::MoveTo(x, y))?;
        
        if y < 8 {
            // Folhagem verde no topo
            let leaf_chars = ["&", "*", "o", "º"];
            let idx = (rng.gen_range(0, leaf_chars.len() as u32)) as usize;
            stdout.queue(SetForegroundColor(Color::Green))?;
            stdout.queue(Print(leaf_chars[idx]))?;
        } else {
            // Tronco e ramos secundários
            stdout.queue(SetForegroundColor(Color::DarkYellow))?;
            stdout.queue(Print("/"))?;
        }
    }

    // Desenhar a base / vaso do bonsai na parte inferior
    stdout.queue(cursor::MoveTo(30, 21))?;
    stdout.queue(SetForegroundColor(Color::Grey))?;
    stdout.queue(Print("_---~~~~~~~~~~~~~~~~---------___"))?;
    stdout.queue(cursor::MoveTo(28, 22))?;
    stdout.queue(Print(" \\                             /"))?;
    stdout.queue(cursor::MoveTo(27, 23))?;
    stdout.queue(Print("  \\___________________________/"))?;

    stdout.queue(ResetColor)?;
    stdout.queue(cursor::MoveTo(0, 25))?;
    stdout.flush()?;
    Ok(())
}

fn main() {
    if let Err(e) = draw_bonsai() {
        eprintln!("Erro ao desenhar o rbonsai: {}", e);
    }
}

```

### Como testar agora:

```bash
cargo run --bin rbonsai

```


Tem toda a razão! A versão anterior estava a desenhar os caracteres diretamente com um passo linear vertical e sem expansão lateral em grelha, o que resultava num tronco reto e sem copa orgânica.

Para resolver isto e obter um aspeto verdadeiramente semelhante ao `cbonsai` original — com ramos ramificados para os lados, copas de folhagem densas e um vaso estruturado na base —, vamos mudar a abordagem para um **sistema baseado em matriz/grelha virtual em memória**. O programa desenha a árvore completa numa grelha bidimensional e depois imprime-a colorida no terminal de uma só vez.

Substitua o conteúdo de `src/bin/rbonsai.rs` pelo seguinte código melhorado:

```rust
use std::io::{self, Write};
use std::time::{SystemTime, UNIX_EPOCH};
use crossterm::{
    cursor,
    style::{Color, Print, ResetColor, SetForegroundColor},
    terminal::{Clear, ClearType},
    QueueableCommand,
};

struct SimpleRng {
    state: u64,
}

impl SimpleRng {
    fn new() -> Self {
        let seed = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos() as u64;
        Self { state: seed }
    }

    fn next_u32(&mut self) -> u32 {
        self.state ^= self.state << 13;
        self.state ^= self.state >> 7;
        self.state ^= self.state << 17;
        self.state as u32
    }

    fn gen_range(&mut self, min: u32, max: u32) -> u32 {
        if min >= max { return min; }
        min + (self.next_u32() % (max - min))
    }
}

fn draw_bonsai() -> io::Result<()> {
    let mut stdout = io::stdout();
    let mut rng = SimpleRng::new();

    const WIDTH: usize = 80;
    const HEIGHT: usize = 24;
    let mut grid = vec![vec![' '; WIDTH]; HEIGHT];

    // Posição inicial do tronco no vaso
    let mut x = WIDTH as i32 / 2;
    let mut y = HEIGHT as i32 - 5;

    // Desenhar o tronco principal com alguma sinuosidade para cima
    for _ in 0..9 {
        if y >= 0 && (0..WIDTH as i32).contains(&x) {
            grid[y as usize][x as usize] = '|';
        }
        y -= 1;
        if rng.gen_range(0, 2) == 0 {
            x += (rng.gen_range(0, 3) as i32) - 1;
        }
    }

    // Gerar ramos secundários e espalhar folhagem lateral
    let branch_count = 7;
    for _ in 0..branch_count {
        let mut bx = x;
        let mut by = y + rng.gen_range(1, 4) as i32;
        let dir = if rng.gen_range(0, 2) == 0 { 1 } else { -1 };
        let branch_length = rng.gen_range(6, 14);

        for step in 0..branch_length {
            bx += dir * (rng.gen_range(0, 2) as i32 + 1);
            by -= rng.gen_range(0, 2) as i32;

            let ux = bx.clamp(2, WIDTH as i32 - 3) as usize;
            let uy = by.clamp(2, HEIGHT as i32 - 7) as usize;

            if step > 2 {
                // Criar nuvens de folhas nas pontas dos ramos
                for dx in -2..=2 {
                    for dy in -1..=1 {
                        let fx = (ux as i32 + dx).clamp(1, WIDTH as i32 - 2) as usize;
                        let fy = (uy as i32 + dy).clamp(1, HEIGHT as i32 - 7) as usize;
                        let leaves = ['&', '*', 'o', '•', 'º'];
                        let l_idx = rng.gen_range(0, leaves.len() as u32) as usize;
                        if grid[fy][fx] == ' ' {
                            grid[fy][fx] = leaves[l_idx];
                        }
                    }
                }
            } else {
                grid[uy][ux] = if dir > 0 { '/' } else { '\\' };
            }
        }
    }

    // Copa de folhagem principal no topo
    for dx in -5..=5 {
        for dy in -2..=1 {
            let fx = (x as i32 + dx).clamp(1, WIDTH as i32 - 2) as usize;
            let fy = (y as i32 + dy).clamp(1, HEIGHT as i32 - 7) as usize;
            if grid[fy][fx] == ' ' {
                let leaves = ['*', '&', 'o'];
                let l_idx = rng.gen_range(0, leaves.len() as u32) as usize;
                grid[fy][fx] = leaves[l_idx];
            }
        }
    }

    // Desenhar o vaso de bonsai na parte inferior
    let pot_y = HEIGHT - 4;
    let pot_start = WIDTH / 2 - 16;
    
    let pot_lines = [
        "    ___---~~~~~~~~~~~~~~~~---------___    ",
        "   /                                  \\   ",
        "  /____________________________________\\  ",
        "      ||                          ||      "
    ];

    for (row_idx, line) in pot_lines.iter().enumerate() {
        for (col_idx, c) in line.chars().enumerate() {
            let target_x = pot_start + col_idx;
            let target_y = pot_y + row_idx;
            if target_x < WIDTH && target_y < HEIGHT {
                grid[target_y][target_x] = c;
            }
        }
    }

    // Renderizar a grelha completa no terminal com cores apropriadas
    stdout.queue(Clear(ClearType::All))?;
    stdout.queue(cursor::MoveTo(0, 0))?;

    for row in grid {
        for c in row {
            match c {
                '*' | '&' | 'o' | '•' | 'º' => {
                    stdout.queue(SetForegroundColor(Color::Green))?;
                }
                '|' | '/' | '\\' => {
                    stdout.queue(SetForegroundColor(Color::DarkYellow))?;
                }
                '_' | '~' | '(' | ')' | '[' | ']' => {
                    stdout.queue(SetForegroundColor(Color::Grey))?;
                }
                _ => {
                    stdout.queue(SetForegroundColor(Color::Reset))?;
                }
            }
            stdout.queue(Print(c))?;
        }
        stdout.queue(Print("\n"))?;
    }

    stdout.queue(ResetColor)?;
    stdout.flush()?;
    Ok(())
}

fn main() {
    if let Err(e) = draw_bonsai() {
        eprintln!("Erro ao desenhar o rbonsai: {}", e);
    }
}

```

### Como testar agora:

```bash
cargo run --bin rbonsai

```

Desta forma, o programa gera uma estrutura orgânica em 2D com ramos expansivos, copa detalhada e um vaso elegante na base. Pode voltar a testar também a combinação com o `rlolcat`:

```bash
cargo run --bin rbonsai | cargo run --bin rlolcat

```


Tem toda a razão! A versão anterior estava a desenhar os caracteres diretamente com um passo linear vertical e sem expansão lateral em grelha, o que resultava num tronco reto e sem copa orgânica.

Para resolver isto e obter um aspeto verdadeiramente semelhante ao `cbonsai` original — com ramos ramificados para os lados, copas de folhagem densas e um vaso estruturado na base —, vamos mudar a abordagem para um **sistema baseado em matriz/grelha virtual em memória**. O programa desenha a árvore completa numa grelha bidimensional e depois imprime-a colorida no terminal de uma só vez.

Substitua o conteúdo de `src/bin/rbonsai.rs` pelo seguinte código melhorado:

```rust
use std::io::{self, Write};
use std::time::{SystemTime, UNIX_EPOCH};
use crossterm::{
    cursor,
    style::{Color, Print, ResetColor, SetForegroundColor},
    terminal::{Clear, ClearType},
    QueueableCommand,
};

struct SimpleRng {
    state: u64,
}

impl SimpleRng {
    fn new() -> Self {
        let seed = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos() as u64;
        Self { state: seed }
    }

    fn next_u32(&mut self) -> u32 {
        self.state ^= self.state << 13;
        self.state ^= self.state >> 7;
        self.state ^= self.state << 17;
        self.state as u32
    }

    fn gen_range(&mut self, min: u32, max: u32) -> u32 {
        if min >= max { return min; }
        min + (self.next_u32() % (max - min))
    }
}

fn draw_bonsai() -> io::Result<()> {
    let mut stdout = io::stdout();
    let mut rng = SimpleRng::new();

    const WIDTH: usize = 80;
    const HEIGHT: usize = 24;
    let mut grid = vec![vec![' '; WIDTH]; HEIGHT];

    // Posição inicial do tronco no vaso
    let mut x = WIDTH as i32 / 2;
    let mut y = HEIGHT as i32 - 5;

    // Desenhar o tronco principal com alguma sinuosidade para cima
    for _ in 0..9 {
        if y >= 0 && (0..WIDTH as i32).contains(&x) {
            grid[y as usize][x as usize] = '|';
        }
        y -= 1;
        if rng.gen_range(0, 2) == 0 {
            x += (rng.gen_range(0, 3) as i32) - 1;
        }
    }

    // Gerar ramos secundários e espalhar folhagem lateral
    let branch_count = 7;
    for _ in 0..branch_count {
        let mut bx = x;
        let mut by = y + rng.gen_range(1, 4) as i32;
        let dir = if rng.gen_range(0, 2) == 0 { 1 } else { -1 };
        let branch_length = rng.gen_range(6, 14);

        for step in 0..branch_length {
            bx += dir * (rng.gen_range(0, 2) as i32 + 1);
            by -= rng.gen_range(0, 2) as i32;

            let ux = bx.clamp(2, WIDTH as i32 - 3) as usize;
            let uy = by.clamp(2, HEIGHT as i32 - 7) as usize;

            if step > 2 {
                // Criar nuvens de folhas nas pontas dos ramos
                for dx in -2..=2 {
                    for dy in -1..=1 {
                        let fx = (ux as i32 + dx).clamp(1, WIDTH as i32 - 2) as usize;
                        let fy = (uy as i32 + dy).clamp(1, HEIGHT as i32 - 7) as usize;
                        let leaves = ['&', '*', 'o', '•', 'º'];
                        let l_idx = rng.gen_range(0, leaves.len() as u32) as usize;
                        if grid[fy][fx] == ' ' {
                            grid[fy][fx] = leaves[l_idx];
                        }
                    }
                }
            } else {
                grid[uy][ux] = if dir > 0 { '/' } else { '\\' };
            }
        }
    }

    // Copa de folhagem principal no topo
    for dx in -5..=5 {
        for dy in -2..=1 {
            let fx = (x as i32 + dx).clamp(1, WIDTH as i32 - 2) as usize;
            let fy = (y as i32 + dy).clamp(1, HEIGHT as i32 - 7) as usize;
            if grid[fy][fx] == ' ' {
                let leaves = ['*', '&', 'o'];
                let l_idx = rng.gen_range(0, leaves.len() as u32) as usize;
                grid[fy][fx] = leaves[l_idx];
            }
        }
    }

    // Desenhar o vaso de bonsai na parte inferior
    let pot_y = HEIGHT - 4;
    let pot_start = WIDTH / 2 - 16;
    
    let pot_lines = [
        "    ___---~~~~~~~~~~~~~~~~---------___    ",
        "   /                                  \\   ",
        "  /____________________________________\\  ",
        "      ||                          ||      "
    ];

    for (row_idx, line) in pot_lines.iter().enumerate() {
        for (col_idx, c) in line.chars().enumerate() {
            let target_x = pot_start + col_idx;
            let target_y = pot_y + row_idx;
            if target_x < WIDTH && target_y < HEIGHT {
                grid[target_y][target_x] = c;
            }
        }
    }

    // Renderizar a grelha completa no terminal com cores apropriadas
    stdout.queue(Clear(ClearType::All))?;
    stdout.queue(cursor::MoveTo(0, 0))?;

    for row in grid {
        for c in row {
            match c {
                '*' | '&' | 'o' | '•' | 'º' => {
                    stdout.queue(SetForegroundColor(Color::Green))?;
                }
                '|' | '/' | '\\' => {
                    stdout.queue(SetForegroundColor(Color::DarkYellow))?;
                }
                '_' | '~' | '(' | ')' | '[' | ']' => {
                    stdout.queue(SetForegroundColor(Color::Grey))?;
                }
                _ => {
                    stdout.queue(SetForegroundColor(Color::Reset))?;
                }
            }
            stdout.queue(Print(c))?;
        }
        stdout.queue(Print("\n"))?;
    }

    stdout.queue(ResetColor)?;
    stdout.flush()?;
    Ok(())
}

fn main() {
    if let Err(e) = draw_bonsai() {
        eprintln!("Erro ao desenhar o rbonsai: {}", e);
    }
}

```

### Como testar agora:

```bash
cargo run --bin rbonsai

```

Desta forma, o programa gera uma estrutura orgânica em 2D com ramos expansivos, copa detalhada e um vaso elegante na base. Pode voltar a testar também a combinação com o `rlolcat`:

```bash
cargo run --bin rbonsai | cargo run --bin rlolcat

```

Excelente desafio! O **`cmatrix`** é um dos clássicos absolutos do terminal Unix, simulando a famosa chuva de códigos verde-escuro do filme *Matrix*.

Para manter o nosso projeto unificado, **100% autónomo, portátil e sem dependências externas**, vamos adicionar o **`rmatrix`** como o sexto binário.

Como o `cmatrix` é uma animação em tempo real que redesenha o ecrã continuamente, utilizaremos a biblioteca `crossterm` para gerir o ecrã alternativo, ler teclas sem bloqueio (para permitir sair ao premir `q` ou `Ctrl+C`) e atualizar os caracteres fluidamente.

---

### 1. Implementação do `rmatrix`

Crie um novo ficheiro chamado `src/bin/rmatrix.rs` com o seguinte código:

```rust
use std::io::{self, Write};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use crossterm::{
    cursor,
    event::{self, Event, KeyCode},
    style::{Color, Print, ResetColor, SetForegroundColor},
    terminal::{self, Clear, ClearType},
    ExecutableCommand, QueueableCommand,
};

struct SimpleRng {
    state: u64,
}

impl SimpleRng {
    fn new() -> Self {
        let seed = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_nanos() as u64;
        Self { state: seed }
    }

    fn next_u32(&mut self) -> u32 {
        self.state ^= self.state << 13;
        self.state ^= self.state >> 7;
        self.state ^= self.state << 17;
        self.state as u32
    }

    fn gen_range(&mut self, min: u32, max: u32) -> u32 {
        if min >= max { return min; }
        min + (self.next_u32() % (max - min))
    }
}

// Representa uma coluna de pingos de chuva digital
struct Drop {
    y: i32,
    speed: i32,
    length: i32,
}

fn main() -> io::Result<()> {
    let mut stdout = io::stdout();
    let mut rng = SimpleRng::new();

    // Entrar no modo de ecrã alternativo e esconder o cursor
    terminal::enable_raw_mode()?;
    stdout.execute(terminal::EnterAlternateScreen)?;
    stdout.execute(cursor::Hide)?;
    stdout.execute(Clear(ClearType::All))?;

    let (width, height) = terminal::size()?;
    let mut drops: Vec<Drop> = (0..width).map(|_| {
        Drop {
            y: -(rng.gen_range(0, height as u32) as i32),
            speed: rng.gen_range(1, 3) as i32,
            length: rng.gen_range(5, (height / 2) as u32) as i32,
        }
    }).collect();

    let target_fps = Duration::from_millis(50); // ~20 FPS

    loop {
        // Verificar se o utilizador premiu 'q' para sair
        if event::poll(Duration::from_millis(1))? {
            if let Event::Key(key) = event::read()? {
                if key.code == KeyCode::Char('q') || key.code == KeyCode::Esc {
                    break;
                }
            }
        }

        let frame_start = Instant::now();
        let (w, h) = terminal::size()?;

        // Se o tamanho do terminal mudou, ajusta o vetor de gotas
        if drops.len() != w as usize {
            drops.resize_with(w as usize, || {
                Drop {
                    y: -(rng.gen_range(0, h as u32) as i32),
                    speed: rng.gen_range(1, 3) as i32,
                    length: rng.gen_range(5, (h / 2) as u32) as i32,
                }
            });
        }

        stdout.queue(Clear(ClearType::All))?;

        for (x, drop) in drops.iter_mut().enumerate() {
            drop.y += drop.speed;

            // Desenhar a cauda da gota (caracteres verdes escuros)
            for i in 0..drop.length {
                let current_y = drop.y - i;
                if current_y >= 0 && current_y < h as i32 {
                    // Escolher um caractere aleatório (alfanumérico / japonês estilizado em ASCII)
                    let chars = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789@#$%*<>~+-/|";
                    let char_idx = rng.gen_range(0, chars.len() as u32) as usize;
                    let random_char = chars.chars().nth(char_idx).unwrap();

                    stdout.queue(cursor::MoveTo(x as u16, current_y as u16))?;

                    if i == 0 {
                        // A cabeça da gota é branca/verde muito brilhante
                        stdout.queue(SetForegroundColor(Color::White))?;
                    } else if i < 3 {
                        // O topo da cauda é verde claro
                        stdout.queue(SetForegroundColor(Color::Green))?;
                    } else {
                        // O resto da cauda é verde escuro
                        stdout.queue(SetForegroundColor(Color::DarkGreen))?;
                    }

                    stdout.queue(Print(random_char))?;
                }
            }

            // Se a gota passou o fundo do ecrã, reinicia no topo com novos parâmetros
            if drop.y - drop.length > h as i32 {
                drop.y = -(rng.gen_range(0, 10) as i32);
                drop.speed = rng.gen_range(1, 3) as i32;
                drop.length = rng.gen_range(5, (h / 2) as u32) as i32;
            }
        }

        stdout.flush()?;

        // Controlo de FPS
        let elapsed = frame_start.elapsed();
        if elapsed < target_fps {
            std::thread::sleep(target_fps - elapsed);
        }
    }

    // Restaurar o terminal original ao sair
    stdout.execute(cursor::Show)?;
    stdout.execute(terminal::LeaveAlternateScreen)?;
    terminal::disable_raw_mode()?;

    Ok(())
}

```

---

### 2. Como Executar

Agora tem 6 ferramentas icónicas integradas no seu projeto Rust (`rlolcat`, `rcowsay`, `rfortune`, `rboxes`, `rbonsai` e `rmatrix`).

Para testar a simulação do Matrix (prima **`q`** ou **`Esc`** para sair):

```bash
cargo run --bin rmatrix

```

### 3. Compilação Final para Produção

Para compilar todas as ferramentas otimizadas de uma só vez:

```bash
cargo build --release

```

Os executáveis autónomos estarão disponíveis na pasta `target/release/` prontos a ser distribuídos para Windows, Linux ou macOS sem dependências externas!

Excelente ideia! O **`sl`** (*Steam Locomotive*) é um dos *easter eggs* mais hilariantes e clássicos do ecossistema Unix, concebido originalmente para ensinar os utilizadores a escreverem corretamente `ls` em vez de se enganarem.

Como sugeriu inteligentemente, para o ecossistema Windows ou janelas de comandos onde os erros típicos costumam ser outros (como escrever `rid` em vez de `dir`, ou `dc` em vez de `cd`), podemos adaptar o conceito num **`rsl`** em Rust que anime uma locomotiva a vapor a cruzar o terminal quando detetar um comando invertido ou uma falha de digitação!

Para manter o nosso projeto unificado, **100% autónomo, portátil e sem dependências externas**, vamos adicionar o **`rsl`** como o sétimo binário.

---

### 1. Implementação do `rsl`

Crie um novo ficheiro chamado `src/bin/rsl.rs` com o seguinte código. Ele aceitará opcionalmente os erros comuns do Windows (`rid`, `dc`) ou funcionará diretamente se executado de forma autónoma:

```rust
use std::io::{self, Write};
use std::thread;
use std::time::Duration;
use crossterm::{
    cursor,
    style::{Color, Print, ResetColor, SetForegroundColor},
    terminal::{self, Clear, ClearType},
    ExecutableCommand, QueueableCommand,
};

// Arte ASCII detalhada de uma locomotiva a vapor em movimento
const LOCOMOTIVE: &[&str] = &[
    "      o00000000000000000000000000000000000000000000000000000000000000000000000000000000000o      ",
    "      |                                                                                |      ",
    "      |      ____________________________________________________________________      |      ",
    "      |     |                                                                    |     |      ",
    "      |     |  _      _      _      _      _      _      _      _      _      _  |     |      ",
    "      |     | | |    | |    | |    | |    | |    | |    | |    | |    | |    | | |     |      ",
    "      |     |_|______|_|____|_|____|_|____|_|____|_|____|_|____|_|____|_|____|_|_|_____|      ",
    "     ==.    |                                                                    |          ",
    "   .----'-----.__________________________________________________________________|          ",
    "  /  == == == /                                                                    |          ",
    " |  (_)(_)(_) /                                        ____________________________|          ",
    "  \\          /                                        /                            |          ",
    "   `--------'----------------------------------------'-----------------------------'          ",
    "      __oo_oo__      (_)_(_)_      (_)_(_)_      (_)_(_)_      (_)_(_)_      (_)_(_)_         ",
    "     /         \\    /         \\    /         \\    /         \\    /         \\    /         \\        ",
    "    [___________]  [___________]  [___________]  [___________]  [___________]  [___________]       ",
    "     O       O      O       O      O       O      O       O      O       O      O       O        ",
];

fn main() -> io::Result<()> {
    let args: Vec<String> = std::env::args().collect();
    
    // Se o utilizador invocou o programa propositadamente a tentar escrever um erro típico
    if args.len() > 1 {
        let typo = &args[1].to_lowercase();
        match typo.as_str() {
            "rid" => println!("Aviso: Queria dizer 'dir'? A despachar a locomotiva corretiva..."),
            "dc"  => println!("Aviso: Queria dizer 'cd'? A despachar a locomotiva corretiva..."),
            "clsx"| "clrs" => println!("Aviso: Erro de limpeza de ecrã detetado..."),
            _ => {}
        }
        thread::sleep(Duration::from_millis(800));
    }

    let mut stdout = io::stdout();

    // Configuração do terminal para animação fluida
    terminal::enable_raw_mode()?;
    stdout.execute(terminal::EnterAlternateScreen)?;
    stdout.execute(cursor::Hide)?;

    let (width, height) = terminal::size()?;
    let train_width = LOCOMOTIVE[0].chars().count() as i32;
    let start_y = (height as i32 / 2) - (LOCOMOTIVE.len() as i32 / 2);

    // O comboio começa fora da margem direita e desloca-se para a esquerda
    let mut x_pos = width as i32;

    while x_pos > -train_width {
        stdout.queue(Clear(ClearType::All))?;

        // Desenhar cada linha da locomotiva na nova posição horizontal
        for (i, line) in LOCOMOTIVE.iter().enumerate() {
            let current_y = start_y + i as i32;
            if current_y >= 0 && current_y < height as i32 {
                // Posicionar cursor
                let print_x = if x_pos < 0 { 0 } else { x_pos as u16 };
                stdout.queue(cursor::MoveTo(print_x, current_y as u16))?;

                // Cortar a string do comboio se estiver parcialmente fora do ecrã à esquerda
                let display_str = if x_pos < 0 {
                    let skip = (-x_pos) as usize;
                    if skip < line.chars().count() {
                        line.chars().skip(skip).collect::<String>()
                    } else {
                        String::new()
                    }
                } else {
                    line.to_string()
                };

                // Pintar o comboio com cor vermelha/castanha alusiva a vapor
                stdout.queue(SetForegroundColor(Color::DarkRed))?;
                stdout.queue(Print(display_str))?;
            }
        }

        stdout.flush()?;
        thread::sleep(Duration::from_millis(30)); // Velocidade da animação

        x_pos -= 2; // Passo de movimento por frame
    }

    // Restaurar o terminal original ao terminar
    stdout.execute(cursor::Show)?;
    stdout.execute(terminal::LeaveAlternateScreen)?;
    terminal::disable_raw_mode()?;

    Ok(())
}

```

---

### 2. Como Testar o `rsl`

Agora tem **7 ferramentas lendárias** integradas no seu projeto em Rust (`rlolcat`, `rcowsay`, `rfortune`, `rboxes`, `rbonsai`, `rmatrix` e `rsl`).

Pode testar o comportamento corretivo fingindo que se enganou num comando típico do Windows:

```bash
cargo run --bin rsl -- rid

```

Ou testar o comando `dc`:

```bash
cargo run --bin rsl -- dc

```

### 3. Compilação Final para Produção

Para compilar todos os utilitários de produção otimizados e autónomos de uma só vez:

```bash
cargo build --release

```

Os 7 executáveis ficarão prontos na pasta `target/release/`, permitindo-lhe criar o seu próprio toolkit de utilitários retro/fun em Rust sem dependências externas!

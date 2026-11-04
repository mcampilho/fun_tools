# fun_tools

Brinquedos e ferramentas clássicos do terminal Unix, reescritos em Rust para **Windows e Linux**.
Todas as tools começam por **r** (de Rust).

[English](README.md)

![Demonstração](docs/demo.png)

| Tool | Ao estilo de | O que faz |
|---|---|---|
| `rfetch` | neofetch | Informação do sistema ao lado de um logótipo do SO / shell ([detalhes](docs/rfetch.pt.md)) |
| `rcowsay`, `rcowthink` | cowsay | Um animal diz ou pensa uma mensagem |
| `rlolcat` | lolcat | Texto com as cores do arco-íris |
| `rboxes` | boxes | Uma caixa à volta do texto |
| `rfortune` | fortune | Uma frase ao acaso |
| `rbonsai` | cbonsai | Faz crescer um bonsai ao acaso |
| `rmatrix` | cmatrix | Chuva digital a cair |
| `rsl` | sl | Um comboio a vapor atravessa o terminal |

## Compilar e instalar

```bash
cargo build --release
```

Os programas ficam em `target/release/`. Para os instalar todos em `~/.cargo/bin`:

```bash
cargo install --path .
```

- **Windows:** precisa das ferramentas de compilação MSVC (`link.exe`, `cl.exe`) no ambiente.
- **Linux:** precisa de um compilador de C (`gcc` ou `clang`), usado pela biblioteca de TLS do rfetch.

`test.bat` (Windows) e `test.sh` (Linux) compilam tudo e mostram uma pequena demonstração de cada tool.

## Comum a todas as tools

- `-h, --help` e `-V, --version`.
- `--lang <código>`: idioma das mensagens, `en`, `pt` ou `es`. Também `RTOOLS_LANG`;
  senão, o idioma do sistema, e inglês quando não é nenhum destes.
- As cores só vão para um terminal (a não ser com `--force`/`--color`) e nunca com `NO_COLOR`.
  A profundidade de cor é detetada (24 bits, 256 ou 16 cores); `RTOOLS_COLORS=16|256|truecolor`
  força-a.
- As larguras são medidas em colunas do terminal: acentos, CJK e emoji mantêm as caixas e os
  balões direitos.
- O texto recebido por pipe do Windows PowerShell perde a marca de ordem dos bytes (BOM); os fins
  de linha `\r\n` não causam problemas.
- Os erros de utilização terminam com o código 2, os outros erros com 1.

## As tools

### rcowsay / rcowthink

```bash
rcowsay "Olá"                      # mensagem nos argumentos
rfortune | rcowsay -f cat          # ou recebida por pipe
rcowthink -f owl -d "..."          # pensar, com olhos de morto
rcowsay -l                         # animais: cow, calf, sheep, cat, owl, crab, ghost
```

Opções: `-f` animal, `-r` animal ao acaso, `-e` olhos, `-T` língua, `-W` largura de quebra (40),
`-n` sem quebra de linhas, e os aspetos `-b -d -g -p -s -t -w -y`.

### rlolcat

```bash
rfortune | rlolcat
rlolcat -a ficheiro.txt            # animado
rcowsay olá | rlolcat -f | rboxes  # --force: cores para um pipe
```

Lê linha a linha, à medida que o texto chega. Opções: `-p` espalhamento, `-F` frequência,
`-S` semente, `-i` colorir o fundo, `-a`/`-d`/`-s` animação, `-f` forçar as cores.

### rboxes

```bash
echo "Olá" | rboxes double
rboxes -s bold -a c -p 2 -t Título ficheiro.txt
rboxes --list
```

Estilos: round, single, double, bold, dashed, block, shadow, classic, stars, hash, markdown.
Opções: `-a` alinhamento (l/c/r), `-p`/`-P` margens, `-w` largura mínima, `-t` título,
`-c` cor da moldura (ou `rainbow`).

### rfortune

```bash
rfortune                           # frases embutidas, no idioma atual
rfortune -s                        # só as curtas
rfortune -m rust                   # todas as frases com "rust"
rfortune frases.txt pasta/         # ficheiros fortune (entradas separadas por uma linha "%")
rfortune --system                  # Linux: também /usr/share/games/fortunes
```

`RFORTUNE_PATH` (uma lista de ficheiros/pastas) é usado quando não se indica nenhum ficheiro.

### rbonsai

```bash
rbonsai                            # mostrar uma árvore
rbonsai -l                         # vê-la crescer
rbonsai -s 42 -M 8 -b 2            # semente, mais ramos, vaso pequeno
```

A mesma semente dá a mesma árvore (`-v` mostra-a). `-L` comprimento do tronco, `-W`/`-H` tamanho.

### rmatrix

```bash
rmatrix
rmatrix -c rainbow -s 8 -k         # arco-íris, rápido, com katakana
```

q, Esc ou Ctrl+C para sair; as teclas de 1 a 9 mudam a velocidade. O ecrã só é redesenhado onde
muda, por isso não cintila.

### rsl

```bash
rsl
rsl -l -F                          # um comboio pequeno a voar
rsl -n 5                           # cinco carruagens
```

q, Esc ou Ctrl+C para o parar.

## Estrutura do projeto

```text
src/lib.rs        código comum: cli, i18n, input, rng, term (cores), text (larguras), tui
src/cow.rs        rcowsay e rcowthink
src/bin/*.rs      um ficheiro por tool
src/bin/rfetch/   rfetch (vários módulos)
tools/            gerador dos logótipos do rfetch
docs/             imagens, documentação do rfetch
```

## Idiomas

Cada tool tem os seus textos numa tabela por idioma (inglês, português europeu, espanhol); os
textos em falta ficam em inglês. A lista de idiomas é `LANGS` em `src/i18n.rs`.

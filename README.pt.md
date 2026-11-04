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
| `rfiglet` | figlet | Letras grandes feitas de caracteres |
| `rtoilet` | toilet | Letras grandes e coloridas, com filtros |
| `rtty-clock` | tty-clock | Um relógio digital grande |
| `rasciiquarium` | asciiquarium | Um aquário: peixes, tubarões, uma baleia, um barco... |
| `rnyancat` | nyancat | Um gato com corpo de bolo voa pelo espaço num arco-íris |
| `rhollywood` | hollywood | Painéis com atividade de "hacker" muito ocupada, como nos filmes |
| `rrig` | rig | Identidades fictícias ao acaso para dados de teste |

Todos os desenhos, fontes, frases e dados são originais; também se podem usar as fontes
FIGlet/TOIlet instaladas no sistema.

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

As tools de ecrã inteiro (rmatrix, rsl, rtty-clock, rasciiquarium, rnyancat, rhollywood) precisam de um terminal.

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
rbonsai -u -f blossom -l           # aspeto Unicode, em flor (ver abaixo)
```

A mesma semente dá a mesma árvore (`-v` mostra-a). `-L` comprimento do tronco, `-W`/`-H` tamanho.
Com `-u`, `-f` escolhe as folhas: green (verde), autumn (outono), blossom (em flor) ou random.

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
rsl -u                             # aspeto Unicode (ver abaixo)
```

q, Esc ou Ctrl+C para o parar.

### rfiglet / rtoilet

```bash
rfiglet "Olá"                      # "standard" se estiver instalada, senão a "hash" embutida
rfiglet -f pixel -c "Olá, Ação!"   # as fontes embutidas têm as letras acentuadas
rtoilet --gay "Arco-íris"          # rtoilet: fonte embutida "pixel" e filtros de cor
rtoilet -f tiny -F border:metal "Tiny"
rfiglet -L                         # fontes: embutidas e instaladas
```

Fontes embutidas: pixel, tiny, hash, shade, drop (desenhadas a partir de uma fonte de
pixels original, com as letras acentuadas do português, do espanhol e de outras línguas
ocidentais). As fontes FIGlet `.flf` e TOIlet `.tlf` são procuradas pelo nome em
`FIGLET_FONTDIR`, `TOILET_FONT_PATH`, numa pasta `fonts` ao lado do programa,
`%LOCALAPPDATA%\rfiglet\fonts` (Windows), `~/.local/share/rfiglet/fonts` e `/usr/share/figlet`
(Linux), ou indicadas como ficheiro; são seguidas as suas regras de encaixe e fusão. Opções:
`-w` largura, `-c`/`-l`/`-r` alinhamento, `-k` encaixe, `-W` largura total, `-s` fusão; o
rtoilet tem também `-F` filtros (crop, rainbow/gay, metal, fire, flip, flop, rotate, border).

### rtty-clock

```bash
rtty-clock -s                      # com segundos
rtty-clock -t -C cyan -x           # 12 horas, ciano, numa caixa
rtty-clock -r -S                   # protetor de ecrã aos saltos: qualquer tecla sai
```

Teclas: q sair, s segundos, t 12/24 h, d data, r saltar, x caixa, b dois pontos a piscar,
1 a 8 cor. A data é escrita no idioma escolhido (ou no formato strftime de `-f`). Os
algarismos são células coloridas, por isso não têm riscos entre linhas.

### rasciiquarium

```bash
rasciiquarium
rasciiquarium -n 9 -s 7            # mais peixes, mais depressa
rasciiquarium -u                   # aspeto Unicode (ver abaixo)
```

Peixes de vários tipos nadam para os dois lados e deitam bolhas, as algas ondulam e, de
vez em quando, passa um tubarão (come os peixes pequenos), uma baleia com o seu repuxo, um
barco ou uma medusa. Teclas: q sair, p pausa, r recomeçar.

### O aspeto Unicode (`-u`) do rasciiquarium, do rsl e do rbonsai

![Aspeto Unicode](docs/unicode.png)

Com `-u, --unicode`, tudo passa a pixel art a cores: cada célula do terminal mostra dois
pixels quadrados com os meios blocos `▀ ▄`, e alguns símbolos ficam por cima (bolhas `○ ◦`,
estrelas `✦ · ∙`). O aquário ganha um céu noturno com quarto crescente, ondas, raios de luz,
um fundo de areia e criaturas em pixels. O comboio ganha um tender, carruagens com
passageiros, rodas que giram com bielas, fumo a subir e uma paisagem com nuvens, colinas e a
via. O bonsai ganha um tronco sombreado em S, ramos com nuvens de folhas (verdes, de outono
ou em flor) e um vaso vidrado com terra e musgo; também pode crescer ao vivo (`-l`).
É uma opção porque precisa de uma fonte com estes caracteres (Cascadia, Consolas,
DejaVu Sans Mono, Meslo, as Nerd Fonts...) e fica melhor com cor de 24 bits (Windows
Terminal, a maioria dos terminais Linux); com 256 ou 16 cores, as cores são aproximadas.

### rnyancat

```bash
rnyancat
rnyancat -n -s 2                   # sem contador, pixels do desenho duas vezes maiores
rnyancat -f 100                    # parar ao fim de 100 imagens
```

Um gato com corpo de bolo voa por um céu estrelado, com um arco-íris atrás: sobe e desce,
mexe as patas e a cauda, e as estrelas cintilam ao passar. O contador em baixo diz há
quanto tempo está a voar (`-n` esconde-o). O desenho é original e é sempre feito no
aspeto Unicode, ampliado para caber no terminal (ou `-s`). `-d` dá o tempo de cada
imagem. q, Esc ou Ctrl+C para sair.

### rhollywood

```bash
rhollywood
rhollywood -p 4 -i 5               # quatro painéis, um muda a cada 5 s
```

Painéis: memória, código a compilar, registo do sistema, ligações de rede, lista de
processos, medidores de CPU, barras de desencriptação, um osciloscópio e um fluxo de
algarismos. É tudo inventado. Teclas: q sair, espaço baralhar.

### rrig

```bash
rrig                               # uma identidade, país conforme o idioma
rrig -C pt -c 5 -e                 # cinco identidades portuguesas, com e-mail
rrig -C es -o csv -c 100 > teste.csv
rrig -o json -s 42                 # as mesmas identidades outra vez, com a mesma semente
```

Países: us, pt, es. É tudo fictício: os telefones dos EUA usam a gama 555-01xx, reservada
para ficção, e os e-mails usam os domínios example.com/.org/.net.

## Estrutura do projeto

```text
src/lib.rs        código comum: cli, i18n, input, rng, term (cores), text (larguras), tui
src/cow.rs        rcowsay e rcowthink
src/fig.rs        rfiglet e rtoilet (layout FIGlet, fontes, filtros)
src/pixfont.rs    a fonte de pixels original (fontes embutidas, rtty-clock)
src/bin/*.rs      um ficheiro por tool
src/bin/rfetch/   rfetch (vários módulos)
tools/            gerador dos logótipos do rfetch
docs/             imagens, documentação do rfetch
```

## Idiomas

Cada tool tem os seus textos numa tabela por idioma (inglês, português europeu, espanhol); os
textos em falta ficam em inglês. A lista de idiomas é `LANGS` em `src/i18n.rs`.

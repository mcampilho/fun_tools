# fun_tools

Classic Unix terminal toys and tools, rewritten in Rust for **Windows and Linux**.
Every tool starts with **r** (for Rust).

[Português](README.pt.md)

![Demo](docs/demo.png)

| Tool | In the style of | What it does |
|---|---|---|
| `rfetch` | neofetch | System information next to a logo of the OS / shell ([details](docs/rfetch.md)) |
| `rcowsay`, `rcowthink` | cowsay | An animal says or thinks a message |
| `rlolcat` | lolcat | Rainbow-coloured text |
| `rboxes` | boxes | A box around text |
| `rfortune` | fortune | A random saying |
| `rbonsai` | cbonsai | Grows a random bonsai tree |
| `rmatrix` | cmatrix | Falling digital rain |
| `rsl` | sl | A steam train crosses the terminal |
| `rfiglet` | figlet | Large letters made of characters |
| `rtoilet` | toilet | Large coloured letters, with filters |
| `rtty-clock` | tty-clock | A big digital clock |
| `rasciiquarium` | asciiquarium | An aquarium: fish, sharks, a whale, a ship... |
| `rhollywood` | hollywood | Panes of busy-looking "hacker" activity, as in the films |
| `rrig` | rig | Random fictional identities for test data |

All the art, fonts, sayings and data are original; FIGlet/TOIlet fonts installed on the
system can be used too.

## Build and install

```bash
cargo build --release
```

The programs are in `target/release/`. To install them all in `~/.cargo/bin`:

```bash
cargo install --path .
```

- **Windows:** needs the MSVC build tools (`link.exe`, `cl.exe`) in the environment.
- **Linux:** needs a C compiler (`gcc` or `clang`), used by rfetch's TLS library.

The full-screen tools (rmatrix, rsl, rtty-clock, rasciiquarium, rhollywood) need a terminal.

`test.bat` (Windows) and `test.sh` (Linux) build everything and run a short demo of each tool.

## Common to all the tools

- `-h, --help` and `-V, --version`.
- `--lang <code>`: language of the messages, `en`, `pt` or `es`. Also `RTOOLS_LANG`;
  otherwise the system language, and English when it is none of these.
- Colours only go to a terminal (unless `--force`/`--color`) and never with `NO_COLOR`.
  The colour depth is detected (24-bit, 256 or 16 colours); `RTOOLS_COLORS=16|256|truecolor`
  forces it.
- Widths are measured in terminal columns: accents, CJK and emoji keep boxes and balloons straight.
- Text piped from Windows PowerShell loses its byte-order mark; `\r\n` line endings are fine.
- Usage errors exit with code 2, other errors with 1.

## The tools

### rcowsay / rcowthink

```bash
rcowsay "Hello"                    # message from the arguments
rfortune | rcowsay -f cat          # or piped in
rcowthink -f owl -d "..."          # think, with dead eyes
rcowsay -l                         # animals: cow, calf, sheep, cat, owl, crab, ghost
```

Options: `-f` animal, `-r` random animal, `-e` eyes, `-T` tongue, `-W` wrap width (40),
`-n` no wrapping, and the looks `-b -d -g -p -s -t -w -y`.

### rlolcat

```bash
rfortune | rlolcat
rlolcat -a file.txt                # animated
rcowsay hi | rlolcat -f | rboxes   # --force: colours into a pipe
```

Reads line by line, as the text arrives. Options: `-p` spread, `-F` frequency, `-S` seed,
`-i` colour the background, `-a`/`-d`/`-s` animation, `-f` force colours.

### rboxes

```bash
echo "Hello" | rboxes double
rboxes -s bold -a c -p 2 -t Title file.txt
rboxes --list
```

Styles: round, single, double, bold, dashed, block, shadow, classic, stars, hash, markdown.
Options: `-a` align (l/c/r), `-p`/`-P` padding, `-w` minimum width, `-t` title,
`-c` border colour (or `rainbow`).

### rfortune

```bash
rfortune                           # built-in sayings, in the current language
rfortune -s                        # short ones only
rfortune -m rust                   # every saying with "rust"
rfortune my-fortunes.txt folder/   # fortune files (entries separated by a "%" line)
rfortune --system                  # Linux: also /usr/share/games/fortunes
```

`RFORTUNE_PATH` (a list of files/folders) is used when no file is given.

### rbonsai

```bash
rbonsai                            # print a tree
rbonsai -l                         # watch it grow
rbonsai -s 42 -M 8 -b 2            # seed, more branches, small pot
```

The same seed gives the same tree (`-v` prints it). `-L` trunk length, `-W`/`-H` size.

### rmatrix

```bash
rmatrix
rmatrix -c rainbow -s 8 -k         # rainbow, fast, with katakana
```

q, Esc or Ctrl+C quit; the keys 1 to 9 change the speed. The screen is only redrawn
where it changes, so it does not flicker.

### rsl

```bash
rsl
rsl -l -F                          # a little flying train
rsl -n 5                           # five carriages
rsl -u                             # Unicode look (see below)
```

q, Esc or Ctrl+C stop it.

### rfiglet / rtoilet

```bash
rfiglet "Hello"                    # "standard" if installed, else the built-in "hash"
rfiglet -f pixel -c "Olá, Ação!"   # accented letters work in the built-in fonts
rtoilet --gay "Rainbow"            # rtoilet: built-in "pixel" font and colour filters
rtoilet -f tiny -F border:metal "Tiny"
rfiglet -L                         # fonts: built-in and installed
```

Built-in fonts: pixel, tiny, hash, shade, drop (drawn from an original pixel font with
the accented letters of Portuguese, Spanish and other western languages). FIGlet `.flf` and
TOIlet `.tlf` fonts are found by name in `FIGLET_FONTDIR`, `TOILET_FONT_PATH`, a `fonts`
folder next to the program, `%LOCALAPPDATA%\rfiglet\fonts` (Windows),
`~/.local/share/rfiglet/fonts` and `/usr/share/figlet` (Linux), or given as a file; their
kerning and smushing rules are followed. Options: `-w` width, `-c`/`-l`/`-r` alignment,
`-k` kerning, `-W` full width, `-s` smushing; rtoilet also has `-F` filters
(crop, rainbow/gay, metal, fire, flip, flop, rotate, border).

### rtty-clock

```bash
rtty-clock -s                      # with seconds
rtty-clock -t -C cyan -x           # 12 hours, cyan, in a box
rtty-clock -r -S                   # bouncing screensaver: any key quits
```

Keys: q quit, s seconds, t 12/24 h, d date, r bounce, x box, b blinking colons,
1 to 8 colour. The date is written in the chosen language (or `-f` strftime format).
The digits are coloured cells, so they have no gaps between rows.

### rasciiquarium

```bash
rasciiquarium
rasciiquarium -n 9 -s 7            # more fish, faster
rasciiquarium -u                   # Unicode look (see below)
```

Fish of several kinds swim both ways and blow bubbles, the seaweed sways, and now and
then a shark (it eats small fish), a whale with its spout, a ship or a jellyfish passes by.
Keys: q quit, p pause, r restart.

### The Unicode look (`-u`) of rasciiquarium and rsl

![Unicode look](docs/unicode.png)

With `-u, --unicode`, everything is colour pixel art: each terminal cell shows two square
pixels with the half blocks `▀ ▄`, and a few symbols go on top (`○ ◦` bubbles, `✦ · ∙`
stars). The aquarium gets a night sky with a crescent moon, waves, light rays, a sandy
bottom and pixel creatures. The train gets a tender, carriages with passengers, turning
wheels with coupling rods, rising smoke, and a landscape with clouds, hills and the track.
It is an option because it needs a font with these characters (Cascadia, Consolas, DejaVu
Sans Mono, Meslo, the Nerd Fonts...) and looks best with 24-bit colour (Windows Terminal,
most Linux terminals); with 256 or 16 colours the colours are approximated.

### rhollywood

```bash
rhollywood
rhollywood -p 4 -i 5               # four panes, one changes every 5 s
```

Panes: memory dump, code being built, system log, network connections, process list,
CPU meters, decryption progress bars, a signal scope and a digit stream. Everything shown
is made up. Keys: q quit, space shuffle.

### rrig

```bash
rrig                               # one identity, country from the language
rrig -C pt -c 5 -e                 # five Portuguese identities, with e-mail
rrig -C es -o csv -c 100 > test.csv
rrig -o json -s 42                 # the same identities again with the same seed
```

Countries: us, pt, es. All of it is fictional: US phone numbers use the 555-01xx range
reserved for fiction and e-mails use the example.com/.org/.net domains.

## Project structure

```text
src/lib.rs        shared code: cli, i18n, input, rng, term (colours), text (widths), tui
src/cow.rs        rcowsay and rcowthink
src/fig.rs        rfiglet and rtoilet (FIGlet layout, fonts, filters)
src/pixfont.rs    the original pixel font (built-in fonts, rtty-clock)
src/bin/*.rs      one file per tool
src/bin/rfetch/   rfetch (several modules)
tools/            rfetch logo generator
docs/             images, rfetch documentation
```

## Languages

Each tool keeps its texts in one table per language (English, European Portuguese,
Spanish); missing texts fall back to English. The list of languages is `LANGS` in
`src/i18n.rs`.

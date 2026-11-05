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
| `rnyancat` | nyancat | A cat with a pastry body flies through space on a rainbow |
| `rpipes` | pipes.sh | Pipes grow across the terminal, turning at random |
| `rnms` | no-more-secrets | Scrambled text decrypts itself, as in the films |
| `rponysay` | ponysay | A pixel-art pony says or thinks a message |
| `rgenact` | genact | Fake busy activity: builds, boot logs, downloads, deployments |
| `rhollywood` | hollywood | Panes of busy-looking "hacker" activity, as in the films |
| `rblocks` | bastet | Falling blocks; the evil mode always gives the piece you need least |
| `rinvaders` | ninvaders | Shoot down the rows of invaders before they land |
| `rsudoku` | nudoku | Sudoku: puzzles with one solution, in four difficulties, or printed |
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

The full-screen tools (rmatrix, rsl, rtty-clock, rasciiquarium, rnyancat, rpipes, rhollywood) and the games need a terminal.

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
rbonsai -u -f blossom -l           # Unicode look, in blossom (see below)
```

The same seed gives the same tree (`-v` prints it). `-L` trunk length, `-W`/`-H` size.
With `-u`, `-f` picks the leaves: green, autumn, blossom or random.

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

### The Unicode look (`-u`) of rasciiquarium, rsl and rbonsai

![Unicode look](docs/unicode.png)

With `-u, --unicode`, everything is colour pixel art: each terminal cell shows two square
pixels with the half blocks `▀ ▄`, and a few symbols go on top (`○ ◦` bubbles, `✦ · ∙`
stars). The aquarium gets a night sky with a crescent moon, waves, light rays, a sandy
bottom and pixel creatures. The train gets a tender, carriages with passengers, turning
wheels with coupling rods, rising smoke, and a landscape with clouds, hills and the track.
The bonsai gets a shaded trunk with an S-curve, branches with pads of leaves (green,
autumn or blossom), and a glazed pot with soil and moss; it can still grow live (`-l`).
It is an option because it needs a font with these characters (Cascadia, Consolas, DejaVu
Sans Mono, Meslo, the Nerd Fonts...) and looks best with 24-bit colour (Windows Terminal,
most Linux terminals); with 256 or 16 colours the colours are approximated.

### rnyancat

```bash
rnyancat
rnyancat -n -s 2                   # no counter, art pixels twice as big
rnyancat -f 100                    # stop after 100 frames
```

A cat with a pastry body flies through a starry sky, a rainbow behind it: it bobs,
paddles and swings its tail, and the stars twinkle as they go by. The counter at the
bottom says how long it has been flying (`-n` hides it). The art is original and is
always drawn in the Unicode look, enlarged to fit the terminal (or `-s`). `-d` sets the
time per frame. q, Esc or Ctrl+C quit.

### rpipes

```bash
rpipes
rpipes -p 5 -t 1 -t 2 -R           # five pipes, light and rounded, random starts
rpipes -t c"|-++++" -s 5           # your own pieces, twistier
rpipes -l                          # the styles
```

Styles: heavy, light, rounded, double, dashed, mixed, blocks, dots, ascii (`-t 0` to `-t 8`).
Options: `-p` pipes, `-f` steps per second, `-s` straightness, `-r` clear after so many pieces,
`-R` random starts, `-K` keep colour and style across the edges, `-B` no bold, `-C` no colours.
Keys: q quit, space clear, 1 to 9 speed.

### rnms

```bash
echo "The secret is 42" | rnms     # scrambled, a key, then decrypted
ls -l | rnms -a -f green           # decrypt at once, in green
rnms -u "Top secret"               # scramble with block and geometric symbols too
```

The text stays on the screen at the end. Options: `-a` no key, `-s` scramble the spaces too,
`-f` colour, `-c` clear the screen first, `-t` seconds of scrambling. Without input it shows a
short demo.

### rponysay

```bash
rponysay "Hello!"
rfortune | rponysay -k unicorn -f lilac
rponysay -t -k pegasus "Hmm..."    # think
rponysay -l                        # coats and kinds
```

![Ponies and pipes](docs/ponies-pipes.png)

The ponies are original pixel art drawn by the program (they are not the characters of the
original ponysay): three kinds (pony, unicorn, pegasus) and eight coats (coral, lilac, azure,
lemon, jade, plum, snow, cocoa), chosen at random unless given. `-W` wrap width, `-n` no wrap,
`-s` seed.

### rgenact

```bash
rgenact                            # one module after another, forever
rgenact -m cargo -m docker -s 2    # only these, twice as fast
rgenact --exit-after-time 60
rgenact -l
```

Modules: cargo, bootlog, kernel, docker, download, memdump, packages, terraform, ansible,
weblog, hashcrack, loading. Everything is made up: names, hashes and addresses (only the
ranges set aside for documentation). Ctrl+C stops it.

### rhollywood

```bash
rhollywood
rhollywood -p 4 -i 5               # four panes, one changes every 5 s
```

Panes: memory dump, code being built, system log, network connections, process list,
CPU meters, decryption progress bars, a signal scope and a digit stream. Everything shown
is made up. Keys: q quit, space shuffle.

### The games: rblocks, rinvaders, rsudoku

![Games](docs/games.png)

The games are original: they share their genre and rules with the classics, not their
names, art or look.

```bash
rblocks                            # evil mode, like bastet
rblocks -m random -L 5             # fair pieces, from level 5
rinvaders
rsudoku -d hard
rsudoku -p 4 -d expert --solution > sudokus.txt   # puzzles for paper
```

- **rblocks**: ← → move, ↑ rotate, z rotate back, ↓ faster, space drop, p pause.
  Modes (`-m`): `evil` (default; every new piece is the one that suits you least),
  `random` (fair, shuffled sets of seven, with the next piece shown) and `nice`
  (the most helpful piece). `-G` hides where the piece will land.
- **rinvaders**: ← → move, space shoot, p pause. The formation speeds up as it shrinks;
  the shields wear away; the saucer is worth a surprise. An extra life at 1500 points;
  each level starts lower. `-L` starting level. The number of rows of invaders depends on
  the terminal's height (2 to 5).
- **rsudoku**: arrows or h j k l move, 1-9 write, 0 erase, c check, ? hint, u undo,
  n new puzzle, S solution. Clashes show in red at once. Every puzzle is made on the spot
  and has a single solution; `-s` repeats one.

rblocks and rinvaders keep the ten best scores (`-S` shows them) in `%LOCALAPPDATA%\rtools`
(Windows) or `~/.local/share/rtools` (Linux); `RTOOLS_DATA` chooses another folder.
In every game, q or Esc quits and n starts again after a game ends.

### rrig

```bash
rrig                               # one identity, country from the language
rrig -C pt -c 5 -e                 # five Portuguese identities, with e-mail
rrig -C es -o csv -c 100 > test.csv
rrig -o json -s 42                 # the same identities again with the same seed
```

Countries: us, pt, es. All of it is fictional: US phone numbers use the 555-01xx range
reserved for fiction and e-mails use the example.com/.org/.net domains.

## Origins

Every tool here is a new program written from scratch, with original art and texts; none of
the code comes from the originals. These are the classics that inspired them, with thanks to
their authors.

| Tool | Inspired by | Author(s) | First released | Home |
|---|---|---|---|---|
| rfetch | neofetch, screenFetch | Dylan Araps; Brett Bohnenkamper | 2015 (archived in 2024); 2010 | [neofetch](https://github.com/dylanaraps/neofetch), [screenFetch](https://github.com/KittyKatt/screenFetch) |
| rcowsay, rcowthink | cowsay | Tony Monroe; maintained since 2016 by Andrew Janke | 1999 | [cowsay-org/cowsay](https://github.com/cowsay-org/cowsay) |
| rlolcat | lolcat | Moe (busyloop) | 2011 | [busyloop/lolcat](https://github.com/busyloop/lolcat) |
| rboxes | boxes | Thomas Jensen | 1999 | [boxes.thomasjensen.com](https://boxes.thomasjensen.com/) |
| rfortune | fortune | Ken Arnold (BSD fortune) | 1979 (Version 7 Unix) | [fortune-mod](https://github.com/shlomif/fortune-mod) |
| rbonsai | cbonsai | John Allbritten (from his bonsai.sh, 2019) | 2020 | [jallbrit/cbonsai](https://gitlab.com/jallbrit/cbonsai) |
| rmatrix | cmatrix | Chris Allegretta; maintained by Abishek V. Ashok | 1999 | [abishekvashok/cmatrix](https://github.com/abishekvashok/cmatrix) |
| rsl | sl | Toyoda Masashi | 1993 | [mtoyoda/sl](https://github.com/mtoyoda/sl) |
| rfiglet | FIGlet | Glenn Chappell and Ian Chai; maintained by Claudio Matsuoka | 1991 ("newban"), 1993 (FIGlet 2.0) | [figlet.org](http://www.figlet.org/) |
| rtoilet | TOIlet | Sam Hocevar (libcaca project) | 2006 | [caca.zoy.org/wiki/toilet](http://caca.zoy.org/wiki/toilet) |
| rtty-clock | tty-clock | Martin Duquesnoy (xorg62) | 2008 | [xorg62/tty-clock](https://github.com/xorg62/tty-clock) |
| rasciiquarium | Asciiquarium | Kirk Baucom | 2005 (1.0: 2 September 2005) | [robobunny.com](https://robobunny.com/projects/asciiquarium/html/) |
| rnyancat | nyancat | Kevin Lange (klange); the Nyan Cat animation is by Christopher Torres (prguitarman), April 2011 | 2011 | [klange/nyancat](https://github.com/klange/nyancat) |
| rhollywood | hollywood | Dustin Kirkland | 2014 | [dustinkirkland/hollywood](https://github.com/dustinkirkland/hollywood) |
| rrig | rig (Random Identity Generator) | Ian Turner, after the MS-DOS program "fake" | 1999 | [rig.sourceforge.net](https://rig.sourceforge.net/) |
| rpipes | pipes.sh | Matthew Simpson (Arch Linux forums); Yu-Jie Lin; Pipeseroni since 2015 | 2010 | [pipeseroni/pipes.sh](https://github.com/pipeseroni/pipes.sh) |
| rponysay | ponysay | Erkin Batu Altunbaş, with Mattias Andrée and others | 2012 | [erkin/ponysay](https://github.com/erkin/ponysay) |
| rnms | no-more-secrets | Brian Barto (bartobri); the effect is from the 1992 film *Sneakers* | 2016 | [bartobri/no-more-secrets](https://github.com/bartobri/no-more-secrets) |
| rgenact | genact | Sven-Hendrik Haase (svenstaro) | 2018 (first version on crates.io) | [svenstaro/genact](https://github.com/svenstaro/genact) |
| rblocks | bastet ("bastard tetris") | Federico Poloni, based on Petris by Peter Seidler | 2004 | [bastet](https://fph.altervista.org/prog/bastet.html) |
| rinvaders | nInvaders | Thomas Dettbarn | 2002 | [ninvaders.sourceforge.net](http://ninvaders.sourceforge.net/) |
| rsudoku | nudoku | Michael Vetter (jubalh) | 2015 (on GitHub) | [jubalh/nudoku](https://github.com/jubalh/nudoku) |

Notes on the dates: the year is the first public release when it is known (from the project's
own history, changelog or man page), otherwise the oldest trace found. For nInvaders, bastet
and nudoku it is an estimate from the oldest copyright notice, package or repository; tty-clock's
code carries a 2008 copyright. fortune first shipped with Version 7 Unix in 1979; the
BSD fortune by Ken Arnold is the one most systems use today. The game names, characters
and art of the originals are not used.

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

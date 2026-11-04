"""Generates the rfetch pixel-art logos.

Usage (from the project folder; needs Pillow for the preview):
    python3 tools/gen_logos.py preview.png src/logo/art.rs

Each logo is drawn here with simple shapes (circles, polygons, lines) and written
to src/logo/art.rs as text grids; preview.png shows them all, including the
Windows + badge and WSL combinations.
"""
import math
import sys
from pathlib import Path


class Canvas:
    def __init__(s, w, h):
        s.w, s.h = w, h
        s.p = [[None] * w for _ in range(h)]

    def fill(s, f, c):
        for y in range(s.h):
            for x in range(s.w):
                if f(x + 0.5, y + 0.5):
                    s.p[y][x] = c

    def disk(s, cx, cy, r, c):
        s.fill(lambda x, y: (x - cx) ** 2 + (y - cy) ** 2 <= r * r, c)

    def ellipse(s, cx, cy, rx, ry, c, rot=0.0, clip=None):
        cr, sr = math.cos(rot), math.sin(rot)

        def f(x, y):
            if clip and not clip(x, y):
                return False
            dx, dy = x - cx, y - cy
            u, v = dx * cr + dy * sr, -dx * sr + dy * cr
            return (u / rx) ** 2 + (v / ry) ** 2 <= 1

        s.fill(f, c)

    def ring(s, cx, cy, r0, r1, c, gaps=()):
        def f(x, y):
            d = math.hypot(x - cx, y - cy)
            if not r0 <= d <= r1:
                return False
            a = math.degrees(math.atan2(-(y - cy), x - cx))
            return all(abs((a - g + 180) % 360 - 180) > hw for g, hw in gaps)

        s.fill(f, c)

    def rect(s, x0, y0, x1, y1, c):
        s.fill(lambda x, y: x0 <= x <= x1 and y0 <= y <= y1, c)

    def rrect(s, x0, y0, x1, y1, r, c):
        def f(x, y):
            if not (x0 <= x <= x1 and y0 <= y <= y1):
                return False
            qx = min(max(x, x0 + r), x1 - r)
            qy = min(max(y, y0 + r), y1 - r)
            return (x - qx) ** 2 + (y - qy) ** 2 <= r * r

        s.fill(f, c)

    def poly(s, pts, c):
        def f(x, y):
            inside = False
            for i in range(len(pts)):
                (x1, y1), (x2, y2) = pts[i], pts[(i + 1) % len(pts)]
                if (y1 > y) != (y2 > y) and x < (x2 - x1) * (y - y1) / (y2 - y1) + x1:
                    inside = not inside
            return inside

        s.fill(f, c)

    def line(s, pts, t, c):
        segs = list(zip(pts, pts[1:]))

        def f(x, y):
            for (ax, ay), (bx, by) in segs:
                dx, dy = bx - ax, by - ay
                L = dx * dx + dy * dy
                k = 0 if L == 0 else max(0, min(1, ((x - ax) * dx + (y - ay) * dy) / L))
                if (x - ax - k * dx) ** 2 + (y - ay - k * dy) ** 2 <= t * t:
                    return True
            return False

        s.fill(f, c)

    def blobs(s, pts, c):
        """pts: (x, y, radius) circles, unioned."""
        s.fill(lambda x, y: any((x - px) ** 2 + (y - py) ** 2 <= r * r for px, py, r in pts), c)

    def stamp(s, x0, y0, rows, c):
        for j, r in enumerate(rows):
            for i, ch in enumerate(r):
                if ch == "#" and 0 <= x0 + i < s.w and 0 <= y0 + j < s.h:
                    s.p[y0 + j][x0 + i] = c

    def rows(s):
        return ["".join(ch or "." for ch in r) for r in s.p]


def polar(cx, cy, r, deg):
    a = math.radians(deg)
    return (cx + r * math.cos(a), cy - r * math.sin(a))


ARTS = []  # (name, accent, palette dict, holes, canvas)


def art(name, accent, palette, holes=""):
    def deco(fn):
        ARTS.append((name, accent, palette, holes, fn()))
        return fn
    return deco


# ---------------------------------------------------------------- Windows
# Pane edges fall on even pixel rows, so every pane line is a full cell
# (some terminals leave a thin gap between half-block glyphs).
@art("windows", 39, {"a": 39, "b": 33})
def _():
    c = Canvas(30, 30)
    c.rect(0, 0, 14, 14, "a"); c.rect(16, 0, 30, 14, "a")
    c.rect(0, 16, 14, 30, "b"); c.rect(16, 16, 30, 30, "b")
    return c


@art("badge-winsmall", 39, {"a": 39, "b": 33})
def _():
    c = Canvas(14, 14)
    c.rect(0, 0, 6, 6, "a"); c.rect(8, 0, 14, 6, "a")
    c.rect(0, 8, 6, 14, "b"); c.rect(8, 8, 14, 14, "b")
    return c


# Steps two pixels tall, stamped on an even row: every cell of ">" is either
# fully white or fully background (no thin half blocks, which some consoles
# draw a little off and leave loose white lines).
CHEVRON = [
    "##......",
    "##......",
    "..##....",
    "..##....",
    "....##..",
    "....##..",
    "..##....",
    "..##....",
    "##......",
    "##......",
]


@art("badge-powershell", 75, {"n": 25, "w": 231}, holes="w")
def _():
    # Windows PowerShell 5.1: upright navy console window with ">_".
    c = Canvas(20, 16)
    c.rrect(0, 0, 20, 16, 1.6, "n")
    c.stamp(3, 2, CHEVRON, "w")
    c.rect(10, 10, 17, 11.9, "w")
    return c


@art("badge-pwsh", 39, {"c": 45, "d": 33, "w": 231}, holes="w")
def _():
    # PowerShell 7: slanted parallelogram, light-to-deep blue, ">_".
    c = Canvas(24, 16)
    c.poly([(8, 0), (24, 0), (16, 16), (0, 16)], "d")
    c.fill(lambda x, y: y < 8 and c.p[int(y)][int(x)] == "d", "c")
    c.stamp(6, 2, CHEVRON, "w")
    c.poly([(11.5, 10), (18.5, 10), (17.5, 12), (10.5, 12)], "w")
    return c


@art("badge-git", 202, {"o": 202, "w": 231}, holes="w")
def _():
    c = Canvas(20, 20)
    c.fill(lambda x, y: abs(x - 10) + abs(y - 10) <= 10 and max(abs(x - 10), abs(y - 10)) <= 9.0, "o")
    a, b, d = (8.6, 7.6), (8.6, 14.2), (13.6, 11.2)
    c.line([(4.6, 3.6), a], 1.0, "w")    # incoming history
    c.line([a, b], 1.0, "w")             # trunk
    c.line([a, d], 1.0, "w")             # branch
    for p in (a, b, d):
        c.disk(p[0], p[1], 1.8, "w")
    return c


M_GLYPH = [
    "##......##",
    "###....###",
    "####..####",
    "##.####.##",
    "##..##..##",
    "##......##",
    "##......##",
    "##......##",
]


# One badge per MSYS2 environment, in the colours of its icon:
# (environment, accent, top colour, bottom colour).
MSYS2_ENVS = [
    ("msys2", 98, 98, 134),               # MSYS: purple
    ("msys2-ucrt64", 137, 137, 94),       # UCRT64: brown
    ("msys2-mingw64", 75, 26, 19),        # MINGW64: blue
    ("msys2-mingw32", 248, 248, 242),     # MINGW32: grey
    ("msys2-clang64", 208, 208, 166),     # CLANG64: orange
    ("msys2-clangarm64", 70, 70, 28),     # CLANGARM64: green
]


def msys2_badge(env, accent, top, bottom):
    @art("badge-" + env, accent, {"m": top, "n": bottom, "w": 231}, holes="w")
    def _():
        c = Canvas(18, 16)
        c.rrect(0, 0, 18, 16, 3.5, "m")
        c.fill(lambda x, y: c.p[int(y)][int(x)] == "m" and y > 9, "n")
        c.stamp(4, 4, M_GLYPH, "w")
        return c


for _env in MSYS2_ENVS:
    msys2_badge(*_env)


@art("badge-cygwin", 34, {"g": 34})
def _():
    c = Canvas(18, 18)
    c.ring(9, 9, 3.6, 8.8, "g", gaps=[(0, 42)])
    return c


# ---------------------------------------------------------------- Linux
@art("linux", 250, {"k": 240, "w": 231, "y": 220}, holes="w")
def _():
    c = Canvas(26, 30)
    c.ellipse(13, 18.5, 10.5, 10.5, "k")          # body + flippers
    c.ellipse(13, 8, 6.8, 7.5, "k")               # head
    c.ellipse(13, 19.5, 6.8, 8.6, "w")            # belly
    c.ellipse(10.6, 6.8, 1.8, 2.4, "w"); c.ellipse(15.4, 6.8, 1.8, 2.4, "w")
    c.disk(11.0, 7.4, 0.9, "k"); c.disk(15.0, 7.4, 0.9, "k")
    c.ellipse(13, 11.3, 3.6, 1.7, "y")            # beak
    c.ellipse(6.5, 27.5, 5.5, 2.4, "y"); c.ellipse(19.5, 27.5, 5.5, 2.4, "y")  # feet
    return c


@art("ubuntu", 202, {"o": 202, "w": 231}, holes="w")
def _():
    c = Canvas(30, 30)
    c.disk(15, 15, 15, "o")
    angles = (0, 120, 240)
    c.ring(15, 15, 5.4, 8.0, "w", gaps=[(a, 24) for a in angles])
    for a in angles:
        x, y = polar(15, 15, 11.0, a)
        c.disk(x, y, 2.7, "w")
    return c


@art("debian", 161, {"r": 161})
def _():
    c = Canvas(28, 30)
    cx, cy = 15.0, 13.5
    pts = []
    th = 0.0
    while th <= 2 * math.pi * 1.62:
        r = 2.2 + th * 1.12
        t = 0.7 + th * 0.2
        x, y = cx + r * math.cos(-th + 2.3), cy + r * math.sin(-th + 2.3)
        pts.append((x, y, t))
        th += 0.04
    # tail sweeping down-left
    lx, ly, lt = pts[-1]
    for i in range(1, 40):
        k = i / 40
        pts.append((lx - 1.5 * k - 4 * k * k, ly + 9 * k, lt * (1 - 0.85 * k)))
    c.blobs(pts, "r")
    return c


@art("fedora", 32, {"b": 32, "w": 231}, holes="w")
def _():
    c = Canvas(28, 28)
    c.disk(14, 14, 14, "b")
    c.rrect(14, 14, 28, 28, 2.5, "b")             # speech-bubble corner
    c.line([(12.5, 23.5), (12.5, 11), (13.2, 8.6), (15.2, 7.2), (17.6, 7.0), (19.2, 7.8)], 1.6, "w")
    c.line([(8.8, 15.2), (16.8, 15.2)], 1.25, "w")
    c.line([(12.5, 21.0), (11.6, 23.6), (9.4, 24.6), (7.6, 23.8)], 1.6, "w")
    return c


@art("arch", 39, {"c": 39})
def _():
    c = Canvas(30, 28)
    c.poly([(15, 0), (21, 12), (30, 28), (0, 28), (9, 12)], "c")
    c.poly([(15, 11.5), (17.5, 15), (20, 21), (22.5, 28.5), (7.5, 28.5), (10, 21), (12.5, 15)], None)
    return c


@art("mint", 113, {"g": 113, "w": 231}, holes="w")
def _():
    c = Canvas(28, 26)
    c.rrect(0, 0, 28, 26, 6, "g")
    c.rect(0, 0, 7, 7, "g")
    c.line([(6, 3.5), (6, 17), (7.2, 19.6), (9.8, 20.8), (20.2, 20.8), (22.8, 19.6), (23.6, 17), (23.6, 11.5)], 1.2, "w")
    c.line([(11.2, 17.5), (11.2, 11.5), (12.2, 9.4), (14.1, 8.6), (16.0, 9.4), (17.1, 11.5), (17.1, 16.5)], 1.2, "w")
    c.line([(17.1, 11.5), (18.2, 9.4), (20.2, 8.6), (22.4, 9.4), (23.6, 11.5)], 1.2, "w")
    return c


@art("manjaro", 35, {"g": 35})
def _():
    c = Canvas(26, 26)
    c.rect(0, 0, 17, 8, "g"); c.rect(0, 0, 8, 26, "g")
    c.rect(10, 10, 17, 26, "g")
    c.rect(19, 0, 26, 26, "g")
    return c


@art("alpine", 38, {"b": 24, "w": 231}, holes="w")
def _():
    c = Canvas(30, 26)
    c.poly([(7.5, 0), (22.5, 0), (30, 13), (22.5, 26), (7.5, 26), (0, 13)], "b")
    c.line([(4.5, 18.5), (11.5, 9.5), (18.8, 18.5)], 1.35, "w")
    c.line([(14.3, 13.0), (18.6, 9.5), (25.5, 18.5)], 1.35, "w")
    return c


@art("opensuse", 112, {"g": 112, "w": 231, "d": 28}, holes="w")
def _():
    c = Canvas(32, 22)
    c.ellipse(15, 13, 11, 8.5, "g", clip=lambda x, y: y <= 16.5)   # body dome
    c.poly([(21, 5.5), (29, 8), (31.5, 12), (29, 15), (22, 16)], "g")  # head
    c.ring(6, 14, 1.6, 4.0, "g", gaps=[(15, 50)])                  # curled tail
    c.rect(8, 16, 11, 19, "g"); c.rect(18, 16, 21, 19, "g")        # legs
    c.disk(26.5, 9.8, 2.0, "w"); c.disk(27.0, 10.0, 0.9, "d")      # eye
    return c


@art("gentoo", 141, {"p": 141, "q": 97, "w": 231}, holes="w")
def _():
    c = Canvas(26, 28)
    c.ellipse(14.5, 10.5, 11, 8.6, "p", rot=-0.45)
    c.poly([(7.5, 13), (3, 27), (15, 18)], "p")
    c.ellipse(14.5, 10.5, 11, 8.6, "q", rot=-0.45, clip=lambda x, y: y > 13.5 and x > 12)
    c.ellipse(16.2, 9.4, 3.8, 2.6, "w", rot=-0.45)
    return c


@art("pop", 74, {"c": 74, "w": 231}, holes="w")
def _():
    c = Canvas(28, 28)
    c.disk(14, 14, 14, "c")
    c.line([(16.8, 6.0), (12.6, 16.8)], 2.1, "w")
    c.disk(11.2, 21.0, 1.9, "w")
    c.line([(15.4, 21.4), (21.0, 21.4)], 1.1, "w")
    return c


@art("redhat", 160, {"r": 160, "k": 88})
def _():
    c = Canvas(32, 20)
    c.ellipse(16, 15, 15.5, 4.6, "r")
    c.ellipse(16.5, 9.5, 8.5, 8.0, "r", clip=lambda x, y: y <= 14)
    c.line([(8.5, 12.6), (24.5, 12.6)], 1.0, "k")
    return c


@art("almalinux", 75, {"r": 203, "y": 220, "g": 112, "c": 39, "b": 26})
def _():
    # Five curved petals around the centre, like a flower.
    c = Canvas(30, 30)
    for k, col in enumerate("rygbc"):
        a = 108 + k * 72
        px, py = polar(15, 15, 8.6, a)
        c.ellipse(px, py, 6.6, 3.7, col, rot=-math.radians(a + 90 - 18))
    return c


@art("rocky", 36, {"g": 36, "w": 231}, holes="w")
def _():
    # Green disk crossed by a white mountain line.
    c = Canvas(30, 30)
    c.disk(15, 15, 15, "g")
    c.line([(0, 21.5), (10.5, 12.0), (15.5, 17.0), (19.5, 13.5), (30, 24.5)], 1.25, "w")
    c.fill(lambda x, y: c.p[int(y)][int(x)] == "w" and math.hypot(x - 15, y - 15) > 13.4, "g")
    c.fill(lambda x, y: math.hypot(x - 15, y - 15) > 15, None)
    return c


@art("centos", 214, {"y": 214, "g": 148, "p": 127, "b": 61})
def _():
    c = Canvas(28, 28)
    for (cx, cy, col) in [(14, 6.6, "y"), (21.4, 14, "g"), (14, 21.4, "p"), (6.6, 14, "b")]:
        c.fill(lambda x, y, cx=cx, cy=cy: abs(x - cx) + abs(y - cy) <= 6.3, col)
    return c


@art("void", 71, {"g": 71, "h": 65})
def _():
    c = Canvas(28, 28)
    c.ring(14, 14, 9.0, 14, "g")
    c.disk(14, 14, 6.2, "h")
    c.fill(lambda x, y: abs((x - 14) + (y - 14)) / math.sqrt(2) < 1.4, None)
    return c


@art("nixos", 68, {"a": 68, "b": 110})
def _():
    c = Canvas(30, 30)
    for k in range(6):
        th = k * 60
        n = th + 90                      # bar normal
        mx, my = polar(15, 15, 5.2, n)   # bar passes at distance 5.2 from the centre
        s, e = polar(mx, my, -5.5, th), polar(mx, my, 9.5, th)
        c.line([s, e], 1.55, "a" if k % 2 == 0 else "b")
    return c


# ---------------------------------------------------------------- output
def overlay(base, badge, at=None):
    """Same rule as the Rust code: badge in the bottom-right corner, 1-px gap."""
    bw, bh = len(badge[0]), len(badge)
    # y stays even, so the badge starts on a terminal line.
    ox, oy = at or (len(base[0]) - bw * 2 // 3, (len(base) - bh * 2 // 3) & ~1)
    W, H = max(len(base[0]), ox + bw), max(len(base), oy + bh)
    g = [["."] * W for _ in range(H)]
    for y, r in enumerate(base):
        for x, ch in enumerate(r):
            g[y][x] = ch
    for y in range(H):
        for x in range(W):
            if ox - 1 <= x <= ox + bw and oy - 1 <= y <= oy + bh:
                g[y][x] = "."
    for y, r in enumerate(badge):
        for x, ch in enumerate(r):
            if ch != ".":
                g[oy + y][ox + x] = ch.upper() if ch.islower() else ch
    return ["".join(r) for r in g]


XTERM = []
for i in range(16):
    XTERM.append([(0, 0, 0), (128, 0, 0), (0, 128, 0), (128, 128, 0), (0, 0, 128), (128, 0, 128), (0, 128, 128), (192, 192, 192),
                  (128, 128, 128), (255, 0, 0), (0, 255, 0), (255, 255, 0), (0, 0, 255), (255, 0, 255), (0, 255, 255), (255, 255, 255)][i])
lv = [0, 95, 135, 175, 215, 255]
for r in lv:
    for g in lv:
        for b in lv:
            XTERM.append((r, g, b))
for i in range(24):
    v = 8 + 10 * i
    XTERM.append((v, v, v))


def preview(path):
    by = {n: (a, p, h, cv.rows()) for n, a, p, h, cv in ARTS}
    items = [(n, p, rows) for n, (a, p, h, rows) in by.items()]
    for b in ["powershell", "pwsh", "git", "cygwin"] + [e[0] for e in MSYS2_ENVS]:
        _, bp, _, brows = by["badge-" + b]
        pal = dict(by["windows"][1]); pal.update({k.upper(): v for k, v in bp.items()})
        items.append(("windows-" + b, pal, overlay(by["windows"][3], brows, (16, 16))))
    _, wp, _, wrows = by["badge-winsmall"]
    for d in ["ubuntu", "debian"]:
        pal = dict(by[d][1]); pal.update({k.upper(): v for k, v in wp.items()})
        items.append((d + "-wsl", pal, overlay(by[d][3], wrows)))
    from PIL import Image, ImageDraw
    S, cols, cell = 6, 6, 40 * 6
    img = Image.new("RGB", (cols * cell, ((len(items) + cols - 1) // cols) * (cell + 16)), (30, 30, 30))
    d = ImageDraw.Draw(img)
    for i, (n, pal, rows) in enumerate(items):
        ox, oy = (i % cols) * cell + 8, (i // cols) * (cell + 16) + 4
        d.text((ox, oy), n, fill=(220, 220, 220))
        for y, r in enumerate(rows):
            for x, ch in enumerate(r):
                if ch != ".":
                    d.rectangle([ox + x * S, oy + 14 + y * S, ox + x * S + S - 1, oy + 14 + y * S + S - 1], fill=XTERM[pal[ch]])
    img.save(path)


def rust(path):
    o = ["//! Pixel-art logos. Generated, then kept as plain data: one character per pixel,",
         "//! `.` is transparent, each letter is a colour from the logo's palette",
         "//! (xterm-256 indices). Two pixel rows make one terminal line.",
         "//! `holes` are the light details that become transparent without colours.",
         "",
         "pub struct Art {",
         "    pub name: &'static str,",
         "    pub accent: u8,",
         "    pub palette: &'static [(char, u8)],",
         "    pub holes: &'static str,",
         "    pub rows: &'static [&'static str],",
         "}",
         "",
         "pub const ARTS: &[Art] = &["]
    for n, a, p, h, cv in ARTS:
        pal = ", ".join(f"('{k}', {v})" for k, v in p.items())
        o.append("    Art {")
        o.append(f'        name: "{n}",')
        o.append(f"        accent: {a},")
        o.append(f"        palette: &[{pal}],")
        o.append(f'        holes: "{h}",')
        o.append("        rows: &[")
        for r in cv.rows():
            o.append(f'            "{r}",')
        o.append("        ],")
        o.append("    },")
    o.append("];")
    Path(path).write_text("\n".join(o) + "\n", encoding="utf-8")


if __name__ == "__main__":
    preview(sys.argv[1])
    if len(sys.argv) > 2:
        rust(sys.argv[2])

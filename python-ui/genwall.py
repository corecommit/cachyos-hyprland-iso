#!/usr/bin/env python3
"""Random dark wallpaper (pure python, no dependencies).

usage: genwall.py png OUT.png [--style glow|aurora|mesh] [--seed N] [--size WxH]

Three looks (picked at random unless --style is given):
  glow    soft coloured light blobs on a dark gradient
  aurora  flowing ribbons of light
  mesh    smooth multi-colour gradient, gently warped
Colours come from a random colour harmony, so they always go together. A light vignette
and fine grain stop colour banding when the picture is scaled up.
The seed is the current time unless --seed is given, so every run is different. The file
is written next to its destination and moved over the old one: the old picture is deleted
and only one copy ever exists.
"""
import colorsys, math, os, random, struct, sys, time, zlib

W, H = 512, 288   # small on purpose: fast to make; smooth pictures scale up cleanly


def hsv(h, s, v):
    return [255.0 * c for c in colorsys.hsv_to_rgb(h % 1.0, min(1.0, max(0.0, s)), min(1.0, max(0.0, v)))]


def palette(rnd):
    """(background colour, [accent colours]) from a random colour harmony."""
    hue = rnd.random()
    offsets = rnd.choice([
        (0.0, 0.06, -0.06, 0.12),        # analogous: neighbours
        (0.0, 0.5, 0.04, 0.46),          # complementary
        (0.0, 0.333, 0.667, 0.05),       # triadic
        (0.0, 0.42, 0.58, 0.08),         # split complementary
    ])
    bg = hsv(hue, rnd.uniform(0.45, 0.75), rnd.uniform(0.045, 0.09))
    accents = [hsv(hue + o, rnd.uniform(0.40, 0.80), rnd.uniform(0.60, 0.98)) for o in offsets]
    rnd.shuffle(accents)
    return bg, accents


def mix(a, b, t):
    return [a[i] + (b[i] - a[i]) * t for i in range(3)]


def style_glow(rnd, w, h):
    bg, accents = palette(rnd)
    top = mix(bg, accents[0], 0.07)
    blobs = []
    for i in range(rnd.randint(4, 6)):
        col = accents[i % len(accents)]
        cx, cy = rnd.uniform(0.05, 0.95), rnd.uniform(0.05, 0.95)
        rx, ry = rnd.uniform(0.10, 0.30), rnd.uniform(0.10, 0.34)
        s = rnd.uniform(0.40, 0.85)
        ex = [s * math.exp(-((x / w - cx) * w / h) ** 2 / (2 * rx * rx)) for x in range(w)]
        ey = [math.exp(-(y / h - cy) ** 2 / (2 * ry * ry)) for y in range(h)]
        blobs.append((col, ex, ey))
    for y in range(h):
        row = []
        base = mix(top, bg, y / (h - 1))
        for x in range(w):
            r, g, b = base
            for col, ex, ey in blobs:
                k = ex[x] * ey[y]
                r += (col[0] - r) * k
                g += (col[1] - g) * k
                b += (col[2] - b) * k
            row.append((r, g, b))
        yield row


def style_aurora(rnd, w, h):
    bg, accents = palette(rnd)
    ribbons = []
    for i in range(rnd.randint(3, 4)):
        ribbons.append((
            accents[i % len(accents)],
            rnd.uniform(0.25, 0.75),               # centre line height
            rnd.uniform(0.05, 0.14),               # wave size
            rnd.uniform(1.0, 2.6), rnd.uniform(0, 6.28),    # wave 1: frequency, phase
            rnd.uniform(2.5, 5.0), rnd.uniform(0, 6.28),    # wave 2
            rnd.uniform(0.045, 0.10),              # thickness
            rnd.uniform(0.5, 1.0),                 # brightness
            rnd.uniform(0.8, 2.0), rnd.uniform(0, 6.28),    # slow brightness change along x
        ))
    cols = []
    for (col, cy, amp, f1, p1, f2, p2, th, br, fb, pb) in ribbons:
        centre, bright = [], []
        for x in range(w):
            t = x / w
            centre.append(cy + amp * math.sin(f1 * 6.283 * t + p1) + amp * 0.4 * math.sin(f2 * 6.283 * t + p2))
            bright.append(br * (0.55 + 0.45 * math.sin(fb * 6.283 * t + pb)))
        cols.append((col, centre, bright, th))
    for y in range(h):
        v = y / h
        base = mix(mix(bg, accents[0], 0.05), bg, v)
        row = []
        for x in range(w):
            r, g, b = base
            for col, centre, bright, th in cols:
                d = (v - centre[x]) / th
                if d > 3.2 or d < -3.2:
                    continue
                # sharper on the lower edge, soft glow upwards: like a curtain of light
                k = math.exp(-d * d * (0.9 if d > 0 else 0.35)) * bright[x] * 0.8
                r += (col[0] - r) * k
                g += (col[1] - g) * k
                b += (col[2] - b) * k
            row.append((r, g, b))
        yield row


def style_mesh(rnd, w, h):
    bg, accents = palette(rnd)
    corners = [mix(bg, accents[i % len(accents)], rnd.uniform(0.35, 0.75)) for i in range(4)]
    wa, wb = rnd.uniform(1.0, 2.2), rnd.uniform(1.0, 2.2)
    pa, pb = rnd.uniform(0, 6.28), rnd.uniform(0, 6.28)
    amp = rnd.uniform(0.10, 0.22)
    for y in range(h):
        v = y / (h - 1)
        row = []
        for x in range(w):
            u = x / (w - 1)
            uu = min(1.0, max(0.0, u + amp * math.sin(wa * 6.283 * v + pa)))
            vv = min(1.0, max(0.0, v + amp * math.sin(wb * 6.283 * u + pb)))
            uu = uu * uu * (3 - 2 * uu)     # smoothstep: no visible grid lines
            vv = vv * vv * (3 - 2 * vv)
            top = mix(corners[0], corners[1], uu)
            bot = mix(corners[2], corners[3], uu)
            c = mix(top, bot, vv)
            row.append((c[0] * 0.55, c[1] * 0.55, c[2] * 0.55))   # keep it dark: it is a desktop
        yield row


STYLES = {"glow": style_glow, "aurora": style_aurora, "mesh": style_mesh}


def render(seed, style=None, w=W, h=H):
    rnd = random.Random(seed)
    name = style if style in STYLES else rnd.choice(sorted(STYLES))
    grain = [rnd.uniform(-2.6, 2.6) for _ in range(4096)]
    raw = bytearray()
    cx, cy = (w - 1) / 2.0, (h - 1) / 2.0
    for y, row in enumerate(STYLES[name](rnd, w, h)):
        raw.append(0)
        dy = (y - cy) / cy
        for x, (r, g, b) in enumerate(row):
            dx = (x - cx) / cx
            vig = 1.0 - 0.30 * max(0.0, (dx * dx + dy * dy) / 2.0)   # darker towards the corners
            n = grain[(x * 31 + y * 17) & 4095]
            raw += bytes((min(255, max(0, int(r * vig + n))),
                          min(255, max(0, int(g * vig + n))),
                          min(255, max(0, int(b * vig + n)))))

    def chunk(t, d):
        c = struct.pack(">I", len(d)) + t + d
        return c + struct.pack(">I", zlib.crc32(t + d) & 0xFFFFFFFF)

    return (b"\x89PNG\r\n\x1a\n"
            + chunk(b"IHDR", struct.pack(">IIBBBBB", w, h, 8, 2, 0, 0, 0))
            + chunk(b"IDAT", zlib.compress(bytes(raw), 6))
            + chunk(b"IEND", b""))


def write_atomic(path, data):
    os.makedirs(os.path.dirname(os.path.abspath(path)), exist_ok=True)
    tmp = path + ".tmp"
    with open(tmp, "wb") as f:
        f.write(data)
    os.replace(tmp, path)


def main():
    a = sys.argv[1:]
    if len(a) < 2 or a[0] != "png":
        sys.exit(__doc__)
    out, seed, style, w, h = a[1], time.time_ns(), None, W, H
    i = 2
    try:
        while i < len(a):
            if a[i] == "--seed":
                seed = int(a[i + 1])
            elif a[i] == "--style" and a[i + 1] in STYLES:
                style = a[i + 1]
            elif a[i] == "--size":
                w, h = (int(n) for n in a[i + 1].lower().split("x"))
            else:
                raise ValueError(a[i])
            i += 2
    except (ValueError, IndexError):
        sys.exit(__doc__)
    write_atomic(out, render(seed, style, w, h))


if __name__ == "__main__":
    main()

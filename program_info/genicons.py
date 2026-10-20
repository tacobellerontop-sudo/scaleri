#!/usr/bin/env python3
# Generates every Scaleri logo and icon in this folder from the pixel art defined below.
# Requires rsvg-convert (librsvg) for the raster outputs. Run from anywhere:
#   python3 program_info/genicons.py
import os
import struct
import subprocess
import tempfile

HERE = os.path.dirname(os.path.abspath(__file__))
APPID = "io.github.tacobellerontop_sudo.Scaleri"

# 12x12 "S" glyph, drawn on a 16x16 pixel grid with a 2px margin
GLYPH = [
    "..########..",
    ".##########.",
    "###......##.",
    "###.........",
    "###.........",
    ".#########..",
    "..#########.",
    ".........###",
    ".........###",
    ".##......###",
    ".##########.",
    "..########..",
]

# 5x7 pixel font for the wordmark (rendered bold)
FONT = {
    "S": [".####", "#....", "#....", ".###.", "....#", "....#", "####."],
    "C": [".####", "#....", "#....", "#....", "#....", "#....", ".####"],
    "A": [".###.", "#...#", "#...#", "#####", "#...#", "#...#", "#...#"],
    "L": ["#....", "#....", "#....", "#....", "#....", "#....", "#####"],
    "E": ["#####", "#....", "#....", "####.", "#....", "#....", "#####"],
    "R": ["####.", "#...#", "#...#", "####.", "#.#..", "#..#.", "#...#"],
    "I": ["###", ".#.", ".#.", ".#.", ".#.", ".#.", "###"],
}

GRASS_TOP = "#9be15d"
GRASS_LEFT = "#7cc444"
GRASS_FACE = "#5fae34"
GRASS_BOTTOM = "#4a8a2a"
GRASS_SHADOW = "#1d4f12"


def glyph_pixels():
    return {(2 + x, 2 + y) for y, row in enumerate(GLYPH) for x, ch in enumerate(row) if ch == "#"}


def shade(pixels, grid, bounds=None):
    for x, y in pixels:
        s = (x + 1, y + 1)
        if s not in pixels and (bounds is None or s in bounds):
            grid[s] = GRASS_SHADOW
    for x, y in pixels:
        if (x, y - 1) not in pixels:
            grid[(x, y)] = GRASS_TOP
        elif (x - 1, y) not in pixels:
            grid[(x, y)] = GRASS_LEFT
        elif (x, y + 1) not in pixels:
            grid[(x, y)] = GRASS_BOTTOM
        else:
            grid[(x, y)] = GRASS_FACE


def tile_grid():
    """A deepslate tile with pixel-stepped corners and a bevelled edge."""
    corners = {(0, 0), (1, 0), (0, 1), (15, 0), (14, 0), (15, 1), (0, 15), (1, 15), (0, 14), (15, 15), (14, 15), (15, 14)}
    inner_corners = {(1, 1), (14, 1), (1, 14), (14, 14)}
    grid = {}
    for y in range(16):
        for x in range(16):
            if (x, y) in corners:
                continue
            if x in (0, 15) or y in (0, 15) or (x, y) in inner_corners:
                c = "#121213"
            elif y <= 1 or x <= 1:
                c = "#4a4b4d"
            elif y >= 14 or x >= 14:
                c = "#232324"
            else:
                h = (x * 7 + y * 13 + x * y) % 11
                c = "#333436" if h < 3 else ("#2d2e30" if h < 6 else "#303133")
            grid[(x, y)] = c
    return grid


def rects(grid, indent=" "):
    return "\n".join(
        f'{indent}<rect x="{x}" y="{y}" width="1" height="1" fill="{c}"/>'
        for (x, y), c in sorted(grid.items(), key=lambda k: (k[0][1], k[0][0]))
    )


def svg(width, height, viewbox, body):
    return (
        f'<svg xmlns="http://www.w3.org/2000/svg" width="{width}" height="{height}" viewBox="{viewbox}" shape-rendering="crispEdges">\n'
        f" <title>Scaleri Logo</title>\n{body}\n</svg>\n"
    )


def icon_grid():
    grid = tile_grid()
    shade(glyph_pixels(), grid, bounds=set(grid))
    return grid


def glyph_grid():
    grid = {}
    shade(glyph_pixels(), grid)
    return grid


def wordmark_grid(text):
    px = set()
    x0 = 0
    for ch in text:
        g = FONT[ch]
        for y, row in enumerate(g):
            for x, c in enumerate(row):
                if c == "#":
                    px.update({(x0 + x, y), (x0 + x + 1, y)})
        x0 += len(g[0]) + 2
    grid = {}
    shade(px, grid)
    return grid, x0 + 1, 9


def write(name, data):
    with open(os.path.join(HERE, name), "w" if isinstance(data, str) else "wb") as f:
        f.write(data)


def render(svg_path, size, out):
    subprocess.run(["rsvg-convert", "-w", str(size), "-h", str(size), svg_path, "-o", out], check=True)
    with open(out, "rb") as f:
        return f.read()


def main():
    icon = svg(48, 48, "0 0 16 16", rects(icon_grid()))
    write(f"{APPID}.svg", icon)
    write("../launcher/resources/multimc/scalable/launcher.svg", icon)
    write("Scaleri.icon/Assets/glyph.svg", svg(48, 48, "0 0 16 16", rects(glyph_grid())))
    # macOS Big Sur grid: 824px artwork centred on a 1024px canvas
    write(f"{APPID}.bigsur.svg", svg(1024, 1024, "0 0 1024 1024", f' <g transform="translate(100 100) scale(51.5)">\n{rects(icon_grid(), "  ")}\n </g>'))

    words, w, h = wordmark_grid("SCALERI")
    scale = 16 / h * 0.8
    tx, ty = 19, (16 - h * scale) / 2
    total = tx + w * scale
    logo = svg(
        int(total * 12), 192, f"0 0 {total:.3f} 16",
        f'{rects(icon_grid())}\n <g transform="translate({tx} {ty:.3f}) scale({scale:.4f})">\n{rects(words, "  ")}\n </g>',
    )
    write(f"{APPID}.logo.svg", logo)
    write(f"{APPID}.logo-darkmode.svg", logo)

    with tempfile.TemporaryDirectory() as d:
        icon_svg = os.path.join(HERE, f"{APPID}.svg")
        bigsur_svg = os.path.join(HERE, f"{APPID}.bigsur.svg")
        write(f"{APPID}_256.png", render(icon_svg, 256, os.path.join(d, "256.png")))

        # Windows ICO with PNG-compressed entries
        sizes = [256, 128, 64, 48, 32, 24, 16]
        pngs = [render(icon_svg, s, os.path.join(d, f"ico{s}.png")) for s in sizes]
        header = struct.pack("<HHH", 0, 1, len(sizes))
        offset = 6 + 16 * len(sizes)
        entries = b""
        for s, p in zip(sizes, pngs):
            entries += struct.pack("<BBBBHHII", s % 256, s % 256, 0, 0, 1, 32, len(p), offset)
            offset += len(p)
        write("scaleri.ico", header + entries + b"".join(pngs))

        # macOS ICNS with PNG entries
        body = b""
        for kind, s in [(b"icp4", 16), (b"icp5", 32), (b"ic07", 128), (b"ic08", 256), (b"ic09", 512), (b"ic10", 1024)]:
            p = render(bigsur_svg, s, os.path.join(d, f"icns{s}.png"))
            body += kind + struct.pack(">I", len(p) + 8) + p
        write("scaleri.icns", b"icns" + struct.pack(">I", len(body) + 8) + body)


if __name__ == "__main__":
    main()

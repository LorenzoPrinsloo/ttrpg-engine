"""Draws the markup examples in docs/images by running them through the real
`render` command and turning its terminal colours into SVG.

Dice are rolled each time, so those numbers change when this is run again.

Run from the repository root after `cargo build`:  python3 docs/charts/markup.py
"""

import html
import os
import re
import subprocess
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
OUT = ROOT / "docs" / "images"
BINARY = ROOT / "target" / "debug" / "ttrpg-engine"

BACKGROUND = "#1e1f29"
PANEL = "#272936"
FOREGROUND = "#e6e6e6"
SOURCE = "#9aa0b4"
HEADER = "#c8cbe0"
# The 16 terminal colours, as a dark terminal theme shows them.
PALETTE = {
    30: "#4d4d4d", 31: "#ff5c57", 32: "#5af78e", 33: "#f3f99d", 34: "#57c7ff", 35: "#ff6ac1", 36: "#9aedfe", 37: "#f1f1f0",
    90: "#7f8496", 91: "#ff8a85", 92: "#8cfab0", 93: "#f7fbbd", 94: "#8ad7ff", 95: "#ff9ad6", 96: "#c2f4ff", 97: "#ffffff",
}
SGR = re.compile(r"\x1b\[([0-9;]*)m")

CHAR_WIDTH = 8.4
ROW_HEIGHT = 34
FONT = "Menlo, Consolas, 'DejaVu Sans Mono', monospace"


def render(markup):
    env = dict(os.environ, CLICOLOR_FORCE="1")
    result = subprocess.run([str(BINARY), "render", markup], capture_output=True, text=True, env=env, check=True)
    return result.stdout.rstrip("\n")


def spans(ansi):
    """Splits terminal output into (text, style) runs."""
    style = {"bold": False, "italic": False, "underline": False, "colour": None}
    runs, position = [], 0
    for match in SGR.finditer(ansi):
        if match.start() > position:
            runs.append((ansi[position:match.start()], dict(style)))
        codes = [int(c) for c in match.group(1).split(";") if c] or [0]
        i = 0
        while i < len(codes):
            code = codes[i]
            if code == 0:
                style = {"bold": False, "italic": False, "underline": False, "colour": None}
            elif code == 1:
                style["bold"] = True
            elif code == 3:
                style["italic"] = True
            elif code == 4:
                style["underline"] = True
            elif code in PALETTE:
                style["colour"] = PALETTE[code]
            elif code == 38 and codes[i + 1:i + 2] == [2]:
                r, g, b = codes[i + 2:i + 5]
                style["colour"] = f"#{r:02x}{g:02x}{b:02x}"
                i += 4
            i += 1
        position = match.end()
    if position < len(ansi):
        runs.append((ansi[position:], dict(style)))
    return runs


def tspans(runs):
    out = []
    for text, style in runs:
        attributes = [f'fill="{style["colour"] or FOREGROUND}"']
        if style["bold"]:
            attributes.append('font-weight="bold"')
        if style["italic"]:
            attributes.append('font-style="italic"')
        if style["underline"]:
            attributes.append('text-decoration="underline"')
        out.append(f'<tspan {" ".join(attributes)}>{html.escape(text)}</tspan>')
    return "".join(out)


def panel(name, title, examples):
    """A two-column card: the markup as written, and what the terminal shows."""
    source_width = max(len(markup) for _, markup in examples) * CHAR_WIDTH
    rendered = [render(markup) for _, markup in examples]
    output_width = max(len(SGR.sub("", text)) for text in rendered) * CHAR_WIDTH
    left, gap, top = 24, 40, 74
    width = int(left + source_width + gap + max(output_width, 140) + 24)
    height = top + ROW_HEIGHT * len(examples) + 16
    parts = [
        f'<svg xmlns="http://www.w3.org/2000/svg" width="{width}" height="{height}" viewBox="0 0 {width} {height}" font-family="{FONT}" font-size="14">',
        f'<rect width="{width}" height="{height}" rx="10" fill="{BACKGROUND}"/>',
        f'<text x="{left}" y="30" fill="{HEADER}" font-size="15" font-weight="bold" font-family="Helvetica, Arial, sans-serif">{html.escape(title)}</text>',
        f'<text x="{left}" y="56" fill="{SOURCE}" font-size="12" font-family="Helvetica, Arial, sans-serif">you write</text>',
        f'<text x="{left + source_width + gap}" y="56" fill="{SOURCE}" font-size="12" font-family="Helvetica, Arial, sans-serif">the terminal shows</text>',
    ]
    for i, ((_, markup), shown) in enumerate(zip(examples, rendered)):
        y = top + ROW_HEIGHT * i
        if i % 2 == 0:
            parts.append(f'<rect x="12" y="{y}" width="{width - 24}" height="{ROW_HEIGHT}" rx="4" fill="{PANEL}"/>')
        baseline = y + ROW_HEIGHT / 2 + 5
        parts.append(f'<text x="{left}" y="{baseline}" fill="{SOURCE}" xml:space="preserve">{html.escape(markup)}</text>')
        parts.append(f'<text x="{left + source_width + gap}" y="{baseline}" xml:space="preserve">{tspans(spans(shown))}</text>')
    parts.append("</svg>")
    OUT.mkdir(parents=True, exist_ok=True)
    (OUT / name).write_text("\n".join(parts) + "\n")
    print(f"wrote docs/images/{name}")


if __name__ == "__main__":
    panel("markup-styles.svg", "Styles", [
        ("bold", "**Black Marble**"),
        ("italic", "*Nothing* on the scopes."),
        ("underline", "reads back __your own transmission__"),
        ("nested", "**A shape, *barely* moving**"),
        ("combined", "__**Make a Fear Save.**__"),
        ("bold wins", "***x***"),
    ])
    panel("markup-colours.svg", "Colours", [
        ("named", "{red|red} {green|green} {yellow|yellow} {blue|blue} {purple|purple} {cyan|cyan}"),
        ("bright", "{bright red|bright red} {bright cyan|bright cyan} {bright white|bright white}"),
        ("extra names", "{grey|grey} {orange|orange} {pink|pink} {brown|brown}"),
        ("hex", "{#f80|#f80} {#4c78a8|#4c78a8} {#B279A2|#B279A2}"),
        ("with styles", "{bright cyan|**Something answers.**} Make a *Fear Save*."),
        ("styles outside", "**{red|blood} and {grey|ash}**"),
    ])
    panel("markup-dice.svg", "Dice, rolled when the entry is shown", [
        ("one die", "[1d10] Escaped Child Android(s)"),
        ("modifiers", "Damage: [2d6+3]"),
        ("several terms", "Drift: [1d20-1d4+2] metres"),
        ("styled", "**[1d5] bodies** in the corridor"),
        ("coloured", "Everyone Close takes {red|[1d5] Stress}."),
        ("in prose", "worth **[1d5]mcr**"),
    ])
    panel("markup-literal.svg", "Escapes and mistakes show as written", [
        ("escaped", r"\*not italic\* and \[1d6\]"),
        ("unclosed", "**never closed"),
        ("unknown colour", "{ultraviolet|text}"),
        ("bad dice", "[2d0] and [1d]"),
        ("too many dice", "[5000d6]"),
        ("markup in dice", "[**1d6**]"),
    ])

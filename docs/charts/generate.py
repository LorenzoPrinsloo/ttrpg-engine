"""Draws the charts in docs/images.

Each chart is computed exactly or simulated with a fixed seed, so running this
again gives the same pictures. Where a chart shows something the engine does,
the model here follows the engine's rules: a bag holds marbles in its ratios
(reduced by their greatest common divisor) and draws without replacement; a
unique bag also removes every copy of what was drawn; a table rolls uniformly
over its rows.

Run from the repository root:  python3 docs/charts/generate.py
"""

import math
import random
import tomllib
from functools import reduce
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
OUT = ROOT / "docs" / "images"

INK = "#2b2b2b"
MUTED = "#777777"
GRID = "#e6e6e6"
COLOURS = {
    "stateless": "#4c78a8",
    "bag": "#e45756",
    "pity": "#f2a33a",
    "ratchet": "#54a24b",
    "cautious": "#9ecae9",
    "standard": "#4c78a8",
    "grinder": "#1d3557",
    "polya": "#b279a2",
}


def fmt(x):
    return f"{x:.2f}"


def svg(width, height, body):
    return (
        f'<svg xmlns="http://www.w3.org/2000/svg" width="{width}" height="{height}" '
        f'viewBox="0 0 {width} {height}" font-family="Helvetica, Arial, sans-serif">\n'
        f'<rect width="{width}" height="{height}" fill="#ffffff"/>\n{body}\n</svg>\n'
    )


def text(x, y, s, size=12, anchor="start", weight="normal", fill=INK, rotate=None):
    transform = f' transform="rotate({rotate} {fmt(x)} {fmt(y)})"' if rotate else ""
    return (
        f'<text x="{fmt(x)}" y="{fmt(y)}" font-size="{size}" text-anchor="{anchor}" '
        f'font-weight="{weight}" fill="{fill}"{transform}>{s}</text>'
    )


class Frame:
    """A plot area with axes; maps data coordinates to pixels."""

    def __init__(self, width, height, x_range, y_range, left=70, right=30, top=56, bottom=70):
        self.width, self.height = width, height
        self.x0, self.x1 = x_range
        self.y0, self.y1 = y_range
        self.left, self.top = left, top
        self.w = width - left - right
        self.h = height - top - bottom

    def x(self, v):
        return self.left + (v - self.x0) / (self.x1 - self.x0) * self.w

    def y(self, v):
        return self.top + self.h - (v - self.y0) / (self.y1 - self.y0) * self.h

    def axes(self, title, x_label, y_label, x_ticks, y_ticks, y_format=lambda v: f"{v:.0%}"):
        parts = [text(20, 30, title, 16, weight="bold")]
        for t in y_ticks:
            parts.append(f'<line x1="{fmt(self.left)}" y1="{fmt(self.y(t))}" x2="{fmt(self.left + self.w)}" y2="{fmt(self.y(t))}" stroke="{GRID}"/>')
            parts.append(text(self.left - 8, self.y(t) + 4, y_format(t), 11, "end", fill=MUTED))
        for t in x_ticks:
            parts.append(text(self.x(t), self.top + self.h + 18, str(t), 11, "middle", fill=MUTED))
        parts.append(f'<line x1="{fmt(self.left)}" y1="{fmt(self.top + self.h)}" x2="{fmt(self.left + self.w)}" y2="{fmt(self.top + self.h)}" stroke="{MUTED}"/>')
        parts.append(text(self.left + self.w / 2, self.top + self.h + 40, x_label, 12, "middle", fill=MUTED))
        parts.append(text(18, self.top + self.h / 2, y_label, 12, "middle", fill=MUTED, rotate=-90))
        return "\n".join(parts)

    def line(self, points, colour, width=2.5, dash=None):
        d = " ".join(f"{fmt(self.x(a))},{fmt(self.y(b))}" for a, b in points)
        dashed = f' stroke-dasharray="{dash}"' if dash else ""
        return f'<polyline points="{d}" fill="none" stroke="{colour}" stroke-width="{width}" stroke-linejoin="round"{dashed}/>'

    def bars(self, series, colour, offset, bar_width):
        return "\n".join(
            f'<rect x="{fmt(self.x(a) + offset)}" y="{fmt(self.y(b))}" width="{fmt(bar_width)}" '
            f'height="{fmt(self.y(0) - self.y(b))}" fill="{colour}"/>'
            for a, b in series if b > 0
        )

    def legend(self, entries):
        """Lays the entries out left to right, wrapping onto new rows, below the plot."""
        rows, row, x = [], [], self.left
        for label, colour in entries:
            width = 34 + 6.2 * len(label)
            if row and x + width > self.width - 10:
                rows.append(row)
                row, x = [], self.left
            row.append((x, label, colour))
            x += width
        rows.append(row)
        parts = []
        for i, row in enumerate(rows):
            y = self.height - 16 - 18 * (len(rows) - 1 - i)
            for x, label, colour in row:
                parts.append(f'<rect x="{fmt(x)}" y="{fmt(y - 9)}" width="14" height="10" fill="{colour}"/>')
                parts.append(text(x + 20, y, label, 11))
        return "\n".join(parts)


def write(name, content):
    OUT.mkdir(parents=True, exist_ok=True)
    (OUT / name).write_text(content)
    print(f"wrote docs/images/{name}")


# --- the mechanics -------------------------------------------------------

def bag_marbles(ratios):
    """The engine's starting bag: hundredths reduced by one shared divisor."""
    parts = {item: round(ratio * 100) for item, ratio in ratios.items()}
    divisor = reduce(math.gcd, parts.values(), 0) or 1
    return [item for item, part in parts.items() for _ in range(part // divisor)]


def pity_chance(misses):
    """1.5% on the first check and 1.5 points more after every miss, which averages
    one encounter in about ten checks like the others."""
    return min(1.0, 0.015 * (misses + 1))


def gap_distribution_stateless(p, max_gap):
    return {k: (1 - p) ** (k - 1) * p for k in range(1, max_gap + 1)}


def gap_distribution_pity(max_gap):
    dist, survive = {}, 1.0
    for k in range(1, max_gap + 1):
        chance = pity_chance(k - 1)
        dist[k] = survive * chance
        survive *= 1 - chance
    return dist


def gap_distribution_bag(size, max_gap):
    # One encounter token per bag of `size`, reshuffled when empty: the gap is
    # (size - first position) + second position, positions uniform in each bag.
    counts = {k: 0 for k in range(1, max_gap + 1)}
    for first in range(1, size + 1):
        for second in range(1, size + 1):
            gap = size - first + second
            counts[gap] = counts.get(gap, 0) + 1
    total = size * size
    return {k: v / total for k, v in counts.items() if k <= max_gap}


def binomial(n, k, p):
    return math.comb(n, k) * p**k * (1 - p) ** (n - k)


def dice_sum_distribution(n, sides):
    dist = {0: 1.0}
    for _ in range(n):
        nxt = {}
        for total, chance in dist.items():
            for face in range(1, sides + 1):
                nxt[total + face] = nxt.get(total + face, 0) + chance / sides
        dist = nxt
    return dist


# --- charts ----------------------------------------------------------------

def encounter_gaps():
    f = Frame(820, 400, (0.5, 30.5), (0, 0.12), bottom=90)
    stateless = gap_distribution_stateless(0.1, 30)
    bag = gap_distribution_bag(10, 30)
    pity = gap_distribution_pity(30)
    body = [f.axes("Checks between encounters, all averaging about one in ten", "checks since the last encounter", "chance", [1, 5, 10, 15, 20, 25, 30], [0, 0.04, 0.08, 0.12])]
    width = f.w / 30 / 3.4
    body.append(f.bars(sorted(stateless.items()), COLOURS["stateless"], -1.5 * width, width))
    body.append(f.bars(sorted(bag.items()), COLOURS["bag"], -0.5 * width, width))
    body.append(f.bars(sorted(pity.items()), COLOURS["pity"], 0.5 * width, width))
    body.append(f.legend([
        ("stateless d100 doubles: the most likely gap is 1", COLOURS["stateless"]),
        ("shuffle bag of 10: never more than 19", COLOURS["bag"]),
        ("pity timer: bunched around 10", COLOURS["pity"]),
    ]))
    write("encounter-gaps.svg", svg(820, 400, "\n".join(body)))


def encounters_per_ten():
    f = Frame(820, 360, (-0.5, 5.5), (0, 1.0))
    stateless = [(k, binomial(10, k, 0.1)) for k in range(0, 6)]
    bag = [(1, 1.0)]
    body = [f.axes("How many encounters in 10 checks", "encounters", "share of sessions", [0, 1, 2, 3, 4, 5], [0, 0.25, 0.5, 0.75, 1.0])]
    width = f.w / 6 / 3
    body.append(f.bars(stateless, COLOURS["stateless"], -width, width))
    body.append(f.bars(bag, COLOURS["bag"], 0, width))
    for k, p in stateless:
        if p > 0.005:
            body.append(text(f.x(k) - width / 2, f.y(p) - 5, f"{p:.0%}", 10, "middle", fill=MUTED))
    body.append(text(f.x(1) + width / 2, f.y(1.0) - 5, "100%", 10, "middle", fill=MUTED))
    body.append(f.legend([("stateless d100 doubles: 35% of the time, nothing at all", COLOURS["stateless"]), ("shuffle bag of 10: exactly one, every time", COLOURS["bag"])]))
    write("encounters-per-ten.svg", svg(820, 360, "\n".join(body)))


def due_effect():
    f = Frame(820, 380, (0, 9), (0, 1.0))
    stateless = [(n, 0.1) for n in range(0, 10)]
    bag = [(n, 1 / (10 - n)) for n in range(0, 10)]
    pity = [(n, pity_chance(n)) for n in range(0, 10)]
    body = [f.axes("Chance the next check is an encounter, after a quiet stretch", "quiet checks so far", "chance of an encounter next", list(range(0, 10)), [0, 0.25, 0.5, 0.75, 1.0])]
    body.append(f.line(stateless, COLOURS["stateless"]))
    body.append(f.line(bag, COLOURS["bag"]))
    body.append(f.line(pity, COLOURS["pity"]))
    body.append(text(f.x(8.6), f.y(0.1) - 8, "dice never remember", 11, "end", fill=COLOURS["stateless"]))
    body.append(text(f.x(8.7), f.y(1.0) + 16, "the bag makes it a certainty", 11, "end", fill=COLOURS["bag"]))
    body.append(f.legend([("stateless d100 doubles", COLOURS["stateless"]), ("shuffle bag of 10", COLOURS["bag"]), ("pity timer", COLOURS["pity"])]))
    write("due-effect.svg", svg(820, 380, "\n".join(body)))


def dice_sums():
    shapes = [((1, 20), "1d20: flat", COLOURS["stateless"]), ((2, 6), "2d6: a triangle", COLOURS["bag"]), ((3, 6), "3d6: a bell", COLOURS["ratchet"]), ((2, 10), "2d10: the panic check", COLOURS["grinder"])]
    f = Frame(820, 380, (0, 1), (0, 0.18))
    body = [f.axes("Adding dice changes the shape, not just the range", "roll, from lowest to highest possible", "chance", [], [0, 0.05, 0.10, 0.15])]
    for (n, sides), label, colour in shapes:
        dist = dice_sum_distribution(n, sides)
        low, high = n, n * sides
        points = [((total - low) / (high - low), chance) for total, chance in sorted(dist.items())]
        body.append(f.line(points, colour))
    body.append(text(f.x(0), f.top + f.h + 18, "lowest", 11, "start", fill=MUTED))
    body.append(text(f.x(1), f.top + f.h + 18, "highest", 11, "end", fill=MUTED))
    body.append(f.legend([(label, colour) for _, label, colour in shapes]))
    write("dice-sums.svg", svg(820, 380, "\n".join(body)))


def unique_bag():
    ratios = {}
    data = tomllib.loads((ROOT / "config" / "bags" / "artifacts.toml").read_text())
    for i, item in enumerate(data["items"]):
        ratios[i] = item["ratio"]
    rare = next(i for i, r in ratios.items() if abs(r - 0.01) < 1e-9)
    rng = random.Random(7)
    runs, draws = 20000, 30
    found_unique = [0] * (draws + 1)
    for _ in range(runs):
        marbles = bag_marbles(ratios)
        seen = False
        for k in range(1, draws + 1):
            drawn = marbles.pop(rng.randrange(len(marbles)))
            marbles = [m for m in marbles if m != drawn]
            seen = seen or drawn == rare
            found_unique[k] += seen
    found_dice = [(k, 1 - 0.99**k) for k in range(0, draws + 1)]
    f = Frame(820, 380, (0, draws), (0, 1.0))
    body = [f.axes("Finding a 1-in-100 artifact: a unique bag versus fresh rolls", "draws from the artifacts bag", "chance it has turned up", [0, 5, 10, 15, 20, 25, 30], [0, 0.25, 0.5, 0.75, 1.0])]
    body.append(f.line([(k, found_unique[k] / runs) for k in range(0, draws + 1)], COLOURS["bag"]))
    body.append(f.line(found_dice, COLOURS["stateless"]))
    body.append(f.legend([("unique bag (artifacts.toml): common finds leave, so its chance rises", COLOURS["bag"]), ("rolling fresh: 1% on every draw", COLOURS["stateless"])]))
    write("unique-bag.svg", svg(820, 380, "\n".join(body)))


def ratchet_and_vessel():
    beats, runs = 20, 20000
    rng = random.Random(11)

    def vessel(stress_chance, check_chance):
        paid = [0] * (beats + 1)
        for _ in range(runs):
            stress, first = 2, None
            for beat in range(1, beats + 1):
                if rng.random() < stress_chance:
                    stress += 1
                if rng.random() < check_chance:
                    roll = rng.randint(1, 10) + rng.randint(1, 10)
                    if roll <= stress:
                        first = first or beat
                        if 10 <= roll <= 15:
                            stress += 1
                        elif roll >= 16:
                            stress += 2
            if first:
                for b in range(first, beats + 1):
                    paid[b] += 1
        return [(b, paid[b] / runs) for b in range(0, beats + 1)]

    def ratchet(exposure_chance, threshold):
        paid = [0] * (beats + 1)
        for _ in range(runs):
            exposure = 0
            for beat in range(1, beats + 1):
                if rng.random() < exposure_chance:
                    exposure += 1
                if exposure >= threshold:
                    for b in range(beat, beats + 1):
                        paid[b] += 1
                    break
        return [(b, paid[b] / runs) for b in range(0, beats + 1)]

    f = Frame(820, 400, (0, beats), (0, 1.0))
    body = [f.axes("When does the horror pay off? A ratchet versus a pressure vessel", "story beat", "sessions with a payoff so far", [0, 5, 10, 15, 20], [0, 0.25, 0.5, 0.75, 1.0])]
    curves = [
        (vessel(0.3, 0.15), "vessel, cautious crew", COLOURS["cautious"]),
        (vessel(0.5, 0.25), "vessel, standard crew", COLOURS["standard"]),
        (vessel(0.7, 0.35), "vessel, meat grinder", COLOURS["grinder"]),
        (ratchet(0.6, 10), "ratchet (threshold 10)", COLOURS["ratchet"]),
    ]
    for points, _, colour in curves:
        body.append(f.line(points, colour))
    body.append(f.legend([(label, colour) for _, label, colour in curves]))
    write("ratchet-vs-vessel.svg", svg(820, 400, "\n".join(body)))
    return {label: points[-1][1] for points, label, _ in curves}


def urns():
    rng = random.Random(5)
    runs, draws = 20000, 20
    outcomes = {"without": [0] * (draws + 1), "with": [0] * (draws + 1), "polya": [0] * (draws + 1)}
    for _ in range(runs):
        tokens = ["red"] * 10 + ["blue"] * 10
        reds = sum(1 for _ in range(draws) if tokens.pop(rng.randrange(len(tokens))) == "red")
        outcomes["without"][reds] += 1
        reds = sum(1 for _ in range(draws) if rng.random() < 0.5)
        outcomes["with"][reds] += 1
        red, blue, reds = 1, 1, 0
        for _ in range(draws):
            if rng.random() < red / (red + blue):
                red, reds = red + 1, reds + 1
            else:
                blue += 1
        outcomes["polya"][reds] += 1
    f = Frame(820, 400, (-0.5, draws + 0.5), (0, 0.3), bottom=90)
    body = [f.axes("Twenty draws from 10 red and 10 blue, three replacement rules", "red tokens drawn out of 20", "share of runs", [0, 5, 10, 15, 20], [0, 0.1, 0.2, 0.3])]
    width = f.w / (draws + 1) / 3.4
    series = [("without", "bag, no replacement: always exactly 10", COLOURS["bag"]), ("with", "dice, put it back: a bell around 10", COLOURS["stateless"]), ("polya", "Pólya urn, put it back plus another: anything goes", COLOURS["polya"])]
    for i, (key, _, colour) in enumerate(series):
        # The bag's single bar is 100%: cap it at the top of the axis and label it.
        shares = [(k, min(n / runs, f.y1)) for k, n in enumerate(outcomes[key])]
        body.append(f.bars(shares, colour, (i - 1.5) * width, width))
    body.append(text(f.x(10) - 1.5 * width + width / 2, f.y(f.y1) - 6, "100%", 11, "middle", fill=COLOURS["bag"]))
    body.append(f.legend([(label, colour) for _, label, colour in series]))
    write("urns.svg", svg(820, 400, "\n".join(body)))


def memory_plane():
    width, height = 820, 470
    left, top, w, h = 110, 60, 640, 330

    def px(memory, feedback):
        return left + memory * w, top + (1 - (feedback + 1) / 2) * h

    parts = [text(20, 30, "Two axes: how much a mechanic remembers, and which way the memory pushes", 16, weight="bold")]
    parts.append(f'<rect x="{left}" y="{top}" width="{w}" height="{h / 2}" fill="#fbeff3"/>')
    parts.append(f'<rect x="{left}" y="{top + h / 2}" width="{w}" height="{h / 2}" fill="#eef4fb"/>')
    parts.append(f'<line x1="{left}" y1="{top + h / 2}" x2="{left + w}" y2="{top + h / 2}" stroke="{MUTED}" stroke-dasharray="4 3"/>')
    parts.append(f'<rect x="{left}" y="{top}" width="{w}" height="{h}" fill="none" stroke="{MUTED}"/>')
    parts.append(text(left + w - 8, top + 18, "positive feedback: luck compounds (vindictive)", 11, "end", fill=MUTED))
    parts.append(text(left + w - 8, top + h - 10, "negative feedback: luck cancels out (fair)", 11, "end", fill=MUTED))
    parts.append(text(left, top + h + 22, "no memory", 11, "start", fill=MUTED))
    parts.append(text(left + w, top + h + 22, "full memory", 11, "end", fill=MUTED))
    parts.append(text(left + w / 2, top + h + 42, "memory", 12, "middle", fill=MUTED))
    parts.append(text(60, top + h / 2, "feedback", 12, "middle", fill=MUTED, rotate=-90))
    points = [
        (0.04, 0.0, "stateless dice, table rows", COLOURS["stateless"], True),
        (0.80, -0.70, "shuffle bag (Bag)", COLOURS["bag"], True),
        (0.88, -0.45, "unique bag or table", COLOURS["bag"], True),
        (0.80, -0.20, "pity timer", COLOURS["pity"], False),
        (0.96, 0.05, "ratchet (the Bends)", COLOURS["ratchet"], False),
        (0.80, 0.55, "pressure vessel (Stress + Panic)", COLOURS["grinder"], False),
        (0.92, 0.82, "Pólya urn", COLOURS["polya"], False),
    ]
    for memory, feedback, label, colour, built in points:
        x, y = px(memory, feedback)
        fill = colour if built else "#ffffff"
        parts.append(f'<circle cx="{fmt(x)}" cy="{fmt(y)}" r="8" fill="{fill}" stroke="{colour}" stroke-width="3"/>')
        anchor, dx = ("end", -14) if memory > 0.5 else ("start", 14)
        parts.append(text(x + dx, y + 4, label, 12, anchor))
    parts.append(f'<circle cx="{left + 6}" cy="{height - 22}" r="6" fill="{INK}"/>')
    parts.append(text(left + 18, height - 18, "in the engine today", 11))
    parts.append(f'<circle cx="{left + 176}" cy="{height - 22}" r="6" fill="#ffffff" stroke="{INK}" stroke-width="2"/>')
    parts.append(text(left + 188, height - 18, "on the TODO list", 11))
    write("memory-plane.svg", svg(width, height, "\n".join(parts)))


def mean_gap(dist):
    return sum(k * p for k, p in dist.items()) / sum(dist.values())


if __name__ == "__main__":
    print(f"  mean gap: stateless {mean_gap(gap_distribution_stateless(0.1, 400)):.2f}, "
          f"bag {mean_gap(gap_distribution_bag(10, 400)):.2f}, pity {mean_gap(gap_distribution_pity(400)):.2f}")
    encounter_gaps()
    encounters_per_ten()
    due_effect()
    dice_sums()
    unique_bag()
    final = ratchet_and_vessel()
    urns()
    memory_plane()
    for label, share in final.items():
        print(f"  by beat 20, {label}: {share:.0%}")

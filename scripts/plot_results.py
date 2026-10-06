#!/usr/bin/env python3
"""Render experiment_results.csv as a diverging bar chart (static SVG).

Each bar is Team A's mean final score differential for one "A vs B" matchup:
right of zero (blue) = A ahead, left (red) = B ahead. Sign is also carried by
bar direction and the printed value, so identity is never color-alone.

Usage: python3 scripts/plot_results.py [experiment_results.csv] [docs/images/strategy-battle.svg]
Standard library only.
"""
import csv
import re
import sys
from html import escape

src = sys.argv[1] if len(sys.argv) > 1 else "experiment_results.csv"
dst = sys.argv[2] if len(sys.argv) > 2 else "docs/images/strategy-battle.svg"

rows = []
with open(src, newline="") as f:
    for r in csv.DictReader(f):
        m = re.match(r"\[(.+?)\]\s*(.*)", r["strategy"])
        group, label = (m.group(1), m.group(2)) if m else ("small", r["strategy"])
        rows.append((group, label, float(r["average_score_differential"]),
                     int(r["wins"]), int(r["losses"]), int(r["draws"])))

groups = []
for g, *_ in rows:
    if g not in groups:
        groups.append(g)

# Layout
W, label_w, value_w, pad = 760, 230, 52, 16
row_h, group_h, top = 22, 30, 64
plot_x0 = pad + label_w
plot_x1 = W - pad - value_w
lim = max(1.0, max(abs(v) for _, _, v, *_ in rows))
lim = float(int(lim) + 1) if lim > int(lim) else lim
zero_x = plot_x0 + (plot_x1 - plot_x0) / 2
scale = (plot_x1 - plot_x0) / 2 / lim
H = top + len(groups) * group_h + len(rows) * row_h + 40


def x_of(v):
    return zero_x + v * scale


out = [f'<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {W} {H}" width="{W}" height="{H}" '
       f'font-family="system-ui, -apple-system, Segoe UI, sans-serif" role="img" '
       f'aria-label="Mean final score margin for Team A by matchup">',
       """<style>
  .bg { fill: #fcfcfb; } .t1 { fill: #0b0b0b; } .t2 { fill: #52514e; }
  .grid { stroke: #e4e3df; } .zero { stroke: #52514e; }
  .pos { fill: #2a78d6; } .neg { fill: #e34948; }
  @media (prefers-color-scheme: dark) {
    .bg { fill: #1a1a19; } .t1 { fill: #ffffff; } .t2 { fill: #c3c2b7; }
    .grid { stroke: #383835; } .zero { stroke: #c3c2b7; }
    .pos { fill: #3987e5; } .neg { fill: #e66767; }
  }
</style>""",
       f'<rect class="bg" width="{W}" height="{H}" rx="8"/>',
       f'<text class="t1" x="{pad}" y="26" font-size="16" font-weight="600">Strategy battle: Team A\'s mean final margin</text>',
       f'<text class="t2" x="{pad}" y="46" font-size="12">Rows are "A vs B", 100 games each. '
       f'Right/blue = A ahead, left/red = B ahead. Hover a bar for W/L/D.</text>']

# Gridlines and axis labels
y_plot_top = top - 8
y_plot_bot = H - 34
step = 1.0
v = -lim
while v <= lim + 1e-9:
    x = x_of(v)
    cls = "zero" if abs(v) < 1e-9 else "grid"
    out.append(f'<line class="{cls}" x1="{x:.1f}" x2="{x:.1f}" y1="{y_plot_top}" y2="{y_plot_bot}" stroke-width="1"/>')
    out.append(f'<text class="t2" x="{x:.1f}" y="{H - 16}" font-size="11" text-anchor="middle">{v:+.0f}</text>')
    v += step

y = top
for g in groups:
    y += group_h
    out.append(f'<text class="t1" x="{pad}" y="{y - 10}" font-size="12" font-weight="600">{escape(g)}</text>')
    for group, label, val, w, l, d in rows:
        if group != g:
            continue
        cy = y + row_h / 2
        out.append(f'<text class="t2" x="{plot_x0 - 8}" y="{cy + 4:.1f}" font-size="12" text-anchor="end">{escape(label)}</text>')
        x0, x1 = sorted((zero_x, x_of(val)))
        width = max(x1 - x0, 1.0)
        cls = "pos" if val >= 0 else "neg"
        tip = f"{label}: margin {val:+.2f}, A won {w}, lost {l}, drew {d}"
        out.append(f'<g><title>{escape(tip)}</title>'
                   f'<rect x="{x0:.1f}" y="{y + 4}" width="{width:.1f}" height="{row_h - 8}" rx="3" class="{cls}"/>'
                   f'<rect x="{plot_x0}" y="{y}" width="{plot_x1 - plot_x0}" height="{row_h}" fill="transparent"/></g>')
        out.append(f'<text class="t1" x="{plot_x1 + 6}" y="{cy + 4:.1f}" font-size="12" '
                   f'font-variant-numeric="tabular-nums">{val:+.2f}</text>')
        y += row_h

out.append("</svg>")
with open(dst, "w") as f:
    f.write("\n".join(out) + "\n")
print(f"wrote {dst} ({len(rows)} rows)")

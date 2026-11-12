"""kobito tools の壁紙（2560x1440 の SVG）を作る。

主張しない静かな風景にする。水彩のにじみ・鉛筆の線・紙の地を SVG のフィルターで出す。
使い方: python3 day.py 出力.svg（夕方・夜の壁紙は evening.py・night.py。まとめて作るのは scripts/make-wallpapers.sh）
"""
import math
import random
import sys

W, H = 2560, 1440
PENCIL = "#6E675E"
random.seed(7)


def ridge(y0, amp, waves, x0=-60, x1=W + 60, step=40, bottom=H + 40):
    """手で描いたような、ゆるい起伏の丘の形。"""
    pts = []
    x = x0
    while x <= x1:
        y = y0 + sum(a * math.sin(x / l + p) for a, l, p in waves) + random.uniform(-amp, amp)
        pts.append((x, y))
        x += step + random.uniform(-8, 8)
    d = f"M{pts[0][0]:.0f} {pts[0][1]:.0f}"
    for (ax, ay), (bx, by) in zip(pts, pts[1:]):
        d += f" Q{ax:.0f} {ay:.0f} {(ax + bx) / 2:.0f} {(ay + by) / 2:.0f}"
    return d + f" L{x1} {bottom} L{x0} {bottom}Z"


def wc_filter(fid, seed, wobble=14, freq=0.012, edge=0.8, mfreq=0.018, mott=0.75):
    """水彩：縁が揺れ、色むらがあり、縁に絵の具がたまる。"""
    return f'''
    <filter id="{fid}" x="-15%" y="-15%" width="130%" height="130%" color-interpolation-filters="sRGB">
      <feTurbulence type="fractalNoise" baseFrequency="{freq}" numOctaves="3" seed="{seed}" result="n"/>
      <feDisplacementMap in="SourceGraphic" in2="n" scale="{wobble}" xChannelSelector="R" yChannelSelector="G" result="d"/>
      <feTurbulence type="fractalNoise" baseFrequency="{mfreq}" numOctaves="3" seed="{seed + 50}" result="m"/>
      <feColorMatrix in="m" type="matrix" values="0 0 0 0 0  0 0 0 0 0  0 0 0 0 0  0 0 0 -{mott} {0.55 + mott * 0.6:.2f}" result="ma"/>
      <feComposite in="d" in2="ma" operator="in" result="mott"/>
      <feMorphology in="d" operator="erode" radius="3" result="er"/>
      <feComposite in="d" in2="er" operator="out" result="ring"/>
      <feGaussianBlur in="ring" stdDeviation="1.2" result="ringb"/>
      <feComponentTransfer in="ringb" result="ringc"><feFuncA type="linear" slope="{edge}"/></feComponentTransfer>
      <feMerge><feMergeNode in="mott"/><feMergeNode in="ringc"/></feMerge>
    </filter>'''


defs = [wc_filter(f"wc{i}", 3 + i * 11, wobble=10 + i * 2) for i in range(6)]
defs.append(wc_filter("wcsky", 91, wobble=40, freq=0.004, edge=0, mfreq=0.0025, mott=0.5))
defs.append(wc_filter("wck", 71, wobble=1.2, freq=0.12, edge=0.4, mfreq=0.12, mott=0.4))
defs.append(wc_filter("wcleaf", 61, wobble=26, freq=0.02, edge=1.0, mfreq=0.03, mott=0.5))
defs.append('''
    <filter id="pencil" x="-10%" y="-10%" width="120%" height="120%">
      <feTurbulence type="fractalNoise" baseFrequency="0.06" numOctaves="2" seed="5" result="n"/>
      <feDisplacementMap in="SourceGraphic" in2="n" scale="3.5" xChannelSelector="R" yChannelSelector="G" result="d"/>
      <feTurbulence type="fractalNoise" baseFrequency="0.5" numOctaves="1" seed="8" result="g"/>
      <feColorMatrix in="g" type="matrix" values="0 0 0 0 0  0 0 0 0 0  0 0 0 0 0  0 0 0 -2.2 1.9" result="ga"/>
      <feComposite in="d" in2="ga" operator="in"/>
    </filter>
    <filter id="pencilk" x="-10%" y="-10%" width="120%" height="120%">
      <feTurbulence type="fractalNoise" baseFrequency="0.3" numOctaves="2" seed="5" result="n"/>
      <feDisplacementMap in="SourceGraphic" in2="n" scale="1" xChannelSelector="R" yChannelSelector="G"/>
    </filter>
    <filter id="paper" x="0" y="0" width="100%" height="100%">
      <feTurbulence type="fractalNoise" baseFrequency="0.75" numOctaves="3" seed="21" result="f"/>
      <feColorMatrix in="f" type="matrix" values="0 0 0 0 0.42  0 0 0 0 0.37  0 0 0 0 0.30  0.45 0 0 0 -0.2"/>
    </filter>
    <filter id="blotch" x="0" y="0" width="100%" height="100%">
      <feTurbulence type="fractalNoise" baseFrequency="0.0035" numOctaves="3" seed="33" result="f"/>
      <feColorMatrix in="f" type="matrix" values="0 0 0 0 0.60  0 0 0 0 0.52  0 0 0 0 0.40  0.35 0 0 0 -0.12"/>
    </filter>
    <linearGradient id="skywash" x1="0" y1="0" x2="0" y2="1">
      <stop offset="0" stop-color="#9DB8C2" stop-opacity=".62"/>
      <stop offset=".45" stop-color="#B9CCD0" stop-opacity=".30"/>
      <stop offset=".72" stop-color="#E9D9BC" stop-opacity=".25"/>
      <stop offset="1" stop-color="#E9D9BC" stop-opacity="0"/>
    </linearGradient>''')

out = []
add = out.append


def wash(d, color, op, f=0, extra=""):
    add(f'<path d="{d}" fill="{color}" opacity="{op}" filter="url(#wc{f})" {extra}/>')


def line(d, w=2.4, op=0.75, color=PENCIL):
    add(f'<path d="{d}" fill="none" stroke="{color}" stroke-width="{w}" stroke-linecap="round" '
        f'stroke-linejoin="round" opacity="{op}" filter="url(#pencil)"/>')


# --- 紙と空 ---
add(f'<rect width="{W}" height="{H}" fill="#F4EFE5"/>')
add(f'<rect x="-80" y="-80" width="{W + 160}" height="{H * 0.75:.0f}" fill="url(#skywash)" filter="url(#wcsky)"/>')
# --- 遠くの山から手前へ ---
wash(ridge(905, 6, [(40, 260, 0.3), (18, 90, 1.7)]), "#AFC0C6", 0.40, 0)
wash(ridge(965, 5, [(26, 330, 2.1), (12, 120, 0.4)]), "#B4C3AA", 0.45, 1)
wash(ridge(1040, 4, [(30, 420, 1.0), (10, 140, 2.6)]), "#A7BA98", 0.50, 2)
# 右手の丘（小人の座る場所）
hill = "M-60 1215 C500 1190 1050 1160 1450 1090 C1700 1030 1880 930 2120 905 C2330 892 2470 930 2640 990 L2640 1500 L-60 1500Z"
wash(hill, "#9CB28E", 0.52, 3)
wash(hill, "#B9C79C", 0.30, 4, 'transform="translate(30 26)"')
# 手前の草地
wash(ridge(1250, 5, [(22, 500, 0.6), (8, 160, 1.9)]), "#D3C7A0", 0.40, 5)
wash(ridge(1340, 6, [(16, 380, 2.2)]), "#C2B98F", 0.30, 0)

# --- 遠くの小さな家 ---
hx, hy = 1210, 1018
wash(f"M{hx} {hy} L{hx} {hy - 30} L{hx + 34} {hy - 30} L{hx + 34} {hy}Z", "#EFE6D2", 0.9, 1)
wash(f"M{hx - 6} {hy - 28} L{hx + 17} {hy - 50} L{hx + 40} {hy - 28}Z", "#B97C6A", 0.65, 2)
line(f"M{hx} {hy} L{hx} {hy - 29} M{hx + 34} {hy - 29} L{hx + 34} {hy} M{hx - 6} {hy - 28} L{hx + 17} {hy - 50} L{hx + 40} {hy - 28}", 1.6, 0.55)
wash(f"M{hx + 12} {hy - 22} h9 v9 h-9Z", "#8EA3B5", 0.6, 3)

# --- 遠くの丘をゆく、おもちゃの汽車 ---
line("M330 1078 C520 1060 700 1052 900 1060 C1000 1064 1060 1070 1120 1076", 1.6, 0.45)
tx, ty = 640, 1056
for i, (dx, col, wdt) in enumerate([(0, "#C29282", 30), (36, "#8FA2BC", 30), (72, "#6F9479", 40)]):
    x = tx + dx
    wash(f"M{x} {ty} l{wdt} 0 l0 -18 l-{wdt} 0Z", col, 0.75, i)
    line(f"M{x} {ty} l{wdt} 0 l0 -18 l-{wdt} 0Z", 1.4, 0.55)
    for wx in (x + 7, x + wdt - 7):
        add(f'<circle cx="{wx}" cy="{ty + 2}" r="3.2" fill="{PENCIL}" opacity=".5" filter="url(#pencil)"/>')
line(f"M{tx + 72} {ty - 18} l0 -10 l16 0 l0 10 M{tx + 100} {ty - 18} l0 -12", 1.4, 0.55)
for k, (sx, sy, r) in enumerate([(tx + 104, ty - 42, 7), (tx + 116, ty - 60, 10), (tx + 134, ty - 82, 13)]):
    add(f'<circle cx="{sx}" cy="{sy}" r="{r}" fill="#FFFFFF" opacity="{0.55 - k * 0.12:.2f}" filter="url(#wc{k})"/>')

# --- 丘の上の木 ---
tx, ty = 2190, 908
wash(f"M{tx - 9} {ty + 4} C{tx - 6} {ty - 60} {tx - 4} {ty - 120} {tx - 2} {ty - 150} L{tx + 8} {ty - 150} C{tx + 8} {ty - 110} {tx + 10} {ty - 60} {tx + 14} {ty + 4}Z", "#8C6C52", 0.7, 1)
line(f"M{tx - 9} {ty + 2} C{tx - 6} {ty - 60} {tx - 4} {ty - 120} {tx - 2} {ty - 150} M{tx + 14} {ty + 2} C{tx + 10} {ty - 60} {tx + 9} {ty - 100} {tx + 8} {ty - 140}", 2, 0.6)
leaf = random.Random(4)
for i in range(26):
    ang = leaf.uniform(0, 2 * math.pi)
    r = leaf.uniform(0, 1) ** 0.6
    cx, cy = tx + 8 + math.cos(ang) * r * 130, ty - 235 + math.sin(ang) * r * 95
    rx, ry = leaf.uniform(34, 62), leaf.uniform(24, 44)
    col = leaf.choice(["#93AE8C", "#A6BC90", "#86A282", "#B3C59A"])
    add(f'<ellipse cx="{cx:.0f}" cy="{cy:.0f}" rx="{rx:.0f}" ry="{ry:.0f}" fill="{col}" opacity="{leaf.uniform(.22, .38):.2f}" '
        f'transform="rotate({leaf.uniform(-30, 30):.0f} {cx:.0f} {cy:.0f})" filter="url(#wcleaf)"/>')
# 木の影
wash(f"M{tx - 150} {ty + 6} q150 -16 300 -2 q-150 18 -300 2Z", "#7E9677", 0.35, 0)

# --- 木の下で、メモを読む小人 ---
def kobito_sitting(cx, base, s):
    ox, oy = cx - 24 * s, base - 42 * s
    T = lambda d: f'transform="translate({ox:.1f} {oy:.1f}) scale({s})"'
    body = "M10 23C10 11 16 5 25 5c8 0 13 6 13 16 0 11-6 16-15 16-8 0-13-5-13-14Z"
    visor = "M15 13c5-4 13-4 18 0v12c-5 4-13 4-18 0Z"
    add(f'<g {T(0)}>')
    add(f'<path d="{body}" fill="#E2DCCD" opacity=".95" filter="url(#wck)"/>')
    add(f'<path d="{visor}" fill="#FBF9F3" opacity=".9" filter="url(#wck)"/>')
    add(f'<path d="M17 26.5 L29 25 L30 33 L18 34.5Z" fill="#E9CC7E" opacity=".85" filter="url(#wck)"/>')
    sw = 2.6 / s
    for d in [body, visor, "M17 26.5 L29 25 L30 33 L18 34.5Z",
              "M19.5 36 Q15 40 10.5 41", "M28.5 36 Q33 40 37.5 41",
              "M11.5 26 Q14 30 17.5 30", "M37 25 Q35 30 30.5 30.5",
              "M22.6 21.4c1.5 2 3.3 2 4.8 0"]:
        add(f'<path d="{d}" fill="none" stroke="{PENCIL}" stroke-width="{sw:.3f}" stroke-linecap="round" '
            f'stroke-linejoin="round" opacity=".75" filter="url(#pencilk)"/>')
    add(f'<circle cx="21" cy="17.5" r="1.1" fill="{PENCIL}" opacity=".85"/><circle cx="28" cy="17.5" r="1.1" fill="{PENCIL}" opacity=".85"/>')
    add('</g>')


kobito_sitting(2085, 916, 2.6)

# --- 手前の草（筆のはらい）---
grass = random.Random(9)
for _ in range(34):
    gx, gy = grass.uniform(40, W - 40), grass.uniform(1240, 1420)
    h = grass.uniform(14, 30)
    line(f"M{gx:.0f} {gy:.0f} q{grass.uniform(-4, 4):.0f} -{h * .6:.0f} {grass.uniform(-8, 8):.0f} -{h:.0f}", 1.6, grass.uniform(.18, .32), "#6F8064")

# --- 空の鳥 ---
line("M1580 560 q9 -9 18 0 q9 -9 18 0", 2, 0.45)
line("M1640 530 q7 -7 14 0 q7 -7 14 0", 1.8, 0.4)

# --- 紙の質感 ---
add(f'<rect width="{W}" height="{H}" filter="url(#blotch)"/>')
add(f'<rect width="{W}" height="{H}" filter="url(#paper)"/>')

svg = f'''<svg xmlns="http://www.w3.org/2000/svg" width="{W}" height="{H}" viewBox="0 0 {W} {H}">
  <title>kobito tools wallpaper</title>
  <defs>{''.join(defs)}
  </defs>
  {chr(10).join(out)}
</svg>
'''
open(sys.argv[1], "w").write(svg)

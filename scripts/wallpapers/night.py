"""kobito tools の壁紙・夜版（2560x1440 の SVG）。

小高い丘に座った小人が、湾の向こうのネオンの街を眺めている。昼・夕方の壁紙と同じく、主張しない静かな絵にする。
使い方: python3 night.py 出力.svg
"""
import random
import sys

from watercolor import Canvas, H, W, blob, ridge

c = Canvas(pencil="#3B3652", seed=23, paper_alpha=0.22, blotch_alpha=0.14, blotch_rgb=(0.36, 0.34, 0.50))
rng = c.rng
SKYLINE = 1004   # 街の足もと（湾の岸）

sky = '''
    <linearGradient id="nightsky" x1="0" y1="0" x2="0" y2="1">
      <stop offset="0" stop-color="#262C4E" stop-opacity=".94"/>
      <stop offset=".45" stop-color="#363A68" stop-opacity=".90"/>
      <stop offset=".78" stop-color="#5C4C7C" stop-opacity=".86"/>
      <stop offset="1" stop-color="#7E5E86" stop-opacity=".82"/>
    </linearGradient>
    <linearGradient id="nightsea" x1="0" y1="0" x2="0" y2="1">
      <stop offset="0" stop-color="#3A3B66" stop-opacity=".92"/>
      <stop offset="1" stop-color="#2A2E50" stop-opacity=".94"/>
    </linearGradient>
    <mask id="crescent"><rect width="2560" height="1440" fill="#000"/>
      <circle cx="1950" cy="258" r="56" fill="#fff"/><circle cx="1972" cy="244" r="50" fill="#000"/></mask>'''

# --- 紙と空 ---
c.add(f'<rect width="{W}" height="{H}" fill="#F1ECE2"/>')
# 重ね塗りで夜の深さを出す（1回目は広く、2回目は少しずらして）
c.add(f'<rect x="-80" y="-80" width="{W + 160}" height="{SKYLINE + 120}" fill="url(#nightsky)" filter="url(#wcdark)"/>')
c.add(f'<rect x="-80" y="-120" width="{W + 160}" height="{SKYLINE + 100}" fill="url(#nightsky)" opacity=".75" filter="url(#wcsky)"/>')
# 街の明かりが空を染める
for cx, rx, ry, col, op in [(1000, 1000, 240, "#B8708C", 0.50), (1100, 620, 170, "#D98A92", 0.45),
                            (600, 380, 120, "#9A80C0", 0.36), (1500, 380, 120, "#D4908E", 0.36)]:
    c.wash(blob(rng, cx, SKYLINE - 90, rx, ry, points=14, jitter=0.18), col, op, "wcwet")

# 星（白い絵の具を筆先で置いた点）
star = random.Random(31)
for _ in range(90):
    x, y = star.uniform(30, W - 30), star.uniform(30, 640) ** 1.0
    if 1820 < x < 2080 and 170 < y < 350:
        continue   # 月のまわり
    r = star.uniform(1.4, 3.2)
    c.add(f'<path d="{blob(star, x, y, r, r * star.uniform(.8, 1.2), points=6, jitter=.3)}" fill="#FFF6DE" '
          f'opacity="{star.uniform(.45, .9):.2f}" filter="url(#pencilk)"/>')
for x, y in [(420, 180), (1460, 120), (2300, 420), (980, 300)]:
    c.line(f"M{x - 9} {y} L{x + 9} {y} M{x} {y - 9} L{x} {y + 9}", 1.4, 0.55, "#FFF6DE")

# 月（にじみの淡い光と、細い三日月）
mx, my = 1950, 258
c.wash(blob(rng, mx, my, 150, 140, points=12, jitter=.1), "#8E90B8", 0.35, "wcwet")
c.add('<g mask="url(#crescent)">')
c.wash(blob(rng, mx, my, 60, 60, points=14, jitter=.03), "#F6E9BE", 0.92, "wcfine")
c.add('</g>')

# --- 遠くの山 ---
c.wash(ridge(rng, SKYLINE - 70, 5, [(26, 300, 0.4), (12, 110, 2.0)], bottom=SKYLINE + 20), "#4A4572", 0.75, 1)

# --- 街 ---
city = random.Random(41)
windows = []
neon = []
TOWER = []


def building(x, w, h, color, op, f, lit):
    top = SKYLINE - h
    roof = city.random()
    if roof < 0.2:      # 片流れ
        d = f"M{x} {SKYLINE + 6} L{x} {top + 14} L{x + w} {top} L{x + w} {SKYLINE + 6}Z"
    elif roof < 0.4:    # 段のある屋上
        d = (f"M{x} {SKYLINE + 6} L{x} {top + 12} L{x + w * .3:.0f} {top + 12} L{x + w * .3:.0f} {top} "
             f"L{x + w * .75:.0f} {top} L{x + w * .75:.0f} {top + 12} L{x + w} {top + 12} L{x + w} {SKYLINE + 6}Z")
    else:
        d = f"M{x} {SKYLINE + 6} L{x} {top} L{x + w} {top} L{x + w} {SKYLINE + 6}Z"
    c.wash(d, color, op, f)
    if lit:
        for wy in range(int(top) + 14, SKYLINE - 10, 16):
            for wx in range(int(x) + 8, int(x + w) - 8, 13):
                if city.random() < lit:
                    windows.append((wx, wy))


# 奥の並び
x = 120
while x < 1980:
    w = city.uniform(50, 110)
    h = city.uniform(60, 150)
    building(x, w, h, "#504C7E", 0.80, city.randint(0, 5), 0.30)
    x += w + city.uniform(-6, 10)
# 手前の並び
x = 160
while x < 1900:
    w = city.uniform(60, 130)
    h = city.uniform(36, 110)
    if 1040 < x < 1140:
        w, h = 70, 220    # 塔
        TOWER.append(x + 35)
    building(x, w, h, "#3A3964", 0.90, city.randint(0, 5), 0.55)
    if city.random() < 0.6:
        neon.append((x + city.uniform(10, w - 24), SKYLINE - h + city.uniform(20, h * 0.5)))
    x += w + city.uniform(4, 22)
# 塔のアンテナと赤い灯
tower_x = next(b for b in TOWER)
c.line(f"M{tower_x} {SKYLINE - 218} L{tower_x} {SKYLINE - 270}", 3, 0.9, "#2B2A48")
c.wash(blob(rng, tower_x, SKYLINE - 274, 5, 5, points=7), "#F0606A", 0.95, "wcfine")
c.add(f'<path d="{blob(rng, tower_x, SKYLINE - 274, 16, 16)}" fill="#F0606A" opacity=".45" filter="url(#bloom)"/>')

# 街全体からあふれる光（窓の集まりのまわりににじむ）
for gx in range(200, 1960, 140):
    c.add(f'<path d="{blob(city, gx + city.uniform(-30, 30), SKYLINE - 40, 120, 60)}" fill="#F7D48C" '
          f'opacity="{city.uniform(.28, .42):.2f}" filter="url(#bloom)"/>')
# 窓の明かり（明るい芯と、ときどきまわりのにじみ）
for wx, wy in windows:
    col = city.choice(["#FFE6A0", "#FFD98A", "#FFF1C4", "#E2F1FF"])
    if city.random() < 0.25:
        c.add(f'<path d="{blob(city, wx + 4, wy + 4, 12, 12, points=7)}" fill="{col}" opacity=".55" filter="url(#bloom)"/>')
    c.wash(f"M{wx} {wy} h{city.uniform(5, 8):.0f} v{city.uniform(6, 9):.0f} h-{city.uniform(5, 8):.0f}Z", col,
           city.uniform(.8, 1), "wcfine")
# ネオンの看板（縦長のにじみと、まわりの光）
NEON = ["#FF8FC0", "#7FF0EA", "#D2A8FF", "#FFBE70"]
for nx, ny in neon:
    col = city.choice(NEON)
    tall = city.random() < 0.6
    w, h = (14, city.uniform(46, 80)) if tall else (city.uniform(46, 80), 14)
    c.add(f'<path d="{blob(city, nx + w / 2, ny + h / 2, w * 2.2, h * 1.6)}" fill="{col}" opacity=".7" filter="url(#bloom)"/>')
    c.wash(f"M{nx:.0f} {ny:.0f} h{w:.0f} v{h:.0f} h-{w:.0f}Z", col, 0.95, "wcfine")
    c.wash(f"M{nx + w * .3:.0f} {ny + h * .2:.0f} h{w * .4:.0f} v{h * .6:.0f} h-{w * .4:.0f}Z", "#FFFFFF", 0.7, "wcfine")
    c.line(f"M{nx:.0f} {ny:.0f} h{w:.0f} v{h:.0f} h-{w:.0f}Z", 1.2, 0.5, "#FFFFFF")

# --- 湾と橋 ---
c.wash(f"M-80 {SKYLINE} L{W + 80} {SKYLINE} L{W + 80} {H + 80} L-80 {H + 80}Z", "url(#nightsea)", 1, "wcdarkb")
c.wash(f"M-80 {SKYLINE + 14} L{W + 80} {SKYLINE + 10} L{W + 80} {H + 80} L-80 {H + 80}Z", "url(#nightsea)", 0.7, "wcdark")
# 湾の小さな波（明るい鉛筆）
for _ in range(18):
    wx, wy = rng.uniform(100, 1900), rng.uniform(SKYLINE + 120, 1380)
    ww = rng.uniform(26, 70)
    c.line(f"M{wx:.0f} {wy:.0f} q{ww / 4:.0f} -5 {ww / 2:.0f} 0 q{ww / 4:.0f} 5 {ww / 2:.0f} 0", 1.4, rng.uniform(.18, .3), "#8C88B0")
# 水面に映る明かり（縦に揺れる短いはらい）
refl = random.Random(51)
for nx, ny in neon + windows[::5]:
    col = refl.choice(NEON) if (nx, ny) in neon else "#FFDE96"
    for k in range(refl.randint(3, 6)):
        y = SKYLINE + 18 + k * refl.uniform(14, 24)
        c.wash(blob(refl, nx + refl.uniform(-6, 6), y, refl.uniform(8, 20), refl.uniform(2, 3.5), points=7, jitter=.25),
               col, 0.75 - k * 0.1, "wcfine")
# 水面ににじむ街の光
for gx in range(240, 1940, 180):
    c.add(f'<path d="{blob(refl, gx, SKYLINE + 60, 110, 40)}" fill="#E9B97E" opacity=".22" filter="url(#bloom)"/>')
# 湾をわたる鉄橋と、明かりの灯った汽車（昼の壁紙の汽車）
bridge_y = SKYLINE + 70
c.line(f"M200 {bridge_y} C700 {bridge_y - 6} 1300 {bridge_y - 6} 1800 {bridge_y}", 2.4, 0.7, "#252443")
for px in range(260, 1800, 120):
    c.line(f"M{px} {bridge_y} L{px} {bridge_y + 60}", 1.8, 0.45, "#252443")
tx = 820
for i, w in enumerate([46, 46, 58]):
    x = tx + i * 52
    c.wash(f"M{x} {bridge_y - 2} l{w} 0 l0 -22 l-{w} 0Z", "#2F2E52", 0.95, i)
    for k in range(3 if w == 46 else 2):
        c.wash(f"M{x + 7 + k * 13} {bridge_y - 17} h8 v8 h-8Z", "#F7DA92", 0.9, "wcfine")
c.add(f'<path d="{blob(rng, tx + 80, bridge_y - 12, 110, 18)}" fill="#F7DA92" opacity=".25" filter="url(#bloom)"/>')

# --- 手前の丘と小人 ---
hill = "M1580 1500 C1700 1370 1860 1260 2040 1200 C2200 1150 2380 1136 2640 1146 L2640 1500Z"
c.wash(hill, "#33405A", 0.92, 2)
c.wash(hill, "#4A5670", 0.32, 3, 'transform="translate(-16 12)"')
# 丘の木（葉は筆の点。街灯に照らされた側は少し明るい）
c.tree(random.Random(61), 2450, 1156, 170, 120, 100, ["#3D5560", "#4A6068", "#56646E", "#3A4A5E"],
       trunk="#4B4552", dabs=58, op=(0.35, 0.55))
for _ in range(10):
    c.dab(rng, rng.uniform(2310, 2410), rng.uniform(892, 982), rng.uniform(18, 30), rng.uniform(12, 22), "#8F8A6E", 0.28)
# 街灯
lx, ly = 2300, 1160
c.line(f"M{lx} {ly} L{lx} {ly - 150} q0 -14 14 -14 l12 0", 2.4, 0.8, "#2A2840")
c.add(f'<path d="{blob(rng, lx + 26, ly - 158, 70, 60)}" fill="#F7D9A0" opacity=".35" filter="url(#bloom)"/>')
c.wash(blob(rng, lx + 26, ly - 158, 9, 7, points=8), "#FBE7B6", 0.95, "wcfine")
c.wash(f"M{lx - 140} {ly + 4} q140 -22 280 0 q-140 26 -280 0Z", "#F7D9A0", 0.20, "wcwet")
# 草（筆のはらい）
for _ in range(26):
    gx, gy = rng.uniform(1750, W - 30), rng.uniform(1200, 1420)
    h = rng.uniform(14, 28)
    c.line(f"M{gx:.0f} {gy:.0f} q{rng.uniform(-4, 4):.0f} -{h * .6:.0f} {rng.uniform(-8, 8):.0f} -{h:.0f}", 1.6,
           rng.uniform(.25, .4), "#56677A")
# 街を眺める小人（街のほう＝左を向く）
c.kobito(2160, 1172, 2.6, pose="sit", facing=-1, body="#CFCAD8", visor="#ECEAF2", solid=True)

c.paper()
open(sys.argv[1], "w").write(c.svg("kobito tools wallpaper (night)", sky))

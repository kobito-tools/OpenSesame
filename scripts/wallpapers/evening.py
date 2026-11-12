"""kobito tools の壁紙・夕方版（2560x1440 の SVG）。

見晴らしのいい展望台に小人が立ち、海岸線の向こうへ夕日が沈んでいく。昼の壁紙と同じく、主張しない静かな絵にする。
使い方: python3 evening.py 出力.svg
"""
import math
import random
import sys

from watercolor import Canvas, H, W, blob, ridge

c = Canvas(pencil="#6B5D5A", seed=11, paper_alpha=0.42, blotch_alpha=0.30, blotch_rgb=(0.66, 0.50, 0.40))
rng = c.rng
HORIZON = 868

sky = '''
    <linearGradient id="eveningsky" x1="0" y1="0" x2="0" y2="1">
      <stop offset="0" stop-color="#9C7FA8" stop-opacity=".78"/>
      <stop offset=".30" stop-color="#D98C8E" stop-opacity=".72"/>
      <stop offset=".62" stop-color="#EE9A6C" stop-opacity=".78"/>
      <stop offset="1" stop-color="#F6BD6E" stop-opacity=".82"/>
    </linearGradient>
    <linearGradient id="eveningsea" x1="0" y1="0" x2="0" y2="1">
      <stop offset="0" stop-color="#D99486" stop-opacity=".78"/>
      <stop offset=".45" stop-color="#B98A9A" stop-opacity=".68"/>
      <stop offset="1" stop-color="#9C86A2" stop-opacity=".62"/>
    </linearGradient>
    <clipPath id="abovesea"><rect x="0" y="0" width="2560" height="{HORIZON + 2}"/></clipPath>'''.replace("{HORIZON + 2}", str(HORIZON + 2))

# --- 紙と空 ---
c.add(f'<rect width="{W}" height="{H}" fill="#F4EFE5"/>')
c.add(f'<rect x="-80" y="-80" width="{W + 160}" height="{HORIZON + 140}" fill="url(#eveningsky)" filter="url(#wcsky)"/>')
c.add(f'<rect x="-80" y="-120" width="{W + 160}" height="{HORIZON + 120}" fill="url(#eveningsky)" opacity=".55" filter="url(#wcdark)"/>')
# 夕日の光が空ににじむ（濡れた紙に色を置いたような、重ねたにじみ）
sx, sy = 1240, HORIZON + 6
for rx, ry, col, op in [(1300, 420, "#F7B46E", 0.38), (820, 300, "#F8C67C", 0.42), (480, 200, "#F9D58C", 0.45),
                        (240, 110, "#FBE0A0", 0.50)]:
    c.wash(blob(rng, sx + rng.uniform(-40, 40), sy - ry * 0.3, rx, ry, points=14, jitter=0.18), col, op, "wcwet")

# 横にたなびく雲（筆を横に引いた形）
for cx, cy, rx, ry, col, op in [
    (520, 330, 330, 26, "#A9789A", 0.40), (760, 372, 210, 18, "#C27F8E", 0.36),
    (1720, 250, 380, 30, "#9A7398", 0.38), (1960, 300, 240, 18, "#B97C90", 0.34),
    (1380, 610, 260, 16, "#E5876E", 0.40), (980, 690, 200, 12, "#EB946E", 0.38),
    (1830, 640, 300, 16, "#DC8278", 0.36),
]:
    c.wash(blob(rng, cx, cy, rx, ry, points=12, jitter=0.18), col, op, "wcsky")
    # 雲の下側が夕日に照らされる
    c.wash(blob(rng, cx + rx * 0.05, cy + ry * 0.55, rx * 0.8, ry * 0.45, points=10, jitter=0.2), "#F7B072", 0.45, "wcsky")

# 夕日（下は海が隠す）
c.add('<g clip-path="url(#abovesea)">')
c.wash(blob(rng, sx, sy, 64, 64, points=14, jitter=0.03), "#F09A58", 0.85, 1)
c.wash(blob(rng, sx - 8, sy - 8, 40, 40, points=12, jitter=0.05), "#F6BE7A", 0.55, 2)
c.add('</g>')

# --- 遠くの岬と島 ---
cape = ("M-60 806 C120 796 260 768 380 784 C470 796 540 828 640 846 C720 860 800 866 880 "
        f"{HORIZON + 2} L-60 {HORIZON + 2}Z")
c.wash(cape, "#9A86A0", 0.50, 2)
c.wash(cape, "#B28E98", 0.25, 3, 'transform="translate(18 10)"')
c.wash(f"M1700 {HORIZON + 2} C1740 846 1800 842 1850 852 C1880 858 1900 864 1915 {HORIZON + 2}Z", "#9A86A0", 0.45, 4)
c.wash(f"M1950 {HORIZON + 2} C1965 860 1990 858 2010 {HORIZON + 2}Z", "#9A86A0", 0.40, 5)
# 岬の灯台
lx, ly = 640, 846
c.wash(f"M{lx - 7} {ly} L{lx - 5} {ly - 40} L{lx + 5} {ly - 40} L{lx + 7} {ly}Z", "#F1E8DC", 0.95, "wcfine")
c.wash(f"M{lx - 6} {ly - 40} L{lx} {ly - 50} L{lx + 6} {ly - 40}Z", "#B5675A", 0.75, "wcfine")
c.line(f"M{lx - 7} {ly} L{lx - 5} {ly - 40} L{lx + 5} {ly - 40} L{lx + 7} {ly} M{lx - 6} {ly - 40} L{lx} {ly - 50} L{lx + 6} {ly - 40}", 1.4, 0.55)
c.glow(lx, ly - 44, 16, "#FBE3A6", 0.8)

# --- 海 ---
c.wash(f"M-80 {HORIZON} L{W + 80} {HORIZON} L{W + 80} {H + 80} L-80 {H + 80}Z", "url(#eveningsea)", 1, 0)
# 水平線の明るさ
c.wash(f"M-80 {HORIZON} L{W + 80} {HORIZON} L{W + 80} {HORIZON + 60} L-80 {HORIZON + 60}Z", "#F4B276", 0.45, "wcsky")
# 夕日の照り返し（水面の短いはらい）
glint = random.Random(3)
for i in range(46):
    t = i / 45
    y = HORIZON + 8 + t ** 1.4 * 330
    half = 40 + t * 150
    x = sx + glint.uniform(-half, half) * (0.6 + 0.4 * glint.random())
    c.wash(blob(glint, x, y, glint.uniform(24, 70) * (0.6 + t), glint.uniform(2.5, 4.5), points=8, jitter=0.2),
           glint.choice(["#F5B676", "#F7C891", "#F2A46A"]), (0.62 - t * 0.38), "wcfine")
# 波のしわ（鉛筆）
for _ in range(16):
    wx, wy = rng.uniform(300, 2300), rng.uniform(HORIZON + 40, 1250)
    if abs(wx - sx) < 160:
        continue
    ww = rng.uniform(26, 60)
    c.line(f"M{wx:.0f} {wy:.0f} q{ww / 4:.0f} -5 {ww / 2:.0f} 0 q{ww / 4:.0f} 5 {ww / 2:.0f} 0", 1.4, rng.uniform(.18, .3))

# --- 手前の海岸線（左下の陸と砂浜）---
COAST = "M-60 884 C80 890 170 902 260 914 C330 924 360 950 420 958 C500 968 540 1000 600 1030 C660 1060 700 1100 760 1130 C820 1160 850 1220 900 1260 C950 1300 990 1380 1016 1500"
shore = COAST + " L-60 1500Z"
c.wash(shore, "#EAD2A8", 0.75, 3, 'transform="translate(34 -12)"')     # 砂浜
c.wash(shore, "#B39470", 0.62, 4)                                        # 草地（夕日の色）
c.wash("M-60 1040 C120 1030 300 1080 460 1170 C560 1230 640 1330 690 1500 L-60 1500Z", "#8E7C66", 0.34, 5)  # 手前の丘
# 畑のうね（鉛筆のかすれた線）
for k, (y0, x1) in enumerate([(968, 300), (1010, 250), (1046, 340)]):
    c.line(f"M{10 + k * 30} {y0} C{110 + k * 20} {y0 + 6 + k * 3} {x1 - 80} {y0 + 22} {x1} {y0 + 44}", 1.2, 0.16)
c.wash("M-60 900 C160 912 340 950 500 1012 C380 1000 160 960 -60 950Z", "#D9A775", 0.30, 2)  # 夕日の当たり
# 打ち寄せる波
c.add(f'<path d="{COAST}" transform="translate(40 -6)" fill="none" stroke="#FFFFFF" stroke-width="2" stroke-dasharray="140 26 220 40 90 30 260 22" '
      'stroke-linecap="round" opacity=".55" filter="url(#pencil)"/>')
c.add(f'<path d="{COAST}" transform="translate(4 0)" fill="none" stroke="{c.pencil}" stroke-width="1.5" stroke-dasharray="300 18 120 60 200 14 80 40" '
      'stroke-linecap="round" opacity=".42" filter="url(#pencil)"/>')
# 海辺の木立（筆の点）
grove = random.Random(5)
for gx, gy in [(150, 930), (230, 944), (600, 1060), (720, 1110), (790, 1160), (300, 1210), (120, 1300)]:
    for _ in range(6):
        c.dab(grove, gx + grove.uniform(-26, 26), gy + grove.uniform(-24, 6), grove.uniform(14, 26),
              grove.uniform(10, 20), grove.choice(["#8E7E62", "#A98A68", "#7C705E"]), grove.uniform(.3, .5))
# 海辺の小さな家並み
for i, (hx, hw, roof) in enumerate([(380, 30, "#B97C6A"), (424, 24, "#8EA3B5"), (462, 28, "#C49A6C")]):
    hy = 978 + i * 10
    c.wash(f"M{hx} {hy} l0 -22 l{hw} 0 l0 22Z", "#F1E6D4", 0.9, i)
    c.wash(f"M{hx - 4} {hy - 21} l{hw / 2 + 4:.0f} -14 l{hw / 2 + 4:.0f} 14Z", roof, 0.65, i + 1)
    c.line(f"M{hx} {hy} l0 -22 l{hw} 0 l0 22 M{hx - 4} {hy - 21} l{hw / 2 + 4:.0f} -14 l{hw / 2 + 4:.0f} 14", 1.3, 0.5)

# --- 右手の高台と展望台 ---
cliff = ("M1520 1500 C1600 1380 1680 1250 1790 1160 C1880 1092 1990 1058 2140 1048 "
         "C2300 1040 2460 1046 2640 1062 L2640 1500Z")
c.wash(cliff, "#8A7468", 0.62, 2)
c.wash(cliff, "#D38E62", 0.30, 3, 'transform="translate(-14 10)"')
# 崖の岩肌（鉛筆と、濃いめのにじみ）
c.wash("M1540 1500 C1610 1400 1680 1310 1770 1226 C1800 1250 1790 1300 1760 1360 C1730 1420 1720 1470 1716 1500Z", "#86806E", 0.26, 4)
c.line("M1590 1450 C1640 1380 1690 1316 1750 1250 M1650 1490 C1690 1420 1720 1370 1756 1320", 1.3, 0.30)
# 高台の草むら（筆の点）
bush = random.Random(8)
for gx, gy in [(1840, 1120), (1900, 1086), (2350, 1062), (2560, 1080), (2050, 1100)]:
    for _ in range(5):
        c.dab(bush, gx + bush.uniform(-30, 30), gy + bush.uniform(-14, 10), bush.uniform(16, 30),
              bush.uniform(10, 18), bush.choice(["#7C705E", "#9A8066", "#B98868"]), bush.uniform(.25, .42))
c.line("M1500 1500 C1540 1470 1580 1440 1620 1420", 1.6, 0.45, "#FFFFFF")   # 崖の下の波
# 高台の木（葉は筆の点）
c.tree(random.Random(21), 2440, 1046, 160, 125, 96,
       ["#8D8A6C", "#A99070", "#D49A6A", "#7F7A66", "#C08466"], dabs=56)
# 展望台の床と柵
deck_l, deck_r, deck_y = 1930, 2300, 1046
c.wash(f"M{deck_l} {deck_y} L{deck_r} {deck_y} L{deck_r + 10} {deck_y + 16} L{deck_l - 12} {deck_y + 16}Z", "#A9805F", 0.70, 1)
c.line(f"M{deck_l - 12} {deck_y + 16} L{deck_r + 10} {deck_y + 16} M{deck_l} {deck_y} L{deck_r} {deck_y}", 1.8, 0.65)
for px in range(deck_l, deck_r + 1, 52):
    c.line(f"M{px} {deck_y + 16} L{px} {deck_y + 40}", 1.6, 0.5)   # 床を支える脚
posts = list(range(deck_l + 6, deck_r, 70))
for px in posts:
    c.line(f"M{px} {deck_y} L{px + 1} {deck_y - 66}", 2.2, 0.7)
c.line(f"M{posts[0] - 6} {deck_y - 64} L{posts[-1] + 8} {deck_y - 68}", 2.6, 0.75)   # 手すり
c.line(f"M{posts[0] - 4} {deck_y - 34} L{posts[-1] + 6} {deck_y - 36}", 1.6, 0.55)
c.wash(f"M{posts[0] - 6} {deck_y - 68} L{posts[-1] + 8} {deck_y - 72} L{posts[-1] + 8} {deck_y - 62} L{posts[0] - 6} {deck_y - 58}Z", "#A47658", 0.55, 2)
# 望遠鏡
tx = 2240
c.line(f"M{tx} {deck_y} L{tx} {deck_y - 52} M{tx - 10} {deck_y} L{tx + 10} {deck_y}", 2.2, 0.7)
c.wash(f"M{tx - 26} {deck_y - 62} L{tx + 12} {deck_y - 70} L{tx + 14} {deck_y - 54} L{tx - 24} {deck_y - 48}Z", "#7D8C9C", 0.75, "wcfine")
c.line(f"M{tx - 26} {deck_y - 62} L{tx + 12} {deck_y - 70} L{tx + 14} {deck_y - 54} L{tx - 24} {deck_y - 48}Z", 1.6, 0.65)
# 柵にもたれて、夕日を眺める小人（夕日のほう＝左を向く）
c.kobito(2090, deck_y, 2.6, pose="stand", facing=-1, body="#E6D8C6", visor="#FBF3E6")

# --- 手前の草（筆のはらい）---
for _ in range(30):
    gx = rng.uniform(1700, W - 40) if rng.random() < 0.6 else rng.uniform(20, 1100)
    gy = rng.uniform(1300, 1420) if gx > 1600 else rng.uniform(1160, 1420)
    h = rng.uniform(14, 30)
    c.line(f"M{gx:.0f} {gy:.0f} q{rng.uniform(-4, 4):.0f} -{h * .6:.0f} {rng.uniform(-8, 8):.0f} -{h:.0f}", 1.6,
           rng.uniform(.18, .32), "#6C6B52")

# --- 空の鳥 ---
c.line("M1420 640 q9 -9 18 0 q9 -9 18 0", 2, 0.5)
c.line("M1478 610 q7 -7 14 0 q7 -7 14 0", 1.8, 0.45)
c.line("M1380 680 q6 -6 12 0 q6 -6 12 0", 1.6, 0.4)

# 全体にかかる夕日の色（薄い橙の重ね塗り）
c.add(f'<rect x="-80" y="-80" width="{W + 160}" height="{H + 160}" fill="#F2A15E" opacity=".16" filter="url(#wcdark)"/>')
c.paper()
open(sys.argv[1], "w").write(c.svg("kobito tools wallpaper (evening)", sky))

"""水彩風の壁紙を描くための共通の部品（昼の壁紙 day.py と同じ作り）。

手描きのタッチは SVG のフィルターで出す。
- 水彩のにじみ：塗りの縁を少し揺らし、色むらと、縁に絵の具がたまったような濃さを付ける（wc_filter）。
- 鉛筆の線：細く、少しよれてかすれる（pencil）。
- 紙の地：全体に、紙の目と大きなむらを薄く重ねる（paper・blotch）。
- 木の葉：丸ではなく、半透明の筆の点（不定形の形）を重ねる（Canvas.dab）。
"""
import math
import random

W, H = 2560, 1440


def ridge(rng, y0, amp, waves, x0=-60, x1=W + 60, step=40, bottom=H + 40):
    """手で描いたような、ゆるい起伏の形（下は画面の外まで塗る）。"""
    pts = []
    x = x0
    while x <= x1:
        y = y0 + sum(a * math.sin(x / l + p) for a, l, p in waves) + rng.uniform(-amp, amp)
        pts.append((x, y))
        x += step + rng.uniform(-8, 8)
    d = f"M{pts[0][0]:.0f} {pts[0][1]:.0f}"
    for (ax, ay), (bx, by) in zip(pts, pts[1:]):
        d += f" Q{ax:.0f} {ay:.0f} {(ax + bx) / 2:.0f} {(ay + by) / 2:.0f}"
    return d + f" L{x1} {bottom} L{x0} {bottom}Z"


def blob(rng, cx, cy, rx, ry, points=9, jitter=0.28, angle=0.0):
    """筆で置いたような、不定形の点の形。丸や楕円にならないよう、縁をでこぼこにする。"""
    pts = []
    for i in range(points):
        a = 2 * math.pi * i / points + rng.uniform(-0.25, 0.25)
        k = 1 + rng.uniform(-jitter, jitter)
        x, y = math.cos(a) * rx * k, math.sin(a) * ry * k
        ca, sa = math.cos(angle), math.sin(angle)
        pts.append((cx + x * ca - y * sa, cy + x * sa + y * ca))
    mids = [((ax + bx) / 2, (ay + by) / 2) for (ax, ay), (bx, by) in zip(pts, pts[1:] + pts[:1])]
    d = f"M{mids[-1][0]:.1f} {mids[-1][1]:.1f}"
    for (px, py), (mx, my) in zip(pts, mids):
        d += f" Q{px:.1f} {py:.1f} {mx:.1f} {my:.1f}"
    return d + "Z"


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


def common_defs(paper_alpha=0.45, blotch_alpha=0.35, blotch_rgb=(0.60, 0.52, 0.40)):
    defs = [wc_filter(f"wc{i}", 3 + i * 11, wobble=10 + i * 2) for i in range(6)]
    defs.append(wc_filter("wcsky", 91, wobble=40, freq=0.004, edge=0, mfreq=0.0025, mott=0.5))
    defs.append(wc_filter("wck", 71, wobble=1.2, freq=0.12, edge=0.4, mfreq=0.12, mott=0.4))
    defs.append(wc_filter("wcleaf", 61, wobble=18, freq=0.03, edge=1.0, mfreq=0.03, mott=0.5))
    defs.append(wc_filter("wcfine", 17, wobble=3, freq=0.05, edge=0.6, mfreq=0.06, mott=0.45))
    # 夜の空や水のような、広く暗い塗り。色むらを控えめにする。
    defs.append(wc_filter("wcdark", 93, wobble=30, freq=0.004, edge=0, mfreq=0.003, mott=0.22))
    defs.append(wc_filter("wcdarkb", 95, wobble=8, freq=0.01, edge=0.5, mfreq=0.008, mott=0.24))
    r, g, b = blotch_rgb
    defs.append(f'''
    <filter id="pencil" filterUnits="userSpaceOnUse" x="-200" y="-200" width="{W + 400}" height="{H + 400}">
      <feTurbulence type="fractalNoise" baseFrequency="0.06" numOctaves="2" seed="5" result="n"/>
      <feDisplacementMap in="SourceGraphic" in2="n" scale="3.5" xChannelSelector="R" yChannelSelector="G" result="d"/>
      <feTurbulence type="fractalNoise" baseFrequency="0.5" numOctaves="1" seed="8" result="g"/>
      <feColorMatrix in="g" type="matrix" values="0 0 0 0 0  0 0 0 0 0  0 0 0 0 0  0 0 0 -2.2 1.9" result="ga"/>
      <feComposite in="d" in2="ga" operator="in"/>
    </filter>
    <filter id="pencilk" filterUnits="userSpaceOnUse" x="-200" y="-200" width="{W + 400}" height="{H + 400}">
      <feTurbulence type="fractalNoise" baseFrequency="0.3" numOctaves="2" seed="5" result="n"/>
      <feDisplacementMap in="SourceGraphic" in2="n" scale="1" xChannelSelector="R" yChannelSelector="G"/>
    </filter>
    <filter id="wcwet" x="-40%" y="-40%" width="180%" height="180%" color-interpolation-filters="sRGB">
      <feTurbulence type="fractalNoise" baseFrequency="0.006" numOctaves="3" seed="44" result="n"/>
      <feDisplacementMap in="SourceGraphic" in2="n" scale="90" xChannelSelector="R" yChannelSelector="G" result="d"/>
      <feGaussianBlur in="d" stdDeviation="22" result="b"/>
      <feTurbulence type="fractalNoise" baseFrequency="0.012" numOctaves="3" seed="45" result="m"/>
      <feColorMatrix in="m" type="matrix" values="0 0 0 0 0  0 0 0 0 0  0 0 0 0 0  0 0 0 -0.7 1.05" result="ma"/>
      <feComposite in="b" in2="ma" operator="in"/>
    </filter>
    <filter id="bloom" x="-120%" y="-120%" width="340%" height="340%" color-interpolation-filters="sRGB">
      <feTurbulence type="fractalNoise" baseFrequency="0.05" numOctaves="2" seed="46" result="n"/>
      <feDisplacementMap in="SourceGraphic" in2="n" scale="14" xChannelSelector="R" yChannelSelector="G" result="d"/>
      <feGaussianBlur in="d" stdDeviation="9" result="b"/>
      <feTurbulence type="fractalNoise" baseFrequency="0.08" numOctaves="2" seed="47" result="m"/>
      <feColorMatrix in="m" type="matrix" values="0 0 0 0 0  0 0 0 0 0  0 0 0 0 0  0 0 0 -0.8 1.1" result="ma"/>
      <feComposite in="b" in2="ma" operator="in"/>
    </filter>
    <filter id="glow" x="-50%" y="-50%" width="200%" height="200%">
      <feGaussianBlur stdDeviation="14"/>
    </filter>
    <filter id="glowsoft" x="-80%" y="-80%" width="260%" height="260%">
      <feGaussianBlur stdDeviation="40"/>
    </filter>
    <filter id="paper" x="0" y="0" width="100%" height="100%">
      <feTurbulence type="fractalNoise" baseFrequency="0.75" numOctaves="3" seed="21" result="f"/>
      <feColorMatrix in="f" type="matrix" values="0 0 0 0 0.42  0 0 0 0 0.37  0 0 0 0 0.30  {paper_alpha} 0 0 0 -0.2"/>
    </filter>
    <filter id="blotch" x="0" y="0" width="100%" height="100%">
      <feTurbulence type="fractalNoise" baseFrequency="0.0035" numOctaves="3" seed="33" result="f"/>
      <feColorMatrix in="f" type="matrix" values="0 0 0 0 {r}  0 0 0 0 {g}  0 0 0 0 {b}  {blotch_alpha} 0 0 0 -0.12"/>
    </filter>''')
    return defs


class Canvas:
    """SVG の部品を順に積む。後に足したものほど手前。"""

    def __init__(self, pencil="#6E675E", seed=7, **paper):
        self.pencil = pencil
        self.rng = random.Random(seed)
        self.defs = common_defs(**paper)
        self.out = []

    def add(self, s):
        self.out.append(s)

    def wash(self, d, color, op, f=0, extra=""):
        flt = f if isinstance(f, str) else f"wc{f}"
        self.add(f'<path d="{d}" fill="{color}" opacity="{op}" filter="url(#{flt})" {extra}/>')

    def line(self, d, w=2.4, op=0.75, color=None):
        self.add(f'<path d="{d}" fill="none" stroke="{color or self.pencil}" stroke-width="{w}" stroke-linecap="round" '
                 f'stroke-linejoin="round" opacity="{op}" filter="url(#pencil)"/>')

    def dab(self, rng, cx, cy, rx, ry, color, op):
        """木の葉などの、半透明の筆の点。"""
        d = blob(rng, cx, cy, rx, ry, points=rng.randint(7, 10), angle=rng.uniform(-0.6, 0.6))
        self.add(f'<path d="{d}" fill="{color}" opacity="{op:.2f}" filter="url(#wcleaf)"/>')

    def glow(self, cx, cy, r, color, op, soft=False):
        self.add(f'<circle cx="{cx:.0f}" cy="{cy:.0f}" r="{r:.0f}" fill="{color}" opacity="{op:.2f}" '
                 f'filter="url(#{"glowsoft" if soft else "glow"})"/>')

    def tree(self, rng, x, base, height, crown_w, crown_h, colors, trunk="#8C6C52", dabs=34, op=(0.22, 0.4)):
        """幹は鉛筆と水彩、葉は筆の点を重ねる。"""
        top = base - height
        self.wash(f"M{x - 9} {base + 4} C{x - 6} {base - height * .4:.0f} {x - 4} {base - height * .8:.0f} {x - 2} {top} "
                  f"L{x + 8} {top} C{x + 8} {base - height * .7:.0f} {x + 10} {base - height * .4:.0f} {x + 14} {base + 4}Z", trunk, 0.7, 1)
        self.line(f"M{x - 9} {base + 2} C{x - 6} {base - height * .4:.0f} {x - 4} {base - height * .8:.0f} {x - 2} {top} "
                  f"M{x + 14} {base + 2} C{x + 10} {base - height * .4:.0f} {x + 9} {base - height * .7:.0f} {x + 8} {top + 10}", 2, 0.6)
        for _ in range(dabs):
            ang = rng.uniform(0, 2 * math.pi)
            r = rng.uniform(0, 1) ** 0.6
            cx, cy = x + 4 + math.cos(ang) * r * crown_w, top - crown_h * 0.55 + math.sin(ang) * r * crown_h
            self.dab(rng, cx, cy, rng.uniform(24, 46), rng.uniform(18, 34), rng.choice(colors), rng.uniform(*op))

    def kobito(self, cx, base, s, pose="sit", facing=0, body="#E2DCCD", visor="#FBF9F3", accent=None, solid=False):
        """小人。pose は sit（座る）か stand（立つ）。facing は顔の向き（-1 左・0 正面・1 右）。
        solid にすると、暗い背景でも透けないよう、下に不透明な地を一度塗る。"""
        ox, oy = cx - 24 * s, base - (42 if pose == "sit" else 48) * s
        egg = "M10 23C10 11 16 5 25 5c8 0 13 6 13 16 0 11-6 16-15 16-8 0-13-5-13-14Z"
        # 顔の枠・目・口は同じ中心にそろえる。横を向くときは中心を向きのほうへ寄せ、枠と目の間隔を狭める。
        if facing == 0:
            face, half, eye = 24.0, 9.0, 3.5
            vis = "M15 13c5-4 13-4 18 0v12c-5 4-13 4-18 0Z"
        else:
            face, half, eye = 24.0 + 4.5 * facing, 7.5, 3.0
            vis = f"M{face - half:.1f} 14c4-3 11-3 15 0v10c-4 3-11 3-15 0Z"
        if pose == "sit":
            limbs = ["M19.5 36 Q15 40 10.5 41", "M28.5 36 Q33 40 37.5 41",
                     "M11.5 26 Q14 30 17.5 30", "M37 25 Q35 30 30.5 30.5"]
        else:
            limbs = ["M20 36 L18.5 47", "M28 36 L29.5 47",
                     "M11 25 Q8 30 9.5 34", "M37 25 Q40 30 38.5 34"]
        self.add(f'<g transform="translate({ox:.1f} {oy:.1f}) scale({s})">')
        if solid:
            self.add(f'<path d="{egg}" fill="{body}"/>')
        self.add(f'<path d="{egg}" fill="{body}" opacity=".95" filter="url(#wck)"/>')
        self.add(f'<path d="{vis}" fill="{visor}" opacity=".9" filter="url(#wck)"/>')
        if accent:
            self.add(f'<path d="M17 26.5 L29 25 L30 33 L18 34.5Z" fill="{accent}" opacity=".85" filter="url(#wck)"/>')
        sw = 2.6 / s
        mouth = f"M{face - 2.2:.1f} 21.4c1.3 1.9 3.1 1.9 4.4 0"
        for d in [egg, vis, mouth, *limbs] + (["M17 26.5 L29 25 L30 33 L18 34.5Z"] if accent else []):
            self.add(f'<path d="{d}" fill="none" stroke="{self.pencil}" stroke-width="{sw:.3f}" stroke-linecap="round" '
                     f'stroke-linejoin="round" opacity=".75" filter="url(#pencilk)"/>')
        self.add(f'<circle cx="{face - eye:.1f}" cy="17.5" r="1.1" fill="{self.pencil}" opacity=".85"/>'
                 f'<circle cx="{face + eye:.1f}" cy="17.5" r="1.1" fill="{self.pencil}" opacity=".85"/>')
        self.add('</g>')

    def paper(self):
        self.add(f'<rect width="{W}" height="{H}" filter="url(#blotch)"/>')
        self.add(f'<rect width="{W}" height="{H}" filter="url(#paper)"/>')

    def svg(self, title, extra_defs=""):
        return f'''<svg xmlns="http://www.w3.org/2000/svg" width="{W}" height="{H}" viewBox="0 0 {W} {H}">
  <title>{title}</title>
  <defs>{''.join(self.defs)}{extra_defs}
  </defs>
  {chr(10).join(self.out)}
</svg>
'''

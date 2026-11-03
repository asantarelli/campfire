# This file is part of Campfire.
# Copyright (c) Pandula Péter 2017-2026.
# https://github.com/pandulapeter/campfire
#
# This Source Code Form is subject to the terms of the Mozilla Public License, v. 2.0.
# If a copy of the MPL was not distributed with this file, You can obtain one at
# https://mozilla.org/MPL/2.0/.

"""Paints the JSON of `render_svg` as a picture, so that a layout can be looked at: `python preview.py in.json out.png [pixels per unit]`."""

import json
import sys

from PIL import Image, ImageDraw, ImageFont

COLORS = {'ink': (17, 17, 17), 'muted': (136, 136, 136), 'accent': (194, 87, 10)}
SUPER = 3


def main():
    source, target = sys.argv[1], sys.argv[2]
    unit = float(sys.argv[3]) if len(sys.argv) > 3 else 12.0
    data = json.load(open(source, encoding='utf-8'))
    scale = unit * SUPER
    width, height = int((data['width'] + 2) * scale), int((data['height'] + 2) * scale)
    image = Image.new('RGB', (width, height), 'white')
    draw = ImageDraw.Draw(image)
    margin = scale

    def point(x, y):
        return (margin + x * scale, margin + y * scale)

    fonts = {}

    def font(size, style):
        key = (round(size * scale), style)
        if key not in fonts:
            name = {'n': 'segoeui.ttf', 'b': 'segoeuib.ttf', 'i': 'segoeuii.ttf'}[style]
            fonts[key] = ImageFont.truetype('C:/Windows/Fonts/' + name, key[0])
        return fonts[key]

    for mark in data['marks']:
        color = COLORS[mark['t']]
        kind = mark['k']
        if kind == 'line':
            x1, y1, x2, y2 = mark['p']
            draw.line([point(x1, y1), point(x2, y2)], fill=color, width=max(1, round(mark['w'] * scale)))
        elif kind == 'poly':
            points = [point(x, y) for x, y in mark['p']]
            w = max(1, round(mark['w'] * scale))
            draw.line(points, fill=color, width=w, joint='curve')
            for p in (points[0], points[-1]):
                draw.ellipse([p[0] - w / 2, p[1] - w / 2, p[0] + w / 2, p[1] + w / 2], fill=color)
        elif kind == 'fill':
            draw.polygon([point(x, y) for x, y in mark['p']], fill=color)
        elif kind == 'ellipse':
            import math
            x, y, rx, ry, angle = mark['p']
            points = []
            for step in range(40):
                t = step / 40 * 2 * math.pi
                px, py = rx * math.cos(t), ry * math.sin(t)
                points.append(point(x + px * math.cos(angle) - py * math.sin(angle), y + px * math.sin(angle) + py * math.cos(angle)))
            if mark['f']:
                draw.polygon(points, fill=color)
            else:
                draw.line(points + [points[0]], fill=color, width=max(1, round(mark['w'] * scale)))
        elif kind == 'text':
            x, y = mark['p']
            f = font(mark['z'], mark['y'])
            text = mark['s']
            box = draw.textbbox((0, 0), text, font=f)
            w, h = box[2] - box[0], box[3] - box[1]
            px, py = point(x, y)
            left = {'l': px, 'c': px - w / 2, 'r': px - w}[mark['a']]
            top = py - h / 2 - box[1]
            if mark['m']:
                draw.rectangle([left - 2, py - h / 2 - 2, left + w + 2, py + h / 2 + 2], fill='white')
            draw.text((left - box[0], top), text, font=f, fill=color)
    image = image.resize((width // SUPER, height // SUPER), Image.LANCZOS)
    image.save(target)


if __name__ == '__main__':
    main()

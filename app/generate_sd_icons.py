# This file is part of Campfire.
# Copyright (c) Pandula Péter 2017-2026.
# https://github.com/pandulapeter/campfire
#
# This Source Code Form is subject to the terms of the Mozilla Public License, v. 2.0.
# If a copy of the MPL was not distributed with this file, You can obtain one at
# https://mozilla.org/MPL/2.0/.

"""
Writes every app icon SDCancionero ships out of the one picture drawn by hand, `app/icons/sd_pick.png`: the guitar pick
cropped to its edges, on a transparent background.

    python3 app/generate_sd_icons.py

Run from anywhere, with Pillow installed, after the source has changed; the output is committed. The icon does not follow
the theme color, so the launcher aliases, the favicons, the desktop window icons and the iOS alternate icon sets of
every color are all the same picture: the code that picks one by color keeps working, it just always lands on the pick.

What is written:

- Android: the adaptive icon's foreground and monochrome layers in every density (`res/mipmap-*`), whose background is
  a plain color (`ic_launcher_background` in `res/values/colors.xml`), and the Play listing icon `app/android/appIcon.png`.
  The monochrome layer is the blue mark alone, which is what Android 13's themed icons tint.
- The desktop: `appIcon.ico` and `appIcon.icns` (the packaged icons, in `app/icons` and in the desktop module's
  resources) and the window icons `app_icon*.png`.
- The web: `icon-192.png` and the favicons, which keep the names the page looks for.
- iOS: the light, dark and tinted appearance of every `AppIcon` set.
- Inside the app: `ic_sd_mark.xml`, the blue mark alone traced into a 24dp vector, which the screens tint with the
  theme's colors like every other icon. Tracing needs `pip install potracer numpy` as well.
"""

import glob
import os
import shutil

from PIL import Image, ImageChops, ImageFilter

ROOT = os.path.join(os.path.dirname(os.path.abspath(__file__)), "..")
SOURCE = os.path.join(ROOT, "app", "icons", "sd_pick.png")
ANDROID_RES = os.path.join(ROOT, "app", "android", "src", "main", "res")
WEB_RESOURCES = os.path.join(ROOT, "app", "web", "src", "wasmJsMain", "resources")
DESKTOP_DRAWABLE = os.path.join(ROOT, "app", "desktop", "src", "main", "composeResources", "drawable")
DESKTOP_RESOURCES = os.path.join(ROOT, "app", "desktop", "src", "main", "resources")
IOS_ASSETS = os.path.join(ROOT, "app", "ios", "iosApp", "iosApp", "Assets.xcassets")
PRESENTATION_DRAWABLE = os.path.join(ROOT, "presentation", "src", "commonMain", "composeResources", "drawable")

# Android's adaptive icon is a 108dp canvas of which only the central 66dp circle is always visible.
ANDROID_DENSITIES = {"mdpi": 108, "hdpi": 162, "xhdpi": 216, "xxhdpi": 324, "xxxhdpi": 432}
ANDROID_PICK_HEIGHT = 0.58
LAUNCHER_BACKGROUND = (255, 255, 255, 255)
# In pixels of the source picture, and odd, as the filter that cuts the pick's rim away needs.
RIM_WIDTH = 61


def load_pick():
    return Image.open(SOURCE).convert("RGBA")


def render(pick, size, height, background=None):
    """The pick, [height] of the canvas tall, centered on a [size] square that is transparent unless [background] is."""
    canvas = Image.new("RGBA", (size, size), background or (0, 0, 0, 0))
    target_height = round(size * height)
    target_width = round(pick.width * target_height / pick.height)
    scaled = pick.resize((target_width, target_height), Image.LANCZOS)
    canvas.alpha_composite(scaled, ((size - target_width) // 2, (size - target_height) // 2))
    return canvas


def blue_mark(pick):
    """The pick with only its blue mark left, white, whose opacity is how much of the mark each pixel is."""
    red, green, blue, alpha = pick.split()
    brightness = ImageChops.lighter(green, blue).point(lambda value: max(0, min(255, (value - 90) * 255 // 80)))
    # The pick's rim catches the light and reads as bright as the mark does, so the outer edge is cut away first: the mark
    # sits well inside it.
    interior = alpha.filter(ImageFilter.MinFilter(RIM_WIDTH))
    mask = ImageChops.multiply(brightness, interior)
    mark = Image.new("RGBA", pick.size, (255, 255, 255, 0))
    mark.putalpha(mask)
    return mark


def save(image, *path, **options):
    target = os.path.join(*path)
    image.save(target, **options)


def write_android(pick):
    mark = blue_mark(pick)
    for density, size in ANDROID_DENSITIES.items():
        folder = os.path.join(ANDROID_RES, f"mipmap-{density}")
        save(render(pick, size, ANDROID_PICK_HEIGHT), folder, "ic_launcher_foreground.webp", lossless=True)
        save(render(mark, size, ANDROID_PICK_HEIGHT), folder, "ic_launcher_monochrome.webp", lossless=True)
        background = os.path.join(folder, "ic_launcher_background.webp")
        if os.path.exists(background):
            os.remove(background)
    save(render(pick, 512, 0.82, LAUNCHER_BACKGROUND).convert("RGB"), ROOT, "app", "android", "appIcon.png")


def write_desktop(pick):
    window = render(pick, 512, 0.96)
    save(window, ROOT, "app", "icons", "app_icon.png")
    for path in glob.glob(os.path.join(DESKTOP_DRAWABLE, "app_icon*.png")):
        save(window, path)
    sizes = [(16, 16), (24, 24), (32, 32), (48, 48), (64, 64), (128, 128), (256, 256)]
    ico = os.path.join(ROOT, "app", "icons", "appIcon.ico")
    save(render(pick, 256, 0.96), ico, sizes=sizes)
    icns = os.path.join(ROOT, "app", "icons", "appIcon.icns")
    save(render(pick, 1024, 0.82, LAUNCHER_BACKGROUND), icns)
    for name in ("appIcon.ico", "appIcon.icns"):
        shutil.copyfile(os.path.join(ROOT, "app", "icons", name), os.path.join(DESKTOP_RESOURCES, name))


def write_web(pick):
    icon = render(pick, 192, 0.96)
    save(icon, ROOT, "app", "icons", "icon-192.png")
    for path in glob.glob(os.path.join(WEB_RESOURCES, "favicon-*.png")):
        save(icon, path)


def write_ios(pick):
    light = render(pick, 1024, 0.82, LAUNCHER_BACKGROUND).convert("RGB")
    dark = render(pick, 1024, 0.82, (38, 38, 42, 255)).convert("RGB")
    tinted = render(pick, 1024, 0.82, (0, 0, 0, 255)).convert("L").point(lambda value: min(255, value * 3)).convert("RGB")
    for folder in [os.path.join(ROOT, "app", "icons", "ios")] + glob.glob(os.path.join(IOS_ASSETS, "AppIcon*.appiconset")):
        save(light, folder, "icon.png")
        save(dark, folder, "icon-dark.png")
        save(tinted, folder, "icon-tinted.png")


def trace_path(mask, height):
    """The outline of [mask], a black and white picture, as the path data of a vector that is [height] units tall."""
    import numpy
    import potrace

    bitmap = potrace.Bitmap(numpy.array(mask.convert("L")) <= 127)
    scale = height / mask.height
    offset_x = (24 - mask.width * scale) / 2
    offset_y = (24 - height) / 2

    def point(p):
        return f"{p.x * scale + offset_x:.2f},{p.y * scale + offset_y:.2f}"

    commands = []
    for curve in bitmap.trace(turdsize=12, alphamax=1.0, opttolerance=0.4):
        commands.append(f"M{point(curve.start_point)}")
        for segment in curve.segments:
            if segment.is_corner:
                commands.append(f"L{point(segment.c)}L{point(segment.end_point)}")
            else:
                commands.append(f"C{point(segment.c1)} {point(segment.c2)} {point(segment.end_point)}")
        commands.append("Z")
    return "".join(commands)


def write_in_app_mark(pick):
    mark = blue_mark(pick)
    bounds = mark.split()[3].point(lambda value: 255 if value > 24 else 0).getbbox()
    shape = mark.crop(bounds).split()[3]
    path_data = trace_path(shape, height=22)
    header = open(os.path.join(PRESENTATION_DRAWABLE, "ic_add.xml"), encoding="utf-8").read().split("<vector")[0]
    body = [
        '<vector xmlns:android="http://schemas.android.com/apk/res/android"',
        '    android:width="24dp"',
        '    android:height="24dp"',
        '    android:viewportWidth="24"',
        '    android:viewportHeight="24">',
        "    <path",
        '        android:fillColor="#FF000000"',
        '        android:fillType="evenOdd"',
        f'        android:pathData="{path_data}"/>',
        "</vector>",
        "",
    ]
    vector = header + chr(10).join(body)
    with open(os.path.join(PRESENTATION_DRAWABLE, "ic_sd_mark.xml"), "w", encoding="utf-8", newline="") as file:
        file.write(vector)


if __name__ == "__main__":
    source = load_pick()
    write_android(source)
    write_desktop(source)
    write_web(source)
    write_ios(source)
    write_in_app_mark(source)

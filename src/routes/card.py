import sys
from pathlib import Path
from PIL import Image, ImageDraw, ImageFont

name, tagline, category = sys.argv[1:]
image = Image.new("RGB", (1200, 630), "#fff2d9")
draw = ImageDraw.Draw(image)
ink = "#251e19"


def font(size, bold=False):
    face = "DejaVuSans-Bold" if bold else "DejaVuSans"
    return ImageFont.truetype(face + ".ttf", size)


def fit(text, face, width):
    if draw.textlength(text, font=face) <= width:
        return text
    while text and draw.textlength(text + "…", font=face) > width:
        text = text[:-1]
    return text.rstrip() + "…"


icon_path = Path("/app/card-icon.png")
if not icon_path.exists():
    icon_path = Path("assets/apple-touch-icon.png")
icon = Image.open(icon_path).convert("RGBA").resize((64, 64), Image.Resampling.LANCZOS)
image.paste(icon, (56, 32), icon)
draw.text((138, 40), "Built with Rust", font=font(36, True), fill=ink)
draw.line((56, 122, 1144, 122), fill=ink, width=2)
draw.text((56, 158), fit(category.upper()[:128], font(25, True), 1080), font=font(25, True), fill="#964021")
title = " ".join(name.split())[:180]
size = 94
while size > 54 and draw.textlength(title, font=font(size, True)) > 1088:
    size -= 4
face = font(size, True)
draw.text((52, 214), fit(title, face, 1088), font=face, fill=ink)
face = font(40)
words = tagline[:1000].split()
for row in range(3):
    if not words:
        break
    line = words.pop(0)
    while words and draw.textlength(line + " " + words[0], font=face) <= 1088:
        line += " " + words.pop(0)
    if row == 2 and words:
        line += "…"
    draw.text((56, 344 + row * 52), fit(line, face, 1088), font=face, fill=ink)
draw.rectangle((0, 560, 1200, 630), fill="#ff804e")
draw.text((56, 578), "builtwithrust.com  /  Project directory", font=font(24, True), fill=ink)
image.save(sys.stdout.buffer, format="PNG")

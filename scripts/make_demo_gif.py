#!/usr/bin/env python3
"""Render docs/dev-scaffold.gif from a canned dry-run session."""

from pathlib import Path

from PIL import Image, ImageDraw, ImageFont

WIDTH, HEIGHT = 920, 560
BG = (13, 17, 23)
FG = (230, 237, 243)
CYAN = (56, 189, 190)
GREEN = (63, 185, 80)
YELLOW = (210, 168, 66)
DIM = (125, 133, 144)
WHITE = (255, 255, 255)
CURSOR = (56, 189, 190)

ROOT = Path(__file__).resolve().parents[1]
OUT = ROOT / "docs" / "dev-scaffold.gif"

COMMAND = "dev-scaffold payment-service --type api --dry-run"

OUTPUT = """\
  ◆  DEV-SCAFFOLD  v0.1.0
     scaffold a service · secure defaults
  ────────────────────────────────────────

  Dry run   payment-service
  template  api

  payment-service/
  ├── .github/
  │   ├── workflows/
  │   │   └── ci.yml
  │   └── dependabot.yml
  ├── app/
  │   ├── main.py
  │   └── config.py
  ├── tests/
  ├── Dockerfile
  ├── pyproject.toml
  └── .env.example
""".rstrip("\n")

FONT_CANDIDATES = [
    "/System/Library/Fonts/Menlo.ttc",
    "/System/Library/Fonts/Monaco.ttf",
    "/Library/Fonts/Menlo.ttc",
]


def load_font(size: int) -> ImageFont.FreeTypeFont | ImageFont.ImageFont:
    for path in FONT_CANDIDATES:
        try:
            return ImageFont.truetype(path, size, index=0)
        except OSError:
            continue
    return ImageFont.load_default()


def color_for(line: str) -> tuple[int, int, int]:
    stripped = line.lstrip()
    if "DEV-SCAFFOLD" in line or stripped.startswith("◆"):
        return CYAN
    if stripped.startswith("──"):
        return CYAN
    if "Dry run" in line:
        return YELLOW
    if "template" in line and "api" in line:
        return GREEN
    if stripped.startswith("payment-service/"):
        return CYAN
    if stripped.startswith("├") or stripped.startswith("│") or stripped.startswith("└"):
        if stripped.rstrip().endswith("/"):
            return CYAN
        return FG
    if stripped.startswith("scaffold a service"):
        return DIM
    return FG


def render(text: str, show_cursor: bool) -> Image.Image:
    img = Image.new("RGB", (WIDTH, HEIGHT), BG)
    draw = ImageDraw.Draw(img)
    font = load_font(16)
    x0, y = 28, 24
    line_h = 22
    for line in text.split("\n"):
        draw.text((x0, y), line, font=font, fill=color_for(line))
        y += line_h
        if y > HEIGHT - 36:
            break
    if show_cursor:
        cursor_line = text.split("\n")[-1] if text else ""
        bbox = draw.textbbox((x0, 0), cursor_line, font=font)
        cx = x0 + (bbox[2] - bbox[0]) + 2
        cy = 24 + line_h * (text.count("\n"))
        draw.rectangle((cx, cy + 3, cx + 8, cy + 16), fill=CURSOR)
    return img


def main() -> None:
    OUT.parent.mkdir(parents=True, exist_ok=True)
    frames: list[Image.Image] = []
    durations: list[int] = []

    prompt = "$ "
    for i in range(len(COMMAND) + 1):
        frames.append(render(prompt + COMMAND[:i], show_cursor=True))
        durations.append(55 if i else 400)

    typed = prompt + COMMAND
    frames.append(render(typed, show_cursor=False))
    durations.append(350)

    lines = OUTPUT.split("\n")
    body = typed + "\n"
    for i, _line in enumerate(lines):
        body = typed + "\n" + "\n".join(lines[: i + 1])
        frames.append(render(body, show_cursor=False))
        durations.append(90)

    frames.append(render(typed + "\n" + OUTPUT, show_cursor=False))
    durations.append(3500)

    frames[0].save(
        OUT,
        save_all=True,
        append_images=frames[1:],
        duration=durations,
        loop=0,
        optimize=True,
    )
    print(f"wrote {OUT} ({len(frames)} frames)")


if __name__ == "__main__":
    main()

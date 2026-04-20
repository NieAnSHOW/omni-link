"""
Generate macOS app icon following Apple HIG specifications.

macOS icon specs (since Big Sur):
- Canvas: 1024x1024 px
- Squircle inner: 824x824 px centered (100px gutter)
- Corner radius: ~185.4 px (superellipse n≈5)
- Drop shadow: 28px blur, 12px Y-offset, black 50%
- Transparency: allowed
- Color: sRGB or Display P3
"""

import math
import os
from pathlib import Path
from PIL import Image, ImageDraw, ImageFilter

SOURCE = "rounded_image.png"
OUTPUT_DIR = "src-tauri/icons"
CANVAS = 1024
INNER = 824
GUTTER = (CANVAS - INNER) // 2  # 100px
N = 5  # superellipse exponent


def create_squircle_mask(size, exponent=N):
    """Create a squircle (superellipse) mask."""
    a = size / 2
    mask = Image.new("L", (size, size), 0)
    pixels = mask.load()

    for y in range(size):
        for x in range(size):
            dx = (x - a + 0.5) / a
            dy = (y - a + 0.5) / a
            # superellipse: |dx|^n + |dy|^n <= 1
            val = abs(dx) ** exponent + abs(dy) ** exponent
            if val <= 1.0:
                # Anti-alias edge
                edge_dist = (1.0 - val) * a
                if edge_dist < 1.0:
                    pixels[x, y] = int(edge_dist * 255)
                else:
                    pixels[x, y] = 255
    return mask


def create_macos_icon(source_path):
    """Create a macOS-compliant app icon from a pre-rounded source image."""
    source = Image.open(source_path).convert("RGBA")

    # Resize source to fill the inner area (source is already rounded with transparency)
    source_resized = source.resize((INNER, INNER), Image.LANCZOS)

    # Create drop shadow from source alpha
    print("Adding drop shadow...")
    source_alpha = source_resized.split()[3]
    shadow_inner = Image.new("RGBA", (INNER, INNER), (0, 0, 0, 0))
    black_layer = Image.new("RGBA", (INNER, INNER), (0, 0, 0, 128))
    black_layer.putalpha(source_alpha)

    shadow_layer = Image.new("RGBA", (CANVAS, CANVAS), (0, 0, 0, 0))
    shadow_y_offset = 12
    shadow_layer.paste(black_layer, (GUTTER, GUTTER + shadow_y_offset))

    shadow_blurred = shadow_layer.filter(ImageFilter.GaussianBlur(radius=28))

    # Compose final icon
    print("Composing final icon...")
    canvas = Image.new("RGBA", (CANVAS, CANVAS), (0, 0, 0, 0))
    canvas.paste(shadow_blurred, (0, 0), shadow_blurred)
    canvas.paste(source_resized, (GUTTER, GUTTER), source_resized)

    return canvas


def generate_sizes(icon, output_dir):
    """Generate all required macOS icon sizes."""
    sizes = [
        ("icon_512x512@2x", 1024),
        ("icon_512x512", 512),
        ("icon_256x256@2x", 512),
        ("icon_256x256", 256),
        ("icon_128x128@2x", 256),
        ("icon_128x128", 128),
        ("icon_32x32@2x", 64),
        ("icon_32x32", 32),
        ("icon_16x16@2x", 32),
        ("icon_16x16", 16),
    ]

    for name, size in sizes:
        resized = icon.resize((size, size), Image.LANCZOS)
        path = os.path.join(output_dir, f"{name}.png")
        resized.save(path, "PNG")
        print(f"  Generated: {path} ({size}x{size})")


def generate_appstore_icon(icon, output_dir):
    """Generate 1024x1024 App Store icon (no shadow, squircle mask only)."""
    # App Store icon is just the 1024x1024 icon as-is
    path = os.path.join(output_dir, "icon_1024x1024.png")
    icon.save(path, "PNG")
    print(f"  Generated: {path} (1024x1024 App Store)")


def main():
    output_dir = OUTPUT_DIR
    os.makedirs(output_dir, exist_ok=True)

    print(f"Source: {SOURCE}")
    print(f"Output: {output_dir}")
    print(f"Canvas: {CANVAS}x{CANVAS}, Inner: {INNER}x{INNER}")
    print()

    icon = create_macos_icon(SOURCE)

    print("\nGenerating icon sizes...")
    generate_sizes(icon, output_dir)
    generate_appstore_icon(icon, output_dir)

    # Also save the master icon for reference
    master_path = os.path.join(output_dir, "macos_master_1024.png")
    icon.save(master_path, "PNG")
    print(f"\n  Master: {master_path}")

    print("\nDone! All macOS icons generated.")


if __name__ == "__main__":
    main()

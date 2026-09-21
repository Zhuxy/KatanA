#!/usr/bin/env python3
"""
Generate the Dark Gold (暗金色) icon for KatanB from KatanA's master icon.
Preserves 100% of the original katana blade lighting, gradients, and atmospheric dark background,
while grading the palette into a rich antique gold / dark bronze aesthetic.
"""

import sys
import os
import shutil
import subprocess
from PIL import Image
import numpy as np

def generate_dark_gold_image(input_path: str) -> Image.Image:
    im_orig = Image.open(input_path).convert('RGB')
    arr = np.array(im_orig, dtype=float)

    # Luminance of original
    luma = (0.299 * arr[:, :, 0] + 0.587 * arr[:, :, 1] + 0.114 * arr[:, :, 2]) / 255.0

    # Color grading control points (Dark Gold / Antique Bronze Gold)
    # Deep warm bronze-black shadows -> antique gold midtones -> radiant golden sword -> white-gold edge
    ctrl_luma = [0.0,   0.06,  0.15,  0.30,  0.50,  0.75,  0.92,  1.0]
    ctrl_r =    [16.0,  32.0,  68.0, 135.0, 195.0, 245.0, 255.0, 255.0]
    ctrl_g =    [11.0,  22.0,  48.0,  95.0, 145.0, 195.0, 242.0, 255.0]
    ctrl_b =    [ 4.0,   7.0,  12.0,  22.0,  35.0,  65.0, 150.0, 240.0]

    out = np.zeros_like(arr)
    out[:, :, 0] = np.interp(luma, ctrl_luma, ctrl_r)
    out[:, :, 1] = np.interp(luma, ctrl_luma, ctrl_g)
    out[:, :, 2] = np.interp(luma, ctrl_luma, ctrl_b)

    return Image.fromarray(out.clip(0, 255).astype(np.uint8))

def build_icns(img: Image.Image, output_icns_path: str):
    iconset_dir = output_icns_path + ".iconset"
    if os.path.exists(iconset_dir):
        shutil.rmtree(iconset_dir)
    os.makedirs(iconset_dir, exist_ok=True)

    sizes = [
        (16, 'icon_16x16.png'),
        (32, 'icon_16x16@2x.png'),
        (32, 'icon_32x32.png'),
        (64, 'icon_32x32@2x.png'),
        (128, 'icon_128x128.png'),
        (256, 'icon_128x128@2x.png'),
        (256, 'icon_256x256.png'),
        (512, 'icon_256x256@2x.png'),
        (512, 'icon_512x512.png'),
    ]

    for sz, fname in sizes:
        resized = img.resize((sz, sz), Image.Resampling.LANCZOS)
        resized.save(os.path.join(iconset_dir, fname), 'PNG')

    if os.path.exists(output_icns_path):
        os.remove(output_icns_path)

    os.makedirs(os.path.dirname(os.path.abspath(output_icns_path)), exist_ok=True)
    subprocess.check_call(['iconutil', '-c', 'icns', iconset_dir, '-o', output_icns_path])
    shutil.rmtree(iconset_dir)
    print(f"Generated ICNS: {output_icns_path}")

def main():
    root_dir = os.path.abspath(os.path.join(os.path.dirname(__file__), "..", ".."))
    default_input = os.path.join(root_dir, "assets", "icon.iconset", "icon_512x512.png")
    default_output = os.path.join(root_dir, "target", "icon_b.icns")

    input_path = sys.argv[1] if len(sys.argv) > 1 else default_input
    output_path = sys.argv[2] if len(sys.argv) > 2 else default_output

    if not os.path.exists(input_path):
        print(f"Error: input file not found at {input_path}")
        sys.exit(1)

    print(f"Reading master icon from {input_path}...")
    gold_img = generate_dark_gold_image(input_path)
    build_icns(gold_img, output_path)

if __name__ == "__main__":
    main()

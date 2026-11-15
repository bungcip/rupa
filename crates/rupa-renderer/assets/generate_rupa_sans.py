#!/usr/bin/env python3
"""Generate the vendored Rupa Sans default font.

The font is a small 5x7 pixel-outline face covering printable ASCII. It is
authored for this repository and released under the SIL Open Font License 1.1
(see OFL.txt). Regenerate both weights with:

    python3 crates/rupa-renderer/assets/generate_rupa_sans.py

The renderer pins the resulting SHA-256 digests in `crates/rupa-renderer/src/fonts.rs`.
"""

from __future__ import annotations

import hashlib
import struct
from pathlib import Path

PIXEL = 128
UNITS_PER_EM = 1024
ASCENT = 7 * PIXEL
DESCENT = PIXEL
ADVANCE = 6 * PIXEL
SPACE_ADVANCE = 4 * PIXEL

# Printable ASCII, 5 columns x 7 rows. '#' is ink.
GLYPHS = {
    " ": ["00000"] * 7,
    "!": ["..#..", "..#..", "..#..", "..#..", "..#..", ".....", "..#.."],
    '"': [".#.#.", ".#.#.", ".....", ".....", ".....", ".....", "....."],
    "#": [".#.#.", ".#.#.", "#####", ".#.#.", "#####", ".#.#.", ".#.#."],
    "$": ["..#..", ".####", "#.#..", ".###.", "..#.#", "####.", "..#.."],
    "%": ["##...", "##..#", "...#.", "..#..", ".#...", "#..##", "...##"],
    "&": [".##..", "#..#.", "#.#..", ".#...", "#.#.#", "#..#.", ".##.#"],
    "'": ["..#..", "..#..", ".....", ".....", ".....", ".....", "....."],
    "(": ["...#.", "..#..", ".#...", ".#...", ".#...", "..#..", "...#."],
    ")": [".#...", "..#..", "...#.", "...#.", "...#.", "..#..", ".#..."],
    "*": [".....", "#.#.#", ".###.", "#####", ".###.", "#.#.#", "....."],
    "+": [".....", "..#..", "..#..", "#####", "..#..", "..#..", "....."],
    ",": [".....", ".....", ".....", ".....", ".....", "..##.", "..#.."],
    "-": [".....", ".....", ".....", "#####", ".....", ".....", "....."],
    ".": [".....", ".....", ".....", ".....", ".....", ".##..", ".##.."],
    "/": ["....#", "...#.", "...#.", "..#..", ".#...", ".#...", "#...."],
    "0": [".###.", "#...#", "#..##", "#.#.#", "##..#", "#...#", ".###."],
    "1": ["..#..", ".##..", "..#..", "..#..", "..#..", "..#..", ".###."],
    "2": [".###.", "#...#", "....#", "...#.", "..#..", ".#...", "#####"],
    "3": ["#####", "...#.", "..#..", "...#.", "....#", "#...#", ".###."],
    "4": ["...#.", "..##.", ".#.#.", "#..#.", "#####", "...#.", "...#."],
    "5": ["#####", "#....", "####.", "....#", "....#", "#...#", ".###."],
    "6": ["..##.", ".#...", "#....", "####.", "#...#", "#...#", ".###."],
    "7": ["#####", "....#", "...#.", "..#..", ".#...", ".#...", ".#..."],
    "8": [".###.", "#...#", "#...#", ".###.", "#...#", "#...#", ".###."],
    "9": [".###.", "#...#", "#...#", ".####", "....#", "...#.", ".##.."],
    ":": [".....", ".##..", ".##..", ".....", ".##..", ".##..", "....."],
    ";": [".....", ".##..", ".##..", ".....", ".##..", "..#..", ".#..."],
    "<": ["...#.", "..#..", ".#...", "#....", ".#...", "..#..", "...#."],
    "=": [".....", ".....", "#####", ".....", "#####", ".....", "....."],
    ">": [".#...", "..#..", "...#.", "....#", "...#.", "..#..", ".#..."],
    "?": [".###.", "#...#", "....#", "...#.", "..#..", ".....", "..#.."],
    "@": [".###.", "#...#", "#.###", "#.#.#", "#.###", "#....", ".###."],
    "A": ["..#..", ".#.#.", "#...#", "#...#", "#####", "#...#", "#...#"],
    "B": ["####.", "#...#", "#...#", "####.", "#...#", "#...#", "####."],
    "C": [".###.", "#...#", "#....", "#....", "#....", "#...#", ".###."],
    "D": ["###..", "#..#.", "#...#", "#...#", "#...#", "#..#.", "###.."],
    "E": ["#####", "#....", "#....", "####.", "#....", "#....", "#####"],
    "F": ["#####", "#....", "#....", "####.", "#....", "#....", "#...."],
    "G": [".###.", "#...#", "#....", "#.###", "#...#", "#...#", ".###."],
    "H": ["#...#", "#...#", "#...#", "#####", "#...#", "#...#", "#...#"],
    "I": [".###.", "..#..", "..#..", "..#..", "..#..", "..#..", ".###."],
    "J": ["..###", "...#.", "...#.", "...#.", "...#.", "#..#.", ".##.."],
    "K": ["#...#", "#..#.", "#.#..", "##...", "#.#..", "#..#.", "#...#"],
    "L": ["#....", "#....", "#....", "#....", "#....", "#....", "#####"],
    "M": ["#...#", "##.##", "#.#.#", "#.#.#", "#...#", "#...#", "#...#"],
    "N": ["#...#", "##..#", "#.#.#", "#..##", "#...#", "#...#", "#...#"],
    "O": [".###.", "#...#", "#...#", "#...#", "#...#", "#...#", ".###."],
    "P": ["####.", "#...#", "#...#", "####.", "#....", "#....", "#...."],
    "Q": [".###.", "#...#", "#...#", "#...#", "#.#.#", "#..#.", ".##.#"],
    "R": ["####.", "#...#", "#...#", "####.", "#.#..", "#..#.", "#...#"],
    "S": [".####", "#....", "#....", ".###.", "....#", "....#", "####."],
    "T": ["#####", "..#..", "..#..", "..#..", "..#..", "..#..", "..#.."],
    "U": ["#...#", "#...#", "#...#", "#...#", "#...#", "#...#", ".###."],
    "V": ["#...#", "#...#", "#...#", "#...#", "#...#", ".#.#.", "..#.."],
    "W": ["#...#", "#...#", "#...#", "#.#.#", "#.#.#", "##.##", "#...#"],
    "X": ["#...#", "#...#", ".#.#.", "..#..", ".#.#.", "#...#", "#...#"],
    "Y": ["#...#", "#...#", ".#.#.", "..#..", "..#..", "..#..", "..#.."],
    "Z": ["#####", "....#", "...#.", "..#..", ".#...", "#....", "#####"],
    "[": [".###.", ".#...", ".#...", ".#...", ".#...", ".#...", ".###."],
    "\\": ["#....", ".#...", ".#...", "..#..", "...#.", "...#.", "....#"],
    "]": [".###.", "...#.", "...#.", "...#.", "...#.", "...#.", ".###."],
    "^": ["..#..", ".#.#.", "#...#", ".....", ".....", ".....", "....."],
    "_": [".....", ".....", ".....", ".....", ".....", ".....", "#####"],
    "`": [".#...", "..#..", ".....", ".....", ".....", ".....", "....."],
    "a": [".....", ".....", ".###.", "....#", ".####", "#...#", ".####"],
    "b": ["#....", "#....", "####.", "#...#", "#...#", "#...#", "####."],
    "c": [".....", ".....", ".###.", "#...#", "#....", "#...#", ".###."],
    "d": ["....#", "....#", ".####", "#...#", "#...#", "#...#", ".####"],
    "e": [".....", ".....", ".###.", "#...#", "#####", "#....", ".###."],
    "f": ["..##.", ".#...", ".#...", "####.", ".#...", ".#...", ".#..."],
    "g": [".....", ".####", "#...#", "#...#", ".####", "....#", ".###."],
    "h": ["#....", "#....", "####.", "#...#", "#...#", "#...#", "#...#"],
    "i": ["..#..", ".....", ".##..", "..#..", "..#..", "..#..", ".###."],
    "j": ["...#.", ".....", "..##.", "...#.", "...#.", "#..#.", ".##.."],
    "k": ["#....", "#....", "#..#.", "#.#..", "##...", "#.#..", "#..#."],
    "l": [".##..", "..#..", "..#..", "..#..", "..#..", "..#..", ".###."],
    "m": [".....", ".....", "##.#.", "#.#.#", "#.#.#", "#...#", "#...#"],
    "n": [".....", ".....", "####.", "#...#", "#...#", "#...#", "#...#"],
    "o": [".....", ".....", ".###.", "#...#", "#...#", "#...#", ".###."],
    "p": [".....", "####.", "#...#", "#...#", "####.", "#....", "#...."],
    "q": [".....", ".####", "#...#", "#...#", ".####", "....#", "....#"],
    "r": [".....", ".....", "#.##.", "##...", "#....", "#....", "#...."],
    "s": [".....", ".....", ".####", "#....", ".###.", "....#", "####."],
    "t": [".#...", ".#...", "####.", ".#...", ".#...", ".#...", "..##."],
    "u": [".....", ".....", "#...#", "#...#", "#...#", "#...#", ".####"],
    "v": [".....", ".....", "#...#", "#...#", "#...#", ".#.#.", "..#.."],
    "w": [".....", ".....", "#...#", "#...#", "#.#.#", "#.#.#", ".#.#."],
    "x": [".....", ".....", "#...#", ".#.#.", "..#..", ".#.#.", "#...#"],
    "y": [".....", "#...#", "#...#", "#...#", ".####", "....#", ".###."],
    "z": [".....", ".....", "#####", "...#.", "..#..", ".#...", "#####"],
    "{": ["...#.", "..#..", "..#..", ".#...", "..#..", "..#..", "...#."],
    "|": ["..#..", "..#..", "..#..", "..#..", "..#..", "..#..", "..#.."],
    "}": [".#...", "..#..", "..#..", "...#.", "..#..", "..#..", ".#..."],
    "~": [".....", ".....", ".#..#", "#.#.#", "#..#.", ".....", "....."],
}

assert set(GLYPHS) == {chr(code) for code in range(32, 127)}


def bold_rows(rows: list[str]) -> list[str]:
    thick = []
    for row in rows:
        expanded = []
        for index in range(len(row)):
            ink = row[index] == "#" or (index + 1 < len(row) and row[index + 1] == "#")
            expanded.append("#" if ink else "0")
        thick.append("".join(expanded))
    return thick


def checksum(data: bytes) -> int:
    padded = data + b"\0" * ((4 - len(data) % 4) % 4)
    return sum(struct.unpack(">" + "I" * (len(padded) // 4), padded)) & 0xFFFFFFFF


def pad4(data: bytes) -> bytes:
    return data + b"\0" * ((4 - len(data) % 4) % 4)


def encode_glyph(rows: list[str]) -> bytes:
    contours = []
    for row_index, row in enumerate(rows):
        for col_index, cell in enumerate(row):
            if cell != "#":
                continue
            x0 = col_index * PIXEL
            x1 = x0 + PIXEL
            y1 = (7 - row_index) * PIXEL
            y0 = y1 - PIXEL
            contours.append([(x0, y0), (x0, y1), (x1, y1), (x1, y0)])

    if not contours:
        return b""

    points = [point for contour in contours for point in contour]
    x_min = min(x for x, _ in points)
    y_min = min(y for _, y in points)
    x_max = max(x for x, _ in points)
    y_max = max(y for _, y in points)

    end_points = []
    total = 0
    for contour in contours:
        total += len(contour)
        end_points.append(total - 1)

    data = bytearray()
    data += struct.pack(">hhhhh", len(contours), x_min, y_min, x_max, y_max)
    data += struct.pack(">" + "H" * len(end_points), *end_points)
    data += struct.pack(">H", 0)  # no instructions
    data += b"\x01" * len(points)  # every point is on-curve, 16-bit deltas
    previous_x = 0
    for x, _ in points:
        data += struct.pack(">h", x - previous_x)
        previous_x = x
    previous_y = 0
    for _, y in points:
        data += struct.pack(">h", y - previous_y)
        previous_y = y
    return bytes(data)


def build_glyf(rows_by_char: dict[str, list[str]]) -> tuple[bytes, list[int]]:
    glyf = bytearray()
    offsets = [0]
    # Glyph 0 is the required .notdef box; printable ASCII starts at glyph 1.
    notdef = ["#####", "#...#", "#...#", "#...#", "#...#", "#...#", "#####"]
    glyf += encode_glyph(notdef)
    glyf += b"\0" * ((2 - len(glyf) % 2) % 2)
    offsets.append(len(glyf))
    for code in range(32, 127):
        data = encode_glyph(rows_by_char[chr(code)])
        glyf += data
        glyf += b"\0" * ((2 - len(glyf) % 2) % 2)
        offsets.append(len(glyf))
    return bytes(glyf), offsets


def encode_name(records: list[tuple[int, str]]) -> bytes:
    format_ = 0
    count = len(records)
    string_offset = 6 + 12 * count
    strings = bytearray()
    entries = bytearray()
    for name_id, value in records:
        encoded = value.encode("utf-16-be")
        entries += struct.pack(">HHHHHH", 3, 1, 0x0409, name_id, len(encoded), len(strings))
        strings += encoded
    return struct.pack(">HHH", format_, count, string_offset) + bytes(entries) + bytes(strings)


def encode_cmap() -> bytes:
    seg_count = 2
    end_codes = [126, 0xFFFF]
    start_codes = [32, 0xFFFF]
    deltas = [(-31) & 0xFFFF, 1]
    range_offsets = [0, 0]
    subtable = bytearray()
    subtable += struct.pack(">HHHHHHH", 4, 0, 0, seg_count * 2, 4, 1, 0)
    subtable += struct.pack(">" + "H" * seg_count, *end_codes)
    subtable += struct.pack(">H", 0)
    subtable += struct.pack(">" + "H" * seg_count, *start_codes)
    subtable += struct.pack(">" + "H" * seg_count, *deltas)
    subtable += struct.pack(">" + "H" * seg_count, *range_offsets)
    # Fill in the subtable length after building the payload.
    struct.pack_into(">H", subtable, 2, len(subtable))
    header = struct.pack(">HHHHI", 0, 1, 3, 1, 12)
    return header + bytes(subtable)


def build_font(bold: bool) -> bytes:
    rows_by_char = {
        char: bold_rows(rows) if bold else rows for char, rows in GLYPHS.items()
    }
    glyf, loca_offsets = build_glyf(rows_by_char)
    num_glyphs = 96
    max_points = max(
        sum(row.count("#") for row in rows) * 4
        for rows in [*rows_by_char.values(), ["#####", "#...#", "#...#", "#...#", "#...#", "#...#", "#####"]]
    )
    max_contours = max(
        sum(row.count("#") for row in rows)
        for rows in [*rows_by_char.values(), ["#####", "#...#", "#...#", "#...#", "#...#", "#...#", "#####"]]
    )

    head = struct.pack(
        ">IIIIHHqqhhhhHHhhh",
        0x00010000,
        0x00010000,
        0,
        0x5F0F3CF5,
        0,
        UNITS_PER_EM,
        0,
        0,
        0,
        -DESCENT,
        5 * PIXEL,
        ASCENT,
        0x20 if bold else 0,
        8,
        2,
        1,
        0,
    )

    hhea = struct.pack(
        ">IhhhHhhhhhhhhhhhH",
        0x00010000,
        ASCENT,
        -DESCENT,
        0,
        ADVANCE,
        0,
        0,
        5 * PIXEL,
        1,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        num_glyphs,
    )

    maxp = struct.pack(
        ">IHHHHHHHHHHHHHH",
        0x00010000,
        num_glyphs,
        max_points,
        max_contours,
        0,
        0,
        2,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
    )

    hmtx = bytearray()
    hmtx += struct.pack(">Hh", ADVANCE, 0)  # .notdef
    for code in range(32, 127):
        advance = SPACE_ADVANCE if chr(code) == " " else ADVANCE
        hmtx += struct.pack(">Hh", advance, 0)

    cmap = encode_cmap()
    loca = b"".join(struct.pack(">I", offset) for offset in loca_offsets)

    family = "Rupa Sans"
    subfamily = "Bold" if bold else "Regular"
    full_name = f"{family} {subfamily}"
    postscript = f"RupaSans-{subfamily}"
    name = encode_name(
        [
            (0, "Copyright (c) 2026 Rupa contributors"),
            (1, family),
            (2, subfamily),
            (4, full_name),
            (6, postscript),
            (13, "SIL Open Font License 1.1"),
            (14, "https://openfontlicense.org"),
        ]
    )

    post = struct.pack(">IIhhIIIII", 0x00030000, 0, 0, -128, 64, 0, 0, 0, 0)

    os2 = struct.pack(
        ">HhHHH" + "h" * 11 + "10sIIII4sHHHhhhHHIIhhHHH",
        4,
        ADVANCE,
        700 if bold else 400,
        5,
        0,
        650,
        600,
        0,
        75,
        650,
        600,
        0,
        350,
        50,
        250,
        0,
        b"\0" * 10,
        1,
        0,
        0,
        0,
        b"RUPU",
        0x20 if bold else 0x40,
        32,
        126,
        ASCENT,
        -DESCENT,
        0,
        ASCENT,
        DESCENT,
        1,
        0,
        512,
        768,
        0,
        32,
        1,
    )

    assert len(head) == 54, len(head)
    assert len(hhea) == 36, len(hhea)
    assert len(maxp) == 32, len(maxp)
    assert len(os2) == 96, len(os2)
    assert len(post) == 32, len(post)

    tables = {
        b"OS/2": os2,
        b"cmap": cmap,
        b"glyf": glyf,
        b"head": head,
        b"hhea": hhea,
        b"hmtx": bytes(hmtx),
        b"loca": loca,
        b"maxp": maxp,
        b"name": name,
        b"post": post,
    }

    tags = sorted(tables)
    num_tables = len(tags)
    header_size = 12 + 16 * num_tables
    offset = header_size
    offsets = {}
    for tag in tags:
        offset += (4 - offset % 4) % 4
        offsets[tag] = offset
        offset += len(pad4(tables[tag]))

    entries = bytearray()
    for tag in tags:
        data = tables[tag]
        entries += struct.pack(">4sIII", tag, checksum(data), offsets[tag], len(data))

    font = bytearray(struct.pack(">IHHHH", 0x00010000, num_tables, 128, 3, 16))
    font += entries
    for tag in tags:
        font += b"\0" * ((4 - len(font) % 4) % 4)
        assert len(font) == offsets[tag], (tag, len(font), offsets[tag])
        font += pad4(tables[tag])

    # head.checkSumAdjustment = 0xB1B0AFBA - checksum(entire font, with the field zero)
    adjustment = (0xB1B0AFBA - checksum(bytes(font))) & 0xFFFFFFFF
    head_offset = offsets[b"head"]
    struct.pack_into(">I", font, head_offset + 8, adjustment)
    return bytes(font)


def main() -> None:
    asset_dir = Path(__file__).resolve().parent
    for bold, suffix in ((False, "Regular"), (True, "Bold")):
        data = build_font(bold)
        path = asset_dir / f"RupaSans-{suffix}.ttf"
        path.write_bytes(data)
        print(f"{path.name}: {len(data)} bytes sha256={hashlib.sha256(data).hexdigest()}")


if __name__ == "__main__":
    main()

#!/usr/bin/env python3
#
# check_image.py - validate the BOOT image header written by scripts/mkimage.py
#
# Author:  Sandesh Ghimire <sandesh@soccentric.com>
# Usage:   scripts/check_image.py build/embassy-app.img [--version 0.1.0]
#
# The image is what actually gets flashed; checking the ELF alone does not
# prove that the header, CRC, load address and reset vector are right.
#

import argparse
import pathlib
import struct
import sys
import zlib

SLOT_BASE = 0x0800C000
SLOT_SIZE = 0x000F4000


def parse_version(text: str) -> int:
    parts = text.split(".")
    if len(parts) != 3:
        raise argparse.ArgumentTypeError("version must be MAJOR.MINOR.PATCH")
    major, minor, patch = (int(part) for part in parts)
    return (major << 24) | (minor << 16) | patch


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("image", type=pathlib.Path)
    parser.add_argument("--version", type=parse_version)
    args = parser.parse_args()

    if not args.image.is_file():
        print(f"image: {args.image} not found", file=sys.stderr)
        return 1

    data = args.image.read_bytes()
    header = struct.unpack_from("<IHHIIIIIII", data, 0)
    magic, hver, hsize, itype, ver, isize, icrc, load, entry, _ts = header

    errors: list[str] = []
    if magic != 0x48574249:
        errors.append("bad image magic")
    if hver != 1:
        errors.append(f"unexpected header version {hver}")
    if hsize != 256:
        errors.append(f"unexpected header size {hsize}")
    if itype != 1:
        errors.append(f"unexpected image type {itype}")
    if args.version is not None and ver != args.version:
        errors.append(f"image version 0x{ver:08X} does not match 0x{args.version:08X}")
    if load != SLOT_BASE:
        errors.append(f"image loads at 0x{load:08X}, expected 0x{SLOT_BASE:08X}")
    if len(data) != hsize + isize:
        errors.append("image size does not match the header")
    if zlib.crc32(data[hsize:]) & 0xFFFFFFFF != icrc:
        errors.append("image CRC32 mismatch")

    sp = struct.unpack_from("<I", data, hsize)[0]
    if not 0x20000000 < sp <= 0x20080000:
        errors.append("initial stack pointer is outside internal RAM")
    if not (entry & 1 and SLOT_BASE <= (entry & ~1) < SLOT_BASE + SLOT_SIZE):
        errors.append("reset vector is outside the slot")

    if errors:
        for error in errors:
            print(f"image: {error}", file=sys.stderr)
        return 1

    print(
        f"image: {args.image} v{(ver >> 24) & 0xFF}.{(ver >> 16) & 0xFF}.{ver & 0xFF} "
        f"({isize} bytes payload, entry 0x{entry:08X})"
    )
    return 0


if __name__ == "__main__":
    sys.exit(main())

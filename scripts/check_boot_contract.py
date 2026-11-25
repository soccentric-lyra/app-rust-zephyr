#!/usr/bin/env python3
#
# check_boot_contract.py - verify the board constants and flash overlay
#
# Author:  Sandesh Ghimire <sandesh@soccentric.com>
# Usage:   scripts/check_boot_contract.py
#
# Fails when src/platform/board.rs or boards/nucleo_f767zi.overlay and the
# BOOT layout have drifted.  The Rust board module holds the addresses the
# application uses; the overlay is what Zephyr actually builds into the image.
#

import pathlib
import re
import sys

BOARD = pathlib.Path("src/platform/board.rs")
OVERLAY = pathlib.Path("boards/nucleo_f767zi.overlay")

EXPECTED_CONSTANTS = {
    "SLOT_BASE": 0x0800C000,
    "CONFIRMED_MAGIC": 0x434F4E46,
}

EXPECTED_PARTITIONS = {
    "app-meta": (0x00008000, 0x00004000),
    "app-slot": (0x0000C000, 0x000F4000),
}

CONST = re.compile(
    r"^\s*pub\s+const\s+([A-Z0-9_]+)\s*:\s*[a-z0-9]+\s*=\s*(0x[0-9A-Fa-f_]+|\d+)\s*;"
)
PARTITION = re.compile(
    r'label\s*=\s*"([^"]+)"\s*;\s*reg\s*=\s*<(0x[0-9A-Fa-f]+)\s+(0x[0-9A-Fa-f]+)>',
    re.DOTALL,
)


def main() -> int:
    if not BOARD.is_file() or not OVERLAY.is_file():
        print("boot contract: board module or devicetree overlay is missing", file=sys.stderr)
        return 1

    failed = False
    constants: dict[str, int] = {}
    for line in BOARD.read_text(encoding="utf-8").splitlines():
        match = CONST.match(line)
        if match:
            constants[match.group(1)] = int(match.group(2).replace("_", ""), 0)

    for name, want in EXPECTED_CONSTANTS.items():
        if name not in constants:
            print(f"boot contract: {name} is missing from {BOARD}", file=sys.stderr)
            failed = True
        elif constants[name] != want:
            print(
                f"boot contract: {name} is 0x{constants[name]:08X}, "
                f"expected 0x{want:08X}",
                file=sys.stderr,
            )
            failed = True

    partitions = {
        label: (int(offset, 16), int(size, 16))
        for label, offset, size in PARTITION.findall(OVERLAY.read_text(encoding="utf-8"))
    }
    for label, want in EXPECTED_PARTITIONS.items():
        if label not in partitions:
            print(f"boot contract: partition {label} is missing from {OVERLAY}", file=sys.stderr)
            failed = True
        elif partitions[label] != want:
            offset, size = partitions[label]
            print(
                f"boot contract: partition {label} is 0x{offset:08X}+0x{size:08X}, "
                f"expected 0x{want[0]:08X}+0x{want[1]:08X}",
                file=sys.stderr,
            )
            failed = True

    if failed:
        return 1
    print("boot contract: board constants and flash overlay match the BOOT layout")
    return 0


if __name__ == "__main__":
    sys.exit(main())

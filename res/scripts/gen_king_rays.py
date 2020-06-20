#!/usr/bin/env python3
"""Pre-compute king ras bitboards for each square."""

TEMPLATE = """\
/// Pre-computed king rays.
const KING_RAYS: [Bitboard; 64] = [
{}
];
"""

DIRS = [(1, 0), (1, 1), (0, 1), (-1, 1), (-1, 0), (-1, -1), (0, -1), (1, -1)]

def bit_pos(square):
    return 1 << square

def get_rays():
    rays = []
    for f in range(8):
        for r in range(8):
            bitboard = 0
            for dir_f, dir_r in DIRS:
                ray_f = f + dir_f
                ray_r = r + dir_r
                if ray_f < 0 or ray_f > 7 or ray_r < 0 or ray_r > 7:
                    continue
                bitboard |= bit_pos(ray_f * 8 + ray_r)
            rays.append("    0b{:064b},".format(bitboard))
    return rays

print(TEMPLATE.format("\n".join(get_rays())))

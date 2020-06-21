#!/usr/bin/env python3
"""Pre-compute pawn captures bitboards for each square."""

TEMPLATE = """\
/// Pre-computed pawn captures.
pub const PAWN_CAPTURES: [[Bitboard; 64]; 2] = [
    [
{}
    ],
    [
{}
    ],
];
"""

def bit_pos(square):
    return 1 << square

def get_captures():
    both_captures = []
    for direction in [1, -1]:
        captures = []
        for f in range(8):
            for r in range(8):
                bitboard = 0
                prog_r = r + direction
                if 0 < prog_r < 7:
                    prev_f = f - 1
                    if prev_f >= 0:
                        bitboard |= bit_pos(prev_f * 8 + prog_r)
                    next_f = f + 1
                    if next_f <= 7:
                        bitboard |= bit_pos(next_f * 8 + prog_r)
                captures.append("        0b{:064b},".format(bitboard))
        both_captures.append(captures)
    return both_captures

CAPTURES = get_captures()
print(TEMPLATE.format("\n".join(CAPTURES[0]), "\n".join(CAPTURES[1])))

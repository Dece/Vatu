#!/usr/bin/env python3
"""Pre-compute pawn progress bitboards for each square."""

TEMPLATE = """\
/// Pre-computed pawn progresses.
pub const PAWN_PROGRESSES: [[Bitboard; 64]; 2] = [
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

def get_progresses():
    both_progresses = []
    for direction in [1, -1]:
        progresses = []
        for f in range(8):
            for r in range(8):
                bitboard = 0
                if 0 < r < 7:
                    prog_r = r + direction
                    bitboard |= bit_pos(f * 8 + prog_r)
                    if direction == 1 and r == 1:
                        bitboard |= bit_pos(f * 8 + prog_r + 1)
                    elif direction == -1 and r == 6:
                        bitboard |= bit_pos(f * 8 + (prog_r - 1))
                progresses.append("        0b{:064b},".format(bitboard))
        both_progresses.append(progresses)
    return both_progresses

PROGRESSES = get_progresses()
print(TEMPLATE.format("\n".join(PROGRESSES[0]), "\n".join(PROGRESSES[1])))

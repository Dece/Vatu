#!/usr/bin/env python3
"""Generate Zobrist keys."""

import random


random.seed("vatu stop playing your queen first you have other pieces")

TEMPLATE = """\
pub const ZOBRIST_PIECES: [[[ZobristHash; 64]; 6]; 2] = [
{}
];

pub const ZOBRIST_BLACK_TURN: ZobristHash = {};

pub const ZOBRIST_CASTLE_WH_K: usize = 0;
pub const ZOBRIST_CASTLE_WH_Q: usize = 1;
pub const ZOBRIST_CASTLE_BL_K: usize = 2;
pub const ZOBRIST_CASTLE_BL_Q: usize = 3;

pub const ZOBRIST_CASTLES: [ZobristHash; 4] = [
{}
];

pub const ZOBRIST_EN_PASSANT: [ZobristHash; 8] = [
{}
];
"""


def gen_hash():
    return random.getrandbits(64)


def gen_pieces_keys():
    pieces_str = ""
    for color in range(2):
        pieces_str += "    [\n"
        for piece in range(6):
            pieces_str += "        [\n"
            for square in range(64):
                k = gen_hash()
                pieces_str += "            {},\n".format(k)
            pieces_str += "        ],\n"
        pieces_str += "    ],\n"
    return pieces_str


print(
    TEMPLATE.format(
        gen_pieces_keys(),
        gen_hash(),
        "\n".join(["    {},".format(gen_hash()) for _ in range(4)]),
        "\n".join(["    {},".format(gen_hash()) for _ in range(8)]),
    )
)

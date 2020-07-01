//! Functions related to Zobrist hashes.

use crate::board::*;

pub type ZobristHash = u64;

pub fn get_new_game_hash() -> ZobristHash {
    return
          get_piece_hash(WHITE, ROOK, A1)
        ^ get_piece_hash(WHITE, KNIGHT, B1)
        ^ get_piece_hash(WHITE, BISHOP, C1)
        ^ get_piece_hash(WHITE, QUEEN, D1)
        ^ get_piece_hash(WHITE, KING, E1)
        ^ get_piece_hash(WHITE, BISHOP, F1)
        ^ get_piece_hash(WHITE, KNIGHT, G1)
        ^ get_piece_hash(WHITE, ROOK, H1)
        ^ get_piece_hash(WHITE, PAWN, A2)
        ^ get_piece_hash(WHITE, PAWN, B2)
        ^ get_piece_hash(WHITE, PAWN, C2)
        ^ get_piece_hash(WHITE, PAWN, D2)
        ^ get_piece_hash(WHITE, PAWN, E2)
        ^ get_piece_hash(WHITE, PAWN, F2)
        ^ get_piece_hash(WHITE, PAWN, G2)
        ^ get_piece_hash(WHITE, PAWN, H2)
        ^ get_piece_hash(BLACK, PAWN, A7)
        ^ get_piece_hash(BLACK, PAWN, B7)
        ^ get_piece_hash(BLACK, PAWN, C7)
        ^ get_piece_hash(BLACK, PAWN, D7)
        ^ get_piece_hash(BLACK, PAWN, E7)
        ^ get_piece_hash(BLACK, PAWN, F7)
        ^ get_piece_hash(BLACK, PAWN, G7)
        ^ get_piece_hash(BLACK, PAWN, H7)
        ^ get_piece_hash(BLACK, ROOK, A8)
        ^ get_piece_hash(BLACK, KNIGHT, B8)
        ^ get_piece_hash(BLACK, BISHOP, C8)
        ^ get_piece_hash(BLACK, QUEEN, D8)
        ^ get_piece_hash(BLACK, KING, E8)
        ^ get_piece_hash(BLACK, BISHOP, F8)
        ^ get_piece_hash(BLACK, KNIGHT, G8)
        ^ get_piece_hash(BLACK, ROOK, H8)
        ^ ZOBRIST_CASTLES[ZOBRIST_CASTLE_WH_K]
        ^ ZOBRIST_CASTLES[ZOBRIST_CASTLE_WH_Q]
        ^ ZOBRIST_CASTLES[ZOBRIST_CASTLE_BL_K]
        ^ ZOBRIST_CASTLES[ZOBRIST_CASTLE_BL_Q]
}

pub fn get_piece_hash(color: Color, piece: Piece, square: Square) -> ZobristHash {
    ZOBRIST_PIECES[color][piece][square as usize]
}

//! Castling flags.

use crate::board::{Bitboard, RANK_1, RANK_8};

pub type Castle = u8;

pub const CASTLE_WH_K: Castle    = 0b00000001;
pub const CASTLE_WH_Q: Castle    = 0b00000010;
pub const CASTLE_WH_MASK: Castle = 0b00000011;
pub const CASTLE_BL_K: Castle    = 0b00000100;
pub const CASTLE_BL_Q: Castle    = 0b00001000;
pub const CASTLE_BL_MASK: Castle = 0b00001100;
pub const CASTLE_K_MASK: Castle  = 0b00000101;
pub const CASTLE_Q_MASK: Castle  = 0b00001010;
pub const CASTLE_MASK: Castle    = 0b00001111;

/// Index castling masks with their color.
pub const CASTLE_MASK_BY_COLOR: [Castle; 2] = [CASTLE_WH_MASK, CASTLE_BL_MASK];

/// Index castling ranks with their color.
pub const CASTLE_RANK_BY_COLOR: [i8; 2] = [RANK_1, RANK_8];

pub const CASTLE_SIDE_K: usize = 0;
pub const CASTLE_SIDE_Q: usize = 1;
pub const NUM_CASTLE_SIDES: usize = 2;

/// Index castling sides using CASTLE_SIDE_K and CASTLE_SIDE_Q.
pub const CASTLE_SIDES: [Castle; 2] = [CASTLE_K_MASK, CASTLE_Q_MASK];

/// Castle paths that must not be under attack, by color and side.
///
/// This includes the original king position, its target square and
/// the square in between.
pub const CASTLE_LEGALITY_PATHS: [[Bitboard; 2]; 2] = [
    [
        0b00000000_00000001_00000001_00000001_00000000_00000000_00000000_00000000,  // White Kside.
        0b00000000_00000000_00000000_00000001_00000001_00000001_00000000_00000000,  // White Qside.
    ], [
        0b00000000_10000000_10000000_10000000_00000000_00000000_00000000_00000000,  // Black Kside.
        0b00000000_00000000_00000000_10000000_10000000_10000000_00000000_00000000,  // Black Qside.
    ]
];

/// Castle paths that must be empty.
pub const CASTLE_MOVE_PATHS: [[Bitboard; 2]; 2] = [
    [
        0b00000000_00000001_00000001_00000000_00000000_00000000_00000000_00000000,  // White Kside.
        0b00000000_00000000_00000000_00000000_00000001_00000001_00000001_00000000,  // White Qside.
    ], [
        0b00000000_10000000_10000000_00000000_00000000_00000000_00000000_00000000,  // Black Kside.
        0b00000000_00000000_00000000_00000000_10000000_10000000_10000000_00000000,  // Black Qside.
    ]
];

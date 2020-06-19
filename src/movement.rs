//! Move functions along with some castling helpers.

use crate::board::*;
use crate::castling::*;
use crate::rules;

const START_WH_K_POS: Square = E1;
const START_BL_K_POS: Square = E8;

/// A movement, with before/after positions and optional promotion.
pub type Move = (Square, Square, Option<Piece>);

/// Apply a move `m` to copies to `board` and `game_state`.
///
/// Can be used for conveniance but it's better to write in existing
/// instances as often as possible using `apply_move_to`.
pub fn apply_move(
    board: &Board,
    game_state: &rules::GameState,
    m: &Move
) -> (Board, rules::GameState) {
    let mut new_board = board.clone();
    let mut new_state = game_state.clone();
    apply_move_to(&mut new_board, &mut new_state, m);
    (new_board, new_state)
}

/// Update `board` and `game_state` to reflect the move `m`.
///
/// The board is updated with correct piece placement.
///
/// The game state is updated with the new player turn and the new
/// castling options.
pub fn apply_move_to(
    board: &mut Board,
    game_state: &mut rules::GameState,
    m: &Move
) {
    let (source, dest) = (m.0, m.1);

    // If a rook is taken, remove its castling option. Needs to be checked before we update board.
    // Note that we only check for a piece going to rook's initial position: it means the rook
    // either moved previously, or it has been taken.
    match source {
        A1 => { game_state.castling &= !CASTLING_WH_Q; }
        H1 => { game_state.castling &= !CASTLING_WH_K; }
        A8 => { game_state.castling &= !CASTLING_BL_Q; }
        H8 => { game_state.castling &= !CASTLING_BL_K; }
    }

    // Update board and game state.
    apply_move_to_board(board, m);
    game_state.color = opposite(game_state.color);

    // If the move is a castle, remove it from castling options.
    if let Some(castle) = get_castle(m) {
        match castle {
            CASTLING_WH_K | CASTLING_WH_Q => game_state.castling &= !CASTLING_WH_MASK,
            CASTLING_BL_K | CASTLING_BL_Q => game_state.castling &= !CASTLING_BL_MASK,
            _ => {}
        };
    }
    // Else, check if the king or a rook moved to update castling options.
    else {
        let color = board.get_color(dest);
        if color == WHITE && game_state.castling & CASTLING_WH_MASK != 0 {
            match board.get_piece(dest) {
                KING => {
                    if source == E1 {
                        game_state.castling &= !CASTLING_WH_MASK;
                    }
                }
                ROOK => {
                    if source == A1 {
                        game_state.castling &= !CASTLING_WH_Q;
                    } else if source == H1 {
                        game_state.castling &= !CASTLING_WH_K;
                    }
                }
                _ => {}
            }
        } else if color == BLACK && game_state.castling & CASTLING_BL_MASK != 0 {
            match board.get_piece(dest) {
                KING => {
                    if source == E8 {
                        game_state.castling &= !CASTLING_BL_MASK;
                    }
                }
                ROOK => {
                    if source == A8 {
                        game_state.castling &= !CASTLING_BL_Q;
                    } else if source == H8 {
                        game_state.castling &= !CASTLING_BL_K;
                    }
                }
                _ => {}
            }
        }
    }
}

/// Apply a move `m` into `board`.
pub fn apply_move_to_board(board: &mut Board, m: &Move) {
    if let Some(castle) = get_castle(m) {
        match castle {
            CASTLING_WH_K => {
                board.move_square(START_WH_K_POS, G1);
                board.move_square(H1, F1);
            }
            CASTLING_WH_Q => {
                board.move_square(START_WH_K_POS, C1);
                board.move_square(A1, D1);
            }
            CASTLING_BL_K => {
                board.move_square(START_BL_K_POS, G8);
                board.move_square(H8, F8);
            }
            CASTLING_BL_Q => {
                board.move_square(START_BL_K_POS, C8);
                board.move_square(A8, D8);
            }
            _ => {}
        }
    } else {
        board.move_square(m.0, m.1);
        if let Some(prom_type) = m.2 {
            let color = board.get_color(m.1);
            board.set_square(m.1, color, prom_type);
        }
    }
}

/// Get the corresponding castling flag for this move.
pub fn get_castle(m: &Move) -> Option<u8> {
    let (source, dest) = (m.0, m.1);
    if source == E1 {
        if dest == C1 {
            Some(CASTLING_WH_Q)
        } else if dest == G1 {
            Some(CASTLING_WH_K)
        } else {
            None
        }
    } else if source == E8 {
        if dest == C8 {
            Some(CASTLING_BL_Q)
        } else if dest == G8 {
            Some(CASTLING_BL_K)
        } else {
            None
        }
    } else {
        None
    }
}

/// Get the move for this castle.
pub fn get_castle_move(castle: u8) -> Move {
    match castle {
        CASTLING_WH_Q => (E1, C1, None),
        CASTLING_WH_K => (E1, G1, None),
        CASTLING_BL_Q => (E8, C8, None),
        CASTLING_BL_K => (E8, G8, None),
        _ => panic!("Illegal castling requested: {:08b}", castle),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::notation::parse_move;

    #[test]
    fn test_apply_move_to_board() {
        let mut b = Board::new_empty();

        // Put 2 enemy knights on board.
        b.set_square(D4, WHITE, KNIGHT);
        b.set_square(F4, BLACK, KNIGHT);
        // Move white knight in a position attacked by black knight.
        apply_move_to_board(&mut b, &(D4, E6, None));
        assert!(b.is_empty(D4));
        assert_eq!(b.get_color(E6), WHITE);
        assert_eq!(b.get_piece(E6), KNIGHT);
        assert_eq!(b.num_pieces(), 2);
        // Sack it with black knight
        apply_move_to_board(&mut b, &(F4, E6, None));
        assert_eq!(b.get_color(E6), BLACK);
        assert_eq!(b.get_piece(E6), KNIGHT);
        assert_eq!(b.num_pieces(), 1);
    }

    #[test]
    fn test_apply_move_to_castling() {
        let mut b = Board::new();
        let mut gs = rules::GameState::new();
        assert_eq!(gs.castling, CASTLING_MASK);

        // On a starting board, start by making place for all castles.
        b.clear_square(B1);
        b.clear_square(C1);
        b.clear_square(D1);
        b.clear_square(F1);
        b.clear_square(G1);
        b.clear_square(B8);
        b.clear_square(C8);
        b.clear_square(D8);
        b.clear_square(F8);
        b.clear_square(G8);
        // White queen-side castling.
        apply_move_to(&mut b, &mut gs, &parse_move("e1c1"));
        assert_eq!(b.get_color(C1), WHITE);
        assert_eq!(b.get_piece(C1), KING);
        assert_eq!(b.get_color(D1), WHITE);
        assert_eq!(b.get_piece(D1), ROOK);
        assert!(b.is_empty(A1));
        assert!(b.is_empty(E1));
        assert_eq!(gs.castling, CASTLING_BL_MASK);
        // Black king-side castling.
        apply_move_to(&mut b, &mut gs, &parse_move("e8g8"));
        assert_eq!(b.get_color(G1), BLACK);
        assert_eq!(b.get_piece(G1), KING);
        assert_eq!(b.get_color(F1), BLACK);
        assert_eq!(b.get_piece(F1), ROOK);
        assert!(b.is_empty(H8));
        assert!(b.is_empty(E8));
        // At the end, no more castling options for both sides.
        assert_eq!(gs.castling, 0);
    }

    #[test]
    fn test_get_castle() {
        assert_eq!(get_castle(&parse_move("e1c1")), Some(CASTLING_WH_Q));
        assert_eq!(get_castle(&parse_move("e1g1")), Some(CASTLING_WH_K));
        assert_eq!(get_castle(&parse_move("e8c8")), Some(CASTLING_BL_Q));
        assert_eq!(get_castle(&parse_move("e8g8")), Some(CASTLING_BL_K));
        assert_eq!(get_castle(&parse_move("d2d4")), None);
    }
}

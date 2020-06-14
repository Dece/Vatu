//! Move functions along with some castling helpers.

use crate::board::*;
use crate::castling::*;
use crate::rules;

const START_WH_K_POS: Pos = pos("e1");
const START_BL_K_POS: Pos = pos("e8");

/// A movement, with before/after positions and optional promotion.
pub type Move = (Pos, Pos, Option<u8>);

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
    apply_move_to_board(board, m);
    apply_move_to_state(game_state, m);
    // If the move is a castle, remove it from castling options.
    if let Some(castle) = get_castle(m) {
        match castle {
            CASTLING_WH_K | CASTLING_WH_Q => game_state.castling &= !CASTLING_WH_MASK,
            CASTLING_BL_K | CASTLING_BL_Q => game_state.castling &= !CASTLING_BL_MASK,
            _ => {}
        };
    }
    // Else, check if it's either a rook or the king that moved.
    else {
        let piece = get_square(board, &m.1);
        if is_white(piece) && game_state.castling & CASTLING_WH_MASK != 0 {
            match get_type(piece) {
                SQ_K => {
                    if m.0 == pos("e1") {
                        game_state.castling &= !CASTLING_WH_MASK;
                    }
                }
                SQ_R => {
                    if m.0 == pos("a1") {
                        game_state.castling &= !CASTLING_WH_Q;
                    } else if m.0 == pos("h1") {
                        game_state.castling &= !CASTLING_WH_K;
                    }
                }
                _ => {}
            }
        } else if is_black(piece) && game_state.castling & CASTLING_BL_MASK != 0 {
            match get_type(piece) {
                SQ_K => {
                    if m.0 == pos("e8") {
                        game_state.castling &= !CASTLING_BL_MASK;
                    }
                }
                SQ_R => {
                    if m.0 == pos("a8") {
                        game_state.castling &= !CASTLING_BL_Q;
                    } else if m.0 == pos("h8") {
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
                move_piece(board, &START_WH_K_POS, &pos("g1"));
                move_piece(board, &pos("h1"), &pos("f1"));
            }
            CASTLING_WH_Q => {
                move_piece(board, &START_WH_K_POS, &pos("c1"));
                move_piece(board, &pos("a1"), &pos("d1"));
            }
            CASTLING_BL_K => {
                move_piece(board, &START_BL_K_POS, &pos("g8"));
                move_piece(board, &pos("h8"), &pos("f8"));
            }
            CASTLING_BL_Q => {
                move_piece(board, &START_BL_K_POS, &pos("c8"));
                move_piece(board, &pos("a8"), &pos("d8"));
            }
            _ => {}
        }
    } else {
        move_piece(board, &m.0, &m.1);
        if let Some(prom_type) = m.2 {
            let color = get_color(get_square(board, &m.1));
            set_square(board, &m.1, color|prom_type);
        }
    }
}

/// Update `game_state` with the move `m`.
///
/// This only updates the player turn. Castling should be updated in a
/// context where the corresponding board is available.
pub fn apply_move_to_state(game_state: &mut rules::GameState, _m: &Move) {
    game_state.color = opposite(game_state.color);
}

/// Get the corresponding castling flag for this move.
pub fn get_castle(m: &Move) -> Option<u8> {
    if m.0 == pos("e1") {
        if m.1 == pos("c1") {
            Some(CASTLING_WH_Q)
        } else if m.1 == pos("g1") {
            Some(CASTLING_WH_K)
        } else {
            None
        }
    } else if m.0 == pos("e8") {
        if m.1 == pos("c8") {
            Some(CASTLING_BL_Q)
        } else if m.1 == pos("g8") {
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
        CASTLING_WH_Q => (pos("e1"), pos("c1"), None),
        CASTLING_WH_K => (pos("e1"), pos("g1"), None),
        CASTLING_BL_Q => (pos("e8"), pos("c8"), None),
        CASTLING_BL_K => (pos("e8"), pos("g8"), None),
        _ => panic!("Illegal castling requested: {:08b}", castle),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::notation::parse_move;

    #[test]
    fn test_apply_move_to_board() {
        let mut b = new_empty();

        // Put 2 enemy knights on board.
        set_square(&mut b, &pos("d4"), SQ_WH_N);
        set_square(&mut b, &pos("f4"), SQ_BL_N);
        // Move white knight in a position attacked by black knight.
        apply_move_to_board(&mut b, &(pos("d4"), pos("e6"), None));
        assert_eq!(get_square(&b, &pos("d4")), SQ_E);
        assert_eq!(get_square(&b, &pos("e6")), SQ_WH_N);
        assert_eq!(num_pieces(&b), 2);
        // Sack it with black knight
        apply_move_to_board(&mut b, &(pos("f4"), pos("e6"), None));
        assert_eq!(get_square(&b, &pos("e6")), SQ_BL_N);
        assert_eq!(num_pieces(&b), 1);
    }

    #[test]
    fn test_apply_move_to_castling() {
        let mut b = new();
        let mut gs = rules::GameState::new();
        assert_eq!(gs.castling, CASTLING_MASK);

        // On a starting board, start by making place for all castles.
        clear_square(&mut b, &pos("b1"));
        clear_square(&mut b, &pos("c1"));
        clear_square(&mut b, &pos("d1"));
        clear_square(&mut b, &pos("f1"));
        clear_square(&mut b, &pos("g1"));
        clear_square(&mut b, &pos("b8"));
        clear_square(&mut b, &pos("c8"));
        clear_square(&mut b, &pos("d8"));
        clear_square(&mut b, &pos("f8"));
        clear_square(&mut b, &pos("g8"));
        // White queen-side castling.
        apply_move_to(&mut b, &mut gs, &parse_move("e1c1"));
        assert!(is_piece(get_square(&b, &pos("c1")), SQ_WH_K));
        assert!(is_piece(get_square(&b, &pos("d1")), SQ_WH_R));
        assert!(is_empty(&b, &pos("a1")));
        assert!(is_empty(&b, &pos("e1")));
        assert_eq!(gs.castling, CASTLING_BL_MASK);
        // Black king-side castling.
        apply_move_to(&mut b, &mut gs, &parse_move("e8g8"));
        assert!(is_piece(get_square(&b, &pos("g8")), SQ_BL_K));
        assert!(is_piece(get_square(&b, &pos("f8")), SQ_BL_R));
        assert!(is_empty(&b, &pos("h8")));
        assert!(is_empty(&b, &pos("e8")));
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

//! Move functions along with some castling helpers.

use std::fmt;

use crate::board::*;
use crate::castling::*;
use crate::rules::GameState;

const START_WH_K_POS: Square = E1;
const START_BL_K_POS: Square = E8;

/// A movement, with before/after positions and optional promotion.
#[derive(PartialEq)]
pub struct Move {
    pub source: Square,
    pub dest: Square,
    pub promotion: Option<Piece>,
}

impl fmt::Debug for Move {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.to_uci_string())
    }
}

pub const SAN_NULL_MOVE: &str = "0000";

impl Move {
    /// Build a move from `source` to `dest`, no promotion.
    pub const fn new(source: Square, dest: Square) -> Move {
        Move { source, dest, promotion: None }
    }

    /// Build a move from `source` to `dest`, with a promotion.
    pub const fn new_promotion(source: Square, dest: Square, promotion: Piece) -> Move {
        Move { source, dest, promotion: Some(promotion) }
    }

    /// Apply this move to `board` and `game_state`.
    pub fn apply_to(&self, board: &mut Board, game_state: &mut GameState) {
        // If a rook is taken, remove its castling option. Needs to be checked before we update
        // board. Note that we only check for a piece going to rook's initial position: it means
        // the rook either moved previously, or it has been taken.
        match self.source {
            A1 => { game_state.castling &= !CASTLING_WH_Q; }
            H1 => { game_state.castling &= !CASTLING_WH_K; }
            A8 => { game_state.castling &= !CASTLING_BL_Q; }
            H8 => { game_state.castling &= !CASTLING_BL_K; }
        }

        // Update board and game state.
        self.apply_to_board(board);
        game_state.color = opposite(game_state.color);

        // If the move is a castle, remove it from castling options.
        if let Some(castle) = self.get_castle() {
            match castle {
                CASTLING_WH_K | CASTLING_WH_Q => game_state.castling &= !CASTLING_WH_MASK,
                CASTLING_BL_K | CASTLING_BL_Q => game_state.castling &= !CASTLING_BL_MASK,
                _ => {}
            };
        }
        // Else, check if the king or a rook moved to update castling options.
        else {
            let color = board.get_color(self.dest);
            if color == WHITE && game_state.castling & CASTLING_WH_MASK != 0 {
                match board.get_piece(self.dest) {
                    KING => {
                        if self.source == E1 {
                            game_state.castling &= !CASTLING_WH_MASK;
                        }
                    }
                    ROOK => {
                        if self.source == A1 {
                            game_state.castling &= !CASTLING_WH_Q;
                        } else if self.source == H1 {
                            game_state.castling &= !CASTLING_WH_K;
                        }
                    }
                    _ => {}
                }
            } else if color == BLACK && game_state.castling & CASTLING_BL_MASK != 0 {
                match board.get_piece(self.dest) {
                    KING => {
                        if self.source == E8 {
                            game_state.castling &= !CASTLING_BL_MASK;
                        }
                    }
                    ROOK => {
                        if self.source == A8 {
                            game_state.castling &= !CASTLING_BL_Q;
                        } else if self.source == H8 {
                            game_state.castling &= !CASTLING_BL_K;
                        }
                    }
                    _ => {}
                }
            }
        }
    }

    /// Apply the move into `board`.
    pub fn apply_to_board(&self, board: &mut Board) {
        if let Some(castle) = self.get_castle() {
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
            board.move_square(self.source, self.dest);
            if let Some(piece) = self.promotion {
                let color = board.get_color(self.dest);
                board.set_square(self.dest, color, piece);
            }
        }
    }

    /// Get the corresponding castling flag for this move.
    pub fn get_castle(&self) -> Option<u8> {
        if self.source == E1 {
            if self.dest == C1 {
                Some(CASTLING_WH_Q)
            } else if self.dest == G1 {
                Some(CASTLING_WH_K)
            } else {
                None
            }
        } else if self.source == E8 {
            if self.dest == C8 {
                Some(CASTLING_BL_Q)
            } else if self.dest == G8 {
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
            CASTLING_WH_Q => Move::new(E1, C1),
            CASTLING_WH_K => Move::new(E1, G1),
            CASTLING_BL_Q => Move::new(E8, C8),
            CASTLING_BL_K => Move::new(E8, G8),
            _ => panic!("Illegal castling requested: {:08b}", castle),
        }
    }

    /// Parse an UCI move algebraic notation string to a Move.
    pub fn from_uci_string(m_str: &str) -> Move {
        Move {
            source: sq_from_string(&m_str[0..2]),
            dest: sq_from_string(&m_str[2..4]),
            promotion: if m_str.len() == 5 {
                Some(match m_str.as_bytes()[4] {
                    b'b' => BISHOP,
                    b'n' => KNIGHT,
                    b'r' => ROOK,
                    b'q' => QUEEN,
                    _ => panic!("What is the opponent doing? This is illegal, I'm out."),
                })
            } else {
                None
            }
        }
    }

    /// Create a string containing the UCI algebraic notation of this move.
    pub fn to_uci_string(&self) -> String {
        let mut move_string = String::new();
        move_string.push_str(&sq_to_string(self.source));
        move_string.push_str(&sq_to_string(self.dest));
        if let Some(piece) = self.promotion {
            move_string.push(match piece {
                QUEEN => 'q',
                BISHOP => 'b',
                KNIGHT => 'n',
                ROOK => 'r',
                _ => panic!("What are you doing? Promote to a legal piece.")
            });
        }
        move_string
    }

    /// Debug only: create a space-separated string of moves.
    pub(crate) fn list_to_uci_string(moves: &Vec<Move>) -> String {
        moves.iter().map(|m| m.to_uci_string()).collect::<Vec<_>>().join(" ")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_apply_move_to_board() {
        let mut b = Board::new_empty();

        // Put 2 enemy knights on board.
        b.set_square(D4, WHITE, KNIGHT);
        b.set_square(F4, BLACK, KNIGHT);
        // Move white knight in a position attacked by black knight.
        Move::new(D4, E6).apply_to_board(&mut b);
        assert!(b.is_empty(D4));
        assert_eq!(b.get_color(E6), WHITE);
        assert_eq!(b.get_piece(E6), KNIGHT);
        assert_eq!(b.num_pieces(), 2);
        // Sack it with black knight
        Move::new(F4, E6).apply_to_board(&mut b);
        assert_eq!(b.get_color(E6), BLACK);
        assert_eq!(b.get_piece(E6), KNIGHT);
        assert_eq!(b.num_pieces(), 1);
    }

    #[test]
    fn test_apply_move_to_castling() {
        let mut b = Board::new();
        let mut gs = GameState::new();
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
        Move::new(E1, C1).apply_to(&mut b, &mut gs);
        assert_eq!(b.get_color(C1), WHITE);
        assert_eq!(b.get_piece(C1), KING);
        assert_eq!(b.get_color(D1), WHITE);
        assert_eq!(b.get_piece(D1), ROOK);
        assert!(b.is_empty(A1));
        assert!(b.is_empty(E1));
        assert_eq!(gs.castling, CASTLING_BL_MASK);
        // Black king-side castling.
        Move::new(E8, G8).apply_to(&mut b, &mut gs);
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
        assert_eq!(Move::new(E1, C1).get_castle(), Some(CASTLING_WH_Q));
        assert_eq!(Move::new(E1, G1).get_castle(), Some(CASTLING_WH_K));
        assert_eq!(Move::new(E8, C8).get_castle(), Some(CASTLING_BL_Q));
        assert_eq!(Move::new(E8, G8).get_castle(), Some(CASTLING_BL_K));
        assert_eq!(Move::new(D2, D4).get_castle(), None);
    }

    #[test]
    fn test_to_uci_string() {
        assert_eq!(Move::new(A1, D4).to_uci_string(), "a1d4");
        assert_eq!(Move::new(H8, A8).to_uci_string(), "h8a8");
        assert_eq!(Move::new_promotion(H7, H8, QUEEN).to_uci_string(), "h7h8q");
        assert_eq!(Move::new_promotion(H7, H8, KNIGHT).to_uci_string(), "h7h8n");
    }

    #[test]
    fn test_from_uci_string() {
        assert_eq!(Move::from_uci_string("a1d4"), Move::new(A1, D4));
        assert_eq!(Move::from_uci_string("a7a8q"), Move::new_promotion(A7, A8, QUEEN));
        assert_eq!(Move::from_uci_string("a7a8r"), Move::new_promotion(A7, A8, ROOK));
    }
}

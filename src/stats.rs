//! Board statistics used for heuristics.

use crate::board::*;
use crate::rules::{GameState, get_player_moves};

/// Storage for board pieces stats.
#[derive(Debug, Clone, PartialEq)]
pub struct BoardStats {
    pub num_pawns: i8,
    pub num_bishops: i8,
    pub num_knights: i8,
    pub num_rooks: i8,
    pub num_queens: i8,
    pub num_kings: i8,
    pub num_doubled_pawns: i8,   // Pawns that are on the same file as a friend.
    pub num_backward_pawns: i8,  // Pawns behind all other pawns on adjacent files.
    pub num_isolated_pawns: i8,  // Pawns that have no friend pawns on adjacent files.
    pub mobility: i32,
}

impl BoardStats {
    pub const fn new() -> BoardStats {
        BoardStats {
            num_pawns: 0, num_bishops: 0, num_knights: 0, num_rooks: 0, num_queens: 0,
            num_kings: 0, num_doubled_pawns: 0, num_backward_pawns: 0, num_isolated_pawns: 0,
            mobility: 0,
        }
    }

    /// Create two new BoardStats objects from the board, for both sides.
    ///
    /// The playing color will have its stats filled in the first
    /// BoardStats object, its opponent in the second.
    pub fn new_from(board: &Board, game_state: &GameState) -> (BoardStats, BoardStats) {
        let mut stats = (BoardStats::new(), BoardStats::new());
        let mut gs = game_state.clone();
        stats.0.compute(board, &gs);
        gs.color = opposite(gs.color);
        stats.1.compute(board, &gs);
        stats
    }

    /// Reset all stats to 0.
    pub fn reset(&mut self) {
        self.num_pawns = 0;
        self.num_bishops = 0;
        self.num_knights = 0;
        self.num_rooks = 0;
        self.num_queens = 0;
        self.num_kings = 0;
        self.num_doubled_pawns = 0;
        self.num_backward_pawns = 0;
        self.num_isolated_pawns = 0;
        self.mobility = 0;
    }

    /// Fill `stats` from given `board` and `game_state`.
    ///
    /// Only the current playing side stats are created,
    /// prepare the game_state accordingly.
    pub fn compute(&mut self, board: &Board, game_state: &GameState) {
        self.reset();
        let color = game_state.color;
        // Compute mobility for all pieces.
        self.mobility = get_player_moves(board, game_state).len() as i32;
        // Compute amount of each piece.
        for file in 0..8 {
            for rank in 0..8 {
                let square = sq(file, rank);
                if board.is_empty(square) || board.get_color_on(square) != color {
                    continue
                }
                match board.get_piece_on(square) {
                    ROOK => self.num_rooks += 1,
                    KNIGHT => self.num_knights += 1,
                    BISHOP => self.num_bishops += 1,
                    QUEEN => self.num_queens += 1,
                    KING => self.num_kings += 1,
                    PAWN => {
                        self.num_pawns += 1;
                        let pawn_bb = board.by_color_and_piece(color, PAWN);

                        // Check for doubled pawns.
                        let file_bb = FILES[file as usize];
                        if (pawn_bb ^ bit_pos(square)) & file_bb != 0 {
                            self.num_doubled_pawns += 1;
                        }

                        // Check for isolated and backward pawns.
                        let (iso_on_prev_file, bw_on_prev_file) = if file > FILE_A {
                            self.find_isolated_and_backward(pawn_bb, square, color, file - 1)
                        } else {
                            (true, true)
                        };
                        let (iso_on_next_file, bw_on_next_file) = if file < FILE_H {
                            self.find_isolated_and_backward(pawn_bb, square, color, file + 1)
                        } else {
                            (true, true)
                        };
                        if iso_on_prev_file && iso_on_next_file {
                            self.num_isolated_pawns += 1;
                        }
                        if bw_on_prev_file && bw_on_next_file {
                            self.num_backward_pawns += 1;
                        }
                    },
                    _ => {}
                }
            }
        }
    }

    /// Find isolated and backward pawns from `square` perspective.
    ///
    /// `bb` is the bitboard of `color`. `square` is only used to have
    /// the reference rank. `file` is the file to inspect. To detect
    /// isolated and backward pawns, `bb` should be the bitboard of
    /// pawns of `color`.
    fn find_isolated_and_backward(
        &mut self,
        bb: Bitboard,
        square: Square,
        color: Color,
        file: i8
    ) -> (bool, bool) {
        if bb & FILES[file as usize] == 0 {
            // If the piece is isolated for this file, it's backward as well.
            (true, true)
        } else {
            let backward_file_bb = if color == WHITE {
                before_on_file(file, sq_rank(square)) | bit_pos(sq(file, sq_rank(square)))
            } else {
                after_on_file(file, sq_rank(square)) | bit_pos(sq(file, sq_rank(square)))
            };
            (false, bb & backward_file_bb == 0)
        }
    }
}

impl std::fmt::Display for BoardStats {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "{}P {}B {}N {}R {}Q {}K {}dp {}bp {}ip {}m",
            self.num_pawns, self.num_bishops, self.num_knights, self.num_rooks,
            self.num_queens, self.num_kings,
            self.num_doubled_pawns, self.num_backward_pawns, self.num_isolated_pawns,
            self.mobility
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compute_stats() {
        // Check that initial stats are correct.
        let b = Board::new();
        let gs = GameState::new();
        let initial_stats = BoardStats {
            num_pawns: 8,
            num_bishops: 2,
            num_knights: 2,
            num_rooks: 2,
            num_queens: 1,
            num_kings: 1,
            num_doubled_pawns: 0,
            num_backward_pawns: 0,
            num_isolated_pawns: 0,
            mobility: 20,
        };
        let mut stats = BoardStats::new_from(&b, &gs);
        assert!(stats.0 == stats.1);
        assert!(stats.0 == initial_stats);

        // Check that doubled pawns are correctly counted.
        let mut b = Board::new_empty();
        b.set_square(D4, WHITE, PAWN);
        b.set_square(D6, WHITE, PAWN);
        stats.0.compute(&b, &gs);
        assert_eq!(stats.0.num_doubled_pawns, 2);
        // Add a pawn on another file, no changes expected.
        b.set_square(E6, WHITE, PAWN);
        stats.0.compute(&b, &gs);
        assert_eq!(stats.0.num_doubled_pawns, 2);
        // Add a pawn backward in the d-file: there are now 3 doubled pawns.
        b.set_square(D2, WHITE, PAWN);
        stats.0.compute(&b, &gs);
        assert_eq!(stats.0.num_doubled_pawns, 3);

        // Check that isolated and backward pawns are correctly counted.
        assert_eq!(stats.0.num_isolated_pawns, 0);
        assert_eq!(stats.0.num_backward_pawns, 2);  // A bit weird?
        // Protect d4 pawn with a friend in e3: it is not isolated nor backward anymore.
        b.set_square(E3, WHITE, PAWN);
        stats.0.compute(&b, &gs);
        assert_eq!(stats.0.num_doubled_pawns, 5);
        assert_eq!(stats.0.num_isolated_pawns, 0);
        assert_eq!(stats.0.num_backward_pawns, 1);
        // Add an adjacent friend to d2 pawn: no pawns are left isolated or backward.
        b.set_square(C2, WHITE, PAWN);
        stats.0.compute(&b, &gs);
        assert_eq!(stats.0.num_doubled_pawns, 5);
        assert_eq!(stats.0.num_isolated_pawns, 0);
        assert_eq!(stats.0.num_backward_pawns, 0);
        // Add an isolated/backward white pawn in a far file.
        b.set_square(A2, WHITE, PAWN);
        stats.0.compute(&b, &gs);
        assert_eq!(stats.0.num_doubled_pawns, 5);
        assert_eq!(stats.0.num_isolated_pawns, 1);
        assert_eq!(stats.0.num_backward_pawns, 1);

        // Check for pawns that are backward but not isolated.
        let mut b = Board::new_empty();
        // Here, d4 pawn protects both e5 and e3, but it is backward.
        b.set_square(D4, WHITE, PAWN);
        b.set_square(E5, WHITE, PAWN);
        b.set_square(E3, WHITE, PAWN);
        stats.0.compute(&b, &gs);
        assert_eq!(stats.0.num_doubled_pawns, 2);
        assert_eq!(stats.0.num_isolated_pawns, 0);
        assert_eq!(stats.0.num_backward_pawns, 1);
    }
}

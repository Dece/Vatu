//! Board statistics used for heuristics.

use crate::board::*;
use crate::rules;

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

/// Create two new BoardStats objects from the board, for white and black.
///
/// See `compute_stats_into` for details.
pub fn compute_stats(board: &Board, game_state: &rules::GameState) -> (BoardStats, BoardStats) {
    let mut stats = (BoardStats::new(), BoardStats::new());
    compute_stats_into(board, game_state, &mut stats);
    stats
}

pub fn compute_stats_into(
    board: &Board,
    game_state: &rules::GameState,
    stats: &mut (BoardStats, BoardStats)
) {
    compute_color_stats_into(board, game_state, &mut stats.0, SQ_WH);
    compute_color_stats_into(board, game_state, &mut stats.1, SQ_BL);
}

/// Update `stats` for `color` from given `board`
///
/// Refresh all stats *except* `mobility`.
pub fn compute_color_stats_into(
    board: &Board,
    game_state: &rules::GameState,
    stats: &mut BoardStats,
    color: u8
) {
    stats.reset();
    // Compute mobility for all pieces.
    stats.mobility = rules::get_player_moves(board, game_state, true).len() as i32;
    // Compute amount of each piece.
    for (piece, p) in get_piece_iterator(board) {
        let (pos_f, pos_r) = p;
        if piece == SQ_E || !is_color(piece, color) {
            continue
        }
        match get_type(piece) {
            SQ_R => stats.num_rooks += 1,
            SQ_N => stats.num_knights += 1,
            SQ_B => stats.num_bishops += 1,
            SQ_Q => stats.num_queens += 1,
            SQ_K => stats.num_kings += 1,
            SQ_P => {
                stats.num_pawns += 1;
                let mut doubled = false;
                let mut isolated = true;
                let mut backward = true;
                for r in 0..8 {
                    // Check for doubled pawns.
                    if
                        !doubled &&
                        is_piece(get_square(board, &(pos_f, r)), color|SQ_P) && r != pos_r
                    {
                        doubled = true;
                    }
                    // Check for isolated pawns.
                    if
                        isolated &&
                        (
                            // Check on the left file if not on a-file...
                            (
                                pos_f > POS_MIN &&
                                is_piece(get_square(board, &(pos_f - 1, r)), color|SQ_P)
                            ) ||
                            // Check on the right file if not on h-file...
                            (
                                pos_f < POS_MAX &&
                                is_piece(get_square(board, &(pos_f + 1, r)), color|SQ_P)
                            )
                        )
                    {
                        isolated = false;
                    }
                    // Check for backward pawns.
                    if backward {
                        if color == SQ_WH && r <= pos_r {
                            if (
                                pos_f > POS_MIN &&
                                is_type(get_square(board, &(pos_f - 1, r)), SQ_P)
                            ) || (
                                pos_f < POS_MAX &&
                                is_type(get_square(board, &(pos_f + 1, r)), SQ_P)
                            ) {
                                backward = false;
                            }
                        } else if color == SQ_BL && r >= pos_r {
                            if (
                                pos_f > POS_MIN &&
                                is_type(get_square(board, &(pos_f - 1, r)), SQ_P)
                            ) || (
                                pos_f < POS_MAX &&
                                is_type(get_square(board, &(pos_f + 1, r)), SQ_P)
                            ) {
                                backward = false;
                            }
                        }
                    }
                }
                if doubled {
                    stats.num_doubled_pawns += 1;
                }
                if isolated {
                    stats.num_isolated_pawns += 1;
                }
                if backward {
                    stats.num_backward_pawns += 1;
                }
            },
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compute_stats() {
        // Check that initial stats are correct.
        let b = new();
        let gs = rules::GameState::new();
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
        let mut stats = compute_stats(&b, &gs);
        eprintln!("{}", stats.0);
        eprintln!("{}", stats.1);
        assert!(stats.0 == stats.1);
        assert!(stats.0 == initial_stats);

        // Check that doubled pawns are correctly counted.
        let mut b = new_empty();
        set_square(&mut b, &pos("d4"), SQ_WH_P);
        set_square(&mut b, &pos("d6"), SQ_WH_P);
        compute_color_stats_into(&b, &gs, &mut stats.0, SQ_WH);
        assert_eq!(stats.0.num_doubled_pawns, 2);
        // Add a pawn on another file, no changes expected.
        set_square(&mut b, &pos("e6"), SQ_WH_P);
        compute_color_stats_into(&b, &gs, &mut stats.0, SQ_WH);
        assert_eq!(stats.0.num_doubled_pawns, 2);
        // Add a pawn backward in the d-file: there are now 3 doubled pawns.
        set_square(&mut b, &pos("d2"), SQ_WH_P);
        compute_color_stats_into(&b, &gs, &mut stats.0, SQ_WH);
        assert_eq!(stats.0.num_doubled_pawns, 3);

        // Check that isolated and backward pawns are correctly counted.
        assert_eq!(stats.0.num_isolated_pawns, 0);
        assert_eq!(stats.0.num_backward_pawns, 2);  // A bit weird?
        // Protect d4 pawn with a friend in e3: it is not isolated nor backward anymore.
        set_square(&mut b, &pos("e3"), SQ_WH_P);
        compute_color_stats_into(&b, &gs, &mut stats.0, SQ_WH);
        assert_eq!(stats.0.num_doubled_pawns, 5);
        assert_eq!(stats.0.num_isolated_pawns, 0);
        assert_eq!(stats.0.num_backward_pawns, 1);
        // Add an adjacent friend to d2 pawn: no pawns are left isolated or backward.
        set_square(&mut b, &pos("c2"), SQ_WH_P);
        compute_color_stats_into(&b, &gs, &mut stats.0, SQ_WH);
        assert_eq!(stats.0.num_doubled_pawns, 5);
        assert_eq!(stats.0.num_isolated_pawns, 0);
        assert_eq!(stats.0.num_backward_pawns, 0);
        // Add an isolated/backward white pawn in a far file.
        set_square(&mut b, &pos("a2"), SQ_WH_P);
        compute_color_stats_into(&b, &gs, &mut stats.0, SQ_WH);
        assert_eq!(stats.0.num_doubled_pawns, 5);
        assert_eq!(stats.0.num_isolated_pawns, 1);
        assert_eq!(stats.0.num_backward_pawns, 1);

        // Check for pawns that are backward but not isolated.
        let mut b = new_empty();
        // Here, d4 pawn protects both e5 and e3, but it is backward.
        set_square(&mut b, &pos("d4"), SQ_WH_P);
        set_square(&mut b, &pos("e5"), SQ_WH_P);
        set_square(&mut b, &pos("e3"), SQ_WH_P);
        compute_color_stats_into(&b, &gs, &mut stats.0, SQ_WH);
        assert_eq!(stats.0.num_doubled_pawns, 2);
        assert_eq!(stats.0.num_isolated_pawns, 0);
        assert_eq!(stats.0.num_backward_pawns, 1);
    }
}

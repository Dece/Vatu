use std::fmt;

use crate::board;
use crate::movement::Move;
use crate::rules;
use crate::stats;

/// Analysis node: a board along with the game state.
#[derive(Clone, PartialEq)]
pub struct Node {
    /// Board for this node.
    pub board: board::Board,
    /// Game state.
    pub game_state: rules::GameState,
}

impl Node {
    /// Create a new node for an empty board and a new game state.
    pub fn new() -> Node {
        Node {
            board: board::Board::new_empty(),
            game_state: rules::GameState::new(),
        }
    }

    /// Apply a move to this node.
    pub fn apply_move(&mut self, m: &mut Move) {
        m.apply_to(&mut self.board, &mut self.game_state);
    }

    pub fn unmake_move(&mut self, m: &Move) {
        m.unmake(&mut self.board, &mut self.game_state);
    }

    /// Return player moves from this node.
    pub fn get_player_moves(&self) -> Vec<Move> {
        rules::get_player_moves(&self.board, &self.game_state)
    }

    /// Compute stats for both players for this node.
    pub fn compute_stats(&self) -> (stats::BoardStats, stats::BoardStats) {
        stats::BoardStats::new_from(&self.board, &self.game_state)
    }
}

impl fmt::Debug for Node {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "Node {{ board: [...], game_state: {:?} }}",
            self.game_state
        )
    }
}

impl fmt::Display for Node {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut s = vec!();
        self.board.draw_to(&mut s);
        let board_drawing = String::from_utf8_lossy(&s).to_string();
        write!(f, "{}{}", board_drawing, self.game_state)
    }
}

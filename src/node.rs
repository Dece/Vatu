use std::fmt;
use std::hash::{Hash, Hasher};

use crate::board;
use crate::movement::{self, Move};
use crate::rules;
use crate::stats;

/// Analysis node: a board along with the game state.
#[derive(Clone)]
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
            board: board::new_empty(),
            game_state: rules::GameState::new(),
        }
    }

    /// Apply a move to this node.
    pub fn apply_move(&mut self, m: &Move) {
        movement::apply_move_to(&mut self.board, &mut self.game_state, m);
    }

    /// Return player moves from this node.
    pub fn get_player_moves(&self, commit: bool) -> Vec<Move> {
        rules::get_player_moves(&self.board, &self.game_state, commit)
    }

    /// Compute stats for both players for this node.
    pub fn compute_stats(&self) -> (stats::BoardStats, stats::BoardStats) {
        stats::compute_stats(&self.board, &self.game_state)
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
        board::draw(&self.board, &mut s);
        let board_drawing = String::from_utf8_lossy(&s).to_string();
        write!(
            f,
            "* Board:\n{}\n\
             * Game state:\n{}",
            board_drawing, self.game_state
        )
    }
}

impl PartialEq for Node {
    fn eq(&self, other: &Self) -> bool {
        self.board.iter().zip(other.board.iter()).all(|(a, b)| a == b)
        && self.game_state == other.game_state
    }
}

impl Eq for Node {}

impl Hash for Node {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.board.iter().for_each(|square| state.write_u8(*square));
        self.game_state.hash(state);
    }
}

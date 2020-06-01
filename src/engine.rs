//! Vatu engine.

use crate::board;
use crate::notation;

pub struct Engine {
    board: board::Board,
}

impl Engine {
    pub fn new() -> Engine {
        Engine {
            board: board::new_empty(),
        }
    }

    pub fn apply_fen(&mut self, fen: &notation::Fen) {
        self.set_placement(&fen.placement);
    }

    fn set_placement(&mut self, placement: &str) {
        self.board = board::new_from_fen(placement);
    }
}

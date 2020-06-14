//! Analysis functions.

use std::sync::{Arc, atomic, mpsc};

use crate::board;
use crate::engine;
use crate::movement::Move;
use crate::node::Node;
use crate::notation;
use crate::rules;
use crate::stats;

const MIN_F32: f32 = std::f32::NEG_INFINITY;
const MAX_F32: f32 = std::f32::INFINITY;

/// Analysis worker.
pub struct Analyzer {
    pub debug: bool,
    node: Node,
    engine_tx: mpsc::Sender<engine::Cmd>,
    max_depth: u32,
}

/// Analysis parameters.
#[derive(Clone)]
pub struct AnalysisParams {
    pub move_time: i32,
    pub white_time: i32,
    pub black_time: i32,
    pub white_inc: i32,
    pub black_inc: i32,
}

impl Analyzer {
    /// Create a new worker to analyze from `node`.
    pub fn new(node: Node, engine_tx: mpsc::Sender<engine::Cmd>) -> Analyzer {
        Analyzer { debug: false, node, engine_tx, max_depth: 1 }
    }

    fn log(&self, message: String) {
        self.engine_tx.send(engine::Cmd::Log(message)).unwrap();
    }

    /// Analyse best moves for the node.
    ///
    /// - `args`: parameters provided for this analysis.
    /// - `score_map`: a NodeEvalMap to read and update.
    /// - `working`: flag telling whether to keep working or to stop.
    pub fn analyze(
        &mut self,
        _args: &AnalysisParams,
        working: Arc<atomic::AtomicBool>,
    ) {
        if !working.load(atomic::Ordering::Relaxed) {
            return;
        }
        if self.debug {
            self.log(format!("Analyzing node:\n{}", &self.node));
            let moves = self.node.get_player_moves(true);
            self.log(format!("Legal moves: {}", notation::move_list_to_string(&moves)));
        }

        self.max_depth = 2;
        let color_factor = if board::is_white(self.node.game_state.color) { 1 } else { -1 } as f32;
        let (max_score, best_move) = self.negamax(&self.node, 0, color_factor);

        if best_move.is_some() {
            let log_str = format!(
                "Best move {} evaluated {}",
                notation::move_to_string(&best_move.unwrap()), max_score
            );
            self.log(log_str);
            self.engine_tx.send(engine::Cmd::TmpBestMove(best_move)).unwrap();
        } else {
            // If no best move could be found, checkmate is unavoidable; send the first legal move.
            self.log("Checkmate is unavoidable.".to_string());
            let moves = rules::get_player_moves(&self.node.board, &self.node.game_state, true);
            let m = if moves.len() > 0 { Some(moves[0]) } else { None };
            self.engine_tx.send(engine::Cmd::TmpBestMove(m)).unwrap();
        }
    }

    fn negamax(
        &self,
        node: &Node,
        depth: u32,
        color_f: f32,
    ) -> (f32, Option<Move>) {
        if depth == self.max_depth {
            let stats = node.compute_stats();
            return (color_f * evaluate(&stats), None)
        }
        let moves = node.get_player_moves(true);
        let mut best_score = MIN_F32;
        let mut best_move = None;
        for m in moves {
            let mut sub_node = node.clone();
            sub_node.apply_move(&m);
            let (score, _) = self.negamax(&mut sub_node, depth + 1, -color_f);
            let score = -score;
            if score >= best_score {
                best_score = score;
                best_move = Some(m);
            }
        }
        (best_score, best_move)
    }
}

/// Compute a score for white/black board stats.
///
/// This uses the formula proposed by Shannon in his 1949 paper called
/// "Programming a Computer for Playing Chess", as it is quite simple
/// yet provide good enough results.
fn evaluate(stats: &(stats::BoardStats, stats::BoardStats)) -> f32 {
    let (player_stats, opponent_stats) = stats;

    200.0 * (player_stats.num_kings - opponent_stats.num_kings) as f32
        + 9.0 * (player_stats.num_queens - opponent_stats.num_queens) as f32
        + 5.0 * (player_stats.num_rooks - opponent_stats.num_rooks) as f32
        + 3.0 * (player_stats.num_bishops - opponent_stats.num_bishops) as f32
        + 3.0 * (player_stats.num_knights - opponent_stats.num_knights) as f32
        + (player_stats.num_pawns - opponent_stats.num_pawns) as f32
        - 0.5 * (
            player_stats.num_doubled_pawns - opponent_stats.num_doubled_pawns +
            player_stats.num_isolated_pawns - opponent_stats.num_isolated_pawns +
            player_stats.num_backward_pawns - opponent_stats.num_backward_pawns
        ) as f32
        + 0.1 * (player_stats.mobility - opponent_stats.mobility) as f32
}

#[cfg(test)]
mod tests {
    // use super::*;

    #[test]
    fn test_minimax() {
        // FIXME
        // let mut node = Node::new();
        // node.game_state.castling = 0;

        // // White mates in 1 move, queen to d7.
        // board::set_square(&mut node.board, &pos("a1"), board::SQ_WH_K);
        // board::set_square(&mut node.board, &pos("c6"), board::SQ_WH_P);
        // board::set_square(&mut node.board, &pos("h7"), board::SQ_WH_Q);
        // board::set_square(&mut node.board, &pos("d8"), board::SQ_BL_K);
        // let (_, m) = minimax(&mut node, 0, 2, true);
        // assert_eq!(m.unwrap(), notation::parse_move("h7d7"));

        // // Check that it works for black as well.
        // board::set_square(&mut node.board, &pos("a1"), board::SQ_BL_K);
        // board::set_square(&mut node.board, &pos("c6"), board::SQ_BL_P);
        // board::set_square(&mut node.board, &pos("h7"), board::SQ_BL_Q);
        // board::set_square(&mut node.board, &pos("d8"), board::SQ_WH_K);
        // node.game_state.color = board::SQ_BL;
        // let (_, m) = minimax(&mut node, 0, 2, true);
        // assert_eq!(m.unwrap(), notation::parse_move("h7d7"));
    }
}

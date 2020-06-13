//! Analysis functions.

use std::fmt;
use std::hash::{Hash, Hasher};
use std::sync::{Arc, atomic, mpsc};

use crate::board;
use crate::engine;
use crate::notation;
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
        (
            self.board.iter().zip(other.board.iter()).all(|(a, b)| a == b) &&
            self.game_state == other.game_state
        )
    }
}

impl Eq for Node {}

impl Hash for Node {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.board.iter().for_each(|square| state.write_u8(*square));
        self.game_state.hash(state);
    }
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

const MIN_F32: f32 = std::f32::NEG_INFINITY;
const MAX_F32: f32 = std::f32::INFINITY;

/// Analyse best moves for a given node.
pub fn analyze(
    node: &mut Node,
    _args: &AnalysisParams,
    working: Arc<atomic::AtomicBool>,
    tx: mpsc::Sender<engine::Cmd>,
    debug: bool,
) {
    if !working.load(atomic::Ordering::Relaxed) {
        return;
    }
    if debug {
        tx.send(engine::Cmd::Log(format!("\tAnalyzing node:\n{}", node))).unwrap();
        let moves = rules::get_player_moves(&node.board, &node.game_state, true);
        let moves_str = format!("\tLegal moves: {}", notation::move_list_to_string(&moves));
        tx.send(engine::Cmd::Log(moves_str)).unwrap();
    }

    let (max_score, best_move) = minimax(node, 0, 2, board::is_white(node.game_state.color));

    if best_move.is_some() {
        let log_str = format!(
            "\tBest move {} evaluated {}",
            notation::move_to_string(&best_move.unwrap()), max_score
        );
        tx.send(engine::Cmd::Log(log_str)).unwrap();
        tx.send(engine::Cmd::TmpBestMove(best_move)).unwrap();
    } else {
        // If no best move could be found, checkmate is unavoidable; send the first legal move.
        tx.send(engine::Cmd::Log("Checkmate is unavoidable.".to_string())).unwrap();
        let moves = rules::get_player_moves(&node.board, &node.game_state, true);
        let m = if moves.len() > 0 { Some(moves[0]) } else { None };
        tx.send(engine::Cmd::TmpBestMove(m)).unwrap();
    }

    // thread::sleep(time::Duration::from_secs(1));
    // for _ in 0..4 {
    //     let board = board.clone();
    //     let wip = wip.clone();
    //     thread::spawn(move || {
    //         analyze(&board, wip);
    //     });
    // }

}

/// Provide a "minimax" score for this node.
///
/// This method recursively looks alternatively for minimum score for
/// one player, then maximum for its opponent; that way it assumes the
/// opponent always does their best.
///
/// `depth` is increased at each recursive call; when `max_depth` is
/// reached, evaluate the current node and return its score.
///
/// `maximizing` specifies whether the method should look for the
/// highest possible score (when true) or the lowest (when false).
fn minimax(
    node: &mut Node,
    depth: u32,
    max_depth: u32,
    maximizing: bool
) -> (f32, Option<rules::Move>) {
    if depth == max_depth {
        let stats = stats::compute_stats(&node.board, &node.game_state);
        return (evaluate(&stats), None);
    }
    let mut minmax = if maximizing { MIN_F32 } else { MAX_F32 };
    let mut minmax_move = None;
    let moves = rules::get_player_moves(&node.board, &node.game_state, true);
    for m in moves {
        let mut sub_node = node.clone();
        rules::apply_move_to(&mut sub_node.board, &mut sub_node.game_state, &m);
        if maximizing {
            let (score, _) = minimax(&mut sub_node, depth + 1, max_depth, false);
            if score >= minmax {
                minmax = score;
                minmax_move = Some(m);
            }
        } else {
            let (score, _) = minimax(&mut sub_node, depth + 1, max_depth, true);
            if score <= minmax {
                minmax = score;
                minmax_move = Some(m);
            }
        }
    }
    (minmax, minmax_move)
}

/// Compute a score for white/black board stats.
///
/// This uses the formula proposed by Shannon in his 1949 paper called
/// "Programming a Computer for Playing Chess", as it is quite simple
/// yet provide good enough results.
fn evaluate(stats: &(stats::BoardStats, stats::BoardStats)) -> f32 {
    let (ws, bs) = stats;

    200.0 * (ws.num_kings - bs.num_kings) as f32
    + 9.0 * (ws.num_queens - bs.num_queens) as f32
    + 5.0 * (ws.num_rooks - bs.num_rooks) as f32
    + 3.0 * (ws.num_bishops - bs.num_bishops) as f32
    + 3.0 * (ws.num_knights - bs.num_knights) as f32
    + (ws.num_pawns - bs.num_pawns) as f32
    - 0.5 * (
        ws.num_doubled_pawns - bs.num_doubled_pawns +
        ws.num_isolated_pawns - bs.num_isolated_pawns +
        ws.num_backward_pawns - bs.num_backward_pawns
    ) as f32
    + 0.1 * (ws.mobility - bs.mobility) as f32
}

#[cfg(test)]
mod tests {
    use super::*;
    use board::pos;

    #[test]
    fn test_minimax() {
        let mut node = Node::new();
        node.game_state.castling = 0;

        // White mates in 1 move, queen to d7.
        board::set_square(&mut node.board, &pos("a1"), board::SQ_WH_K);
        board::set_square(&mut node.board, &pos("c6"), board::SQ_WH_P);
        board::set_square(&mut node.board, &pos("h7"), board::SQ_WH_Q);
        board::set_square(&mut node.board, &pos("d8"), board::SQ_BL_K);
        let (_, m) = minimax(&mut node, 0, 2, true);
        assert_eq!(m.unwrap(), notation::parse_move("h7d7"));

        // Check that it works for black as well.
        board::set_square(&mut node.board, &pos("a1"), board::SQ_BL_K);
        board::set_square(&mut node.board, &pos("c6"), board::SQ_BL_P);
        board::set_square(&mut node.board, &pos("h7"), board::SQ_BL_Q);
        board::set_square(&mut node.board, &pos("d8"), board::SQ_WH_K);
        node.game_state.color = board::SQ_BL;
        let (_, m) = minimax(&mut node, 0, 2, true);
        assert_eq!(m.unwrap(), notation::parse_move("h7d7"));
    }

    #[test]
    fn test_evaluate() {
        let mut node = Node::new();
        let stats = stats::compute_stats(&node.board, &node.game_state);
        assert_eq!(evaluate(&stats), 0.0);

        rules::apply_move_to(&mut node.board, &mut node.game_state, &notation::parse_move("d2d4"));
        let stats = stats::compute_stats(&node.board, &node.game_state);
        assert_eq!(evaluate(&stats), 0.0);
    }
}

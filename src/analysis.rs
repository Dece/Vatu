//! Analysis functions.

use std::sync::{Arc, atomic, mpsc};
use std::time::Instant;

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
    nps_time: Instant,
    num_nodes: u64,
    num_nodes_in_second: u64,
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

/// Analysis info to report.
#[derive(Debug, Clone)]
pub enum AnalysisInfo {
    Nodes(u64),
    Nps(u64),
    CurrentMove(Move),
}

impl Analyzer {
    /// Create a new worker to analyze from `node`.
    pub fn new(node: Node, engine_tx: mpsc::Sender<engine::Cmd>) -> Analyzer {
        Analyzer {
            debug: false,
            node,
            engine_tx,
            max_depth: 1,
            nps_time: Instant::now(),
            num_nodes: 0,
            num_nodes_in_second: 0,
        }
    }

    fn log(&self, message: String) {
        self.engine_tx.send(engine::Cmd::Log(message)).unwrap();
    }

    fn report_info(&self, infos: Vec<AnalysisInfo>) {
        self.engine_tx.send(engine::Cmd::WorkerInfo(infos)).unwrap();
    }

    fn report_best_move(&self, m: Option<Move>) {
        self.engine_tx.send(engine::Cmd::WorkerBestMove(m)).unwrap();
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

        self.nps_time = Instant::now();
        self.max_depth = 4;
        let (max_score, best_move) = self.negamax(&self.node.clone(), MIN_F32, MAX_F32, 0);

        if best_move.is_some() {
            let log_str = format!(
                "Best move {} evaluated {}",
                notation::move_to_string(&best_move.unwrap()), max_score
            );
            self.log(log_str);
            self.report_best_move(best_move);
        } else {
            // If no best move could be found, checkmate is unavoidable; send the first legal move.
            self.log("Checkmate is unavoidable.".to_string());
            let moves = rules::get_player_moves(&self.node.board, &self.node.game_state, true);
            let m = if moves.len() > 0 { Some(moves[0]) } else { None };
            self.report_best_move(m);
        }
    }

    fn negamax(
        &mut self,
        node: &Node,
        alpha: f32,
        beta: f32,
        depth: u32,
    ) -> (f32, Option<Move>) {
        // Increment number of nodes for stats.
        self.num_nodes += 1;
        self.num_nodes_in_second += 1;

        // If we reached max depth, evaluate the node and stop searching.
        if depth == self.max_depth {
            let stats = node.compute_stats();
            let ev = evaluate(&stats);
            return (ev, None)
        }

        // Here's a good time to get some stats!
        if self.nps_time.elapsed().as_millis() >= 1000 {
            self.report_info(vec![
                AnalysisInfo::Nodes(self.num_nodes),
                AnalysisInfo::Nps(self.num_nodes_in_second),
            ]);
            self.num_nodes_in_second = 0;
            self.nps_time = Instant::now();
        }

        // Get negamax for playable moves.
        let moves = node.get_player_moves(true);
        let mut alpha = alpha;
        let mut best_score = MIN_F32;
        let mut best_move = None;
        for m in moves {
            let mut sub_node = node.clone();
            sub_node.apply_move(&m);
            let result = self.negamax(&sub_node, -beta, -alpha, depth + 1);
            let score = -result.0;
            if score > best_score {
                best_score = score;
                best_move = Some(m);
            }
            if best_score > alpha {
                alpha = best_score;
            }
            if alpha >= beta {
                break
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

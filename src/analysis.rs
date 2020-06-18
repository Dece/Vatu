//! Analysis functions.

use std::sync::{Arc, atomic, mpsc};
use std::time::Instant;

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
///
/// Parameters specifying when to stop an analysis (e.g. `max_depth`
/// and `time_limit`) can be used together without issues and the
/// worker will try to stop as soon as the first limit is reached.
pub struct Analyzer {
    /// Enable some debug logs.
    pub debug: bool,
    /// Root node for this analysis.
    node: Node,
    /// Sender for engine commands.
    engine_tx: mpsc::Sender<engine::Cmd>,
    /// Stop working if flag is unset.
    working: Option<Arc<atomic::AtomicBool>>,
    /// Max depth to reach in the next analysis.
    max_depth: u32,
    /// Time limit for the next analysis.
    time_limit: i32,
    /// Instant when the analysis began.
    start_time: Option<Instant>,
    /// Instant of the last "per second" stats calculation.
    current_per_second_timer: Option<Instant>,
    /// Nodes analyzed in this analysis.
    num_nodes: u64,
    /// Node analyzed since the last NPS stat.
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
            working: None,
            max_depth: 1,
            time_limit: 0,
            start_time: None,
            current_per_second_timer: None,
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
        args: &AnalysisParams,
        working: Arc<atomic::AtomicBool>,
    ) {
        self.working = Some(working);
        self.set_limits(args);

        if self.debug {
            self.log(format!("Analyzing node:\n{}", &self.node));
            let moves = self.node.get_player_moves(true);
            self.log(format!("Legal moves: {}", notation::move_list_to_string(&moves)));
            self.log(format!("Move time: {}", self.time_limit));
        }

        self.start_time = Some(Instant::now());
        self.current_per_second_timer = Some(Instant::now());
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

    /// Set search limits.
    fn set_limits(&mut self, args: &AnalysisParams) {
        self.max_depth = 4;
        self.time_limit = if args.move_time != -1 {
            args.move_time
        } else {
            let (time, inc) = if board::is_white(self.node.game_state.color) {
                (args.white_time, args.white_inc)
            } else {
                (args.black_time, args.black_inc)
            };
            // If more than 2 minutes is left, use a 1m time limit.
            if time > 2*60*1000 {
                60*1000
            }
            // Else use 1/4 of the remaining time (plus the increment).
            else if time > 0 {
                (time / 4) + inc
            }
            // Or if there is not remaining time, do not use a time limit.
            else {
                i32::MAX
            }
        };
    }

    /// Return best score and associated move for this node.
    ///
    /// `depth` is the current search depth. `alpha` and `beta` are
    /// used for alpha-beta search tree pruning, where `alpha` is the
    /// lower score bound and `beta` the upper bound.
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

        // If we should stop searching, evaluate the node and stop.
        if self.should_stop_search(depth) {
            let stats = node.compute_stats();
            let ev = evaluate(&stats);
            return (ev, None)
        }

        // Here's a good time to get some stats!
        if self.current_per_second_timer.unwrap().elapsed().as_millis() >= 1000 {
            self.report_info(vec![
                AnalysisInfo::Nodes(self.num_nodes),
                AnalysisInfo::Nps(self.num_nodes_in_second),
            ]);
            self.num_nodes_in_second = 0;
            self.current_per_second_timer = Some(Instant::now());
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

    /// Return true if some parameter requires to stop searching.
    ///
    /// Check for max node depth, time limit and engine stop flag.
    fn should_stop_search(&self, depth: u32) -> bool {
        !self.working.as_ref().unwrap().load(atomic::Ordering::Relaxed)
        || depth == self.max_depth
        || self.start_time.unwrap().elapsed().as_millis() >= self.time_limit as u128
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

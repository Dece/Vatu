//! Vatu engine.

use std::sync::{Arc, atomic, mpsc};
use std::thread;

use crate::board;
use crate::notation;
use crate::rules;
use crate::stats;
use crate::uci;

const MIN_F32: f32 = std::f32::NEG_INFINITY;
const MAX_F32: f32 = std::f32::INFINITY;

/// Analysis engine.
pub struct Engine {
    /// Debug mode, log some data.
    debug: bool,
    /// Current game state, starting point of further analysis.
    node: Node,
    /// Communication mode.
    mode: Mode,
    /// If true, the engine is currently listening to incoming cmds.
    listening: bool,
    /// Shared flag to notify workers if they should keep working.
    working: Arc<atomic::AtomicBool>,
}

/// Analysis node: a board along with the game state.
#[derive(Clone)]
struct Node {
    /// Board for this node.
    board: board::Board,
    /// Game state.
    game_state: rules::GameState,
}

impl Node {
    fn new() -> Node {
        Node {
            board: board::new_empty(),
            game_state: rules::GameState::new(),
        }
    }
}

impl std::fmt::Debug for Node {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "Node {{ board: [...], game_state: {:?} }}",
            self.game_state
        )
    }
}

impl std::fmt::Display for Node {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
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

/// Engine communication mode.
enum Mode {
    /// No mode, sit here and do nothing.
    No,
    /// UCI mode: listen to Cmds, send Uci::Cmd::Engine commands.
    ///
    /// First value is the Uci command sender to report results.
    /// Second value is the receiver for all engine commands, whether
    /// it's from the Uci controller or analysis workers. Third is the
    /// sender that is passed to receive outer Uci and workers cmds.
    Uci(mpsc::Sender<uci::Cmd>, mpsc::Receiver<Cmd>, mpsc::Sender<Cmd>),
}

/// Engine commands.
#[derive(Debug)]
pub enum Cmd {
    // Commands that can be received by the engine.
    UciChannel(mpsc::Sender<Cmd>),        // Provide a sender to UCI to start receiving commands.
    UciDebug(bool),                       // UCI "debug" command.
    UciPosition(Vec<uci::PositionArgs>),  // UCI "position" command.
    UciGo(Vec<uci::GoArgs>),              // UCI "go" command.
    Stop,                                 // Stop working ASAP.
    TmpBestMove(Option<rules::Move>),  // Send best move found by analysis worker (TEMPORARY).
    WorkerInfo(Vec<Info>),                // Informations from a worker.

    // Commands that can be sent by the engine.
    /// Ask for a string to be logged or printed.
    ///
    /// Note that workers can send this command to engine, expecting
    /// the message to be forwarded to whatever can log.
    Log(String),
    /// Report found best move.
    BestMove(Option<rules::Move>),
    /// Report ongoing analysis information.
    Info(Vec<Info>),
}

/// Parameters for starting work.
#[derive(Clone)]
struct WorkArgs {
    move_time: i32,
    white_time: i32,
    black_time: i32,
    white_inc: i32,
    black_inc: i32,
}

/// Information to be transmitted back to whatever is listening.
#[derive(Debug, Clone)]
pub enum Info {
    CurrentMove(rules::Move),
}

/// General engine implementation.
impl Engine {
    pub fn new() -> Engine {
        Engine {
            debug: false,
            node: Node::new(),
            mode: Mode::No,
            listening: false,
            working: Arc::new(atomic::AtomicBool::new(false)),
        }
    }

    /// Listen for incoming commands.
    ///
    /// In UCI mode, read incoming Cmds over the MPSC channel.
    /// In no modes, stop listening immediately.
    pub fn listen(&mut self) {
        self.listening = true;
        while self.listening {
            match &self.mode {
                Mode::Uci(_, rx, _) => {
                    match rx.recv() {
                        Ok(c) => self.handle_command(&c),
                        Err(e) => eprintln!("Engine recv failure: {}", e),
                    }
                }
                _ => break,
            }
        }
    }

    /// Handle UCI commands passed as engine Cmds.
    fn handle_command(&mut self, cmd: &Cmd) {
        match cmd {
            // UCI commands.
            Cmd::UciDebug(on) => self.debug = *on,
            Cmd::UciPosition(args) => self.uci_position(args),
            Cmd::UciGo(args) => self.uci_go(args),
            Cmd::Stop => self.stop(),
            // Workers commands.
            Cmd::Log(s) => self.reply(Cmd::Log(s.to_string())),
            Cmd::WorkerInfo(infos) => self.reply(Cmd::Info(infos.to_vec())),
            Cmd::TmpBestMove(m) => self.reply(Cmd::BestMove(*m)),
            _ => eprintln!("Not an engine input command: {:?}", cmd),
        }
    }

    fn reply(&mut self, cmd: Cmd) {
        match &self.mode {
            Mode::Uci(tx, _, _) => {
                tx.send(uci::Cmd::Engine(cmd)).unwrap();
            }
            _ => {}
        }
    }

    /// Apply a FEN string to the engine state, replacing it.
    ///
    /// For speed purposes, it assumes values are always valid.
    fn apply_fen(&mut self, fen: &notation::Fen) {
        // Placement.
        self.node.board = board::new_from_fen(&fen.placement);
        // Color.
        match fen.color.chars().next().unwrap() {
            'w' => self.node.game_state.color = board::SQ_WH,
            'b' => self.node.game_state.color = board::SQ_BL,
            _ => {}
        };
        // Castling.
        for c in fen.castling.chars() {
            match c {
                'K' => self.node.game_state.castling |= rules::CASTLING_WH_K,
                'Q' => self.node.game_state.castling |= rules::CASTLING_WH_Q,
                'k' => self.node.game_state.castling |= rules::CASTLING_BL_K,
                'q' => self.node.game_state.castling |= rules::CASTLING_BL_Q,
                _ => {}
            }
        }
        // En passant.
        self.node.game_state.en_passant = match fen.en_passant.as_ref() {
            "-" => None,
            p => Some(board::pos(p)),
        };
        // Half moves.
        self.node.game_state.halfmove = fen.halfmove.parse::<i32>().ok().unwrap();
        // Full moves.
        self.node.game_state.fullmove = fen.fullmove.parse::<i32>().ok().unwrap();
    }

    fn apply_moves(&mut self, moves: &Vec<rules::Move>) {
        moves.iter().for_each(|m| self.apply_move(m));
    }

    fn apply_move(&mut self, m: &rules::Move) {
        rules::apply_move_to(&mut self.node.board, &mut self.node.game_state, m);
    }

    /// Start working on board, returning the best move found.
    ///
    /// Stop working after `movetime` ms, or go on forever if it's -1.
    fn work(&mut self, args: &WorkArgs) {
        self.working.store(true, atomic::Ordering::Relaxed);
        let mut node = self.node.clone();
        let args = args.clone();
        let working = self.working.clone();
        let tx = match &self.mode { Mode::Uci(_, _, tx) => tx.clone(), _ => return };
        let debug = self.debug;
        thread::spawn(move || {
            analyze(&mut node, &args, working, tx, debug);
        });
    }

    fn stop(&mut self) {
        self.working.store(false, atomic::Ordering::SeqCst);
    }
}

/// UCI commands management.
impl Engine {
    /// Setup engine for UCI communication.
    pub fn setup_uci(&mut self, uci_s: mpsc::Sender<uci::Cmd>) {
        // Create a channel to receive commands from Uci.
        let (engine_s, engine_r) = mpsc::channel();
        uci_s.send(uci::Cmd::Engine(Cmd::UciChannel(engine_s.clone()))).unwrap();
        self.mode = Mode::Uci(uci_s, engine_r, engine_s);
        self.listen();
    }

    /// Update board state from a "position" command's args.
    fn uci_position(&mut self, p_args: &Vec<uci::PositionArgs>) {
        for arg in p_args {
            match arg {
                uci::PositionArgs::Fen(fen) => {
                    self.apply_fen(&fen);
                },
                uci::PositionArgs::Startpos => {
                    let fen = notation::parse_fen(notation::FEN_START).unwrap();
                    self.apply_fen(&fen);
                },
                uci::PositionArgs::Moves(moves) => {
                    self.apply_moves(&moves);
                }
            }
        }
    }

    /// Start working using parameters passed with a "go" command.
    fn uci_go(&mut self, g_args: &Vec<uci::GoArgs>) {
        let mut args = WorkArgs {
            move_time: -1,
            white_time: -1,
            black_time: -1,
            white_inc: -1,
            black_inc: -1,
        };
        for arg in g_args {
            match arg {
                uci::GoArgs::MoveTime(ms) => args.move_time = *ms,
                uci::GoArgs::Infinite => {}
                uci::GoArgs::WTime(ms) => args.white_time = *ms,
                uci::GoArgs::BTime(ms) => args.black_time = *ms,
                uci::GoArgs::WInc(ms) => args.white_inc = *ms,
                uci::GoArgs::BInc(ms) => args.black_inc = *ms,
                _ => {}
            }
        }
        self.work(&args);
    }
}

fn analyze(
    node: &mut Node,
    _args: &WorkArgs,
    working: Arc<atomic::AtomicBool>,
    tx: mpsc::Sender<Cmd>,
    debug: bool,
) {
    if !working.load(atomic::Ordering::Relaxed) {
        return;
    }
    if debug {
        tx.send(Cmd::Log(format!("\tAnalyzing node:\n{}", node))).unwrap();
        let moves = rules::get_player_moves(&node.board, &node.game_state, true);
        let moves_str = format!("\tLegal moves: {}", notation::move_list_to_string(&moves));
        tx.send(Cmd::Log(moves_str)).unwrap();
    }

    let (max_score, best_move) = minimax(node, 0, 3, board::is_white(node.game_state.color));

    if best_move.is_some() {
        let log_str = format!(
            "\tBest move {} evaluated {}",
            notation::move_to_string(&best_move.unwrap()), max_score
        );
        tx.send(Cmd::Log(log_str)).unwrap();
        tx.send(Cmd::TmpBestMove(best_move)).unwrap();
    } else {
        // If no best move could be found, checkmate is unavoidable; send the first legal move.
        tx.send(Cmd::Log("Checkmate is unavoidable.".to_string())).unwrap();
        let moves = rules::get_player_moves(&node.board, &node.game_state, true);
        let m = if moves.len() > 0 { Some(moves[0]) } else { None };
        tx.send(Cmd::TmpBestMove(m)).unwrap();
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

fn minimax(
    node: &mut Node,
    depth: u32,
    max_depth: u32,
    maximizing: bool
) -> (f32, Option<rules::Move>) {
    if depth == max_depth {
        let stats = stats::compute_stats(&node.board);
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
            if score > minmax {
                minmax = score;
                minmax_move = Some(m);
            }
        } else {
            let (score, _) = minimax(&mut sub_node, depth + 1, max_depth, true);
            if score < minmax {
                minmax = score;
                minmax_move = Some(m);
            }
        }
    }
    (minmax, minmax_move)
}

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
        let stats = stats::compute_stats(&node.board);
        assert_eq!(evaluate(&stats), 0.0);

        rules::apply_move_to_board(&mut node.board, &notation::parse_move("d2d4"));
        let stats = stats::compute_stats(&node.board);
        assert_eq!(evaluate(&stats), 0.0);
    }
}

//! Vatu engine.
//!
//! Hold the various data needed to perform a game analysis,
//! but actual analysis code is in the `analysis` module.

use std::sync::Arc;
use std::sync::mpsc;
use std::sync::atomic::{self, AtomicBool};
use std::thread;

use crate::analysis;
use crate::board;
use crate::castling;
use crate::fen;
use crate::movement::Move;
use crate::node::Node;
use crate::uci;

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
    /// flag to notify workers if they should keep working.
    working: Arc<AtomicBool>,
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

    /// Provide a sender to UCI to start receiving commands.
    UciChannel(mpsc::Sender<Cmd>),
    /// UCI "debug" command.
    UciDebug(bool),
    /// UCI "position" command.
    UciPosition(Vec<uci::PositionArgs>),
    /// UCI "go" command.
    UciGo(Vec<uci::GoArgs>),
    /// Stop working ASAP.
    Stop,
    /// Informations from a worker.
    WorkerInfo(Vec<analysis::AnalysisInfo>),
    /// Send best move found by analysis worker.
    WorkerBestMove(Option<Move>),
    /// Draw board in logs.
    DrawBoard,

    // Commands that can be sent by the engine.

    /// Ask for a string to be logged or printed.
    ///
    /// Note that workers can send this command to engine, expecting
    /// the message to be forwarded to whatever can log.
    Log(String),
    /// Report ongoing analysis information.
    Info(Vec<analysis::AnalysisInfo>),
    /// Report found best move.
    BestMove(Option<Move>),
}

/// General engine implementation.
impl Engine {
    pub fn new() -> Engine {
        Engine {
            debug: false,
            node: Node::new(),
            mode: Mode::No,
            listening: false,
            working: Arc::new(AtomicBool::new(false)),
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

    /// Enable debug output.
    pub fn enable_debug(&mut self) {
        self.debug = true;
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
            Cmd::WorkerBestMove(m) => self.reply(Cmd::BestMove(m.clone())),
            // Other commands.
            Cmd::DrawBoard => {
                let mut s = vec!();
                self.node.board.draw_to(&mut s);
                let s = format!("{}", String::from_utf8_lossy(&s));
                self.reply(Cmd::Log(s));
            }
            _ => eprintln!("Not an engine input command: {:?}", cmd),
        }
    }

    /// Send a command back to the controlling interface.
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
    fn apply_fen(&mut self, fen: &fen::Fen) {
        // Placement.
        self.node.board = board::Board::new_from_fen(&fen.placement);
        // Color.
        match fen.color.chars().next().unwrap() {
            'w' => self.node.game_state.color = board::WHITE,
            'b' => self.node.game_state.color = board::BLACK,
            _ => {}
        };
        // Castling.
        for c in fen.castling.chars() {
            match c {
                'K' => self.node.game_state.castling |= castling::CASTLE_WH_K,
                'Q' => self.node.game_state.castling |= castling::CASTLE_WH_Q,
                'k' => self.node.game_state.castling |= castling::CASTLE_BL_K,
                'q' => self.node.game_state.castling |= castling::CASTLE_BL_Q,
                _ => {}
            }
        }
        // En passant.
        self.node.game_state.en_passant = match fen.en_passant.as_ref() {
            "-" => None,
            s => Some(board::sq_from_string(s)),
        };
        // Half moves.
        self.node.game_state.halfmove = fen.halfmove.parse::<i32>().ok().unwrap();
        // Full moves.
        self.node.game_state.fullmove = fen.fullmove.parse::<i32>().ok().unwrap();
    }

    /// Apply a series of moves to the current node.
    fn apply_moves(&mut self, moves: &mut Vec<Move>) {
        moves.iter_mut().for_each(|m| m.apply_to(&mut self.node.board, &mut self.node.game_state));
    }

    /// Start working on board, returning the best move found.
    ///
    /// Stop working after `movetime` ms, or go on forever if it's -1.
    fn work(&mut self, args: &analysis::AnalysisParams) {
        self.working.store(true, atomic::Ordering::Relaxed);
        let args = args.clone();
        let working = self.working.clone();
        let tx = match &self.mode { Mode::Uci(_, _, tx) => tx.clone(), _ => return };
        let mut worker = analysis::Analyzer::new(self.node.clone(), tx);
        worker.debug = self.debug;
        thread::spawn(move || {
            worker.analyze(&args, working);
        });
    }

    /// Unset the work flag, stopping workers.
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
                    let fen = fen::parse_fen(fen::FEN_START).unwrap();
                    self.apply_fen(&fen);
                },
                uci::PositionArgs::Moves(moves) => {
                    self.apply_moves(&mut moves.clone());
                }
            }
        }
    }

    /// Start working using parameters passed with a "go" command.
    fn uci_go(&mut self, g_args: &Vec<uci::GoArgs>) {
        let mut args = analysis::AnalysisParams {
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

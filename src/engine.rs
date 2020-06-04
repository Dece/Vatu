//! Vatu engine.

use std::sync::{Arc, atomic, mpsc};
use std::thread;
use std::time;

use rand::seq::IteratorRandom;

use crate::board;
use crate::notation;
use crate::rules;
use crate::uci;

/// Analysis engine.
pub struct Engine {
    /// Current game state, starting point of further analysis.
    state: GameState,
    /// Communication mode.
    mode: Mode,
    /// If true, the engine is currently listening to incoming cmds.
    listening: bool,
    /// Shared flag to notify workers if they should keep working.
    working: Arc<atomic::AtomicBool>,
}

/// Representation of a game state that can cloned to analysis workers.
///
/// It does not include various parameters such as clocks so that they
/// can be passed separately using `WorkArgs`.
#[derive(Clone)]
struct GameState {
    board: board::Board,
    color: u8,
    castling: u8,
    en_passant: Option<board::Pos>,
    halfmove: i32,
    fullmove: i32,
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
    UciPosition(Vec<uci::PositionArgs>),  // UCI "position" command.
    UciGo(Vec<uci::GoArgs>),              // UCI "go" command.
    Stop,                                 // Stop working ASAP.
    TmpBestMove(Option<board::Move>),  // Send best move found by analysis worker (TEMPORARY).

    // Commands that can be sent by the engine.
    BestMove(Option<board::Move>),
}

#[derive(Clone)]
struct WorkArgs {
    move_time: i32,
    white_time: i32,
    black_time: i32,
    white_inc: i32,
    black_inc: i32,
}

pub const CASTLING_WH_K: u8 = 0b00000001;
pub const CASTLING_WH_Q: u8 = 0b00000010;
pub const CASTLING_BL_K: u8 = 0b00000100;
pub const CASTLING_BL_Q: u8 = 0b00001000;
pub const CASTLING_MASK: u8 = 0b00001111;

/// General engine implementation.
impl Engine {
    pub fn new() -> Engine {
        Engine {
            state: GameState {
                board: board::new_empty(),
                color: board::SQ_WH,
                castling: CASTLING_MASK,
                en_passant: None,
                halfmove: 0,
                fullmove: 1,
            },
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
            Cmd::UciPosition(args) => self.uci_position(&args),
            Cmd::UciGo(args) => self.uci_go(&args),
            Cmd::Stop => self.stop(),
            // Workers commands.
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
        eprintln!("Applying FEN {:?}", fen);
        self.set_fen_placement(&fen.placement);
        self.set_fen_color(&fen.color);
        self.set_fen_castling(&fen.castling);
        self.set_fen_en_passant(&fen.en_passant);
        self.set_fen_halfmove(&fen.halfmove);
        self.set_fen_fullmove(&fen.fullmove);
    }

    fn set_fen_placement(&mut self, placement: &str) {
        self.state.board = board::new_from_fen(placement);
    }

    fn set_fen_color(&mut self, color: &str) {
        match color.chars().next().unwrap() {
            'w' => self.state.color = board::SQ_WH,
            'b' => self.state.color = board::SQ_BL,
            _ => {}
        }
    }

    fn set_fen_castling(&mut self, castling: &str) {
        for c in castling.chars() {
            match c {
                'K' => self.state.castling |= CASTLING_WH_K,
                'Q' => self.state.castling |= CASTLING_WH_Q,
                'k' => self.state.castling |= CASTLING_BL_K,
                'q' => self.state.castling |= CASTLING_BL_Q,
                _ => {}
            }
        }
    }

    fn set_fen_en_passant(&mut self, en_passant: &str) {
        self.state.en_passant = match en_passant {
            "-" => None,
            p => Some(board::pos(p)),
        };
    }

    fn set_fen_halfmove(&mut self, halfmove: &str) {
        self.state.halfmove = halfmove.parse::<i32>().ok().unwrap();
    }

    fn set_fen_fullmove(&mut self, fullmove: &str) {
        self.state.fullmove = fullmove.parse::<i32>().ok().unwrap();
    }

    fn apply_moves(&mut self, moves: &Vec<board::Move>) {
        moves.iter().for_each(|m| self.apply_move(m));
    }

    fn apply_move(&mut self, m: &board::Move) {
        board::apply_into(&mut self.state.board, m);
    }

    /// Start working on board, returning the best move found.
    ///
    /// Stop working after `movetime` ms, or go on forever if it's -1.
    fn work(&mut self, args: &WorkArgs) {
        self.working.store(true, atomic::Ordering::Relaxed);
        let state = self.state.clone();
        let args = args.clone();
        let working = self.working.clone();
        let tx = match &self.mode { Mode::Uci(_, _, tx) => tx.clone(), _ => return };
        thread::spawn(move || {
            analyze(&state, &args, working, tx);
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
                    self.state.color = if moves.len() % 2 == 0 {
                        board::SQ_WH
                    } else {
                        board::SQ_BL
                    };
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
    state: &GameState,
    _args: &WorkArgs,
    wip: Arc<atomic::AtomicBool>,
    tx: mpsc::Sender<Cmd>,
) {
    if !wip.load(atomic::Ordering::Relaxed) {
        return;
    }

    // Stupid engine! Return a random move.
    let moves = rules::get_player_legal_moves(&state.board, state.color);
    let mut rng = rand::thread_rng();
    let best_move = moves.iter().choose(&mut rng).and_then(|m| Some(*m));
    thread::sleep(time::Duration::from_millis(1000u64));
    tx.send(Cmd::TmpBestMove(best_move)).unwrap();

    // thread::sleep(time::Duration::from_secs(1));
    // for _ in 0..4 {
    //     let board = board.clone();
    //     let wip = wip.clone();
    //     thread::spawn(move || {
    //         analyze(&board, wip);
    //     });
    // }

}

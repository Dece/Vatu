//! Vatu engine.

use std::sync::mpsc;
use std::thread;
use std::time;

use rand::seq::IteratorRandom;

use crate::board;
use crate::notation;
use crate::rules;
use crate::uci;

pub struct Engine {
    board: board::Board,             // Board to analyse.
    color: u8,                       // Color to analyse.
    castling: u8,                    // Castling state.
    en_passant: Option<board::Pos>,  // En passant state.
    halfmove: i32,                   // Current half moves.
    fullmove: i32,                   // Current full moves.
    mode: Mode,
    listening: bool,
}

pub enum Mode {
    // No mode, sit here and do nothing.
    No,
    // UCI mode: listen to Cmds, send Uci::Cmd::Engine commands.
    Uci(mpsc::Sender<uci::Cmd>, mpsc::Receiver<Cmd>),
}

#[derive(Debug)]
pub enum Cmd {
    // Commands that can be received by the engine.
    UciChannel(mpsc::Sender<Cmd>),        // Provide a sender to UCI to start receiving commands.
    UciPosition(Vec<uci::PositionArgs>),  // UCI "position" command.
    UciGo(Vec<uci::GoArgs>),              // UCI "go" command.

    // Commands that can be sent by the engine.
    BestMove(Option<board::Move>),
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
            board: board::new_empty(),
            color: board::SQ_WH,
            castling: CASTLING_MASK,
            en_passant: None,
            halfmove: 0,
            fullmove: 1,
            mode: Mode::No,
            listening: false,
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
                Mode::Uci(_, rx) => {
                    match rx.recv() {
                        Ok(c) => self.handle_uci_command(&c),
                        Err(e) => eprintln!("Engine recv failure: {}", e),
                    }
                }
                _ => break,
            }
        }
    }

    fn reply(&mut self, cmd: Cmd) {
        match &self.mode {
            Mode::Uci(tx, _) => {
                tx.send(uci::Cmd::Engine(cmd)).unwrap();
            }
            _ => {}
        }
    }

    /// Apply a FEN string to the engine state, replacing it.
    ///
    /// For speed purposes, it assumes values are always valid.
    fn apply_fen(&mut self, fen: &notation::Fen) {
        self.set_fen_placement(&fen.placement);
        self.set_fen_color(&fen.color);
        self.set_fen_castling(&fen.castling);
        self.set_fen_en_passant(&fen.en_passant);
        self.set_fen_halfmove(&fen.halfmove);
        self.set_fen_fullmove(&fen.fullmove);
    }

    fn set_fen_placement(&mut self, placement: &str) {
        self.board = board::new_from_fen(placement);
    }

    fn set_fen_color(&mut self, color: &str) {
        match color.chars().next().unwrap() {
            'w' => self.color = board::SQ_WH,
            'b' => self.color = board::SQ_BL,
            _ => {}
        }
    }

    fn set_fen_castling(&mut self, castling: &str) {
        for c in castling.chars() {
            match c {
                'K' => self.castling |= CASTLING_WH_K,
                'Q' => self.castling |= CASTLING_WH_Q,
                'k' => self.castling |= CASTLING_BL_K,
                'q' => self.castling |= CASTLING_BL_Q,
                _ => {}
            }
        }
    }

    fn set_fen_en_passant(&mut self, en_passant: &str) {
        self.en_passant = match en_passant {
            "-" => None,
            p => Some(board::pos(p)),
        };
    }

    fn set_fen_halfmove(&mut self, halfmove: &str) {
        self.halfmove = halfmove.parse::<i32>().ok().unwrap();
    }

    fn set_fen_fullmove(&mut self, fullmove: &str) {
        self.fullmove = fullmove.parse::<i32>().ok().unwrap();
    }

    fn apply_moves(&mut self, moves: &Vec<board::Move>) {
        moves.iter().for_each(|m| self.apply_move(m));
    }

    fn apply_move(&mut self, m: &board::Move) {
        board::apply_into(&mut self.board, m);
    }

    /// Start working on board, returning the best move found.
    ///
    /// Stop working after `movetime` ms, or go on forever if it's -1.
    pub fn work(&mut self, movetime: i32) -> Option<board::Move> {
        // Stupid engine! Return a random move.
        let moves = rules::get_player_legal_moves(&self.board, self.color);
        let mut rng = rand::thread_rng();
        let best_move = moves.iter().choose(&mut rng).and_then(|m| Some(*m));
        // board::draw(&self.board);
        thread::sleep(time::Duration::from_millis(movetime as u64));
        best_move
    }
}

/// UCI commands management.
impl Engine {
    /// Setup engine for UCI communication.
    pub fn setup_uci(&mut self, uci_s: mpsc::Sender<uci::Cmd>) {
        // Create a channel to receive commands from Uci.
        let (engine_s, engine_r) = mpsc::channel();
        uci_s.send(uci::Cmd::Engine(Cmd::UciChannel(engine_s))).unwrap();
        self.mode = Mode::Uci(uci_s, engine_r);
        self.listen();
    }

    /// Handle UCI commands passed as engine Cmds.
    fn handle_uci_command(&mut self, cmd: &Cmd) {
        match cmd {
            Cmd::UciPosition(args) => self.uci_position(&args),
            Cmd::UciGo(args) => self.uci_go(&args),
            _ => eprintln!("Not an UCI command: {:?}", cmd),
        }
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
        let mut movetime = -1;
        for arg in g_args {
            match arg {
                uci::GoArgs::MoveTime(ms) => movetime = *ms,
                uci::GoArgs::Infinite => movetime = -1,
            }
        }
        let best_move = self.work(movetime);
        self.reply(Cmd::BestMove(best_move));
    }

}

//! Vatu engine.

use std::sync::mpsc;

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
}

pub enum Mode {
    // No mode, sit here and do nothing.
    No,
    // UCI mode: listen to Cmds, send Uci::Cmd::Engine commands.
    Uci(mpsc::Receiver<Cmd>, mpsc::Sender<uci::Cmd>),
}

#[derive(Debug)]
pub enum Cmd {
    // Commands that can be received by the engine.
    Ping(String),  // Test if the engine is responding.
    UciPosition(Vec<uci::PositionArgs>),  // UCI "position" command.

    // Commands that can be sent by the engine.
    Pong(String),  // Answer a Ping command with the same payload.
}

pub const CASTLING_WH_K: u8 = 0b00000001;
pub const CASTLING_WH_Q: u8 = 0b00000010;
pub const CASTLING_BL_K: u8 = 0b00000100;
pub const CASTLING_BL_Q: u8 = 0b00001000;
pub const CASTLING_MASK: u8 = 0b00001111;

impl Engine {
    pub fn new(mode: Mode) -> Engine {
        Engine {
            board: board::new_empty(),
            color: board::SQ_WH,
            castling: CASTLING_MASK,
            en_passant: None,
            halfmove: 0,
            fullmove: 1,
            mode,
        }
    }

    /// Listen for incoming commands.
    ///
    /// In UCI mode, read incoming Cmds over the MPSC channel.
    /// In no modes, stop listening immediately.
    pub fn listen(&mut self) {
        loop {
            match self.mode {
                Mode::No => break,
                Mode::Uci(rx, tx) => {
                    self.recv_uci(rx, tx);
                }
            }
        }
    }

    /// Apply a FEN string to the engine state, replacing it.
    ///
    /// For speed purposes, it assumes values are always valid.
    pub fn apply_fen(&mut self, fen: &notation::Fen) {
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

    /// Start working on board, returning the best move found.
    ///
    /// Stop working after `movetime` ms, or go on forever if it's -1.
    pub fn work(&mut self, _movetime: i32) -> board::Move {
        // Stupid engine! Return a random move.
        let moves = rules::get_player_legal_moves(&self.board, self.color);
        let mut rng = rand::thread_rng();
        let best_move = moves.iter().choose(&mut rng).unwrap();
        *best_move
    }

    /// Receive a command from Uci.
    pub fn recv_uci(&mut self, rx: mpsc::Receiver<Cmd>, tx: mpsc::Sender<uci::Cmd>) {
        match rx.recv() {
            Ok(Cmd::Ping(s)) => tx.send(uci::Cmd::Engine(Cmd::Pong(s))).unwrap(),
            Ok(c) => eprintln!("Unhandled command: {:?}", c),
            Err(e) => eprintln!("Engine recv failure: {}", e),
        }
    }
}

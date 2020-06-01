//! UCI management.

use std::fs;
use std::io::{self, Write};

use crate::engine;
use crate::notation;

const VATU_NAME: &str = env!("CARGO_PKG_NAME");
const VATU_AUTHORS: &str = env!("CARGO_PKG_AUTHORS");

/// Hold some values related to UCI comms.
pub struct Uci {
    state: State,
    engine: engine::Engine,
    logfile: Option<fs::File>,
}

/// Internal UCI state.
#[derive(PartialEq)]
pub enum State {
    Init,
    Ready,
}

/// UCI remote commands, received by engine.
#[derive(Debug)]
pub enum RemoteCmd {
    Uci,
    IsReady,
    UciNewGame,
    Stop,
    Position(PositionArgs),
    Quit,
    Unknown(String),
}

/// Arguments for the position remote command.
#[derive(Debug)]
pub enum PositionArgs {
    Startpos,
    Fen(notation::Fen),
}

impl Uci {
    fn listen(&mut self) {
        loop {
            if let Some(cmd) = self.receive() {
                match parse_command(&cmd) {
                    Some(cmd) => {
                        if !self.handle_command(&cmd) {
                            break
                        }
                    }
                    None => {
                        self.log(format!("Unknown command: {}", cmd))
                    }
                }
            }
        }
    }

    fn log(&mut self, s: String) {
        match self.logfile.as_ref()  {
            Some(mut f) => {
                f.write_all(s.as_bytes()).ok();
                f.write_all("\n".as_bytes()).ok();
                f.flush().ok();
            }
            None => {
                eprintln!("{}", s);
            }
        }
    }

    /// Read a command from the interface.
    fn receive(&mut self) -> Option<String> {
        let mut s = String::new();
        match io::stdin().read_line(&mut s) {
            Ok(_) => { self.log(format!(">>> {}", s.trim_end())); Some(s.trim().to_string()) }
            Err(e) => { self.log(format!("Failed to read input: {:?}", e)); None }
        }
    }

    /// Send replies to the interface.
    fn send(&mut self, s: &str) {
        self.log(format!("<<< {}", s));
        println!("{}", s);
    }

    /// Handle a remote command, return false if engine should stop listening.
    fn handle_command(&mut self, cmd: &RemoteCmd) -> bool {
        match cmd {
            RemoteCmd::Uci => if self.state == State::Init { self.identify(); },
            RemoteCmd::IsReady => if self.state == State::Ready { self.ready() },
            RemoteCmd::UciNewGame => if self.state == State::Ready { /* Nothing to do. */ },
            RemoteCmd::Stop => if self.state == State::Ready { /* Nothing to do. */ },
            RemoteCmd::Position(p) => if self.state == State::Ready { self.position(p) }
            RemoteCmd::Quit => return false,
            _ => { self.log(format!("Unknown command: {:?}", cmd)); }
        }
        true
    }

    /// Send IDs to interface.
    fn identify(&mut self) {
        self.send(&format!("id name {}", VATU_NAME));
        self.send(&format!("id author {}", VATU_AUTHORS));
        self.send("uciok");
        self.state = State::Ready;
    }

    /// Notify interface that it is ready.
    fn ready(&mut self) {
        self.send("readyok");
        self.state = State::Ready;
    }

    fn position(&mut self, p_args: &PositionArgs) {
        match p_args {
            PositionArgs::Fen(fen) => {
                self.engine.apply_fen(fen);
            },
            PositionArgs::Startpos => {
                let fen = notation::parse_fen(notation::FEN_START).unwrap();
                self.engine.apply_fen(&fen);
            }
        };
    }
}

/// Create a new Uci object, ready for I/O.
pub fn start(output: Option<&str>) {
    let mut uci = Uci {
        state: State::Init,
        engine: engine::Engine::new(),
        logfile: None
    };
    if let Some(output) = output {
        match fs::File::create(output) {
            Ok(f) => { uci.logfile = Some(f) }
            Err(e) => { eprintln!("Could not open log file: {}", e) }
        }
    }
    uci.listen();
}

fn parse_command(s: &str) -> Option<RemoteCmd> {
    let fields: Vec<&str> = s.split_whitespace().collect();
    match fields[0] {
        "uci" => Some(RemoteCmd::Uci),
        "isready" => Some(RemoteCmd::IsReady),
        "ucinewgame" => Some(RemoteCmd::UciNewGame),
        "stop" => Some(RemoteCmd::Stop),
        "position" => {
            match fields[1] {
                // Subcommand "fen" is followed by a FEN string.
                "fen" => {
                    if let Some(fen) = notation::parse_fen_fields(fields[2..8].to_vec()) {
                        Some(RemoteCmd::Position(PositionArgs::Fen(fen)))
                    } else {
                        None
                    }
                }
                // Subcommand "startpos" assumes the board is a new game.
                "startpos" => Some(RemoteCmd::Position(PositionArgs::Startpos)),
                _ => None
            }
        }
        "quit" => Some(RemoteCmd::Quit),
        c => Some(RemoteCmd::Unknown(c.to_string())),
    }
}

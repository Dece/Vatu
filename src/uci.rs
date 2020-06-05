//! UCI management.

use std::fs;
use std::io::{self, Write};
use std::sync::mpsc;
use std::thread;

use crate::engine;
use crate::notation;
use crate::rules;

const VATU_NAME: &str = env!("CARGO_PKG_NAME");
const VATU_AUTHORS: &str = env!("CARGO_PKG_AUTHORS");

// ************************************
// UCI manager

/// UCI manager with means to send/receive commands and communicate
/// with the engine.
pub struct Uci {
    state: State,                                           // Local UCI state for consistency.
    cmd_channel: (mpsc::Sender<Cmd>, mpsc::Receiver<Cmd>),  // Channel of Cmd, handled by Uci.
    engine_in: Option<mpsc::Sender<engine::Cmd>>,           // Sender for engine comms.
    logfile: Option<fs::File>,                              // If some, write logs to it.
}

/// Internal UCI state.
#[derive(PartialEq)]
pub enum State {
    Init,
    Ready,
    Working,
}

/// Uci MPSC commands.
#[derive(Debug)]
pub enum Cmd {
    Stdin(String),        // String received from standard input.
    Engine(engine::Cmd),  // Engine responses.
}

/// UCI commands.
#[derive(Debug)]
pub enum UciCmd {
    Uci,
    IsReady,
    UciNewGame,
    Stop,
    Position(Vec<PositionArgs>),
    Go(Vec<GoArgs>),
    Quit,
    Unknown(String),
}

/// Arguments for the position remote command.
#[derive(Debug, Clone)]
pub enum PositionArgs {
    Startpos,
    Fen(notation::Fen),
    Moves(Vec<rules::Move>),
}

/// Arguments for the go remote commands.
#[derive(Debug, Clone)]
pub enum GoArgs {
    SearchMoves(Vec<rules::Move>),
    Ponder,
    WTime(i32),
    BTime(i32),
    WInc(i32),
    BInc(i32),
    MovesToGo(i32),
    Depth(i32),
    Nodes(i32),
    Mate(i32),
    MoveTime(i32),
    Infinite,
}

impl Uci {
    /// Start a new UCI listening for standard input.
    pub fn start(output: Option<&str>) {
        // Create the UCI queue, both for standard IO and for engine communication.
        let (uci_s, uci_r): (mpsc::Sender<Cmd>, mpsc::Receiver<Cmd>) = mpsc::channel();
        let stdin_tx = uci_s.clone();
        thread::spawn(move || {
            Uci::read_stdin(stdin_tx);
        });

        let mut uci = Uci {
            state: State::Init,
            cmd_channel: (uci_s, uci_r),
            engine_in: None,
            logfile: None,
        };
        // Configure log output, either a file or stderr.
        if let Some(output) = output {
            match fs::File::create(output) {
                Ok(f) => { uci.logfile = Some(f) }
                Err(e) => { eprintln!("Could not open log file: {}", e) }
            }
        }

        // Start listening for Cmds.
        uci.listen();
    }

    fn listen(&mut self) {
        loop {
            match self.cmd_channel.1.recv() {
                Ok(Cmd::Stdin(cmd)) => {
                    self.log(format!("UCI >>> {}", cmd));
                    if !self.handle_command(&parse_command(&cmd)) {
                        break
                    }
                }
                Ok(Cmd::Engine(cmd)) => {
                    self.handle_engine_command(&cmd);
                }
                Err(e) => self.log(format!("Can't read commands: {}", e))
            }
        }
    }

    fn log(&mut self, s: String) {
        match &mut self.logfile {
            Some(f) => {
                writeln!(f, "{}", s).unwrap();
            }
            None => {
                eprintln!("{}", s);
            }
        }
    }

    /// Read lines over stdin, notifying over an MPSC channel.
    ///
    /// As it is not trivial to add a timeout, or overly complicated
    /// to break the loop with a second channel, simply stop listening
    /// when the UCI "quit" command is received.
    ///
    /// This is not an Uci method as it does not need to act on the
    /// instance itself.
    pub fn read_stdin(tx: mpsc::Sender<Cmd>) {
        let mut s = String::new();
        loop {
            s.clear();
            match io::stdin().read_line(&mut s) {
                Ok(_) => {
                    let s = s.trim();
                    tx.send(Cmd::Stdin(s.to_string())).unwrap();
                    if s == "quit" {
                        break;
                    }
                }
                Err(e) => {
                    eprintln!("Failed to read input: {:?}", e);
                }
            }
        }
    }

    /// Send an UCI reply.
    fn send(&mut self, s: &str) {
        self.log(format!("UCI <<< {}", s));
        println!("{}", s);
    }

    /// Handle an UCI command, return false if it should stop listening.
    fn handle_command(&mut self, cmd: &UciCmd) -> bool {
        match cmd {
            UciCmd::Uci => if self.state == State::Init {
                self.send_identities();
                self.setup_engine();
            },
            UciCmd::IsReady => if self.state == State::Ready { self.send_ready() },
            UciCmd::UciNewGame => if self.state == State::Ready { /* Nothing to do. */ },
            UciCmd::Position(args) => if self.state == State::Ready {
                let args = engine::Cmd::UciPosition(args.to_vec());
                self.engine_in.as_ref().unwrap().send(args).unwrap();
            },
            UciCmd::Go(args) => if self.state == State::Ready {
                let args = engine::Cmd::UciGo(args.to_vec());
                self.engine_in.as_ref().unwrap().send(args).unwrap();
                self.state = State::Working;
            }
            UciCmd::Stop => if self.state == State::Working {
                self.engine_in.as_ref().unwrap().send(engine::Cmd::Stop).unwrap();
            },
            UciCmd::Quit => return false,
            UciCmd::Unknown(c) => { self.log(format!("Unknown command: {}", c)); }
        }
        true
    }

    /// Handle an engine command.
    fn handle_engine_command(&mut self, cmd: &engine::Cmd) {
        match cmd {
            engine::Cmd::UciChannel(s) => {
                self.log("ENG >>> Channel opened.".to_string());
                self.engine_in = Some(s.to_owned());
            }
            engine::Cmd::Log(s) => {
                self.log(s.to_string());
            }
            engine::Cmd::Info(infos) => {
                self.send_infos(infos);
            }
            engine::Cmd::BestMove(m) => {
                self.state = State::Ready;
                self.send_bestmove(m);
            }
            _ => {}
        }
    }

    /// Send IDs to interface.
    fn send_identities(&mut self) {
        self.send(&format!("id name {}", VATU_NAME));
        self.send(&format!("id author {}", VATU_AUTHORS));
        self.send("uciok");
    }

    fn setup_engine(&mut self) {
        let uci_s = self.cmd_channel.0.clone();
        thread::spawn(move || {
            let mut engine = engine::Engine::new();
            engine.setup_uci(uci_s);
        });
        self.state = State::Ready;
    }

    /// Notify interface that it is ready.
    fn send_ready(&mut self) {
        self.send("readyok");
    }

    /// Send engine analysis information.
    fn send_infos(&mut self, infos: &Vec<engine::Info>) {
        let mut s = "info".to_string();
        for i in infos {
            match i {
                engine::Info::CurrentMove(m) => {
                    s.push_str(&format!(" currmove {}", notation::move_to_string(m)));
                }
            }
        }
        self.send(&s);
    }

    /// Send best move.
    fn send_bestmove(&mut self, m: &Option<rules::Move>) {
        let move_str = match m {
            Some(m) => notation::move_to_string(m),
            None => notation::NULL_MOVE.to_string(),
        };
        self.send(&format!("bestmove {}", move_str));
    }
}

// ************************************
// UCI command parsers

/// Parse an UCI command.
fn parse_command(s: &str) -> UciCmd {
    if s.len() == 0 {
        return UciCmd::Unknown("Empty command.".to_string());
    }
    let fields: Vec<&str> = s.split_whitespace().collect();
    match fields[0] {
        "uci" => UciCmd::Uci,
        "isready" => UciCmd::IsReady,
        "ucinewgame" => UciCmd::UciNewGame,
        "stop" => UciCmd::Stop,
        "position" => parse_position_command(&fields[1..]),
        "go" => parse_go_command(&fields[1..]),
        "quit" => UciCmd::Quit,
        c => UciCmd::Unknown(c.to_string()),
    }
}

/// Parse an UCI "position" command.
fn parse_position_command(fields: &[&str]) -> UciCmd {
    let num_fields = fields.len();
    let mut i = 0;
    let mut subcommands = vec!();
    while i < num_fields {
        match fields[i] {
            // Subcommand "fen" is followed by a FEN string.
            "fen" => {
                if let Some(fen) = notation::parse_fen_fields(&fields[i + 1 .. i + 7]) {
                    subcommands.push(PositionArgs::Fen(fen))
                } else {
                    return UciCmd::Unknown(format!("Bad format for position fen"))
                }
                i += 6;
            }
            // Subcommand "startpos" assumes the board is a new game.
            "startpos" => subcommands.push(PositionArgs::Startpos),
            // Subcommand "moves" is followed by moves until the end of the command.
            "moves" => {
                let mut moves = vec!();
                while i + 1 < num_fields {
                    moves.push(notation::parse_move(fields[i + 1]));
                    i += 1;
                }
                subcommands.push(PositionArgs::Moves(moves));
            },
            f => return UciCmd::Unknown(format!("Unknown position subcommand: {}", f)),
        }
        i += 1;
    }
    UciCmd::Position(subcommands)
}

/// Parse an UCI "go" command.
fn parse_go_command(fields: &[&str]) -> UciCmd {
    let num_fields = fields.len();
    let mut i = 0;
    let mut subcommands = vec!();
    while i < num_fields {
        match fields[i] {
            "infinite" => subcommands.push(GoArgs::Infinite),
            "movetime" => {
                i += 1;
                subcommands.push(GoArgs::MoveTime(fields[i].parse::<i32>().unwrap()));
            }
            "wtime" => {
                i += 1;
                subcommands.push(GoArgs::WTime(fields[i].parse::<i32>().unwrap()));
            },
            "btime" => {
                i += 1;
                subcommands.push(GoArgs::BTime(fields[i].parse::<i32>().unwrap()));
            }
            "winc" => {
                i += 1;
                subcommands.push(GoArgs::WInc(fields[i].parse::<i32>().unwrap()));
            }
            "binc" => {
                i += 1;
                subcommands.push(GoArgs::BInc(fields[i].parse::<i32>().unwrap()));
            }
            "movestogo" => {
                i += 1;
                subcommands.push(GoArgs::MovesToGo(fields[i].parse::<i32>().unwrap()));
            }
            "depth" => {
                i += 1;
                subcommands.push(GoArgs::Depth(fields[i].parse::<i32>().unwrap()));
            }
            "nodes" => {
                i += 1;
                subcommands.push(GoArgs::Nodes(fields[i].parse::<i32>().unwrap()));
            }
            "mate" => {
                i += 1;
                subcommands.push(GoArgs::Mate(fields[i].parse::<i32>().unwrap()));
            }
            f => eprintln!("Unknown go subcommand: {}", f),
        }
        i += 1;
    }
    UciCmd::Go(subcommands)
}

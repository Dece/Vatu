//! UCI management.

use std::fs;
use std::io::{self, Write};

use nom::IResult;
use nom::branch::alt;
use nom::character::is_space;
use nom::bytes::complete::{tag, take_while};

use crate::board;

const VATU_NAME: &str = env!("CARGO_PKG_NAME");
const VATU_AUTHORS: &str = env!("CARGO_PKG_AUTHORS");

/// Hold some values related to UCI comms.
pub struct Uci {
    state: State,
    board: board::Board,
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
    Position(String),
    Quit,
    Unknown(String),
}

impl Uci {
    fn listen(&mut self) {
        loop {
            if let Some(cmd) = self.receive() {
                match parse_command(&cmd) {
                    Ok((_, cmd)) => {
                        if !self.handle_command(&cmd) {
                            break
                        }
                    }
                    _ => {
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

    fn ready(&mut self) {
        self.send("readyok");
        self.state = State::Ready;
    }
}

/// Start UCI I/O.
pub fn start(output: Option<&str>) {
    let mut uci = Uci {
        state: State::Init,
        board: board::new_empty(),
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

fn take_non_space(i: &str) -> IResult<&str, &str> {
    take_while(|c| c != ' ')(i)
}

fn parse_command(i: &str) -> IResult<&str, RemoteCmd> {
    let (i, cmd) = take_non_space(i)?;
    match cmd {
        "uci" => Ok((i, RemoteCmd::Uci)),
        "isready" => Ok((i, RemoteCmd::IsReady)),
        "ucinewgame" => Ok((i, RemoteCmd::UciNewGame)),
        "stop" => Ok((i, RemoteCmd::Stop)),
        "position" => Ok((i, RemoteCmd::Position(i.trim().to_string()))),
        "quit" => Ok((i, RemoteCmd::Quit)),
        c => Ok((i, RemoteCmd::Unknown(c.to_string()))),
    }
}

use clap::{App, Arg};

pub mod analysis;
pub mod board;
pub mod castling;
pub mod engine;
pub mod fen;
pub mod movement;
pub mod node;
pub mod precomputed;
pub mod rules;
pub mod stats;
pub mod uci;
pub mod zobrist;

fn main() {
    let args = App::new("Vatu")
        .arg(Arg::with_name("debug")
            .help("Enable debug mode")
            .short("d").long("debug").takes_value(false).required(false))
        .arg(Arg::with_name("log_file")
            .help("Log file path (default is stderr)")
            .long("log-file").takes_value(true).required(false))
        .get_matches();

    let debug = args.is_present("debug");
    let output = args.value_of("log_file");
    uci::Uci::start(debug, output);
}

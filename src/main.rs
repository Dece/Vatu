use std::process;

use clap::{App, AppSettings, Arg, ArgMatches, SubCommand};

pub mod board;
pub mod engine;
pub mod notation;
pub mod rules;
pub mod stats;
pub mod uci;

fn main() {
    let matches = App::new("Vatu")
        .setting(AppSettings::ArgRequiredElseHelp)
        .subcommand(SubCommand::with_name("uci")
            .about("Start engine in UCI mode")
            .arg(Arg::with_name("log_file")
                .help("Log file path (default is stderr)")
                .long("log-file").takes_value(true).required(false)))
        .get_matches();

    process::exit(match matches.subcommand() {
        ("uci", Some(a)) => cmd_uci(a),
        _ => 0,
    })
}

fn cmd_uci(args: &ArgMatches) -> i32 {
    let output = args.value_of("log_file");
    uci::Uci::start(output);
    0
}

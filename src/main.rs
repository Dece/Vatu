use std::process;

use clap::{App, AppSettings, Arg, ArgMatches, SubCommand};

pub mod board;
pub mod cli;
pub mod rules;

fn main() {
    let matches = App::new("Vatu")
        .setting(AppSettings::ArgRequiredElseHelp)
        .subcommand(SubCommand::with_name("cli")
            .about("Start a game in command-line")
            .arg(Arg::with_name("color")
                .help("Color for the player")
                .short("c").long("color").takes_value(true).required(false)
                .possible_values(&["w", "white", "b", "black"])))
        .get_matches();

    process::exit(match matches.subcommand() {
        ("cli", Some(a)) => cmd_cli(a),
        _ => 0,
    })
}

fn cmd_cli(args: &ArgMatches) -> i32 {
    let color = if let Some(c) = args.value_of("color") {
        match c {
            s if ["w", "white"].contains(&s) => board::SQ_WH,
            s if ["b", "black"].contains(&s) => board::SQ_BL,
            _ => { eprintln!("Choose white or black as color."); return 1 }
        }
    } else if rand::random() {
        board::SQ_WH
    } else {
        board::SQ_BL
    };

    cli::start_game(color);
    0
}

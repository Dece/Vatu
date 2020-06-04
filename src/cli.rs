//! Functions to play games over the command-line.

use std::io::{self, Write};

use rand::seq::IteratorRandom;

use crate::board;
use crate::rules;

pub fn start_game(player_color: u8) {
    println!("Starting new game.");
    println!("Player is {}.", if player_color == board::SQ_WH { "white" } else { "black" });
    println!("");
    let ai_color = board::opposite(player_color);
    let mut rng = rand::thread_rng();
    let mut b = board::new();
    let mut turn = board::SQ_WH;
    loop {
        board::draw(&b, &mut io::stdout());
        println!("");
        let m = if turn == player_color {
            println!("Player turn.");
            let legal_moves = rules::get_player_legal_moves(&b, player_color);
            let mut m;
            loop {
                m = get_player_move();
                if legal_moves.contains(&m) {
                    break
                } else {
                    println!("Illegal move.");
                }
            }
            m
        } else {
            println!("Computer turn.");
            let moves = rules::get_player_legal_moves(&b, ai_color);
            *moves.iter().choose(&mut rng).unwrap()
        };
        println!("Move: {:?}", m);
        board::apply_into(&mut b, &m);
        println!("");
        turn = board::opposite(turn);
    }
}

fn get_player_move() -> board::Move {
    loop {
        let from = if let Some(s) = get_input("From: ") { board::pos(&s) } else { continue };
        let to = if let Some(s) = get_input("To: ") { board::pos(&s) } else { continue };
        if board::is_valid_pos(from) && board::is_valid_pos(to) {
            return (from, to, None)  // TODO this does not handle promotion.
        }
        println!("Bad input.");
    }

}

fn get_input(message: &str) -> Option<String> {
    print!("{}", message);
    io::stdout().flush().unwrap();
    let mut input = String::new();
    if io::stdin().read_line(&mut input).is_err() {
        return None
    }
    Some(input)
}

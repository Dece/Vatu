//! Functions using various notations.

use crate::board;

pub const NULL_MOVE: &str = "0000";

/// Create a string containing the UCI algebraic notation of this move.
pub fn move_to_string(m: &board::Move) -> String {
    let mut move_string = String::new();
    move_string.push_str(&board::pos_string(&m.0));
    move_string.push_str(&board::pos_string(&m.1));
    if let Some(prom) = m.2 {
        move_string.push(match prom {
            board::SQ_Q => 'q',
            board::SQ_B => 'b',
            board::SQ_N => 'n',
            board::SQ_R => 'r',
            _ => panic!("What are you doing? Promote to a legal piece.")
        });
    }
    move_string
}

/// Parse an UCI move algebraic notation string to a board::Move.
pub fn parse_move(m_str: &str) -> board::Move {
    let prom = if m_str.len() == 5 {
        Some(match m_str.as_bytes()[4] {
            b'b' => board::SQ_B,
            b'n' => board::SQ_N,
            b'r' => board::SQ_R,
            b'q' => board::SQ_Q,
            _ => panic!("What is the opponent doing? This is illegal, I'm out."),
        })
    } else {
        None
    };
    (board::pos(&m_str[0..2]), board::pos(&m_str[2..4]), prom)
}

/// Create a space-separated string of moves. Used for debugging.
pub fn move_list_to_string(moves: &Vec<board::Move>) -> String {
    moves.iter().map(|m| move_to_string(m)).collect::<Vec<_>>().join(" ")
}

pub const FEN_START: &str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";

/// FEN notation for positions, split into fields.
#[derive(Debug, Clone)]
pub struct Fen {
    pub placement: String,
    pub color: String,
    pub castling: String,
    pub en_passant: String,
    pub halfmove: String,
    pub fullmove: String,
}

pub fn parse_fen(i: &str) -> Option<Fen> {
    let fields: Vec<&str> = i.split_whitespace().collect();
    parse_fen_fields(&fields)
}

pub fn parse_fen_fields(fields: &[&str]) -> Option<Fen> {
    if fields.len() < 6 {
        return None
    }
    Some(Fen {
        placement: fields[0].to_string(),
        color: fields[1].to_string(),
        castling: fields[2].to_string(),
        en_passant: fields[3].to_string(),
        halfmove: fields[4].to_string(),
        fullmove: fields[5].to_string(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_move_to_string() {
        assert_eq!(move_to_string(&((0, 0), (3, 3), None)), "a1d4");
        assert_eq!(move_to_string(&((7, 7), (0, 7), None)), "h8a8");
        assert_eq!(move_to_string(&((7, 6), (7, 7), Some(board::SQ_Q))), "h7h8q");
        assert_eq!(move_to_string(&((7, 6), (7, 7), Some(board::SQ_N))), "h7h8n");
    }

    #[test]
    fn test_parse_move() {
        assert_eq!(parse_move("a1d4"), ((0, 0), (3, 3), None));
        assert_eq!(parse_move("a7a8q"), ((0, 6), (0, 7), Some(board::SQ_Q)));
        assert_eq!(parse_move("a7a8r"), ((0, 6), (0, 7), Some(board::SQ_R)));
    }

    #[test]
    fn test_parse_fen() {
        let fen_start = parse_fen(FEN_START).unwrap();
        assert_eq!(&fen_start.placement, "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR");
        assert_eq!(&fen_start.color, "w");
        assert_eq!(&fen_start.castling, "KQkq");
        assert_eq!(&fen_start.en_passant, "-");
        assert_eq!(&fen_start.halfmove, "0");
        assert_eq!(&fen_start.fullmove, "1");
    }
}

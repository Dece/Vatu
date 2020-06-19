//! Functions to parse FEN strings.

use crate::board;

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

pub fn en_passant_to_string(ep: Option<board::Square>) -> String {
    ep.and_then(|p| Some(board::sq_to_string(p))).unwrap_or("-".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

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

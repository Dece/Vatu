//! Functions using various notations.

use nom::IResult;

/// FEN notation for positions, split into fields.
pub struct Fen {
    placement: String,
    color: String,
    castling: String,
    en_passant: String,
    halfmove: String,
    fullmove: String,
}

fn parse_fen(i: &str) -> IResult<&str, Fen> {

}

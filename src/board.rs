//! Basic type definitions and functions.

// Piece type flags.
pub const SQ_E: u8         = 0;
pub const SQ_P: u8         = 0b00000001;
pub const SQ_B: u8         = 0b00000010;
pub const SQ_N: u8         = 0b00000100;
pub const SQ_R: u8         = 0b00001000;
pub const SQ_Q: u8         = 0b00010000;
pub const SQ_K: u8         = 0b00100000;
pub const SQ_TYPE_MASK: u8 = 0b00111111;

// Piece color flags.
pub const SQ_WH: u8         = 0b01000000;
pub const SQ_BL: u8         = 0b10000000;
pub const SQ_COLOR_MASK: u8 = 0b11000000;

// Piece flags helpers.
pub const SQ_WH_P: u8 = SQ_WH|SQ_P;
pub const SQ_WH_B: u8 = SQ_WH|SQ_B;
pub const SQ_WH_N: u8 = SQ_WH|SQ_N;
pub const SQ_WH_R: u8 = SQ_WH|SQ_R;
pub const SQ_WH_Q: u8 = SQ_WH|SQ_Q;
pub const SQ_WH_K: u8 = SQ_WH|SQ_K;
pub const SQ_BL_P: u8 = SQ_BL|SQ_P;
pub const SQ_BL_B: u8 = SQ_BL|SQ_B;
pub const SQ_BL_N: u8 = SQ_BL|SQ_N;
pub const SQ_BL_R: u8 = SQ_BL|SQ_R;
pub const SQ_BL_Q: u8 = SQ_BL|SQ_Q;
pub const SQ_BL_K: u8 = SQ_BL|SQ_K;

#[inline]
pub const fn has_flag(i: u8, flag: u8) -> bool { i & flag == flag }

// Wrappers for clearer naming.
/// Get type of piece on square, without color.
#[inline]
pub const fn get_type(square: u8) -> u8 { square & SQ_TYPE_MASK }
/// Return true if the piece on this square is of type `piece_type`.
#[inline]
pub const fn is_type(square: u8, piece_type: u8) -> bool { get_type(square) == piece_type }
/// Return true if the piece on this square has this color.
#[inline]
pub const fn is_color(square: u8, color: u8) -> bool { has_flag(square, color) }
/// Return true if this square has a white piece.
#[inline]
pub const fn is_white(square: u8) -> bool { is_color(square, SQ_WH) }
/// Return true if this square has a black piece.
#[inline]
pub const fn is_black(square: u8) -> bool { is_color(square, SQ_BL) }
/// Return the color of the piece on this square.
#[inline]
pub const fn get_color(square: u8) -> u8 { square & SQ_COLOR_MASK }
/// Return true if the piece on this square is the same as `piece`.
#[inline]
pub const fn is_piece(square: u8, piece: u8) -> bool { has_flag(square, piece) }

/// Get opposite color.
#[inline]
pub const fn opposite(color: u8) -> u8 { color ^ SQ_COLOR_MASK }

/// Pretty-print a color.
pub fn color_to_string(color: u8) -> String {
    match color {
        SQ_WH => "white".to_string(),
        SQ_BL => "black".to_string(),
        _ => panic!("Unknown color {}", color),
    }
}

/// Minimum allowed value for stored Pos components.
pub const POS_MIN: i8 = 0;
/// Maximum allowed value for stored Pos components.
pub const POS_MAX: i8 = 7;
/// Coords (file, rank) of a square on a board, both components are in [0, 7].
pub type Pos = (i8, i8);

/// Check if a Pos component is in the [0, 7] range.
#[inline]
pub fn is_valid_pos_c(component: i8) -> bool { component >= 0 && component <= 7 }

/// Check if both `pos` components are valid.
#[inline]
pub fn is_valid_pos(pos: Pos) -> bool { is_valid_pos_c(pos.0) && is_valid_pos_c(pos.1) }

/// Convert string coordinates to Pos.
///
/// `s` has to be valid UTF8, or the very least ASCII because chars
/// are interpreted as raw bytes, and lowercase.
#[inline]
pub const fn pos(s: &str) -> Pos {
    let chars = s.as_bytes();
    ((chars[0] - 0x61) as i8, (chars[1] - 0x31) as i8)
}

/// Return string coordinates from Pos.
pub fn pos_string(p: &Pos) -> String {
    let mut bytes = [0u8; 2];
    bytes[0] = (p.0 + 0x61) as u8;
    bytes[1] = (p.1 + 0x31) as u8;
    String::from_utf8_lossy(&bytes).to_string()
}

/// Bitboard representation of a chess board.
///
/// 64 squares, from A1, A2 to H7, H8. A square is an u8, with bits
/// defining the state of the square.
pub type Board = [u8; 64];

/// Generate the board of a new game.
pub const fn new() -> Board {
    [
        /*            1        2     3     4     5     6        7        8 */
        /* A */ SQ_WH_R, SQ_WH_P, SQ_E, SQ_E, SQ_E, SQ_E, SQ_BL_P, SQ_BL_R,
        /* B */ SQ_WH_N, SQ_WH_P, SQ_E, SQ_E, SQ_E, SQ_E, SQ_BL_P, SQ_BL_N,
        /* C */ SQ_WH_B, SQ_WH_P, SQ_E, SQ_E, SQ_E, SQ_E, SQ_BL_P, SQ_BL_B,
        /* D */ SQ_WH_Q, SQ_WH_P, SQ_E, SQ_E, SQ_E, SQ_E, SQ_BL_P, SQ_BL_Q,
        /* E */ SQ_WH_K, SQ_WH_P, SQ_E, SQ_E, SQ_E, SQ_E, SQ_BL_P, SQ_BL_K,
        /* F */ SQ_WH_B, SQ_WH_P, SQ_E, SQ_E, SQ_E, SQ_E, SQ_BL_P, SQ_BL_B,
        /* G */ SQ_WH_N, SQ_WH_P, SQ_E, SQ_E, SQ_E, SQ_E, SQ_BL_P, SQ_BL_N,
        /* H */ SQ_WH_R, SQ_WH_P, SQ_E, SQ_E, SQ_E, SQ_E, SQ_BL_P, SQ_BL_R,
    ]
}

/// Generate an empty board.
pub const fn new_empty() -> Board {
    [SQ_E; 64]
}

/// Generate a board from a FEN placement string.
pub fn new_from_fen(fen: &str) -> Board {
    let mut board = [SQ_E; 64];
    let mut f = 0;
    let mut r = 7;
    for c in fen.chars() {
        match c {
            'r' => { set_square(&mut board, &(f, r), SQ_BL_R); f += 1 }
            'n' => { set_square(&mut board, &(f, r), SQ_BL_N); f += 1 }
            'b' => { set_square(&mut board, &(f, r), SQ_BL_B); f += 1 }
            'q' => { set_square(&mut board, &(f, r), SQ_BL_Q); f += 1 }
            'k' => { set_square(&mut board, &(f, r), SQ_BL_K); f += 1 }
            'p' => { set_square(&mut board, &(f, r), SQ_BL_P); f += 1 }
            'R' => { set_square(&mut board, &(f, r), SQ_WH_R); f += 1 }
            'N' => { set_square(&mut board, &(f, r), SQ_WH_N); f += 1 }
            'B' => { set_square(&mut board, &(f, r), SQ_WH_B); f += 1 }
            'Q' => { set_square(&mut board, &(f, r), SQ_WH_Q); f += 1 }
            'K' => { set_square(&mut board, &(f, r), SQ_WH_K); f += 1 }
            'P' => { set_square(&mut board, &(f, r), SQ_WH_P); f += 1 }
            '/' => { f = 0; r -= 1; }
            d if d.is_digit(10) => { f += d.to_digit(10).unwrap() as i8 }
            _ => break,
        }
    }
    board
}

/// Return true of both boards are equal.
pub fn eq(b1: &Board, b2: &Board) -> bool {
    b1.iter().zip(b2.iter()).all(|(a, b)| a == b)
}

/// Get value of the square at this position.
#[inline]
pub const fn get_square(board: &Board, coords: &Pos) -> u8 {
    board[(coords.0 * 8 + coords.1) as usize]
}

/// Set a new value for the square at this position.
#[inline]
pub fn set_square(board: &mut Board, coords: &Pos, piece: u8) {
    board[(coords.0 * 8 + coords.1) as usize] = piece;
}

/// Set the square empty at this position.
#[inline]
pub fn clear_square(board: &mut Board, coords: &Pos) {
    set_square(board, coords, SQ_E);
}

/// Move a piece from a position to another, clearing initial square.
#[inline]
pub fn move_piece(board: &mut Board, from: &Pos, to: &Pos) {
    set_square(board, &to, get_square(board, &from));
    clear_square(board, &from);
}

/// Return true of the square at this position is empty.
#[inline]
pub const fn is_empty(board: &Board, coords: &Pos) -> bool {
    get_square(board, coords) == SQ_E
}

/// Return an iterator over the pieces of the board along with pos.
pub fn get_piece_iterator<'a>(board: &'a Board) -> Box<dyn Iterator<Item = (u8, Pos)> + 'a> {
    Box::new(
        board.iter().enumerate()
            .filter(|(_, s)| **s != SQ_E)
            .map(|(i, s)| (*s, ((i / 8) as i8, (i % 8) as i8)))
    )
}

/// Find the king of `color`.
pub fn find_king(board: &Board, color: u8) -> Option<Pos> {
    for f in 0..8 {
        for r in 0..8 {
            let s = get_square(board, &(f, r));
            if is_color(s, color) && is_piece(s, SQ_K) {
                return Some((f, r))
            }
        }
    }
    None
}

/// Count number of pieces on board. Used for debugging.
pub fn num_pieces(board: &Board) -> u8 {
    let mut count = 0;
    for i in board.iter() {
        if *i != SQ_E {
            count += 1;
        }
    }
    count
}

/// Write a text view of the board. Used for debugging.
pub fn draw(board: &Board, f: &mut dyn std::io::Write) {
    for r in (0..8).rev() {
        let mut rank = String::with_capacity(8);
        for f in 0..8 {
            let s = get_square(board, &(f, r));
            let piece =
                if is_piece(s, SQ_P) { 'p' }
                else if is_piece(s, SQ_B) { 'b' }
                else if is_piece(s, SQ_N) { 'n' }
                else if is_piece(s, SQ_R) { 'r' }
                else if is_piece(s, SQ_Q) { 'q' }
                else if is_piece(s, SQ_K) { 'k' }
                else { '.' };
            let piece = if is_color(s, SQ_WH) { piece.to_ascii_uppercase() } else { piece };
            rank.push(piece);
        }
        writeln!(f, "{} {}", r + 1, rank).unwrap();
    }
    write!(f, "  abcdefgh").unwrap();
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::notation;

    #[test]
    fn test_opposite() {
        assert_eq!(opposite(SQ_WH), SQ_BL);
        assert_eq!(opposite(SQ_BL), SQ_WH);
    }

    #[test]
    fn test_pos() {
        assert_eq!(pos("a1"), (0, 0));
        assert_eq!(pos("a2"), (0, 1));
        assert_eq!(pos("a8"), (0, 7));
        assert_eq!(pos("b1"), (1, 0));
        assert_eq!(pos("h8"), (7, 7));
    }

    #[test]
    fn test_pos_string() {
        assert_eq!(pos_string(&(0, 0)), "a1");
        assert_eq!(pos_string(&(0, 1)), "a2");
        assert_eq!(pos_string(&(0, 7)), "a8");
        assert_eq!(pos_string(&(7, 7)), "h8");
    }

    #[test]
    fn test_new_from_fen() {
        let b1 = new();
        let b2 = new_from_fen(notation::FEN_START);
        assert!(eq(&b1, &b2));
    }

    #[test]
    fn test_eq() {
        let mut b1 = new();
        let b2 = new();
        assert!(eq(&b1, &b2));
        set_square(&mut b1, &pos("a1"), SQ_E);
        assert!(!eq(&b1, &b2));
        set_square(&mut b1, &pos("a1"), SQ_WH_R);
        assert!(eq(&b1, &b2));
    }

    #[test]
    fn test_get_square() {
        let b = new();
        assert_eq!(get_square(&b, &pos("a1")), SQ_WH_R);
        assert_eq!(get_square(&b, &pos("a2")), SQ_WH_P);
        assert_eq!(get_square(&b, &pos("a3")), SQ_E);

        assert_eq!(get_square(&b, &pos("a7")), SQ_BL_P);
        assert_eq!(get_square(&b, &pos("a8")), SQ_BL_R);

        assert_eq!(get_square(&b, &pos("d1")), SQ_WH_Q);
        assert_eq!(get_square(&b, &pos("d8")), SQ_BL_Q);
        assert_eq!(get_square(&b, &pos("e1")), SQ_WH_K);
        assert_eq!(get_square(&b, &pos("e8")), SQ_BL_K);
    }

    #[test]
    fn test_is_empty() {
        let b = new();
        assert_eq!(is_empty(&b, &pos("a1")), false);
        assert_eq!(is_empty(&b, &pos("a2")), false);
        assert_eq!(is_empty(&b, &pos("a3")), true);
    }

    #[test]
    fn test_find_king() {
        let b = new_empty();
        assert_eq!(find_king(&b, SQ_WH), None);
        let b = new();
        assert_eq!(find_king(&b, SQ_WH), Some(pos("e1")));
        assert_eq!(find_king(&b, SQ_BL), Some(pos("e8")));
    }

    #[test]
    fn test_num_pieces() {
        assert_eq!(num_pieces(&new_empty()), 0);
        assert_eq!(num_pieces(&new()), 32);
    }
}

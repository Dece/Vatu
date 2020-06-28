//! Basic type definitions and functions.

pub use crate::precomputed::*;

/// Color type, used to index `Board.color`.
pub type Color = usize;

pub const WHITE: usize = 0;
pub const BLACK: usize = 1;
pub const NUM_COLORS: usize = 2;

/// Get opposite color.
#[inline]
pub const fn opposite(color: Color) -> Color { color ^ 1 }

/// Pretty-print a color.
pub fn color_to_string(color: Color) -> String {
    match color {
        0 => "white".to_string(),
        1 => "black".to_string(),
        _ => panic!("Unknown color {}", color),
    }
}

/// Piece type, used to index `Board.piece`.
pub type Piece = usize;

pub const PAWN: usize = 0;
pub const BISHOP: usize = 1;
pub const KNIGHT: usize = 2;
pub const ROOK: usize = 3;
pub const QUEEN: usize = 4;
pub const KING: usize = 5;
pub const NUM_PIECES: usize = 6;

/// Coords (file, rank) of a square on a board.
pub type Square = i8;

/// Get square from file and rank, both starting from 0.
#[inline]
pub const fn sq(file: i8, rank: i8) -> Square { file * 8 + rank }

/// Get file from square.
#[inline]
pub const fn sq_file(square: Square) -> i8 { square / 8 }

/// Get rank from square.
#[inline]
pub const fn sq_rank(square: Square) -> i8 { square % 8 }

/// Get bit mask of `p` in a bitboard.
#[inline]
pub const fn bit_pos(square: Square) -> u64 { 1 << square }

/// Convert string coordinates to Square.
///
/// `s` has to be valid UTF8, or the very least ASCII because chars
/// are interpreted as raw bytes, and lowercase.
#[inline]
pub const fn sq_from_string(square: &str) -> Square {
    let chars = square.as_bytes();
    (chars[0] - 0x61) as i8 * 8 + (chars[1] - 0x31) as i8
}

/// Return string coordinates from Square.
pub fn sq_to_string(square: Square) -> String {
    let mut bytes = [0u8; 2];
    bytes[0] = ((square / 8) + 0x61) as u8;
    bytes[1] = ((square % 8) + 0x31) as u8;
    String::from_utf8_lossy(&bytes).to_string()
}

/// Return the forward square for either white or black.
pub fn forward_square(square: Square, color: Color) -> Square {
    sq(sq_file(square), sq_rank(square) + (if color == WHITE { 1 } else { -1 }))
}

/// Bitboard for color or piece bits.
pub type Bitboard = u64;

pub const FILE_A: i8 = 0;
pub const FILE_B: i8 = 1;
pub const FILE_C: i8 = 2;
pub const FILE_D: i8 = 3;
pub const FILE_E: i8 = 4;
pub const FILE_F: i8 = 5;
pub const FILE_G: i8 = 6;
pub const FILE_H: i8 = 7;
pub const NUM_FILES: usize = 8;

pub const RANK_1: i8 = 0;
pub const RANK_2: i8 = 1;
pub const RANK_3: i8 = 2;
pub const RANK_4: i8 = 3;
pub const RANK_5: i8 = 4;
pub const RANK_6: i8 = 5;
pub const RANK_7: i8 = 6;
pub const RANK_8: i8 = 7;
pub const NUM_RANKS: usize = 8;

pub const FILES: [Bitboard; 8] = [
    0b00000000_00000000_00000000_00000000_00000000_00000000_00000000_11111111,
    0b00000000_00000000_00000000_00000000_00000000_00000000_11111111_00000000,
    0b00000000_00000000_00000000_00000000_00000000_11111111_00000000_00000000,
    0b00000000_00000000_00000000_00000000_11111111_00000000_00000000_00000000,
    0b00000000_00000000_00000000_11111111_00000000_00000000_00000000_00000000,
    0b00000000_00000000_11111111_00000000_00000000_00000000_00000000_00000000,
    0b00000000_11111111_00000000_00000000_00000000_00000000_00000000_00000000,
    0b11111111_00000000_00000000_00000000_00000000_00000000_00000000_00000000,
];

/// Get the bitboard of bits before the square ("left-most" bits).
#[inline]
const fn bits_before(file: i8, rank: i8) -> Bitboard {
    (1 << sq(file, rank)) - 1
}

/// Get the bitboard of bits after the square ("right-most" bits).
#[inline]
const fn bits_after(file: i8, rank: i8) -> Bitboard {
    !bits_before(file, rank) << 1
}

/// Get the bitboard of squares on lower ranks of the file.
#[inline]
pub const fn before_on_file(file: i8, rank: i8) -> Bitboard {
    FILES[file as usize] & bits_before(file, rank)
}

/// Get the bitboard of squares on upper ranks of the file.
#[inline]
pub const fn after_on_file(file: i8, rank: i8) -> Bitboard {
    FILES[file as usize] & bits_after(file, rank)
}

/// Count positive bits of the bitboard.
pub fn count_bits(bitboard: Bitboard) -> u8 {
    let mut bitboard = bitboard;
    let mut count = 0;
    while bitboard > 0 {
        count += bitboard & 1;
        bitboard >>= 1;
    }
    count as u8
}

/// Debug only: pretty-print a bitboard to stderr.
#[allow(dead_code)]
pub(crate) fn draw_bits(bitboard: Bitboard) {
    for rank in (0..8).rev() {
        let mut rank_str = String::with_capacity(8);
        for file in 0..8 {
            rank_str.push(if bitboard & bit_pos(sq(file, rank)) == 0 { '.' } else { '1' });
        }
        eprintln!("{}", rank_str);
    }
}

/// Board representation with color/piece bitboards.
#[derive(Clone, PartialEq)]
pub struct Board {
    pub colors: [Bitboard; 2],
    pub pieces: [Bitboard; 6],
}

/// A direction to move (file and rank).
pub type Direction = (i8, i8);
/// Direction in which bishops moves.
pub const BISHOP_DIRS: [Direction; 4] = [
    (1, 1), (1, -1), (-1, -1), (-1, 1)
];
/// Direction in which rooks moves.
pub const ROOK_DIRS: [Direction; 4] = [
    (1, 0), (0, 1), (-1, 0), (0, -1)
];
/// Direction in which queens moves.
pub const QUEEN_DIRS: [Direction; 8] = [
    (1, 0), (1, 1), (0, 1), (-1, 1), (-1, 0), (-1, -1), (0, -1), (1, -1)
];

// Factories.
impl Board {
    /// Generate the board of a new game.
    pub const fn new() -> Board {
        Board {
            colors: [
                0b00000011_00000011_00000011_00000011_00000011_00000011_00000011_00000011,  // W
                0b11000000_11000000_11000000_11000000_11000000_11000000_11000000_11000000,  // B
            ],
            pieces: [
                0b01000010_01000010_01000010_01000010_01000010_01000010_01000010_01000010,  // P
                0b00000000_00000000_10000001_00000000_00000000_10000001_00000000_00000000,  // B
                0b00000000_10000001_00000000_00000000_00000000_00000000_10000001_00000000,  // N
                0b10000001_00000000_00000000_00000000_00000000_00000000_00000000_10000001,  // R
                0b00000000_00000000_00000000_00000000_10000001_00000000_00000000_00000000,  // Q
                0b00000000_00000000_00000000_10000001_00000000_00000000_00000000_00000000,  // K
            ]
        }
    }

    /// Generate an empty board.
    pub const fn new_empty() -> Board {
        Board {
            colors: [0; 2],
            pieces: [0; 6],
        }
    }

    /// Generate a board from a FEN placement string.
    pub fn new_from_fen(fen: &str) -> Board {
        let mut board = Board::new_empty();
        let mut f = 0;
        let mut r = 7;
        for c in fen.chars() {
            match c {
                'r' => { board.set_square(sq(f, r), BLACK, ROOK); f += 1 }
                'n' => { board.set_square(sq(f, r), BLACK, KNIGHT); f += 1 }
                'b' => { board.set_square(sq(f, r), BLACK, BISHOP); f += 1 }
                'q' => { board.set_square(sq(f, r), BLACK, QUEEN); f += 1 }
                'k' => { board.set_square(sq(f, r), BLACK, KING); f += 1 }
                'p' => { board.set_square(sq(f, r), BLACK, PAWN); f += 1 }
                'R' => { board.set_square(sq(f, r), WHITE, ROOK); f += 1 }
                'N' => { board.set_square(sq(f, r), WHITE, KNIGHT); f += 1 }
                'B' => { board.set_square(sq(f, r), WHITE, BISHOP); f += 1 }
                'Q' => { board.set_square(sq(f, r), WHITE, QUEEN); f += 1 }
                'K' => { board.set_square(sq(f, r), WHITE, KING); f += 1 }
                'P' => { board.set_square(sq(f, r), WHITE, PAWN); f += 1 }
                '/' => { f = 0; r -= 1; }
                d if d.is_digit(10) => { f += d.to_digit(10).unwrap() as i8 }
                _ => break,
            }
        }
        board
    }

    /// Get combined white/black pieces bitboard.
    #[inline]
    pub const fn combined(&self) -> Bitboard {
        self.colors[WHITE] | self.colors[BLACK]
    }

    /// Get the bitboard of a color.
    #[inline]
    pub const fn by_color(&self, color: Color) -> Bitboard {
        self.colors[color]
    }

    /// Get the bitboard of a piece type.
    #[inline]
    pub const fn by_piece(&self, piece: Piece) -> Bitboard {
        self.pieces[piece]
    }

    /// Get the bitboard of a piece type for this color.
    #[inline]
    pub const fn by_color_and_piece(&self, color: Color, piece: Piece) -> Bitboard {
        self.by_color(color) & self.by_piece(piece)
    }

    /// True if this square is empty.
    #[inline]
    pub const fn is_empty(&self, square: Square) -> bool {
        self.combined() & bit_pos(square) == 0
    }

    /// Get color type at position. It must hold a piece!
    pub fn get_color_on(&self, square: Square) -> Color {
        let bp = bit_pos(square);
        if (self.colors[WHITE] & bp) != 0 { WHITE }
        else if (self.colors[BLACK] & bp) != 0 { BLACK }
        else { panic!("Empty square.") }
    }

    /// Get piece type at position. It must hold a piece!
    pub fn get_piece_on(&self, square: Square) -> Piece {
        let bp = bit_pos(square);
        if (self.pieces[PAWN] & bp) != 0 { PAWN }
        else if (self.pieces[BISHOP] & bp) != 0 { BISHOP }
        else if (self.pieces[KNIGHT] & bp) != 0 { KNIGHT }
        else if (self.pieces[ROOK] & bp) != 0 { ROOK }
        else if (self.pieces[QUEEN] & bp) != 0 { QUEEN }
        else if (self.pieces[KING] & bp) != 0 { KING }
        else { panic!("Empty square.") }
    }

    /// Set a new value for the square at this position.
    ///
    /// Clear opposite color bit, but not the previous piece bit.
    #[inline]
    pub fn set_square(&mut self, square: Square, color: Color, piece: Piece) {
        let bp = bit_pos(square);
        self.colors[color] |= bp;
        self.colors[opposite(color)] &= !bp;
        self.pieces[piece] |= bp;
    }

    /// Set the square empty at this position.
    ///
    /// Clear both color and piece bitboards.
    #[inline]
    pub fn clear_square(&mut self, square: Square, color: Color, piece: Piece) {
        let bp = bit_pos(square);
        self.colors[color] &= !bp;
        self.pieces[piece] &= !bp;
    }

    /// Move a piece from a square to another, clearing initial square.
    pub fn move_square(&mut self, source: Square, dest: Square) {
        let (source_color, source_piece) = (self.get_color_on(source), self.get_piece_on(source));
        self.clear_square(source, source_color, source_piece);
        if !self.is_empty(dest) {
            let (dest_color, dest_piece) = (self.get_color_on(dest), self.get_piece_on(dest));
            self.clear_square(dest, dest_color, dest_piece);
        }
        self.set_square(dest, source_color, source_piece);
    }

    /// Change the piece type at square.
    #[inline]
    pub fn set_piece(&mut self, square: Square, from_piece: Piece, to_piece: Piece) {
        let bp = bit_pos(square);
        self.pieces[from_piece] &= !bp;
        self.pieces[to_piece] |= bp;
    }

    /// Find position of this king.
    pub fn find_king(&self, color: Color) -> Option<Square> {
        let king_bb = self.colors[color] & self.pieces[KING];
        for square in 0..64 {
            if king_bb & bit_pos(square) != 0 {
                return Some(square)
            }
        }
        None
    }

    /// Get all rays for all pieces of `color`.
    ///
    /// This function is used to find illegal moves for opposite color.
    ///
    /// This add move rays of all piece types, pawns being a special
    /// case: their diagonal capture are all added even though no enemy
    /// piece is on the target square. Rays include simple moves,
    /// captures and friendly pieces being protected.
    pub fn get_full_rays(&self, color: Color) -> Bitboard {
        let mut ray_bb = 0;
        let color_bb = self.by_color(color);
        for square in 0..NUM_SQUARES {
            if color_bb & bit_pos(square) == 0 {
                continue
            }
            ray_bb |= match self.get_piece_on(square) {
                PAWN => self.get_pawn_protections(square, color),
                BISHOP => self.get_bishop_full_rays(square, color),
                KNIGHT => self.get_knight_full_rays(square),
                ROOK => self.get_rook_full_rays(square, color),
                QUEEN => self.get_queen_full_rays(square, color),
                KING => self.get_king_full_rays(square),
                _ => { panic!("No piece on square {} but color {} bit is set.", square, color) }
            };
        }
        ray_bb
    }

    /// Get pawn progress: only forward moves.
    pub fn get_pawn_progresses(&self, square: Square, color: Color) -> Bitboard {
        let mut progress_bb = PAWN_PROGRESSES[color][square as usize] & !self.combined();
        // Check that we do not jump over a piece when on a starting position.
        if color == WHITE && sq_rank(square) == RANK_2 {
            if self.combined() & bit_pos(sq(sq_file(square), sq_rank(square) + 1)) != 0 {
                progress_bb &= !bit_pos(sq(sq_file(square), sq_rank(square) + 2));
            }
        }
        else if color == BLACK && sq_rank(square) == RANK_7 {
            if self.combined() & bit_pos(sq(sq_file(square), sq_rank(square) - 1)) != 0 {
                progress_bb &= !bit_pos(sq(sq_file(square), sq_rank(square) - 2));
            }
        }
        progress_bb
    }

    /// Get pawn captures: only moves capturing enemy pieces.
    ///
    /// If a pawn is not currently attacking any piece, the bitboard
    /// will be empty.
    #[inline]
    pub const fn get_pawn_captures(&self, square: Square, color: Color) -> Bitboard {
        PAWN_CAPTURES[color][square as usize] & self.by_color(opposite(color))
    }

    /// Get pawn capture bitboard, without considering enemy pieces.
    ///
    /// Both possible diagonals will be set, even if a friendly piece
    /// occupies one.
    #[inline]
    pub const fn get_pawn_protections(&self, square: Square, color: Color) -> Bitboard {
        PAWN_CAPTURES[color][square as usize]
    }

    /// Get bishop rays: moves and captures bitboard.
    #[inline]
    pub fn get_bishop_rays(&self, square: Square, color: Color) -> Bitboard {
        self.get_blockable_rays(square, color, &BISHOP_DIRS, false)
    }

    /// Get all bishop rays: moves, captures and protections bitboard.
    #[inline]
    pub fn get_bishop_full_rays(&self, square: Square, color: Color) -> Bitboard {
        self.get_blockable_rays(square, color, &BISHOP_DIRS, true)
    }

    /// Get rook rays: moves and captures bitboard.
    #[inline]
    pub fn get_rook_rays(&self, square: Square, color: Color) -> Bitboard {
        self.get_blockable_rays(square, color, &ROOK_DIRS, false)
    }

    /// Get all rook rays: moves, captures and protections bitboard.
    #[inline]
    pub fn get_rook_full_rays(&self, square: Square, color: Color) -> Bitboard {
        self.get_blockable_rays(square, color, &ROOK_DIRS, true)
    }

    /// Get queen rays: moves and captures bitboard.
    #[inline]
    pub fn get_queen_rays(&self, square: Square, color: Color) -> Bitboard {
        self.get_blockable_rays(square, color, &QUEEN_DIRS, false)
    }

    /// Get all queen rays: moves, captures and protections bitboard.
    #[inline]
    pub fn get_queen_full_rays(&self, square: Square, color: Color) -> Bitboard {
        self.get_blockable_rays(square, color, &QUEEN_DIRS, true)
    }

    /// Get rays for piece that can move how far they want.
    ///
    /// Used for bishops, rooks and queens. A ray bitboard is the
    /// combination of squares either empty or occupied by an enemy
    /// piece they can reach.
    ///
    /// If `protection` is true, include friend pieces in rays as well.
    fn get_blockable_rays(
        &self,
        square: Square,
        color: Color,
        directions: &[(i8, i8)],
        include_protections: bool
    ) -> Bitboard {
        let mut rays_bb: Bitboard = 0;
        let color_bb = self.by_color(color);
        let combined_bb = self.combined();
        for dir in directions {
            let mut ray_f = sq_file(square);
            let mut ray_r = sq_rank(square);
            loop {
                ray_f += dir.0;
                ray_r += dir.1;
                if ray_f < 0 || ray_f > 7 || ray_r < 0 || ray_r > 7 {
                    break
                }
                let bp = bit_pos(sq(ray_f, ray_r));
                if !include_protections && color_bb & bp != 0 {
                    break
                }
                rays_bb |= bp;
                if combined_bb & bp != 0 {
                    break
                }
            }
        }
        rays_bb
    }

    /// Get knight rays: moves and captures bitboard.
    #[inline]
    pub const fn get_knight_rays(&self, square: Square, color: Color) -> Bitboard {
        KNIGHT_RAYS[square as usize] & !self.by_color(color)
    }

    /// Get all knight rays: moves, captures and protections bitboard.
    #[inline]
    pub const fn get_knight_full_rays(&self, square: Square) -> Bitboard {
        KNIGHT_RAYS[square as usize]
    }

    /// Get king rays: moves and captures bitboard.
    #[inline]
    pub const fn get_king_rays(&self, square: Square, color: Color) -> Bitboard {
        KING_RAYS[square as usize] & !self.by_color(color)
    }

    /// Get all king rays: moves, captures and protections bitboard.
    #[inline]
    pub const fn get_king_full_rays(&self, square: Square) -> Bitboard {
        KING_RAYS[square as usize]
    }

    /// Debug only: write a text view of the board to stderr.
    #[allow(dead_code)]  // For tests only.
    pub(crate) fn draw(&self) {
        self.draw_to(&mut std::io::stderr());
    }

    /// Debug only: write a text view of the board.
    pub(crate) fn draw_to(&self, f: &mut dyn std::io::Write) {
        let cbb = self.combined();
        for rank in (0..8).rev() {
            let mut rank_str = String::with_capacity(8);
            for file in 0..8 {
                let square = sq(file, rank);
                let bp = bit_pos(square);
                let piece_char = if cbb & bp == 0 {
                    '.'
                } else {
                    let (color, piece) = (self.get_color_on(square), self.get_piece_on(square));
                    let mut piece_char = match piece {
                        PAWN => 'p',
                        BISHOP => 'b',
                        KNIGHT => 'n',
                        ROOK => 'r',
                        QUEEN => 'q',
                        KING => 'k',
                        _ => panic!("Invalid piece.")
                    };
                    if color == WHITE {
                        piece_char = piece_char.to_ascii_uppercase();
                    }
                    piece_char
                };
                rank_str.push(piece_char);
            }
            writeln!(f, "{} {}", rank + 1, rank_str).unwrap();
        }
        writeln!(f, "  abcdefgh").unwrap();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Color

    #[test]
    fn test_opposite() {
        assert_eq!(opposite(WHITE), BLACK);
        assert_eq!(opposite(BLACK), WHITE);
    }

    // Square

    #[test]
    fn test_sq_from_string() {
        assert_eq!(sq_from_string("a1"), A1);
        assert_eq!(sq_from_string("a2"), A2);
        assert_eq!(sq_from_string("a8"), A8);
        assert_eq!(sq_from_string("b1"), B1);
        assert_eq!(sq_from_string("h8"), H8);
    }

    #[test]
    fn test_sq_to_string() {
        assert_eq!(sq_to_string(A1), "a1");
        assert_eq!(sq_to_string(A2), "a2");
        assert_eq!(sq_to_string(A8), "a8");
        assert_eq!(sq_to_string(H8), "h8");
    }

    // Bitboard

    #[test]
    fn test_count_bits() {
        assert_eq!(count_bits(Board::new_empty().combined()), 0);
        assert_eq!(count_bits(Board::new().combined()), 32);
    }

    #[test]
    fn test_before_on_file() {
        // Only should the 4 lowest files for readability.
        assert_eq!(before_on_file(FILE_A, RANK_1), 0b00000000_00000000_00000000_00000000);
        assert_eq!(before_on_file(FILE_A, RANK_2), 0b00000000_00000000_00000000_00000001);
        assert_eq!(before_on_file(FILE_A, RANK_4), 0b00000000_00000000_00000000_00000111);
        assert_eq!(before_on_file(FILE_A, RANK_8), 0b00000000_00000000_00000000_01111111);
        assert_eq!(before_on_file(FILE_B, RANK_1), 0b00000000_00000000_00000000_00000000);
        assert_eq!(before_on_file(FILE_C, RANK_1), 0b00000000_00000000_00000000_00000000);
        assert_eq!(before_on_file(FILE_C, RANK_4), 0b00000000_00000111_00000000_00000000);
        // 4 highest files.
        assert_eq!(before_on_file(FILE_H, RANK_4), 0b00000111_00000000_00000000_00000000 << 32);
        assert_eq!(before_on_file(FILE_H, RANK_7), 0b00111111_00000000_00000000_00000000 << 32);
        assert_eq!(before_on_file(FILE_H, RANK_8), 0b01111111_00000000_00000000_00000000 << 32);
    }

    #[test]
    fn test_after_on_square_file() {
        assert_eq!(after_on_file(FILE_A, RANK_1), 0b00000000_00000000_00000000_11111110);
        assert_eq!(after_on_file(FILE_A, RANK_2), 0b00000000_00000000_00000000_11111100);
        assert_eq!(after_on_file(FILE_A, RANK_4), 0b00000000_00000000_00000000_11110000);
        assert_eq!(after_on_file(FILE_A, RANK_8), 0b00000000_00000000_00000000_00000000);
        assert_eq!(after_on_file(FILE_B, RANK_1), 0b00000000_00000000_11111110_00000000);
        assert_eq!(after_on_file(FILE_C, RANK_1), 0b00000000_11111110_00000000_00000000);
        assert_eq!(after_on_file(FILE_C, RANK_4), 0b00000000_11110000_00000000_00000000);
        assert_eq!(after_on_file(FILE_C, RANK_8), 0b00000000_00000000_00000000_00000000);
        // 4 highest files.
        assert_eq!(after_on_file(FILE_H, RANK_4), 0b11110000_00000000_00000000_00000000 << 32);
        assert_eq!(after_on_file(FILE_H, RANK_7), 0b10000000_00000000_00000000_00000000 << 32);
        assert_eq!(after_on_file(FILE_H, RANK_8), 0b00000000_00000000_00000000_00000000 << 32);
    }

    // Board

    #[test]
    fn test_new_from_fen() {
        let b1 = Board::new();
        let b2 = Board::new_from_fen(crate::fen::FEN_START);
        assert!(b1 == b2);
    }

    #[test]
    fn test_get_color() {
        let b = Board::new();
        assert_eq!(b.get_color_on(A1), WHITE);
        assert_eq!(b.get_color_on(A2), WHITE);
        assert_eq!(b.get_color_on(A7), BLACK);
        assert_eq!(b.get_color_on(A8), BLACK);
        assert_eq!(b.get_color_on(D1), WHITE);
        assert_eq!(b.get_color_on(D8), BLACK);
        assert_eq!(b.get_color_on(E1), WHITE);
        assert_eq!(b.get_color_on(E8), BLACK);
    }

    #[test]
    fn test_get_piece() {
        let b = Board::new();
        assert_eq!(b.get_piece_on(A1), ROOK);
        assert_eq!(b.get_piece_on(A2), PAWN);
        assert_eq!(b.get_piece_on(A7), PAWN);
        assert_eq!(b.get_piece_on(A8), ROOK);
        assert_eq!(b.get_piece_on(D1), QUEEN);
        assert_eq!(b.get_piece_on(D8), QUEEN);
        assert_eq!(b.get_piece_on(E1), KING);
        assert_eq!(b.get_piece_on(E8), KING);
    }

    #[test]
    fn test_move_square() {
        let mut b = Board::new_empty();
        b.set_square(D4, WHITE, PAWN);
        b.move_square(D4, D5);
        let bp_d4 = bit_pos(D4);
        let bp_d5 = bit_pos(D5);
        // Source square is cleared.
        assert_eq!(b.combined() & bp_d4, 0);
        assert_eq!(b.by_color(WHITE) & bp_d4, 0);
        assert_eq!(b.by_piece(PAWN) & bp_d4, 0);
        // Destination square is set only to the right color and piece.
        assert_eq!(b.combined() & bp_d5, bp_d5);
        assert_eq!(b.by_color(WHITE) & bp_d5, bp_d5);
        assert_eq!(b.by_piece(PAWN) & bp_d5, bp_d5);

        b.set_square(E6, BLACK, PAWN);
        b.move_square(D5, E6);
        let bp_e6 = bit_pos(E6);
        assert_eq!(b.combined() & bp_d5, 0);
        assert_eq!(b.by_color(WHITE) & bp_d5, 0);
        assert_eq!(b.by_piece(PAWN) & bp_d5, 0);
        assert_eq!(b.combined() & bp_e6, bp_e6);
        assert_eq!(b.by_color(WHITE) & bp_e6, bp_e6);
        assert_eq!(b.by_color(BLACK) & bp_e6, 0);
        assert_eq!(b.by_piece(PAWN) & bp_e6, bp_e6);

    }

    #[test]
    fn test_set_piece() {
        let mut b = Board::new();
        b.set_piece(E1, KING, QUEEN);
        assert_eq!(b.get_color_on(E1), WHITE);
        assert_eq!(b.get_piece_on(E1), QUEEN);
    }

    #[test]
    fn test_find_king() {
        let b = Board::new_empty();
        assert_eq!(b.find_king(WHITE), None);
        let b = Board::new();
        assert_eq!(b.find_king(WHITE), Some(E1));
        assert_eq!(b.find_king(BLACK), Some(E8));
    }

    #[test]
    fn test_get_full_rays() {
        let b = Board::new();
        // Third ranks protected, all pieces protected except rooks = 22 squares.
        assert_eq!(count_bits(b.get_full_rays(WHITE)), 22);
        assert_eq!(count_bits(b.get_full_rays(BLACK)), 22);
    }

    #[test]
    fn test_get_pawn_progresses() {
        let mut b = Board::new_empty();

        // Check for simple or double move for white and black.
        b.set_square(A2, WHITE, PAWN);
        assert_eq!(count_bits(b.get_pawn_progresses(A2, WHITE)), 2);
        b.set_square(B2, WHITE, PAWN);
        assert_eq!(count_bits(b.get_pawn_progresses(B2, WHITE)), 2);
        b.set_square(B3, WHITE, PAWN);
        assert_eq!(count_bits(b.get_pawn_progresses(B3, WHITE)), 1);
        assert!(b.get_pawn_progresses(B3, WHITE) & bit_pos(B4) != 0);
        b.set_square(H7, WHITE, PAWN);
        assert_eq!(count_bits(b.get_pawn_progresses(H7, WHITE)), 1);
        b.set_square(A7, BLACK, PAWN);
        assert_eq!(count_bits(b.get_pawn_progresses(A7, BLACK)), 2);
        assert!(b.get_pawn_progresses(A7, BLACK) & bit_pos(A6) != 0);
        assert!(b.get_pawn_progresses(A7, BLACK) & bit_pos(A5) != 0);

        // Check that a starting pawn cannot jump over another piece.
        // Here, b2 is still blocked by another pawn on b3.
        assert_eq!(count_bits(b.get_pawn_progresses(B2, WHITE)), 0);
        // Move the blocking pawn to b4: one move is freed.
        b.move_square(B3, B4);
        let progress_bb = b.get_pawn_progresses(B2, WHITE);
        assert_eq!(count_bits(progress_bb), 1);
        assert!(progress_bb & bit_pos(B3) != 0);
    }

    #[test]
    fn test_get_pawn_captures() {
        let mut b = Board::new_empty();

        // No capture by default.
        b.set_square(A2, WHITE, PAWN);
        assert_eq!(count_bits(b.get_pawn_captures(A2, WHITE)), 0);
        // Can't capture forward.
        b.set_square(A3, BLACK, PAWN);
        assert_eq!(count_bits(b.get_pawn_captures(A2, WHITE)), 0);
        // Can't capture a frendly piece.
        b.set_square(B3, WHITE, KNIGHT);
        assert_eq!(count_bits(b.get_pawn_captures(A2, WHITE)), 0);
        // Capture that pawn...
        b.set_square(B3, BLACK, PAWN);
        assert_eq!(count_bits(b.get_pawn_captures(A2, WHITE)), 1);
        // But it can capture you back!
        assert_eq!(count_bits(b.get_pawn_captures(B3, BLACK)), 1);
        // This one can capture both b3 and d3 black pawns.
        b.set_square(C2, WHITE, PAWN);
        b.set_square(D3, BLACK, PAWN);
        assert_eq!(count_bits(b.get_pawn_captures(C2, WHITE)), 2);
    }

    #[test]
    fn test_get_pawn_protections() {
        let mut b = Board::new_empty();

        // A pawn not on a border file or rank always protect 2 squares.
        b.set_square(B2, WHITE, PAWN);
        assert_eq!(count_bits(b.get_pawn_protections(B2, WHITE)), 2);
        b.set_square(A2, WHITE, PAWN);
        assert_eq!(count_bits(b.get_pawn_protections(A2, WHITE)), 1);
    }

    #[test]
    fn test_get_bishop_rays() {
        let mut b = Board::new_empty();

        // A bishop has maximum range when it's in a center square.
        b.set_square(D4, WHITE, BISHOP);
        let rays_bb = b.get_bishop_rays(D4, WHITE);
        assert_eq!(count_bits(rays_bb), 13);
        // Going top-right.
        assert!(rays_bb & bit_pos(E5) != 0);
        assert!(rays_bb & bit_pos(F6) != 0);
        assert!(rays_bb & bit_pos(G7) != 0);
        assert!(rays_bb & bit_pos(H8) != 0);
        // Going bottom-right.
        assert!(rays_bb & bit_pos(E3) != 0);
        assert!(rays_bb & bit_pos(F2) != 0);
        assert!(rays_bb & bit_pos(G1) != 0);
        // Going bottom-left.
        assert!(rays_bb & bit_pos(C3) != 0);
        assert!(rays_bb & bit_pos(B2) != 0);
        assert!(rays_bb & bit_pos(A1) != 0);
        // Going top-left.
        assert!(rays_bb & bit_pos(C5) != 0);
        assert!(rays_bb & bit_pos(B6) != 0);
        assert!(rays_bb & bit_pos(A7) != 0);

        // When blocking commit to one square with friendly piece, lose 2 moves.
        b.set_square(B2, WHITE, PAWN);
        let rays_bb = b.get_bishop_rays(D4, WHITE);
        assert_eq!(count_bits(rays_bb), 11);

        // When blocking commit to one square with enemy piece, lose only 1 move.
        b.set_square(B2, BLACK, PAWN);
        let rays_bb = b.get_bishop_rays(D4, WHITE);
        assert_eq!(count_bits(rays_bb), 12);
    }

    #[test]
    fn test_get_knight_moves() {
        let mut b = Board::new_empty();

        // A knight is never blocked; if it's in the center of the board,
        // it can have up to 8 moves.
        b.set_square(D4, WHITE, KNIGHT);
        let rays_bb = b.get_knight_rays(D4, WHITE);
        assert_eq!(count_bits(rays_bb), 8);

        // If on a side if has only 4 moves.
        b.set_square(A4, WHITE, KNIGHT);
        let rays_bb = b.get_knight_rays(A4, WHITE);
        assert_eq!(count_bits(rays_bb), 4);

        // And in a corner, only 2 moves.
        b.set_square(A1, WHITE, KNIGHT);
        let rays_bb = b.get_knight_rays(A1, WHITE);
        assert_eq!(count_bits(rays_bb), 2);

        // Add 2 friendly pieces and it is totally blocked.
        b.set_square(B3, WHITE, PAWN);
        b.set_square(C2, WHITE, PAWN);
        let rays_bb = b.get_knight_rays(A1, WHITE);
        assert_eq!(count_bits(rays_bb), 0);

        // If one of those pieces is an enemy, it can be taken.
        b.set_square(B3, BLACK, PAWN);
        let rays_bb = b.get_knight_rays(A1, WHITE);
        assert_eq!(count_bits(rays_bb), 1);
    }

    #[test]
    fn test_get_rook_moves() {
        let mut b = Board::new_empty();

        b.set_square(D4, WHITE, ROOK);
        let rays_bb = b.get_rook_rays(D4, WHITE);
        assert_eq!(count_bits(rays_bb), 14);
        b.set_square(D6, BLACK, PAWN);
        let rays_bb = b.get_rook_rays(D4, WHITE);
        assert_eq!(count_bits(rays_bb), 12);
        b.set_square(D6, WHITE, PAWN);
        let rays_bb = b.get_rook_rays(D4, WHITE);
        assert_eq!(count_bits(rays_bb), 11);
    }

    #[test]
    fn test_get_queen_moves() {
        let mut b = Board::new_empty();

        b.set_square(D4, WHITE, QUEEN);
        let rays_bb = b.get_queen_rays(D4, WHITE);
        assert_eq!(count_bits(rays_bb), 14 + 13);
    }
}

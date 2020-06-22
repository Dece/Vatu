//! Functions to determine legal moves.

use crate::board::*;
use crate::castling::*;
use crate::fen;
use crate::movement::Move;

pub const POS_MIN: i8 = 0;
pub const POS_MAX: i8 = 7;

/// Characteristics of the state of a game.
///
/// It does not include various parameters such as clocks that are
/// more aimed for engine analysis than typical rules checking.
///
/// - `color`: current player's turn
/// - `castling`: which castling options are available; updated throughout the game.
/// - `en_passant`: position of a pawn that can be taken using en passant attack.
/// - `halfmove`: eh not sure
/// - `fullmove`: same
#[derive(Debug, PartialEq, Clone, Hash)]
pub struct GameState {
    pub color: Color,
    pub castling: u8,
    pub en_passant: Option<Square>,
    pub halfmove: i32,
    pub fullmove: i32,
}

impl GameState {
    pub const fn new() -> GameState {
        GameState {
            color: WHITE,
            castling: CASTLING_MASK,
            en_passant: None,
            halfmove: 0,
            fullmove: 1,
        }
    }
}

impl std::fmt::Display for GameState {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "[color: {}, castling: {:04b}, en_passant: {}, halfmove: {}, fullmove: {}]",
            color_to_string(self.color), self.castling,
            fen::en_passant_to_string(self.en_passant),
            self.halfmove, self.fullmove
        )
    }
}

/// Get a list of moves for all pieces of the playing color.
///
/// If `pseudo_legal` is true, do not check for illegal moves. This is
/// used to avoid endless recursion when checking if a P move is
/// illegal, as it needs to check all possible following enemy moves,
/// e.g. to see if P's king can be taken. Consider a call with true
/// `pseudo_legal` as a collection of attacked squares instead of legal
/// move collection.
pub fn get_player_moves(
    board: &Board,
    game_state: &GameState,
    pseudo_legal: bool,
) -> Vec<Move> {
    let mut moves = Vec::with_capacity(32);
    for r in 0..8 {
        for f in 0..8 {
            let square = sq(f, r);
            if board.is_empty(square) {
                continue
            }
            if board.get_color_on(square) == game_state.color {
                moves.append(
                    &mut get_piece_moves(board, game_state, square, game_state.color, pseudo_legal)
                );
            }
        }
    }
    moves
}

/// Get a list of moves for the piece of `color` on `square`.
///
/// Use `board` and `game_state` to get the moves. `color` is the color
/// of the piece on `square`; it could technically be found from the
/// board but that would require an additional lookup and this function
/// is always called in a context where the piece color is known.
fn get_piece_moves(
    board: &Board,
    game_state: &GameState,
    square: Square,
    color: Color,
    pseudo_legal: bool,
) -> Vec<Move> {
    match board.get_piece_on(square) {
        PAWN => get_pawn_moves(board, game_state, square, color, pseudo_legal),
        BISHOP => get_bishop_moves(board, game_state, square, color, pseudo_legal),
        KNIGHT => get_knight_moves(board, game_state, square, color, pseudo_legal),
        ROOK => get_rook_moves(board, game_state, square, color, pseudo_legal),
        QUEEN => get_queen_moves(board, game_state, square, color, pseudo_legal),
        KING => get_king_moves(board, game_state, square, color, pseudo_legal),
        _ => { panic!("No piece on square.") },
    }
}

// fn get_pawn_moves(
//     board: &Board,
//     game_state: &GameState,
//     square: Square,
//     color: Color,
//     pseudo_legal: bool,
// ) -> Vec<Move> {
//     let (f, r) = (sq_file(square), sq_rank(square));
//     let mut moves = vec!();
//     // Direction: positive for white, negative for black.
//     let dir = if color == WHITE { 1 } else { -1 };
//     // Check 1 or 2 square forward.
//     let move_len = if (color == WHITE && r == 1) || (color == BLACK && r == 6) { 2 } else { 1 };
//     for i in 1..=move_len {
//         let forward_r = r + dir * i;
//         if dir > 0 && forward_r > POS_MAX {
//             return moves
//         }
//         if dir < 0 && forward_r < POS_MIN {
//             return moves
//         }
//         let forward: Square = sq(f, forward_r);
//         // If forward square is empty (and we are not jumping over an occupied square), add it.
//         if board.is_empty(forward) && (i == 1 || board.is_empty(sq(f, forward_r - dir))) {
//             let mut m = Move::new(square, forward);
//             // Pawns that get to the opposite rank automatically promote as queens.
//             if (dir > 0 && forward_r == POS_MAX) || (dir < 0 && forward_r == POS_MIN) {
//                 m.promotion = Some(QUEEN)
//             }
//             if pseudo_legal || !is_illegal(board, game_state, &m) {
//                 moves.push(m);
//             }
//         }
//         // Check diagonals for pieces to attack.
//         if i == 1 {
//             // First diagonal.
//             if f - 1 >= POS_MIN {
//                 let diag = sq(f - 1, forward_r);
//                 if !board.is_empty(diag) {
//                     let diag_color = board.get_color_on(diag);
//                     if let Some(m) = get_capture_move(color, square, diag_color, diag, true) {
//                         if pseudo_legal || !is_illegal(board, game_state, &m) {
//                             moves.push(m);
//                         }
//                     }

//                 }
//             }
//             // Second diagonal.
//             if f + 1 <= POS_MAX {
//                 let diag = sq(f + 1, forward_r);
//                 if !board.is_empty(diag) {
//                     let diag_color = board.get_color_on(diag);
//                     if let Some(m) = get_capture_move(color, square, diag_color, diag, true) {
//                         if pseudo_legal || !is_illegal(board, game_state, &m) {
//                             moves.push(m);
//                         }
//                     }
//                 }
//             }
//         }
//         // TODO en passant
//     }
//     moves
// }

fn get_pawn_moves(
    board: &Board,
    game_state: &GameState,
    square: Square,
    color: Color,
    pseudo_legal: bool,
) -> Vec<Move> {
    get_moves_from_bb(
        board,
        game_state,
        board.get_pawn_progresses(square, color) | board.get_pawn_captures(square, color),
        square,
        color,
        PAWN,
        pseudo_legal
    )
    // TODO en passant
}

fn get_bishop_moves(
    board: &Board,
    game_state: &GameState,
    square: Square,
    color: Color,
    pseudo_legal: bool,
) -> Vec<Move> {
    get_moves_from_bb(
        board,
        game_state,
        board.get_bishop_rays(square, color),
        square,
        color,
        BISHOP,
        pseudo_legal
    )
}

fn get_knight_moves(
    board: &Board,
    game_state: &GameState,
    square: Square,
    color: Color,
    pseudo_legal: bool,
) -> Vec<Move> {
    get_moves_from_bb(
        board,
        game_state,
        board.get_knight_rays(square, color),
        square,
        color,
        KNIGHT,
        pseudo_legal
    )
}

fn get_rook_moves(
    board: &Board,
    game_state: &GameState,
    square: Square,
    color: Color,
    pseudo_legal: bool,
) -> Vec<Move> {
    get_moves_from_bb(
        board,
        game_state,
        board.get_rook_rays(square, color),
        square,
        color,
        ROOK,
        pseudo_legal
    )
}

fn get_queen_moves(
    board: &Board,
    game_state: &GameState,
    square: Square,
    color: Color,
    pseudo_legal: bool,
) -> Vec<Move> {
    get_moves_from_bb(
        board,
        game_state,
        board.get_queen_rays(square, color),
        square,
        color,
        QUEEN,
        pseudo_legal
    )
}

fn get_king_moves(
    board: &Board,
    game_state: &GameState,
    square: Square,
    color: Color,
    pseudo_legal: bool,
) -> Vec<Move> {
    let mut moves = get_moves_from_bb(
        board,
        game_state,
        board.get_king_rays(square, color),
        square,
        color,
        KING,
        pseudo_legal
    );

    // Stop here for pseudo legal moves as castling is not considered along with them.
    if pseudo_legal {
        return moves
    }

    // Castling. Here are the rules that should ALL be respected:
    // 1. The king and the chosen rook are on the player's first rank.
    // 2. Neither the king nor the chosen rook has previously moved.
    // 3. There are no pieces between the king and the chosen rook.
    // 4. The king is not currently in check.
    // 5. The king does not pass through a square that is attacked by an enemy piece.
    // 6. The king does not end up in check.

    // First get the required castling rank and color mask for the player.
    let (castling_rank, castling_color_mask) = if game_state.color == WHITE {
        (0, CASTLING_WH_MASK)
    } else {
        (7, CASTLING_BL_MASK)
    };

    let r = sq_rank(square);
    // Check for castling if the king is on its castling rank (R1)
    // and is not in check (R4).
    if
        r == castling_rank &&
        !is_attacked(board, game_state, square)
    {
        // Check for both castling sides.
        for (path_files, opt_empty_file, castling_side_mask) in CASTLING_SIDES.iter() {
            // Check for castling availability for this color and side.
            if (game_state.castling & castling_color_mask & castling_side_mask) != 0 {
                // Check that squares in the king's path are empty and not attacked (R3.1, R5, R6).
                let mut path_is_clear = true;
                for path_f in path_files {
                    let path_square = sq(*path_f, castling_rank);
                    if
                        !board.is_empty(path_square)
                        || is_illegal(board, game_state, &Move::new(square, path_square)) {
                        path_is_clear = false;
                        break;
                    }
                }
                if !path_is_clear {
                    continue;
                }
                // Check that rook jumps over an empty square on queen-side (R3.2).
                if let Some(rook_path_f) = opt_empty_file {
                    let rook_path_square = sq(*rook_path_f, castling_rank);
                    if !board.is_empty(rook_path_square) {
                        continue;
                    }
                }
                let castle = castling_side_mask & castling_color_mask;
                let m = Move::get_castle_move(castle);
                if pseudo_legal || !is_illegal(board, game_state, &m) {
                    moves.push(m);
                }
            }
        }
    }
    moves
}

/// Get moves from this ray bitboard.
///
/// Inspect all moves from the bitboard and produce a Move for each
/// legal move, or all moves if `pseudo_legal` is true. Pawns that
/// reach the last rank are promoted as queens.
fn get_moves_from_bb(
    board: &Board,
    game_state: &GameState,
    bitboard: Bitboard,
    square: Square,
    color: Color,
    piece: Piece,
    pseudo_legal: bool
) -> Vec<Move> {
    let mut moves = Vec::with_capacity(count_bits(bitboard).into());
    for ray_square in 0..NUM_SQUARES {
        if ray_square == square || bitboard & bit_pos(ray_square) == 0 {
            continue
        }
        if let Some(mut m) = inspect_move(board, game_state, square, ray_square, pseudo_legal) {
            // Automatic queen promotion for pawns moving to the opposite rank.
            if
                piece == PAWN
                && (
                    (color == WHITE && sq_rank(ray_square) == RANK_8)
                    || (color == BLACK && sq_rank(ray_square) == RANK_1)
                )
            {
                m.promotion = Some(QUEEN);
            }
            moves.push(m);
        }
    }
    moves
}

/// Accept or ignore a move from `square` to `ray_square`.
///
/// This function checks that the move is legal, unless `pseudo_legal`
/// is true. It assumes that `ray_square` is either empty or an enemy
/// piece, but not a friend piece: they should have been filtered.
///
/// This function does not set promotions for pawns reaching last rank.
fn inspect_move(
    board: &Board,
    game_state: &GameState,
    square: Square,
    ray_square: Square,
    pseudo_legal: bool
) -> Option<Move> {
    let m = Move::new(square, ray_square);
    if pseudo_legal || !is_illegal(board, game_state, &m) {
        Some(m)
    } else {
        None
    }
}

/// Return a move from `square1` to `square2` if colors are opposite.
fn get_capture_move(
    color1: Color,
    square1: Square,
    color2: Color,
    square2: Square,
    is_pawn: bool,
) -> Option<Move> {
    if color2 == opposite(color1) {
        // Automatic queen promotion for pawns moving to the opposite rank.
        Some(if
            is_pawn
            && (color1 == WHITE && sq_rank(square2) == POS_MAX)
            || (color1 == BLACK && sq_rank(square2) == POS_MIN)
        {
            Move::new_promotion(square1, square2, QUEEN)
        } else {
            Move::new(square1, square2)
        })
    } else {
        None
    }
}

/// Check if a move is illegal.
fn is_illegal(board: &Board, game_state: &GameState, m: &Move) -> bool {
    if let Some(mut king_square) = board.find_king(game_state.color) {
        let mut hypothetic_board = board.clone();
        m.apply_to_board(&mut hypothetic_board);
        // A move is illegal if the king ends up in check.
        // If king moves, use its new position.
        if m.source == king_square {
            king_square = m.dest
        }
        // Check if the move makes the player king in check.
        if is_attacked(&hypothetic_board, &game_state, king_square) {
            return true
        }
    }
    false
}

/// Return true if the piece on `square` is attacked.
///
/// Check all possible enemy moves and return true when one of them
/// ends up attacking the position.
///
/// Beware that the game state must be coherent with the analysed
/// square, i.e. if the piece on `square` is white, the game state
/// should tell that it is white turn. If `square` is empty, simply
/// check if it is getting attacked by the opposite player.
fn is_attacked(board: &Board, game_state: &GameState, square: Square) -> bool {
    let mut enemy_game_state = game_state.clone();
    enemy_game_state.color = opposite(game_state.color);
    // Do not attempt to commit moves, just check for attacked squares.
    let enemy_moves = get_player_moves(board, &enemy_game_state, true);
    for m in enemy_moves.iter() {
        if square == m.dest {
            return true
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_player_moves() {
        let b = Board::new();
        let gs = GameState::new();

        // At first move, white has 16 pawn moves and 4 knight moves.
        let moves = get_player_moves(&b, &gs, false);
        assert_eq!(moves.len(), 20);
    }

    #[test]
    fn test_get_pawn_moves() {
        let mut b = Board::new_empty();
        let gs = GameState::new();

        // Check that a pawn (here white queen's pawn) can move forward if the road is free.
        b.set_square(D3, WHITE, PAWN);
        let moves = get_piece_moves(&b, &gs, D3, WHITE, false);
        assert!(moves.len() == 1 && moves.contains(&Move::new(D3, D4)));

        // Check that a pawn (here white king's pawn) can move 2 square forward on first move.
        b.set_square(E2, WHITE, PAWN);
        let moves = get_piece_moves(&b, &gs, E2, WHITE, false);
        assert_eq!(moves.len(), 2);
        assert!(moves.contains(&Move::new(E2, E3)));
        assert!(moves.contains(&Move::new(E2, E4)));

        // Check that a pawn cannot move forward if a piece is blocking its path.
        // 1. black pawn 2 square forward; only 1 square forward available from start pos.
        b.set_square(E4, BLACK, PAWN);
        let moves = get_piece_moves(&b, &gs, E2, WHITE, false);
        assert!(moves.len() == 1 && moves.contains(&Move::new(E2, E3)));
        // 2. black pawn 1 square forward; no square available.
        b.set_square(E3, BLACK, PAWN);
        let moves = get_piece_moves(&b, &gs, E2, WHITE, false);
        assert_eq!(moves.len(), 0);
        // 3. remove the e4 black pawn; the white pawn should not be able to jump above e3 pawn.
        b.clear_square(E4, BLACK, PAWN);
        let moves = get_piece_moves(&b, &gs, E2, WHITE, false);
        assert_eq!(moves.len(), 0);

        // Check that a pawn can take a piece diagonally.
        b.set_square(F3, BLACK, PAWN);
        let moves = get_piece_moves(&b, &gs, E2, WHITE, false);
        assert!(moves.len() == 1 && moves.contains(&Move::new(E2, F3)));
        b.set_square(D3, BLACK, PAWN);
        let moves = get_piece_moves(&b, &gs, E2, WHITE, false);
        assert_eq!(moves.len(), 2);
        assert!(moves.contains( &Move::new(E2, F3) ));
        assert!(moves.contains( &Move::new(E2, D3) ));

        // Check that a pawn moving to the last rank leads to queen promotion.
        // 1. by simply moving forward.
        b.set_square(A7, WHITE, PAWN);
        let moves = get_piece_moves(&b, &gs, A7, WHITE, false);
        assert!(moves.len() == 1 && moves.contains(&Move::new_promotion(A7, A8, QUEEN)));
    }

    #[test]
    fn test_get_bishop_moves() {
        let mut b = Board::new_empty();
        let gs = GameState::new();

        // A bishop has maximum range when it's in a center square.
        b.set_square(D4, WHITE, BISHOP);
        let moves = get_piece_moves(&b, &gs, D4, WHITE, false);
        assert_eq!(moves.len(), 13);
        // Going top-right.
        assert!(moves.contains(&Move::new(D4, E5)));
        assert!(moves.contains(&Move::new(D4, F6)));
        assert!(moves.contains(&Move::new(D4, G7)));
        assert!(moves.contains(&Move::new(D4, H8)));
        // Going bottom-right.
        assert!(moves.contains(&Move::new(D4, E3)));
        assert!(moves.contains(&Move::new(D4, F2)));
        assert!(moves.contains(&Move::new(D4, G1)));
        // Going bottom-left.
        assert!(moves.contains(&Move::new(D4, C3)));
        assert!(moves.contains(&Move::new(D4, B2)));
        assert!(moves.contains(&Move::new(D4, A1)));
        // Going top-left.
        assert!(moves.contains(&Move::new(D4, C5)));
        assert!(moves.contains(&Move::new(D4, B6)));
        assert!(moves.contains(&Move::new(D4, A7)));

        // When blocking commit to one square with friendly piece, lose 2 moves.
        b.set_square(B2, WHITE, PAWN);
        assert_eq!(get_piece_moves(&b, &gs, D4, WHITE, false).len(), 11);

        // When blocking commit to one square with enemy piece, lose only 1 move.
        b.set_square(B2, BLACK, PAWN);
        assert_eq!(get_piece_moves(&b, &gs, D4, WHITE, false).len(), 12);
    }

    #[test]
    fn test_get_knight_moves() {
        let mut b = Board::new_empty();
        let gs = GameState::new();

        // A knight never has blocked commit; if it's in the center of the board, it can have up to
        // 8 moves.
        b.set_square(D4, WHITE, KNIGHT);
        assert_eq!(get_piece_moves(&b, &gs, D4, WHITE, false).len(), 8);

        // If on a side if has only 4 moves.
        b.set_square(A4, WHITE, KNIGHT);
        assert_eq!(get_piece_moves(&b, &gs, A4, WHITE, false).len(), 4);

        // And in a corner, only 2 moves.
        b.set_square(A1, WHITE, KNIGHT);
        assert_eq!(get_piece_moves(&b, &gs, A1, WHITE, false).len(), 2);

        // Add 2 friendly pieces and it is totally blocked.
        b.set_square(B3, WHITE, PAWN);
        b.set_square(C2, WHITE, PAWN);
        assert_eq!(get_piece_moves(&b, &gs, A1, WHITE, false).len(), 0);
    }

    #[test]
    fn test_get_rook_moves() {
        let mut b = Board::new_empty();
        let gs = GameState::new();

        b.set_square(D4, WHITE, ROOK);
        assert_eq!(get_piece_moves(&b, &gs, D4, WHITE, false).len(), 14);
        b.set_square(D6, BLACK, PAWN);
        assert_eq!(get_piece_moves(&b, &gs, D4, WHITE, false).len(), 12);
        b.set_square(D6, WHITE, PAWN);
        assert_eq!(get_piece_moves(&b, &gs, D4, WHITE, false).len(), 11);
    }

    #[test]
    fn test_get_queen_moves() {
        let mut b = Board::new_empty();
        let gs = GameState::new();

        b.set_square(D4, WHITE, QUEEN);
        assert_eq!(get_piece_moves(&b, &gs, D4, WHITE, false).len(), 14 + 13);
    }

    #[test]
    fn test_get_king_moves() {
        let mut gs = GameState::new();

        // King can move 1 square in any direction.
        let mut b = Board::new_empty();
        b.set_square(D4, WHITE, KING);
        assert_eq!(get_piece_moves(&b, &gs, D4, WHITE, false).len(), 8);
        b.set_square(E5, WHITE, PAWN);
        assert_eq!(get_piece_moves(&b, &gs, D4, WHITE, false).len(), 7);

        // If castling is available, other moves are possible: 5 moves + 2 castles.
        let mut b = Board::new_empty();
        b.set_square(E1, WHITE, KING);
        b.set_square(A1, WHITE, ROOK);
        b.set_square(H1, WHITE, ROOK);
        assert_eq!(get_piece_moves(&b, &gs, E1, WHITE, false).len(), 5 + 2);

        // Castling works as well for black.
        gs.color = BLACK;
        b.set_square(E8, BLACK, KING);
        b.set_square(A8, BLACK, ROOK);
        b.set_square(H8, BLACK, ROOK);
        assert_eq!(get_piece_moves(&b, &gs, E8, BLACK, false).len(), 5 + 2);
    }

    #[test]
    fn test_filter_illegal_moves() {
        let mut b = Board::new_empty();
        let mut gs = GameState::new();

        // Place white's king on first rank.
        b.set_square(E1, WHITE, KING);
        // Place black rook in second rank: king can only move left or right.
        b.set_square(H2, BLACK, ROOK);
        // No castling available.
        gs.castling = 0;
        // 5 moves in absolute but only 2 are legal.
        let all_wh_moves = get_piece_moves(&b, &gs, E1, WHITE, false);
        assert_eq!(all_wh_moves.len(), 2);
    }

    #[test]
    fn test_is_attacked() {
        let mut b = Board::new_empty();
        let gs = GameState::new();

        // Place a black rook in white pawn's file.
        b.set_square(D4, WHITE, PAWN);
        b.set_square(D6, BLACK, ROOK);
        assert!(is_attacked(&b, &gs, D4));
        // Move the rook on another file, no more attack.
        Move::new(D6, E6).apply_to_board(&mut b);
        assert!(!is_attacked(&b, &gs, D4));
    }
}

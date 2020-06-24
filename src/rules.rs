//! Functions to determine legal moves.

use crate::board::*;
use crate::castling::*;
use crate::fen;
use crate::movement::Move;

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
            castling: CASTLE_MASK,
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
    board: &mut Board,
    game_state: &mut GameState,
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
                    &mut get_piece_moves(board, game_state, square, game_state.color)
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
    board: &mut Board,
    game_state: &mut GameState,
    square: Square,
    color: Color,
) -> Vec<Move> {
    let piece = board.get_piece_on(square);
    let mut moves = Vec::with_capacity(32);
    get_moves_from_bb(
        board,
        game_state,
        match piece {
            PAWN => {
                board.get_pawn_progresses(square, color)
                    | board.get_pawn_captures(square, color)
            }
            KING => board.get_king_rays(square, color),
            BISHOP => board.get_bishop_rays(square, color),
            KNIGHT => board.get_knight_rays(square, color),
            ROOK => board.get_rook_rays(square, color),
            QUEEN => board.get_queen_rays(square, color),
            _ => { panic!("Invalid piece.") }
        },
        square,
        color,
        piece,
        &mut moves
    );
    if piece == KING && sq_rank(square) == CASTLE_RANK_BY_COLOR[color] {
        get_king_castles(board, game_state, square, color, &mut moves);
    }
    moves
}

/// Get moves from this ray bitboard.
///
/// Inspect all moves from the bitboard and produce a Move for each
/// legal move. Does not take castle into account. Pawns that reach
/// the last rank are promoted as queens.
fn get_moves_from_bb(
    board: &mut Board,
    game_state: &mut GameState,
    bitboard: Bitboard,
    square: Square,
    color: Color,
    piece: Piece,
    moves: &mut Vec<Move>
) {
    for ray_square in 0..NUM_SQUARES {
        if ray_square == square || bitboard & bit_pos(ray_square) == 0 {
            continue
        }
        if let Some(mut m) = inspect_move(board, game_state, square, ray_square) {
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
}

/// Accept or ignore a move from `square` to `ray_square`.
///
/// This function checks that the move is legal. It assumes that
/// `ray_square` is either empty or an enemy piece, but not a friend
/// piece: they should have been filtered.
///
/// This function, in case a move is accepted, sets the `capture` field
/// if the target square hold a piece.
///
/// This function does not set promotions for pawns reaching last rank.
fn inspect_move(
    board: &mut Board,
    game_state: &mut GameState,
    square: Square,
    ray_square: Square,
) -> Option<Move> {
    let mut m = Move::new(square, ray_square);
    if !is_illegal(board, game_state, &mut m) {
        if !board.is_empty(ray_square) {
            m.capture = Some(board.get_piece_on(ray_square))
        }
        Some(m)
    } else {
        None
    }
}

/// Check if a move is illegal.
fn is_illegal(
    board: &mut Board,
    game_state: &mut GameState,
    m: &mut Move,
) -> bool {
    let color = game_state.color;
    // A move is illegal if the king ends up in check.
    m.apply_to(board, game_state);
    if let Some(king) = board.find_king(color) {
        let attacked_bb = board.get_full_rays(opposite(color));
        m.unmake(board, game_state);
        attacked_bb & bit_pos(king) != 0
    } else {
        m.unmake(board, game_state);
        false
    }
}

/// Get possible castles.
///
/// Here are the rules that should ALL be respected:
/// 1. The king and the chosen rook are on the player's first rank.
/// 2. Neither the king nor the chosen rook has previously moved.
/// 3. There are no pieces between the king and the chosen rook.
/// 4. The king is not currently in check.
/// 5. The king does not pass through a square that is attacked by an enemy piece.
/// 6. The king does not end up in check.
///
/// Rule 1 is NOT checked by this method to avoid creating empty vecs.
/// Check it in the caller.
fn get_king_castles(
    board: &Board,
    game_state: &GameState,
    square: Square,
    color: Color,
    moves: &mut Vec<Move>
) {
    let combined_bb = board.combined();

    // First get the required castling rank and color mask for the player.
    let castle_rank = CASTLE_RANK_BY_COLOR[color];
    let castle_color_mask = CASTLE_MASK_BY_COLOR[color];

    // Check for castling if the king is on its castling rank (R1)
    if sq_rank(square) == castle_rank {
        // Check for both castling sides.
        for castle_side_id in 0..NUM_CASTLE_SIDES {
            let castle_side_mask = CASTLE_SIDES[castle_side_id];
            // Check for castling availability for this color and side (R2).
            if (game_state.castling & castle_color_mask & castle_side_mask) != 0 {
                // Check that squares in the king's path are not attacked (R4, R5, R6).
                let castle_legality_path = CASTLE_LEGALITY_PATHS[color][castle_side_id];
                let attacked_bb = board.get_full_rays(opposite(game_state.color));
                if attacked_bb & castle_legality_path != 0 {
                    continue
                }

                // Check that squares in both the king and rook's path are empty.
                let castle_move_path = CASTLE_MOVE_PATHS[color][castle_side_id];
                if combined_bb & castle_move_path != 0 {
                    continue
                }

                moves.push(Move::get_castle_move(castle_side_mask & castle_color_mask));
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_player_moves() {
        let mut b = Board::new();
        let mut gs = GameState::new();

        // At first move, white has 16 pawn moves and 4 knight moves.
        let moves = get_player_moves(&mut b, &mut gs);
        assert_eq!(moves.len(), 20);
    }

    #[test]
    fn test_get_pawn_progress_moves() {
        let mut b = Board::new_empty();
        let mut gs = GameState::new();

        // Check that a pawn (here white queen's pawn) can move forward if the road is free.
        b.set_square(D3, WHITE, PAWN);
        let moves = get_piece_moves(&mut b, &mut gs, D3, WHITE);
        assert_eq!(moves.len(), 1);
        assert!(moves.iter().any(|m| m.source == D3 && m.dest == D4));

        // Check that a pawn (here white king's pawn) can move 2 square forward on first move.
        b.set_square(E2, WHITE, PAWN);
        let moves = get_piece_moves(&mut b, &mut gs, E2, WHITE);
        assert_eq!(moves.len(), 2);
        assert!(moves.iter().any(|m| m.source == E2 && m.dest == E3));
        assert!(moves.iter().any(|m| m.source == E2 && m.dest == E4));

        // Check that a pawn cannot move forward if a piece is blocking its path.
        // 1. black pawn 2 square forward; only 1 square forward available from start pos.
        b.set_square(E4, BLACK, PAWN);
        let moves = get_piece_moves(&mut b, &mut gs, E2, WHITE);
        assert_eq!(moves.len(), 1);
        assert!(moves.iter().any(|m| m.source == E2 && m.dest == E3));
        // 2. black pawn 1 square forward; no square available.
        b.set_square(E3, BLACK, PAWN);
        let moves = get_piece_moves(&mut b, &mut gs, E2, WHITE);
        assert_eq!(moves.len(), 0);
        // 3. remove the e4 black pawn; the white pawn should not be able to jump above e3 pawn.
        b.clear_square(E4, BLACK, PAWN);
        let moves = get_piece_moves(&mut b, &mut gs, E2, WHITE);
        assert_eq!(moves.len(), 0);
    }

    #[test]
    fn test_get_pawn_capture_moves() {
        let mut b = Board::new_empty();
        let mut gs = GameState::new();

        // Check that a pawn can take a piece diagonally.
        b.set_square(E2, WHITE, PAWN);
        let moves = get_piece_moves(&mut b, &mut gs, E2, WHITE);
        assert_eq!(moves.len(), 2);
        b.set_square(F3, BLACK, PAWN);
        let moves = get_piece_moves(&mut b, &mut gs, E2, WHITE);
        assert_eq!(moves.len(), 3);
        assert!(moves.iter().any(|m| m.source == E2 && m.dest == F3));
        b.set_square(D3, BLACK, PAWN);
        let moves = get_piece_moves(&mut b, &mut gs, E2, WHITE);
        assert_eq!(moves.len(), 4);
        assert!(moves.iter().any(|m| m.source == E2 && m.dest == F3));
        assert!(moves.iter().any(|m| m.source == E2 && m.dest == D3));
    }

    #[test]
    fn test_get_pawn_promotion_moves() {
        let mut b = Board::new_empty();
        let mut gs = GameState::new();

        // Check that a pawn moving to the last rank leads to queen promotion.
        // 1. by simply moving forward.
        b.set_square(A7, WHITE, PAWN);
        let moves = get_piece_moves(&mut b, &mut gs, A7, WHITE);
        assert_eq!(moves.len(), 1);
        let m = &moves[0];
        assert_eq!(m.source, A7);
        assert_eq!(m.dest, A8);
        assert_eq!(m.promotion, Some(QUEEN));
    }

    #[test]
    fn test_get_bishop_moves() {
        let mut b = Board::new_empty();
        let mut gs = GameState::new();

        // A bishop has maximum range when it's in a center square.
        b.set_square(D4, WHITE, BISHOP);
        let moves = get_piece_moves(&mut b, &mut gs, D4, WHITE);
        assert_eq!(moves.len(), 13);
        // Going top-right.
        assert!(moves.iter().any(|m| m.source == D4 && m.dest == E5));
        assert!(moves.iter().any(|m| m.source == D4 && m.dest == F6));
        assert!(moves.iter().any(|m| m.source == D4 && m.dest == G7));
        assert!(moves.iter().any(|m| m.source == D4 && m.dest == H8));
        // Going bottom-right.
        assert!(moves.iter().any(|m| m.source == D4 && m.dest == E3));
        assert!(moves.iter().any(|m| m.source == D4 && m.dest == F2));
        assert!(moves.iter().any(|m| m.source == D4 && m.dest == G1));
        // Going bottom-left.
        assert!(moves.iter().any(|m| m.source == D4 && m.dest == C3));
        assert!(moves.iter().any(|m| m.source == D4 && m.dest == B2));
        assert!(moves.iter().any(|m| m.source == D4 && m.dest == A1));
        // Going top-left.
        assert!(moves.iter().any(|m| m.source == D4 && m.dest == C5));
        assert!(moves.iter().any(|m| m.source == D4 && m.dest == B6));
        assert!(moves.iter().any(|m| m.source == D4 && m.dest == A7));

        // When blocking commit to one square with friendly piece, lose 2 moves.
        b.set_square(B2, WHITE, PAWN);
        assert_eq!(get_piece_moves(&mut b, &mut gs, D4, WHITE).len(), 11);

        // When blocking commit to one square with enemy piece, lose only 1 move.
        b.set_square(B2, BLACK, PAWN);
        assert_eq!(get_piece_moves(&mut b, &mut gs, D4, WHITE).len(), 12);
    }

    #[test]
    fn test_get_knight_moves() {
        let mut b = Board::new_empty();
        let mut gs = GameState::new();

        // A knight never has blocked commit; if it's in the center of the board, it can have up to
        // 8 moves.
        b.set_square(D4, WHITE, KNIGHT);
        assert_eq!(get_piece_moves(&mut b, &mut gs, D4, WHITE).len(), 8);

        // If on a side if has only 4 moves.
        b.set_square(A4, WHITE, KNIGHT);
        assert_eq!(get_piece_moves(&mut b, &mut gs, A4, WHITE).len(), 4);

        // And in a corner, only 2 moves.
        b.set_square(A1, WHITE, KNIGHT);
        assert_eq!(get_piece_moves(&mut b, &mut gs, A1, WHITE).len(), 2);

        // Add 2 friendly pieces and it is totally blocked.
        b.set_square(B3, WHITE, PAWN);
        b.set_square(C2, WHITE, PAWN);
        assert_eq!(get_piece_moves(&mut b, &mut gs, A1, WHITE).len(), 0);
    }

    #[test]
    fn test_get_rook_moves() {
        let mut b = Board::new_empty();
        let mut gs = GameState::new();

        b.set_square(D4, WHITE, ROOK);
        assert_eq!(get_piece_moves(&mut b, &mut gs, D4, WHITE).len(), 14);
        b.set_square(D6, BLACK, PAWN);
        assert_eq!(get_piece_moves(&mut b, &mut gs, D4, WHITE).len(), 12);
        b.set_square(D6, WHITE, PAWN);
        assert_eq!(get_piece_moves(&mut b, &mut gs, D4, WHITE).len(), 11);
    }

    #[test]
    fn test_get_queen_moves() {
        let mut b = Board::new_empty();
        let mut gs = GameState::new();

        b.set_square(D4, WHITE, QUEEN);
        assert_eq!(get_piece_moves(&mut b, &mut gs, D4, WHITE).len(), 14 + 13);
    }

    #[test]
    fn test_get_king_moves() {
        let mut gs = GameState::new();

        // King can move 1 square in any direction.
        let mut b = Board::new_empty();
        b.set_square(D4, WHITE, KING);
        assert_eq!(get_piece_moves(&mut b, &mut gs, D4, WHITE).len(), 8);
        b.set_square(E5, WHITE, PAWN);
        assert_eq!(get_piece_moves(&mut b, &mut gs, D4, WHITE).len(), 7);

        // If castling is available, other moves are possible: 5 moves + 2 castles.
        let mut b = Board::new_empty();
        b.set_square(E1, WHITE, KING);
        b.set_square(A1, WHITE, ROOK);
        b.set_square(H1, WHITE, ROOK);
        assert_eq!(get_piece_moves(&mut b, &mut gs, E1, WHITE).len(), 5 + 2);

        // Castling works as well for black.
        gs.color = BLACK;
        b.set_square(E8, BLACK, KING);
        b.set_square(A8, BLACK, ROOK);
        b.set_square(H8, BLACK, ROOK);
        assert_eq!(get_piece_moves(&mut b, &mut gs, E8, BLACK).len(), 5 + 2);
    }

    #[test]
    fn test_is_illegal() {
        let mut b = Board::new_empty();
        let mut gs = GameState::new();
        gs.castling = 0;

        // Place white's king on first rank.
        b.set_square(E1, WHITE, KING);
        // Place black rook in second rank: king can only move left or right.
        b.set_square(H2, BLACK, ROOK);
        // Check that the king can't go to a rook controlled square.
        assert!(is_illegal(&mut b, &mut gs, &mut Move::new(E1, E2)));
        assert!(is_illegal(&mut b, &mut gs, &mut Move::new(E1, D2)));
        assert!(is_illegal(&mut b, &mut gs, &mut Move::new(E1, F2)));
        assert!(!is_illegal(&mut b, &mut gs, &mut Move::new(E1, D1)));
        assert!(!is_illegal(&mut b, &mut gs, &mut Move::new(E1, F1)));
        let all_wh_moves = get_piece_moves(&mut b, &mut gs, E1, WHITE);
        assert_eq!(all_wh_moves.len(), 2);
    }

    #[test]
    fn test_get_king_moves_legality() {
        let mut b = Board::new_empty();
        let mut gs = GameState::new();

        // Place white's king on first rank.
        b.set_square(E1, WHITE, KING);
        // Place black rook in second rank: king can only move left or right.
        b.set_square(H2, BLACK, ROOK);
        // No castling available.
        gs.castling = 0;
        // 5 moves in absolute but only 2 are legal.
        let all_wh_moves = get_piece_moves(&mut b, &mut gs, E1, WHITE);
        assert_eq!(all_wh_moves.len(), 2);
    }
}

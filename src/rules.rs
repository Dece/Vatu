//! Functions to determine legal moves.

use crate::board::*;

/// Get a list of legal moves for all pieces of either white or black.
pub fn get_legal_player_moves(board: &Board, color: u8) -> Vec<Move> {
    let mut moves = vec!();
    for r in 0..8 {
        for f in 0..8 {
            if is_color(get_square(board, (f, r)), color) {
                moves.append(&mut get_legal_piece_moves(board, (f, r)));
            }
        }
    }
    moves
}

/// Get a list of legal moves for the piece at position `at`.
pub fn get_legal_piece_moves(board: &Board, at: Pos) -> Vec<Move> {
    match get_square(board, at) {
        p if is_piece(p, SQ_P) => get_legal_pawn_moves(board, at, p),
        p if is_piece(p, SQ_B) => get_legal_bishop_moves(board, at, p),
        p if is_piece(p, SQ_N) => get_legal_knight_moves(board, at, p),
        p if is_piece(p, SQ_R) => get_legal_rook_moves(board, at, p),
        p if is_piece(p, SQ_Q) => get_legal_queen_moves(board, at, p),
        p if is_piece(p, SQ_K) => get_legal_king_moves(board, at, p),
        _ => vec!(),
    }
}

fn get_legal_pawn_moves(board: &Board, at: Pos, piece: u8) -> Vec<Move> {
    let (f, r) = at;
    let mut moves = vec!();
    let movement: i8 = if is_white(piece) { 1 } else { -1 };
    // Check 1 or 2 square forward.
    let move_len = if (is_white(piece) && r == 1) || (is_black(piece) && r == 6) { 2 } else { 1 };
    for i in 1..=move_len {
        let forward_r = r + movement * i;
        if movement > 0 && forward_r > POS_MAX {
            return moves
        }
        if movement < 0 && forward_r < POS_MIN {
            return moves
        }
        let forward: Pos = (f, forward_r);
        if is_empty(board, forward) {
            moves.push((at, forward))
        }
        // Check diagonals for pieces to attack.
        if i == 1 {
            let df = f - 1;
            if df >= POS_MIN {
                let diag: Pos = (df, forward_r);
                if let Some(m) = move_on_enemy(piece, at, get_square(board, diag), diag) {
                    moves.push(m);
                }
            }
            let df = f + 1;
            if df <= POS_MAX {
                let diag: Pos = (df, forward_r);
                if let Some(m) = move_on_enemy(piece, at, get_square(board, diag), diag) {
                    moves.push(m);
                }
            }
        }
        // TODO en passant
    }
    moves
}

fn get_legal_bishop_moves(board: &Board, at: Pos, piece: u8) -> Vec<Move> {
    let (f, r) = at;
    let mut sight = [true; 4];  // Store diagonals where a piece blocks sight.
    let mut moves = vec!();
    for dist in 1..=7 {
        for (dir, offset) in [(1, -1), (1, 1), (-1, 1), (-1, -1)].iter().enumerate() {
            if !sight[dir] {
                continue
            }
            let p = (f + offset.0 * dist, r + offset.1 * dist);
            if !is_valid_pos(p) {
                continue
            }
            if is_empty(board, p) {
                moves.push((at, p));
            } else {
                if let Some(m) = move_on_enemy(piece, at, get_square(board, p), p) {
                    moves.push(m);
                }
                sight[dir] = false;  // Stop looking in that direction.
            }
        }
    }
    moves
}

fn get_legal_knight_moves(board: &Board, at: Pos, piece: u8) -> Vec<Move> {
    let (f, r) = at;
    let mut moves = vec!();
    for offset in [(1, 2), (2, 1), (2, -1), (1, -2), (-1, -2), (-2, -1), (-2, 1), (-1, 2)].iter() {
        let p = (f + offset.0, r + offset.1);
        if !is_valid_pos(p) {
            continue
        }
        if is_empty(board, p) {
            moves.push((at, p));
        } else if let Some(m) = move_on_enemy(piece, at, get_square(board, p), p) {
            moves.push(m);
        }
    }
    moves
}

fn get_legal_rook_moves(board: &Board, at: Pos, piece: u8) -> Vec<Move> {
    let (f, r) = at;
    let mut moves = vec!();
    let mut sight = [true; 4];  // Store lines where a piece blocks sight.
    for dist in 1..=7 {
        for (dir, offset) in [(0, 1), (1, 0), (0, -1), (-1, 0)].iter().enumerate() {
            if !sight[dir] {
                continue
            }
            let p = (f + offset.0 * dist, r + offset.1 * dist);
            if !is_valid_pos(p) {
                continue
            }
            if is_empty(board, p) {
                moves.push((at, p));
            } else {
                if let Some(m) = move_on_enemy(piece, at, get_square(board, p), p) {
                    moves.push(m);
                }
                sight[dir] = false;  // Stop looking in that direction.
            }
        }
    }
    moves
}

fn get_legal_queen_moves(board: &Board, at: Pos, piece: u8) -> Vec<Move> {
    let mut moves = vec!();
    // Easy way to get queen moves, but may be a bit quicker if everything was rewritten here.
    moves.append(&mut get_legal_bishop_moves(board, at, piece));
    moves.append(&mut get_legal_rook_moves(board, at, piece));
    moves
}

fn get_legal_king_moves(board: &Board, at: Pos, piece: u8) -> Vec<Move> {
    let (f, r) = at;
    let mut moves = vec!();
    for offset in [(-1, 1), (0, 1), (1, 1), (-1, 0), (1, 0), (-1, -1), (0, -1), (1, -1)].iter() {
        let p = (f + offset.0, r + offset.1);
        if !is_valid_pos(p) {
            continue
        }
        if is_empty(board, p) {
            moves.push((at, p));
        } else if let Some(m) = move_on_enemy(piece, at, get_square(board, p), p) {
            moves.push(m);
        }
    }
    // TODO castling
    moves
}

/// Return a move from pos1 to pos2 if piece1 & piece2 are enemies.
fn move_on_enemy(piece1: u8, pos1: Pos, piece2: u8, pos2: Pos) -> Option<Move> {
    if (is_white(piece1) && is_black(piece2)) || (is_black(piece1) && is_white(piece2)) {
        Some((pos1, pos2))
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_legal_player_moves() {
        let b = new();
        // At first move, white has 16 pawn moves and 4 knight moves.
        let moves = get_legal_player_moves(&b, SQ_WH);
        assert_eq!(moves.len(), 20);
    }

    #[test]
    fn test_get_legal_pawn_moves() {
        let mut b = new_empty();

        // Check that a pawn (here white queen's pawn) can move forward if the road is free.
        set_square(&mut b, pos("d3"), SQ_WH_P);
        let moves = get_legal_piece_moves(&b, pos("d3"));
        assert!(moves.len() == 1 && moves.contains( &(pos("d3"), pos("d4")) ));

        // Check that a pawn (here white king's pawn) can move 2 square forward on first move.
        set_square(&mut b, pos("e2"), SQ_WH_P);
        let moves = get_legal_piece_moves(&b, pos("e2"));
        assert_eq!(moves.len(), 2);
        assert!(moves.contains( &(pos("e2"), pos("e3")) ));
        assert!(moves.contains( &(pos("e2"), pos("e4")) ));

        // Check that a pawn cannot move forward if a piece is blocking its path.
        // 1. black pawn 2 square forward:
        set_square(&mut b, pos("e4"), SQ_BL_P);
        let moves = get_legal_piece_moves(&b, pos("e2"));
        assert!(moves.len() == 1 && moves.contains( &(pos("e2"), pos("e3")) ));
        // 2. black pawn 1 square forward:
        set_square(&mut b, pos("e3"), SQ_BL_P);
        let moves = get_legal_piece_moves(&b, pos("e2"));
        assert_eq!(moves.len(), 0);

        // Check that a pawn can take a piece diagonally.
        set_square(&mut b, pos("f3"), SQ_BL_P);
        let moves = get_legal_piece_moves(&b, pos("e2"));
        assert!(moves.len() == 1 && moves.contains( &(pos("e2"), pos("f3")) ));
        set_square(&mut b, pos("d3"), SQ_BL_P);
        let moves = get_legal_piece_moves(&b, pos("e2"));
        assert_eq!(moves.len(), 2);
        assert!(moves.contains( &(pos("e2"), pos("f3")) ));
        assert!(moves.contains( &(pos("e2"), pos("d3")) ));
    }

    #[test]
    fn test_get_legal_bishop_moves() {
        let mut b = new_empty();

        // A bishop has maximum range when it's in a center square.
        set_square(&mut b, pos("d4"), SQ_WH_B);
        let moves = get_legal_piece_moves(&b, pos("d4"));
        assert_eq!(moves.len(), 13);
        // Going top-right.
        assert!(moves.contains( &(pos("d4"), pos("e5")) ));
        assert!(moves.contains( &(pos("d4"), pos("f6")) ));
        assert!(moves.contains( &(pos("d4"), pos("g7")) ));
        assert!(moves.contains( &(pos("d4"), pos("h8")) ));
        // Going bottom-right.
        assert!(moves.contains( &(pos("d4"), pos("e3")) ));
        assert!(moves.contains( &(pos("d4"), pos("f2")) ));
        assert!(moves.contains( &(pos("d4"), pos("g1")) ));
        // Going bottom-left.
        assert!(moves.contains( &(pos("d4"), pos("c3")) ));
        assert!(moves.contains( &(pos("d4"), pos("b2")) ));
        assert!(moves.contains( &(pos("d4"), pos("a1")) ));
        // Going top-left.
        assert!(moves.contains( &(pos("d4"), pos("c5")) ));
        assert!(moves.contains( &(pos("d4"), pos("b6")) ));
        assert!(moves.contains( &(pos("d4"), pos("a7")) ));

        // When blocking sight to one square with friendly piece, lose 2 moves.
        set_square(&mut b, pos("b2"), SQ_WH_P);
        assert_eq!(get_legal_piece_moves(&b, pos("d4")).len(), 11);

        // When blocking sight to one square with enemy piece, lose only 1 move.
        set_square(&mut b, pos("b2"), SQ_BL_P);
        assert_eq!(get_legal_piece_moves(&b, pos("d4")).len(), 12);
    }

    #[test]
    fn test_get_legal_knight_moves() {
        let mut b = new_empty();

        // A knight never has blocked sight; if it's in the center of the board, it can have up to
        // 8 moves.
        set_square(&mut b, pos("d4"), SQ_WH_N);
        assert_eq!(get_legal_piece_moves(&b, pos("d4")).len(), 8);

        // If on a side if has only 4 moves.
        set_square(&mut b, pos("a4"), SQ_WH_N);
        assert_eq!(get_legal_piece_moves(&b, pos("a4")).len(), 4);

        // And in a corner, only 2 moves.
        set_square(&mut b, pos("a1"), SQ_WH_N);
        assert_eq!(get_legal_piece_moves(&b, pos("a1")).len(), 2);

        // Add 2 friendly pieces and it is totally blocked.
        set_square(&mut b, pos("b3"), SQ_WH_P);
        set_square(&mut b, pos("c2"), SQ_WH_P);
        assert_eq!(get_legal_piece_moves(&b, pos("a1")).len(), 0);
    }

    #[test]
    fn test_get_legal_rook_moves() {
        let mut b = new_empty();

        set_square(&mut b, pos("d4"), SQ_WH_R);
        assert_eq!(get_legal_piece_moves(&b, pos("d4")).len(), 14);
        set_square(&mut b, pos("d6"), SQ_BL_P);
        assert_eq!(get_legal_piece_moves(&b, pos("d4")).len(), 12);
        set_square(&mut b, pos("d6"), SQ_WH_P);
        assert_eq!(get_legal_piece_moves(&b, pos("d4")).len(), 11);
    }

    #[test]
    fn test_get_legal_queen_moves() {
        let mut b = new_empty();

        set_square(&mut b, pos("d4"), SQ_WH_Q);
        assert_eq!(get_legal_piece_moves(&b, pos("d4")).len(), 14 + 13);  // Bishop + rook moves.
    }

    #[test]
    fn test_get_legal_king_moves() {
        let mut b = new_empty();

        set_square(&mut b, pos("d4"), SQ_WH_K);
        assert_eq!(get_legal_piece_moves(&b, pos("d4")).len(), 8);
        set_square(&mut b, pos("e5"), SQ_WH_P);
        assert_eq!(get_legal_piece_moves(&b, pos("d4")).len(), 7);
    }
}

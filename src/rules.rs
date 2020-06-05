//! Functions to determine legal moves.

use crate::board::*;

/// Characteristics of the state of a game.
///
/// It does not include various parameters such as clocks that are
/// more aimed for engine analysis than typical rules checking.
#[derive(Debug, Clone)]
pub struct GameState {
    pub color: u8,
    pub castling: u8,
    pub en_passant: Option<Pos>,
    pub halfmove: i32,
    pub fullmove: i32,
}

impl GameState {
    pub const fn new() -> GameState {
        GameState {
            color: SQ_WH,
            castling: CASTLING_MASK,
            en_passant: None,
            halfmove: 0,
            fullmove: 1,
        }
    }
}

pub const CASTLING_WH_K: u8 = 0b00000001;
pub const CASTLING_WH_Q: u8 = 0b00000010;
pub const CASTLING_BL_K: u8 = 0b00000100;
pub const CASTLING_BL_Q: u8 = 0b00001000;
pub const CASTLING_MASK: u8 = 0b00001111;

pub const START_WH_K_POS: Pos = pos("e1");
pub const START_BL_K_POS: Pos = pos("e8");

/// A movement, with before/after positions and optional promotion.
pub type Move = (Pos, Pos, Option<u8>);

/// Apply a move `m` to copies to `board` and `game_state`.
pub fn apply_move(board: &Board, game_state: &GameState, m: &Move) -> (Board, GameState) {
    let mut new_board = board.clone();
    let mut new_state = game_state.clone();
    apply_move_to(&mut new_board, &mut new_state, m);
    (new_board, new_state)
}

pub fn apply_move_to(board: &mut Board, game_state: &mut GameState, m: &Move) {
    apply_move_to_board(board, game_state, m);
    apply_move_to_state(game_state, m);
}

/// Apply a move `m` into `board`.
pub fn apply_move_to_board(board: &mut Board, _game_state: &GameState, m: &Move) {
    // TODO handle castling
    set_square(board, &m.1, get_square(board, &m.0));
    clear_square(board, &m.0)
}

pub fn apply_move_to_state(game_state: &mut GameState, _m: &Move) {
    game_state.color = opposite(game_state.color);
    // TODO handle castling
}

/// Get a list of legal moves for all pieces of this color.
pub fn get_player_legal_moves(board: &Board, game_state: &GameState) -> Vec<Move> {
    filter_illegal_moves(board, game_state, get_player_moves(board, game_state))
}

/// Get a list of moves for all pieces of this color.
pub fn get_player_moves(board: &Board, game_state: &GameState) -> Vec<Move> {
    let mut moves = vec!();
    for r in 0..8 {
        for f in 0..8 {
            let p = (f, r);
            if is_empty(board, &p) {
                continue
            }
            if is_color(get_square(board, &p), game_state.color) {
                moves.append(&mut get_piece_moves(board, &p, game_state));
            }
        }
    }
    moves
}

/// Get a list of moves for the piece at position `at`.
pub fn get_piece_moves(board: &Board, at: &Pos, game_state: &GameState) -> Vec<Move> {
    match get_square(board, at) {
        p if is_piece(p, SQ_P) => get_pawn_moves(board, at, p),
        p if is_piece(p, SQ_B) => get_bishop_moves(board, at, p),
        p if is_piece(p, SQ_N) => get_knight_moves(board, at, p),
        p if is_piece(p, SQ_R) => get_rook_moves(board, at, p),
        p if is_piece(p, SQ_Q) => get_queen_moves(board, at, p),
        p if is_piece(p, SQ_K) => get_king_moves(board, at, p, game_state),
        _ => vec!(),
    }
}

fn get_pawn_moves(board: &Board, at: &Pos, piece: u8) -> Vec<Move> {
    let (f, r) = *at;
    let mut moves = vec!();
    // Direction: positive for white, negative for black.
    let dir: i8 = if is_white(piece) { 1 } else { -1 };
    // Check 1 or 2 square forward.
    let move_len = if (is_white(piece) && r == 1) || (is_black(piece) && r == 6) { 2 } else { 1 };
    for i in 1..=move_len {
        let forward_r = r + dir * i;
        if dir > 0 && forward_r > POS_MAX {
            return moves
        }
        if dir < 0 && forward_r < POS_MIN {
            return moves
        }
        let forward: Pos = (f, forward_r);
        // If forward square is empty (and we are not jumping over an occupied square), add it.
        if is_empty(board, &forward) && (i == 1 || is_empty(board, &(f, forward_r - dir))) {
            // Pawns that get to the opposite rank automatically promote as queens.
            let prom = if (dir > 0 && forward_r == POS_MAX) || (dir < 0 && forward_r == POS_MIN) {
                Some(SQ_Q)
            } else {
                None
            };
            moves.push((*at, forward, prom))
        }
        // Check diagonals for pieces to attack.
        if i == 1 {
            let df = f - 1;
            if df >= POS_MIN {
                let diag: Pos = (df, forward_r);
                if let Some(m) = move_on_enemy(piece, at, get_square(board, &diag), &diag) {
                    moves.push(m);
                }
            }
            let df = f + 1;
            if df <= POS_MAX {
                let diag: Pos = (df, forward_r);
                if let Some(m) = move_on_enemy(piece, at, get_square(board, &diag), &diag) {
                    moves.push(m);
                }
            }
        }
        // TODO en passant
    }
    moves
}

fn get_bishop_moves(board: &Board, at: &Pos, piece: u8) -> Vec<Move> {
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
            if is_empty(board, &p) {
                moves.push((*at, p, None));
            } else {
                if let Some(m) = move_on_enemy(piece, at, get_square(board, &p), &p) {
                    moves.push(m);
                }
                sight[dir] = false;  // Stop looking in that direction.
            }
        }
    }
    moves
}

fn get_knight_moves(board: &Board, at: &Pos, piece: u8) -> Vec<Move> {
    let (f, r) = at;
    let mut moves = vec!();
    for offset in [(1, 2), (2, 1), (2, -1), (1, -2), (-1, -2), (-2, -1), (-2, 1), (-1, 2)].iter() {
        let p = (f + offset.0, r + offset.1);
        if !is_valid_pos(p) {
            continue
        }
        if is_empty(board, &p) {
            moves.push((*at, p, None));
        } else if let Some(m) = move_on_enemy(piece, at, get_square(board, &p), &p) {
            moves.push(m);
        }
    }
    moves
}

fn get_rook_moves(board: &Board, at: &Pos, piece: u8) -> Vec<Move> {
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
            if is_empty(board, &p) {
                moves.push((*at, p, None));
            } else {
                if let Some(m) = move_on_enemy(piece, at, get_square(board, &p), &p) {
                    moves.push(m);
                }
                sight[dir] = false;  // Stop looking in that direction.
            }
        }
    }
    moves
}

fn get_queen_moves(board: &Board, at: &Pos, piece: u8) -> Vec<Move> {
    let mut moves = vec!();
    // Easy way to get queen moves, but may be a bit quicker if everything was rewritten here.
    moves.append(&mut get_bishop_moves(board, at, piece));
    moves.append(&mut get_rook_moves(board, at, piece));
    moves
}

fn get_king_moves(board: &Board, at: &Pos, piece: u8, _game_state: &GameState) -> Vec<Move> {
    let (f, r) = at;
    let mut moves = vec!();
    for offset in [(-1, 1), (0, 1), (1, 1), (-1, 0), (1, 0), (-1, -1), (0, -1), (1, -1)].iter() {
        let p = (f + offset.0, r + offset.1);
        if !is_valid_pos(p) {
            continue
        }
        if is_empty(board, &p) {
            moves.push((*at, p, None));
        } else if let Some(m) = move_on_enemy(piece, at, get_square(board, &p), &p) {
            moves.push(m);
        }
    }
    // TODO castling
    moves
}

/// Return a move from pos1 to pos2 if piece1 & piece2 are enemies.
fn move_on_enemy(piece1: u8, pos1: &Pos, piece2: u8, pos2: &Pos) -> Option<Move> {
    let color1 = get_color(piece1);
    if is_color(piece2, opposite(color1)) {
        // Automatic queen promotion for pawns moving to the opposite rank.
        let prom = if
            is_piece(piece1, SQ_P) &&
            ((is_white(piece1) && pos2.1 == POS_MAX) || (is_black(piece1) && pos2.1 == POS_MIN))
        {
            Some(SQ_Q)
        } else {
            None
        };
        Some((*pos1, *pos2, prom))
    } else {
        None
    }
}

/// Return an iterator filtering out illegal moves from given list.
///
/// Pass color of moving player to avoid checking it for every move.
fn filter_illegal_moves(board: &Board, game_state: &GameState, moves: Vec<Move>) -> Vec<Move> {
    let king_p = find_king(board, game_state.color);
    moves.into_iter().filter(|m| {
        // If king moved, use its new position.
        let king_p = if m.0 == king_p { m.1 } else { king_p };
        let (new_board, new_state) = apply_move(board, game_state, m);
        // Check if the move makes the player king in check.
        !is_attacked(&new_board, &new_state, &king_p)
    }).collect()
}

/// Return true if the piece at position `at` is attacked.
///
/// Beware that the game state must be coherent with the analysed
/// square, i.e. if the piece at `at` is white, the game state should
/// tell that it is black turn.
fn is_attacked(board: &Board, game_state: &GameState, at: &Pos) -> bool {
    let enemy_moves = get_player_moves(board, game_state);
    for m in enemy_moves.iter() {
        if *at == m.1 {
            return true
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_apply_move_to_board() {
        let mut b = new_empty();
        let s = GameState::new();

        // Put 2 enemy knights on board.
        set_square(&mut b, &pos("d4"), SQ_WH_N);
        set_square(&mut b, &pos("f4"), SQ_BL_N);
        // Move white knight in a position attacked by black knight.
        apply_move_to_board(&mut b, &s, &(pos("d4"), pos("e6"), None));
        assert_eq!(get_square(&b, &pos("d4")), SQ_E);
        assert_eq!(get_square(&b, &pos("e6")), SQ_WH_N);
        assert_eq!(num_pieces(&b), 2);
        // Sack it with black knight
        apply_move_to_board(&mut b, &s, &(pos("f4"), pos("e6"), None));
        assert_eq!(get_square(&b, &pos("e6")), SQ_BL_N);
        assert_eq!(num_pieces(&b), 1);
    }

    #[test]
    fn test_get_player_moves() {
        let b = new();
        let s = GameState::new();

        // At first move, white has 16 pawn moves and 4 knight moves.
        let moves = get_player_moves(&b, &s);
        assert_eq!(moves.len(), 20);
    }

    #[test]
    fn test_get_pawn_moves() {
        let mut b = new_empty();
        let s = GameState::new();

        // Check that a pawn (here white queen's pawn) can move forward if the road is free.
        set_square(&mut b, &pos("d3"), SQ_WH_P);
        let moves = get_piece_moves(&b, &pos("d3"), &s);
        assert!(moves.len() == 1 && moves.contains( &(pos("d3"), pos("d4"), None) ));

        // Check that a pawn (here white king's pawn) can move 2 square forward on first move.
        set_square(&mut b, &pos("e2"), SQ_WH_P);
        let moves = get_piece_moves(&b, &pos("e2"), &s);
        assert_eq!(moves.len(), 2);
        assert!(moves.contains( &(pos("e2"), pos("e3"), None) ));
        assert!(moves.contains( &(pos("e2"), pos("e4"), None) ));

        // Check that a pawn cannot move forward if a piece is blocking its path.
        // 1. black pawn 2 square forward; only 1 square forward available from start pos.
        set_square(&mut b, &pos("e4"), SQ_BL_P);
        let moves = get_piece_moves(&b, &pos("e2"), &s);
        assert!(moves.len() == 1 && moves.contains( &(pos("e2"), pos("e3"), None) ));
        // 2. black pawn 1 square forward; no square available.
        set_square(&mut b, &pos("e3"), SQ_BL_P);
        let moves = get_piece_moves(&b, &pos("e2"), &s);
        assert_eq!(moves.len(), 0);
        // 3. remove the e4 black pawn; the white pawn should not be able to jump above e3 pawn.
        clear_square(&mut b, &pos("e4"));
        let moves = get_piece_moves(&b, &pos("e2"), &s);
        assert_eq!(moves.len(), 0);

        // Check that a pawn can take a piece diagonally.
        set_square(&mut b, &pos("f3"), SQ_BL_P);
        let moves = get_piece_moves(&b, &pos("e2"), &s);
        assert!(moves.len() == 1 && moves.contains( &(pos("e2"), pos("f3"), None) ));
        set_square(&mut b, &pos("d3"), SQ_BL_P);
        let moves = get_piece_moves(&b, &pos("e2"), &s);
        assert_eq!(moves.len(), 2);
        assert!(moves.contains( &(pos("e2"), pos("f3"), None) ));
        assert!(moves.contains( &(pos("e2"), pos("d3"), None) ));

        // Check that a pawn moving to the last rank leads to queen promotion.
        // 1. by simply moving forward.
        set_square(&mut b, &pos("a7"), SQ_WH_P);
        let moves = get_piece_moves(&b, &pos("a7"), &s);
        assert!(moves.len() == 1 && moves.contains( &(pos("a7"), pos("a8"), Some(SQ_Q)) ));
    }

    #[test]
    fn test_get_bishop_moves() {
        let mut b = new_empty();
        let s = GameState::new();

        // A bishop has maximum range when it's in a center square.
        set_square(&mut b, &pos("d4"), SQ_WH_B);
        let moves = get_piece_moves(&b, &pos("d4"), &s);
        assert_eq!(moves.len(), 13);
        // Going top-right.
        assert!(moves.contains( &(pos("d4"), pos("e5"), None) ));
        assert!(moves.contains( &(pos("d4"), pos("f6"), None) ));
        assert!(moves.contains( &(pos("d4"), pos("g7"), None) ));
        assert!(moves.contains( &(pos("d4"), pos("h8"), None) ));
        // Going bottom-right.
        assert!(moves.contains( &(pos("d4"), pos("e3"), None) ));
        assert!(moves.contains( &(pos("d4"), pos("f2"), None) ));
        assert!(moves.contains( &(pos("d4"), pos("g1"), None) ));
        // Going bottom-left.
        assert!(moves.contains( &(pos("d4"), pos("c3"), None) ));
        assert!(moves.contains( &(pos("d4"), pos("b2"), None) ));
        assert!(moves.contains( &(pos("d4"), pos("a1"), None) ));
        // Going top-left.
        assert!(moves.contains( &(pos("d4"), pos("c5"), None) ));
        assert!(moves.contains( &(pos("d4"), pos("b6"), None) ));
        assert!(moves.contains( &(pos("d4"), pos("a7"), None) ));

        // When blocking sight to one square with friendly piece, lose 2 moves.
        set_square(&mut b, &pos("b2"), SQ_WH_P);
        assert_eq!(get_piece_moves(&b, &pos("d4"), &s).len(), 11);

        // When blocking sight to one square with enemy piece, lose only 1 move.
        set_square(&mut b, &pos("b2"), SQ_BL_P);
        assert_eq!(get_piece_moves(&b, &pos("d4"), &s).len(), 12);
    }

    #[test]
    fn test_get_knight_moves() {
        let mut b = new_empty();
        let s = GameState::new();

        // A knight never has blocked sight; if it's in the center of the board, it can have up to
        // 8 moves.
        set_square(&mut b, &pos("d4"), SQ_WH_N);
        assert_eq!(get_piece_moves(&b, &pos("d4"), &s).len(), 8);

        // If on a side if has only 4 moves.
        set_square(&mut b, &pos("a4"), SQ_WH_N);
        assert_eq!(get_piece_moves(&b, &pos("a4"), &s).len(), 4);

        // And in a corner, only 2 moves.
        set_square(&mut b, &pos("a1"), SQ_WH_N);
        assert_eq!(get_piece_moves(&b, &pos("a1"), &s).len(), 2);

        // Add 2 friendly pieces and it is totally blocked.
        set_square(&mut b, &pos("b3"), SQ_WH_P);
        set_square(&mut b, &pos("c2"), SQ_WH_P);
        assert_eq!(get_piece_moves(&b, &pos("a1"), &s).len(), 0);
    }

    #[test]
    fn test_get_rook_moves() {
        let mut b = new_empty();
        let s = GameState::new();

        set_square(&mut b, &pos("d4"), SQ_WH_R);
        assert_eq!(get_piece_moves(&b, &pos("d4"), &s).len(), 14);
        set_square(&mut b, &pos("d6"), SQ_BL_P);
        assert_eq!(get_piece_moves(&b, &pos("d4"), &s).len(), 12);
        set_square(&mut b, &pos("d6"), SQ_WH_P);
        assert_eq!(get_piece_moves(&b, &pos("d4"), &s).len(), 11);
    }

    #[test]
    fn test_get_queen_moves() {
        let mut b = new_empty();
        let s = GameState::new();

        set_square(&mut b, &pos("d4"), SQ_WH_Q);
        assert_eq!(get_piece_moves(&b, &pos("d4"), &s).len(), 14 + 13);  // Bishop + rook moves.
    }

    #[test]
    fn test_get_king_moves() {
        let mut b = new_empty();
        let s = GameState::new();

        set_square(&mut b, &pos("d4"), SQ_WH_K);
        assert_eq!(get_piece_moves(&b, &pos("d4"), &s).len(), 8);
        set_square(&mut b, &pos("e5"), SQ_WH_P);
        assert_eq!(get_piece_moves(&b, &pos("d4"), &s).len(), 7);
    }

    #[test]
    fn test_filter_illegal_moves() {
        let mut b = new_empty();
        let s = GameState::new();

        // Place white's king on first rank.
        set_square(&mut b, &pos("e1"), SQ_WH_K);
        // Place black rook in second rank: king can only move left or right.
        set_square(&mut b, &pos("h2"), SQ_BL_R);
        let all_wh_moves = get_piece_moves(&b, &pos("e1"), &s);
        assert_eq!(all_wh_moves.len(), 5);
        assert_eq!(filter_illegal_moves(&b, &s, all_wh_moves).len(), 2);
    }

    #[test]
    fn test_is_attacked() {
        let mut b = new_empty();
        let mut s = GameState::new();
        s.color = SQ_BL;

        // Place a black rook in white pawn's file.
        set_square(&mut b, &pos("d4"), SQ_WH_P);
        set_square(&mut b, &pos("d6"), SQ_BL_R);
        assert!(is_attacked(&b, &s, &pos("d4")));
        // Move the rook on another file, no more attack.
        apply_move_to(&mut b, &mut s, &(pos("d6"), pos("e6"), None));
        assert!(!is_attacked(&b, &s, &pos("d4")));
    }
}

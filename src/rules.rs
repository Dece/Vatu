//! Functions to determine legal moves.

use crate::board::*;

/// Characteristics of the state of a game.
///
/// It does not include various parameters such as clocks that are
/// more aimed for engine analysis than typical rules checking.
///
/// - `color`: current player's turn
/// - `castling`: which castling options are available; updated throughout the game.
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

pub const CASTLING_WH_K: u8    = 0b00000001;
pub const CASTLING_WH_Q: u8    = 0b00000010;
pub const CASTLING_WH_MASK: u8 = 0b00000011;
pub const CASTLING_BL_K: u8    = 0b00000100;
pub const CASTLING_BL_Q: u8    = 0b00001000;
pub const CASTLING_BL_MASK: u8 = 0b00001100;
pub const CASTLING_K_MASK: u8  = 0b00000101;
pub const CASTLING_Q_MASK: u8  = 0b00001010;
pub const CASTLING_MASK: u8    = 0b00001111;
pub const CASTLING_SIDES: [(std::ops::RangeInclusive<i8>, u8); 2] =
    [(5..=6, CASTLING_K_MASK), (2..=3, CASTLING_Q_MASK)];

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

/// Update `board` and `game_state` to reflect the move `m`.
///
/// The board is updated with correct piece placement.
///
/// The game state is updated with the new player turn and the new
/// castling options.
pub fn apply_move_to(board: &mut Board, game_state: &mut GameState, m: &Move) {
    apply_move_to_board(board, m);
    apply_move_to_state(game_state, m);
    if let Some(castle) = get_castle(m) {
        match castle {
            CASTLING_WH_K | CASTLING_WH_Q => game_state.castling &= !CASTLING_WH_MASK,
            CASTLING_BL_K | CASTLING_BL_Q => game_state.castling &= !CASTLING_BL_MASK,
            _ => {}
        };
    }
}

/// Apply a move `m` into `board`.
pub fn apply_move_to_board(board: &mut Board, m: &Move) {
    if let Some(castle) = get_castle(m) {
        match castle {
            CASTLING_WH_K => {
                move_piece(board, &START_WH_K_POS, &pos("g1"));
                move_piece(board, &pos("h1"), &pos("f1"));
            }
            CASTLING_WH_Q => {
                move_piece(board, &START_WH_K_POS, &pos("c1"));
                move_piece(board, &pos("a1"), &pos("d1"));
            }
            CASTLING_BL_K => {
                move_piece(board, &START_BL_K_POS, &pos("g8"));
                move_piece(board, &pos("h8"), &pos("f8"));
            }
            CASTLING_BL_Q => {
                move_piece(board, &START_BL_K_POS, &pos("c8"));
                move_piece(board, &pos("a8"), &pos("d8"));
            }
            _ => {}
        }
    } else {
        move_piece(board, &m.0, &m.1);
    }
}

/// Update `game_state` with the move `m`.
///
/// This only updates the player turn. Castling should be updated in a
/// context where the corresponding board is available.
pub fn apply_move_to_state(game_state: &mut GameState, _m: &Move) {
    game_state.color = opposite(game_state.color);
}

/// Get the corresponding castling flag for this move.
pub fn get_castle(m: &Move) -> Option<u8> {
    if m.0 == pos("e1") {
        if m.1 == pos("c1") {
            Some(CASTLING_WH_Q)
        } else if m.1 == pos("g1") {
            Some(CASTLING_WH_K)
        } else {
            None
        }
    } else if m.0 == pos("e8") {
        if m.1 == pos("c8") {
            Some(CASTLING_BL_Q)
        } else if m.1 == pos("g8") {
            Some(CASTLING_BL_K)
        } else {
            None
        }
    } else {
        None
    }
}

/// Get the move for this castle.
pub fn get_castle_move(castle: u8) -> Move {
    match castle {
        CASTLING_WH_Q => (pos("e1"), pos("c1"), None),
        CASTLING_WH_K => (pos("e1"), pos("g1"), None),
        CASTLING_BL_Q => (pos("e8"), pos("c8"), None),
        CASTLING_BL_K => (pos("e8"), pos("g8"), None),
        _ => panic!("Illegal castling requested: {:08b}", castle),
    }
}

/// Get a list of moves for all pieces of the playing color.
///
/// If `commit` is false, do not check for illegal moves. This is used
/// to avoid endless recursion when checking if a P move is illegal,
/// as it needs to check all possible following enemy moves, e.g. to
/// see if P's king can be taken. Consider a call with true `commit` as
/// a collection of attacked squares instead of legal move collection.
pub fn get_player_moves(board: &Board, game_state: &GameState, commit: bool) -> Vec<Move> {
    let mut moves = vec!();
    for r in 0..8 {
        for f in 0..8 {
            let p = (f, r);
            if is_empty(board, &p) {
                continue
            }
            if is_color(get_square(board, &p), game_state.color) {
                moves.append(&mut get_piece_moves(board, &p, game_state, commit));
            }
        }
    }
    moves
}

/// Get a list of moves for the piece at position `at`.
pub fn get_piece_moves(board: &Board, at: &Pos, game_state: &GameState, commit: bool) -> Vec<Move> {
    match get_square(board, at) {
        p if is_piece(p, SQ_P) => get_pawn_moves(board, at, p, game_state, commit),
        p if is_piece(p, SQ_B) => get_bishop_moves(board, at, p, game_state, commit),
        p if is_piece(p, SQ_N) => get_knight_moves(board, at, p, game_state, commit),
        p if is_piece(p, SQ_R) => get_rook_moves(board, at, p, game_state, commit),
        p if is_piece(p, SQ_Q) => get_queen_moves(board, at, p, game_state, commit),
        p if is_piece(p, SQ_K) => get_king_moves(board, at, p, game_state, commit),
        _ => vec!(),
    }
}

fn get_pawn_moves(
    board: &Board,
    at: &Pos,
    piece: u8,
    game_state: &GameState,
    commit: bool,
) -> Vec<Move> {
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
            let m = (*at, forward, prom);
            if can_register(commit, board, game_state, &m) {
                moves.push(m);
            }
        }
        // Check diagonals for pieces to attack.
        if i == 1 {
            // First diagonal.
            let df = f - 1;
            if df >= POS_MIN {
                let diag: Pos = (df, forward_r);
                if let Some(m) = move_on_enemy(piece, at, get_square(board, &diag), &diag) {
                    if can_register(commit, board, game_state, &m) {
                        moves.push(m);
                    }
                }
            }
            // Second diagonal.
            let df = f + 1;
            if df <= POS_MAX {
                let diag: Pos = (df, forward_r);
                if let Some(m) = move_on_enemy(piece, at, get_square(board, &diag), &diag) {
                    if can_register(commit, board, game_state, &m) {
                        moves.push(m);
                    }
                }
            }
        }
        // TODO en passant
    }
    moves
}

fn get_bishop_moves(
    board: &Board,
    at: &Pos,
    piece: u8,
    game_state: &GameState,
    commit: bool,
) -> Vec<Move> {
    let (f, r) = at;
    let mut views = [true; 4];  // Store diagonals where a piece blocks commit.
    let mut moves = vec!();
    for dist in 1..=7 {
        for (dir, offset) in [(1, -1), (1, 1), (-1, 1), (-1, -1)].iter().enumerate() {
            if !views[dir] {
                continue
            }
            let p = (f + offset.0 * dist, r + offset.1 * dist);
            if !is_valid_pos(p) {
                continue
            }
            if is_empty(board, &p) {
                let m = (*at, p, None);
                if can_register(commit, board, game_state, &m) {
                    moves.push(m);
                }
            } else {
                if let Some(m) = move_on_enemy(piece, at, get_square(board, &p), &p) {
                    if can_register(commit, board, game_state, &m) {
                        moves.push(m);
                    }
                }
                views[dir] = false;  // Stop looking in that direction.
            }
        }
    }
    moves
}

fn get_knight_moves(
    board: &Board,
    at: &Pos,
    piece: u8,
    game_state: &GameState,
    commit: bool,
) -> Vec<Move> {
    let (f, r) = at;
    let mut moves = vec!();
    for offset in [(1, 2), (2, 1), (2, -1), (1, -2), (-1, -2), (-2, -1), (-2, 1), (-1, 2)].iter() {
        let p = (f + offset.0, r + offset.1);
        if !is_valid_pos(p) {
            continue
        }
        if is_empty(board, &p) {
            let m = (*at, p, None);
            if can_register(commit, board, game_state, &m) {
                moves.push(m);
            }
        } else if let Some(m) = move_on_enemy(piece, at, get_square(board, &p), &p) {
            if can_register(commit, board, game_state, &m) {
                moves.push(m);
            }
        }
    }
    moves
}

fn get_rook_moves(
    board: &Board,
    at: &Pos,
    piece: u8,
    game_state: &GameState,
    commit: bool,
) -> Vec<Move> {
    let (f, r) = at;
    let mut moves = vec!();
    let mut views = [true; 4];  // Store lines where a piece blocks commit.
    for dist in 1..=7 {
        for (dir, offset) in [(0, 1), (1, 0), (0, -1), (-1, 0)].iter().enumerate() {
            if !views[dir] {
                continue
            }
            let p = (f + offset.0 * dist, r + offset.1 * dist);
            if !is_valid_pos(p) {
                continue
            }
            if is_empty(board, &p) {
                let m = (*at, p, None);
                if can_register(commit, board, game_state, &m) {
                    moves.push(m);
                }
            } else {
                if let Some(m) = move_on_enemy(piece, at, get_square(board, &p), &p) {
                    if can_register(commit, board, game_state, &m) {
                        moves.push(m);
                    }
                }
                views[dir] = false;  // Stop looking in that direction.
            }
        }
    }
    moves
}

fn get_queen_moves(
    board: &Board,
    at: &Pos,
    piece: u8,
    game_state: &GameState,
    commit: bool
) -> Vec<Move> {
    let mut moves = vec!();
    // Easy way to get queen moves, but may be a bit quicker if everything was rewritten here.
    moves.append(&mut get_bishop_moves(board, at, piece, game_state, commit));
    moves.append(&mut get_rook_moves(board, at, piece, game_state, commit));
    moves
}

fn get_king_moves(
    board: &Board,
    at: &Pos,
    piece: u8,
    game_state: &GameState,
    commit: bool
) -> Vec<Move> {
    let (f, r) = at;
    let mut moves = vec!();
    for offset in [(-1, 1), (0, 1), (1, 1), (-1, 0), (1, 0), (-1, -1), (0, -1), (1, -1)].iter() {
        let p = (f + offset.0, r + offset.1);
        if !is_valid_pos(p) {
            continue
        }
        if is_empty(board, &p) {
            let m = (*at, p, None);
            if can_register(commit, board, game_state, &m) {
                moves.push(m);
            }
        } else if let Some(m) = move_on_enemy(piece, at, get_square(board, &p), &p) {
            if can_register(commit, board, game_state, &m) {
                moves.push(m);
            }
        }
    }

    // Stop here for uncommitted moves.
    if !commit {
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
    let (castling_rank, castling_color_mask) = if is_white(game_state.color) {
        (0, CASTLING_WH_MASK)
    } else {
        (7, CASTLING_BL_MASK)
    };

    // Check for castling if the king is on its castling rank (R1) and is not in check (R4).
    if
        *r == castling_rank &&               // Part of R1 for the king.
        !is_attacked(board, game_state, at)  // R4
    {
        // Check for both castling sides.
        for (path, castling_side_mask) in CASTLING_SIDES.iter() {
            // Check for castling availability for this color and side.
            if ((game_state.castling & castling_color_mask) | castling_side_mask) != 0 {
                // R3, R5, R6: check that files on the way are empty and not attacked.
                let mut clear_path = true;
                for through_f in path.to_owned() {
                    let p = (through_f, castling_rank);
                    if !is_empty(board, &p) || is_illegal(board, game_state, &(*at, p, None)) {
                        clear_path = false;
                        break;
                    }
                }

                // If the path is clear, the castling can be done.
                if clear_path {
                    let castle = CASTLING_K_MASK & castling_color_mask;
                    let m = get_castle_move(castle);
                    if can_register(commit, board, game_state, &m) {
                        moves.push(m);
                    }
                }
            }
        }
    }
    moves
}

/// Return true if `commit` is false, or the move is not illegal,
///
/// Committing a move means that it can be safely played afterwards.
/// Sometimes it is not what is needed to accept a move in a collection
/// of moves, e.g. when simply checking if some moves would make a
/// previous move illegal.
#[inline]
fn can_register(commit: bool, board: &Board, game_state: &GameState, m: &Move) -> bool {
    !commit || !is_illegal(board, game_state, m)
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

/// Check if a move is illegal.
fn is_illegal(board: &Board, game_state: &GameState, m: &Move) -> bool {
    if let Some(king_p) = find_king(board, game_state.color) {
        // Rule 1: a move is illegal if the king ends up in check.
        // If king moves, use its new position.
        let king_p = if m.0 == king_p { m.1 } else { king_p };
        let mut hypothetic_board = board.clone();
        apply_move_to_board(&mut hypothetic_board, m);
        // Check if the move makes the player king in check.
        if is_attacked(&hypothetic_board, &game_state, &king_p) {
            return true
        }
    }
    false
}

/// Return true if the piece at position `at` is attacked.
///
/// Check all possible enemy moves and return true when one of them
/// ends up attacking the position.
///
/// Beware that the game state must be coherent with the analysed
/// square, i.e. if the piece at `at` is white, the game state should
/// tell that it is white turn. If the square at `at` is empty, simply
/// check if it is getting attacked by the opposite player.
fn is_attacked(board: &Board, game_state: &GameState, at: &Pos) -> bool {
    let mut enemy_game_state = game_state.clone();
    enemy_game_state.color = opposite(game_state.color);
    // Do not attempt to commit moves, just check for attacked squares.
    let enemy_moves = get_player_moves(board, &enemy_game_state, false);
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
    use crate::notation::parse_move;

    #[test]
    fn test_get_castle() {
        assert_eq!(get_castle(&parse_move("e1a1")), Some(CASTLING_WH_Q));
        assert_eq!(get_castle(&parse_move("e1h1")), Some(CASTLING_WH_K));
        assert_eq!(get_castle(&parse_move("e8a8")), Some(CASTLING_BL_Q));
        assert_eq!(get_castle(&parse_move("e8h8")), Some(CASTLING_BL_K));
        assert_eq!(get_castle(&parse_move("d2d4")), None);
    }

    #[test]
    fn test_apply_move_to_board() {
        let mut b = new_empty();

        // Put 2 enemy knights on board.
        set_square(&mut b, &pos("d4"), SQ_WH_N);
        set_square(&mut b, &pos("f4"), SQ_BL_N);
        // Move white knight in a position attacked by black knight.
        apply_move_to_board(&mut b, &(pos("d4"), pos("e6"), None));
        assert_eq!(get_square(&b, &pos("d4")), SQ_E);
        assert_eq!(get_square(&b, &pos("e6")), SQ_WH_N);
        assert_eq!(num_pieces(&b), 2);
        // Sack it with black knight
        apply_move_to_board(&mut b, &(pos("f4"), pos("e6"), None));
        assert_eq!(get_square(&b, &pos("e6")), SQ_BL_N);
        assert_eq!(num_pieces(&b), 1);
    }

    #[test]
    fn test_apply_move_to_castling() {
        let mut b = new();
        let mut gs = GameState::new();
        assert_eq!(gs.castling, CASTLING_MASK);

        // On a starting board, start by making place for all castles.
        clear_square(&mut b, &pos("b1"));
        clear_square(&mut b, &pos("c1"));
        clear_square(&mut b, &pos("d1"));
        clear_square(&mut b, &pos("f1"));
        clear_square(&mut b, &pos("g1"));
        clear_square(&mut b, &pos("b8"));
        clear_square(&mut b, &pos("c8"));
        clear_square(&mut b, &pos("d8"));
        clear_square(&mut b, &pos("f8"));
        clear_square(&mut b, &pos("g8"));
        // White queen-side castling.
        apply_move_to(&mut b, &mut gs, &parse_move("e1a1"));
        assert!(is_piece(get_square(&b, &pos("c1")), SQ_WH_K));
        assert!(is_piece(get_square(&b, &pos("d1")), SQ_WH_R));
        assert!(is_empty(&b, &pos("a1")));
        assert!(is_empty(&b, &pos("e1")));
        assert_eq!(gs.castling, CASTLING_BL_MASK);
        // Black king-side castling.
        apply_move_to(&mut b, &mut gs, &parse_move("e8h8"));
        assert!(is_piece(get_square(&b, &pos("g8")), SQ_BL_K));
        assert!(is_piece(get_square(&b, &pos("f8")), SQ_BL_R));
        assert!(is_empty(&b, &pos("h8")));
        assert!(is_empty(&b, &pos("e8")));
        assert_eq!(gs.castling, 0);
    }

    #[test]
    fn test_get_player_moves() {
        let b = new();
        let gs = GameState::new();

        // At first move, white has 16 pawn moves and 4 knight moves.
        let moves = get_player_moves(&b, &gs, true);
        assert_eq!(moves.len(), 20);
    }

    #[test]
    fn test_get_pawn_moves() {
        let mut b = new_empty();
        let gs = GameState::new();

        // Check that a pawn (here white queen's pawn) can move forward if the road is free.
        set_square(&mut b, &pos("d3"), SQ_WH_P);
        let moves = get_piece_moves(&b, &pos("d3"), &gs, true);
        assert!(moves.len() == 1 && moves.contains( &parse_move("d3d4") ));

        // Check that a pawn (here white king's pawn) can move 2 square forward on first move.
        set_square(&mut b, &pos("e2"), SQ_WH_P);
        let moves = get_piece_moves(&b, &pos("e2"), &gs, true);
        assert_eq!(moves.len(), 2);
        assert!(moves.contains( &parse_move("e2e3") ));
        assert!(moves.contains( &parse_move("e2e4") ));

        // Check that a pawn cannot move forward if a piece is blocking its path.
        // 1. black pawn 2 square forward; only 1 square forward available from start pos.
        set_square(&mut b, &pos("e4"), SQ_BL_P);
        let moves = get_piece_moves(&b, &pos("e2"), &gs, true);
        assert!(moves.len() == 1 && moves.contains( &parse_move("e2e3") ));
        // 2. black pawn 1 square forward; no square available.
        set_square(&mut b, &pos("e3"), SQ_BL_P);
        let moves = get_piece_moves(&b, &pos("e2"), &gs, true);
        assert_eq!(moves.len(), 0);
        // 3. remove the e4 black pawn; the white pawn should not be able to jump above e3 pawn.
        clear_square(&mut b, &pos("e4"));
        let moves = get_piece_moves(&b, &pos("e2"), &gs, true);
        assert_eq!(moves.len(), 0);

        // Check that a pawn can take a piece diagonally.
        set_square(&mut b, &pos("f3"), SQ_BL_P);
        let moves = get_piece_moves(&b, &pos("e2"), &gs, true);
        assert!(moves.len() == 1 && moves.contains( &parse_move("e2f3") ));
        set_square(&mut b, &pos("d3"), SQ_BL_P);
        let moves = get_piece_moves(&b, &pos("e2"), &gs, true);
        assert_eq!(moves.len(), 2);
        assert!(moves.contains( &parse_move("e2f3") ));
        assert!(moves.contains( &parse_move("e2d3") ));

        // Check that a pawn moving to the last rank leads to queen promotion.
        // 1. by simply moving forward.
        set_square(&mut b, &pos("a7"), SQ_WH_P);
        let moves = get_piece_moves(&b, &pos("a7"), &gs, true);
        assert!(moves.len() == 1 && moves.contains( &parse_move("a7a8q") ));
    }

    #[test]
    fn test_get_bishop_moves() {
        let mut b = new_empty();
        let gs = GameState::new();

        // A bishop has maximum range when it's in a center square.
        set_square(&mut b, &pos("d4"), SQ_WH_B);
        let moves = get_piece_moves(&b, &pos("d4"), &gs, true);
        assert_eq!(moves.len(), 13);
        // Going top-right.
        assert!(moves.contains( &parse_move("d4e5") ));
        assert!(moves.contains( &parse_move("d4f6") ));
        assert!(moves.contains( &parse_move("d4g7") ));
        assert!(moves.contains( &parse_move("d4h8") ));
        // Going bottom-right.
        assert!(moves.contains( &parse_move("d4e3") ));
        assert!(moves.contains( &parse_move("d4f2") ));
        assert!(moves.contains( &parse_move("d4g1") ));
        // Going bottom-left.
        assert!(moves.contains( &parse_move("d4c3") ));
        assert!(moves.contains( &parse_move("d4b2") ));
        assert!(moves.contains( &parse_move("d4a1") ));
        // Going top-left.
        assert!(moves.contains( &parse_move("d4c5") ));
        assert!(moves.contains( &parse_move("d4b6") ));
        assert!(moves.contains( &parse_move("d4a7") ));

        // When blocking commit to one square with friendly piece, lose 2 moves.
        set_square(&mut b, &pos("b2"), SQ_WH_P);
        assert_eq!(get_piece_moves(&b, &pos("d4"), &gs, true).len(), 11);

        // When blocking commit to one square with enemy piece, lose only 1 move.
        set_square(&mut b, &pos("b2"), SQ_BL_P);
        assert_eq!(get_piece_moves(&b, &pos("d4"), &gs, true).len(), 12);
    }

    #[test]
    fn test_get_knight_moves() {
        let mut b = new_empty();
        let gs = GameState::new();

        // A knight never has blocked commit; if it's in the center of the board, it can have up to
        // 8 moves.
        set_square(&mut b, &pos("d4"), SQ_WH_N);
        assert_eq!(get_piece_moves(&b, &pos("d4"), &gs, true).len(), 8);

        // If on a side if has only 4 moves.
        set_square(&mut b, &pos("a4"), SQ_WH_N);
        assert_eq!(get_piece_moves(&b, &pos("a4"), &gs, true).len(), 4);

        // And in a corner, only 2 moves.
        set_square(&mut b, &pos("a1"), SQ_WH_N);
        assert_eq!(get_piece_moves(&b, &pos("a1"), &gs, true).len(), 2);

        // Add 2 friendly pieces and it is totally blocked.
        set_square(&mut b, &pos("b3"), SQ_WH_P);
        set_square(&mut b, &pos("c2"), SQ_WH_P);
        assert_eq!(get_piece_moves(&b, &pos("a1"), &gs, true).len(), 0);
    }

    #[test]
    fn test_get_rook_moves() {
        let mut b = new_empty();
        let gs = GameState::new();

        set_square(&mut b, &pos("d4"), SQ_WH_R);
        assert_eq!(get_piece_moves(&b, &pos("d4"), &gs, true).len(), 14);
        set_square(&mut b, &pos("d6"), SQ_BL_P);
        assert_eq!(get_piece_moves(&b, &pos("d4"), &gs, true).len(), 12);
        set_square(&mut b, &pos("d6"), SQ_WH_P);
        assert_eq!(get_piece_moves(&b, &pos("d4"), &gs, true).len(), 11);
    }

    #[test]
    fn test_get_queen_moves() {
        let mut b = new_empty();
        let gs = GameState::new();

        set_square(&mut b, &pos("d4"), SQ_WH_Q);
        assert_eq!(get_piece_moves(&b, &pos("d4"), &gs, true).len(), 14 + 13);
    }

    #[test]
    fn test_get_king_moves() {
        let mut gs = GameState::new();
        return;  // FIXME

        // King can move 1 square in any direction.
        let mut b = new_empty();
        set_square(&mut b, &pos("d4"), SQ_WH_K);
        assert_eq!(get_piece_moves(&b, &pos("d4"), &gs, true).len(), 8);
        set_square(&mut b, &pos("e5"), SQ_WH_P);
        assert_eq!(get_piece_moves(&b, &pos("d4"), &gs, true).len(), 7);

        // If castling is available, other moves are possible: 5 moves + 2 castles.
        let mut b = new_empty();
        set_square(&mut b, &pos("e1"), SQ_WH_K);
        set_square(&mut b, &pos("a1"), SQ_WH_R);
        set_square(&mut b, &pos("h1"), SQ_WH_R);
        assert_eq!(get_piece_moves(&b, &pos("e1"), &gs, true).len(), 5 + 2);

        // Castling works as well for black.
        gs.color = SQ_BL;
        set_square(&mut b, &pos("e8"), SQ_BL_K);
        set_square(&mut b, &pos("a8"), SQ_BL_R);
        set_square(&mut b, &pos("h8"), SQ_BL_R);
        assert_eq!(get_piece_moves(&b, &pos("e8"), &gs, true).len(), 5 + 2);
    }

    #[test]
    fn test_filter_illegal_moves() {
        let mut b = new_empty();
        let gs = GameState::new();

        // Place white's king on first rank.
        set_square(&mut b, &pos("e1"), SQ_WH_K);
        // Place black rook in second rank: king can only move left or right.
        set_square(&mut b, &pos("h2"), SQ_BL_R);
        let all_wh_moves = get_piece_moves(&b, &pos("e1"), &gs, true);
        assert_eq!(all_wh_moves.len(), 2);  // 5 moves in absolute but only 2 are legal.
    }

    #[test]
    fn test_is_attacked() {
        let mut b = new_empty();
        let gs = GameState::new();

        // Place a black rook in white pawn's file.
        set_square(&mut b, &pos("d4"), SQ_WH_P);
        set_square(&mut b, &pos("d6"), SQ_BL_R);
        assert!(is_attacked(&b, &gs, &pos("d4")));
        // Move the rook on another file, no more attack.
        apply_move_to_board(&mut b, &parse_move("d6e6"));
        assert!(!is_attacked(&b, &gs, &pos("d4")));
    }
}

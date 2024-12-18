use pleco::{Board, SQ};
use pleco::tools::eval::Eval;

pub fn material_value(board: &Board) -> i32 {
    let mut white_score = 0;
    let mut black_score = 0;

    for square in 0..64 {
        match board.piece_at_sq(SQ::from(square)) {
            pleco::core::Piece::WhitePawn => white_score += 1,
            pleco::core::Piece::WhiteKnight | pleco::core::Piece::WhiteBishop => {
                white_score += 3
            }
            pleco::core::Piece::WhiteRook => white_score += 5,
            pleco::core::Piece::WhiteQueen => white_score += 9,
            pleco::core::Piece::BlackPawn => black_score += 1,
            pleco::core::Piece::BlackKnight | pleco::core::Piece::BlackBishop => {
                black_score += 3
            }
            pleco::core::Piece::BlackRook => black_score += 5,
            pleco::core::Piece::BlackQueen => black_score += 9,
            _ => {}
        }
    }

    // Return the difference (positive for white advantage, negative for black advantage)
    white_score - black_score
}

pub fn evaluate_board_after_move(board: Board, mv: &pleco::BitMove) -> i32 {
    // General board evaluation for balanced personality
    let mut board_clone = board.clone();
    board_clone.apply_move(*mv);
    Eval::eval_low(&board_clone)
}
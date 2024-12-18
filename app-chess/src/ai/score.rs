use std::vec;
use eframe::egui::Shape::Vec;
use pleco::tools::eval::Eval;
use pleco::{BitMove, Board};

pub fn method1(board: Board) -> std::vec::Vec<BitMove> {
    // Generate all legal moves from the current position.
    let moves = board.generate_moves();

    // println!("Legal moves:");
    // for mv in &moves {
    //     println!("{}", mv);
    // }

    // Find and evaluate the best move.
    let mut best_move: Option<BitMove> = None;
    let mut best_score = i32::MIN;

    for mv in moves {
        // Make the move on a new board.
        let mut new_board = board.clone();
        new_board.apply_move(mv);

        // Evaluate the resulting position.
        let score = Eval::eval_low(&new_board);

        println!("Move: {}, Score: {}", mv, score);

        // Update the best move if this move has a higher score.
        if score > best_score {
            best_score = score;
            best_move = Some(mv);
        }
    }
    // (best_move, best_score)
    let mut best_moves = vec![];
    best_moves.push(best_move.unwrap());
    best_moves
}

pub fn method2(board: Board) -> std::vec::Vec<BitMove> {
    let moves = board.generate_moves();
    let mut best_score = i32::MIN;
    let mut best_moves = vec![];

    for mv in moves.iter() {
        let mut board_clone = board.clone();
        board_clone.apply_move(*mv);

        let score = Eval::eval_low(&board_clone);
        if score > best_score {
            best_score = score;
            best_moves.clear();
            best_moves.push(*mv);
        } else if score == best_score {
            best_moves.push(*mv);
        }
    }
    best_moves
}

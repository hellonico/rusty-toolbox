use pleco::{BitMove, Board};
use pleco::tools::eval::Eval;
use rand::prelude::SliceRandom;
use rand::rng;
use crate::ai::score::{method1, method2};
use crate::utils::convert::{bitmove_to_uci, pretty_print_moves};
use crate::utils::moves::simple_function_to_export_moves;
use crate::utils::score::{evaluate_board_after_move, material_value};

pub fn compute_ai_move(board: &Board, ai_personality: &String) -> Option<BitMove> {
    let mut moves = board.generate_moves();
    moves.shuffle(&mut rng());

    // pretty_print_moves(moves.vec());

    let ai_move = match ai_personality.as_str() {
        "Childish" => moves.first().cloned(),
        "Aggressive" => moves
            .iter()
            .max_by_key(|m| evaluate_aggressiveness(board.clone(),m))
            .cloned(),
        "Defensive" => moves
            .iter()
            .min_by_key(|m| evaluate_aggressiveness(board.clone(),m))
            .cloned(),
        "Exchange-Prone" => moves.iter().find(|m| is_exchange_move(board.clone(), m)).cloned(),
        "Balanced" => moves.iter().max_by_key(|m| evaluate_board_after_move(board.clone(), m)).cloned(),
        "Method1" => Some(method1(board.clone()).get(0).unwrap().clone()),
        "Method2" => Some(method2(board.clone()).get(0).unwrap().clone()),
        _ => None
    };
    println!("Computed {:?} AI move: {:?}", ai_personality, bitmove_to_uci(ai_move.unwrap()));
    ai_move
}

pub fn evaluate_aggressiveness(board:Board, mv: &pleco::BitMove) -> i32 {
    // Clone the board to simulate the move
    let mut board_clone = board.clone();
    board_clone.apply_move(*mv);

    // Calculate material score for both sides
    let material_score = material_value(&board_clone);

    // Combine material advantage and general board evaluation
    material_score + Eval::eval_low(&board_clone)
}

pub fn is_exchange_move(board:Board, mv: &pleco::BitMove) -> bool {
    let mut cloned_board = board.clone();
    cloned_board.apply_move(*mv);
    mv.is_capture() && !cloned_board.checkers().is_empty()
}

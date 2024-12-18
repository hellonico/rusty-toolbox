use pleco::Board;

pub fn get_legal_moves(board: Board, square_index: usize) -> Vec<usize> {
    let moves = board.generate_moves();
    moves
        .iter()
        .filter(|m| m.get_src().0 as usize == square_index)
        .map(|m| m.get_dest().0 as usize)
        .collect()
}

pub fn simple_function_to_export_moves(move_history: Vec<String>) -> Vec<String> {
    let mut moves = Vec::new();
    for (i, chunk) in move_history.chunks(2).enumerate() {
        let turn = format!(
            "{}. {}{}",
            i + 1,
            chunk.get(0).unwrap_or(&String::new()), // White's move
            if let Some(black_move) = chunk.get(1) {
                format!(", {}", black_move)
            } else {
                "".to_string()
            }  // Black's move
        );
        moves.push(turn);
    }
    moves
}

pub fn first_pgn_not_in_history(pgn_moves: &Vec<String>, move_history: Vec<String>) -> Option<String> {
    let first_not_in_history = if pgn_moves.len() > move_history.len() {
        // Get the size of the current move history
        let history_size = move_history.len();

        // Compare all moves up to the size of the current history
        let moves_match = pgn_moves[..history_size]
            .iter()
            .zip(move_history.iter())
            .all(|(pgn_move, history_move)| pgn_move == history_move);

        if moves_match {
            // If the moves match, return the next move from the PGN list
            let next_move = &pgn_moves[history_size];
            Some(next_move.clone())
        } else {
            // If moves don't match, return None or handle the discrepancy
            None
        }
    } else {
        // If PGN list is shorter than the history, there's no next move
        None
    };
    first_not_in_history
}
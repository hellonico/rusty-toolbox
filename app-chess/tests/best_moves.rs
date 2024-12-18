#[cfg(test)]
mod tests {
    // use eframe::egui::accesskit::Role::Search;
    use app_chess::pgn::export::uci_to_pgn;
    use app_chess::pgn::import::{extract_moves, pgn_to_uci};
    use pleco::{BitMove, Board};
    use pleco::tools::eval::Eval;
    use app_chess::ai::score::beast_move;

    #[test]
    fn test_best_moves() {
        // Create a new chess board with the starting position.
        let board = Board::start_pos();

        let (best_move, best_score) = beast_move(board);

        if let Some(best_mv) = best_move {
            println!("\nBest move: {} with score {}", best_mv, best_score);
        } else {
            println!("\nNo moves available.");
        }

    }

}

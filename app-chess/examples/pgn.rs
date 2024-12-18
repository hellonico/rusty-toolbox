use pleco::{Board, Piece};
use std::fs;
use app_chess::pgn::import::{extract_moves, pgn_to_uci};

fn main() {
    // Load the PGN file as a string
    let pgn_path = "app-chess/resources/hello.pgn"; // Replace with your file path
    let pgn_content = fs::read_to_string(pgn_path)
        .expect("Failed to read PGN file");

    // Extract moves from the PGN file
    let pgn_moves = extract_moves(&pgn_content);
    if let Some(moves_section) = pgn_moves {
        let mut board = Board::start_pos(); // Initialize the starting board

        for pgn_move in moves_section {
            if let Some(uci_move) = pgn_to_uci(&board, &pgn_move) {
                board.apply_uci_move(&uci_move);
                println!("Played move: {} -> {}", pgn_move, uci_move);
            } else {
                eprintln!("Invalid move: {}", pgn_move);
                break;
            }
        }

        println!("Final board position:\n{}", board);
    } else {
        eprintln!("Failed to parse moves from PGN file.");
    }


}

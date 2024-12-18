

use pleco::{Board, Piece};
use crate::utils::convert::{char_to_piece, char_to_piecetype};

pub fn pgn_to_uci(board: &Board, pgn_move: &str) -> Option<String> {
    // Remove check (+) or checkmate (#) annotations
    let pgn_move = pgn_move.trim_end_matches('+').trim_end_matches('#');

    // Handle castling
    if pgn_move == "O-O" {
        return Some(if board.turn() == pleco::Player::White { "e1g1" } else { "e8g8" }.to_string());
    } else if pgn_move == "O-O-O" {
        return Some(if board.turn() == pleco::Player::White { "e1c1" } else { "e8c8" }.to_string());
    }

    // Generate all legal moves
    let legal_moves = board.generate_moves();

    if pgn_move.contains('=') {
        let destination = &pgn_move[..2]; // Get the destination square, e.g., "a8"
        let promotion_piece = pgn_move.chars().last().unwrap(); // Get the promotion piece, e.g., 'Q'

        for legal_move in legal_moves {
            let move_piece = board.piece_at_sq(legal_move.get_src());
            let dest_square = legal_move.get_dest().to_string();

            // Check if the move is a promotion
            if legal_move.is_promo()
                && dest_square == destination
                && move_piece == if board.turn() == pleco::Player::White { Piece::WhitePawn } else { Piece::BlackPawn }
                && legal_move.promo_piece() == char_to_piecetype(promotion_piece).unwrap()
            {
                return Some(legal_move.to_string());
            }
        }
        return None;
    }

    if pgn_move.chars().next().unwrap().is_uppercase() {
        let piece_type = pgn_move.chars().next().unwrap(); // Piece type (e.g., 'Q')
        let destination = &pgn_move[pgn_move.len() - 2..]; // Destination (e.g., "a4")

        if piece_type == 'Q' {
            // Handle multiple Queens on the board (disambiguation)
            let mut candidates = Vec::new();
            for legal_move in legal_moves.iter().clone() {
                let move_piece = board.piece_at_sq(legal_move.get_src());
                let dest_square = legal_move.get_dest().to_string();

                if move_piece == if board.turn() == pleco::Player::White { Piece::WhiteQueen } else { Piece::BlackQueen }
                    && dest_square == destination
                {
                    candidates.push(legal_move);
                }
            }

            if candidates.len() == 1 {
                return Some(candidates[0].to_string());
            } else if candidates.len() > 1 {
                // If multiple Queens can move to the same destination, disambiguate by file or rank
                let disambiguated_move = candidates.into_iter().find(|legal_move| {
                    let move_file = legal_move.get_src().to_string().chars().next().unwrap();
                    let pgn_file = pgn_move.chars().nth(1).unwrap(); // file of the move in PGN (e.g., 'a' in "Qaa4")
                    move_file == pgn_file
                });

                if let Some(valid_move) = disambiguated_move {
                    return Some(valid_move.to_string());
                }
            }
        }
    }

    // Handle captures (e.g., "Nxd5", "exd5", "Bxc6")
    if pgn_move.contains('x') {
        let piece_char = pgn_move.chars().next().unwrap(); // Get the piece type or source file
        let destination = &pgn_move[pgn_move.len() - 2..]; // Get the destination square, e.g., "d5"

        for legal_move in legal_moves {
            let move_piece = board.piece_at_sq(legal_move.get_src());
            let dest_square = legal_move.get_dest().to_string();

            // Handle pawn captures (e.g., "exd5")
            if piece_char.is_lowercase() {
                let src_file = piece_char; // e.g., 'e' in "exd5"
                let src_square = legal_move.get_src().to_string();

                if move_piece == if board.turn() == pleco::Player::White { Piece::WhitePawn } else { Piece::BlackPawn }
                    && src_square.starts_with(src_file)
                    && dest_square == destination
                {
                    return Some(legal_move.to_string());
                }
            } else {
                // Handle other piece captures (e.g., "Nxd5", "Bxc6")
                let piece_enum = char_to_piece(piece_char, board.turn() == pleco::Player::White)?;

                if move_piece == piece_enum && dest_square == destination {
                    return Some(legal_move.to_string());
                }
            }
        }
        return None;
    }

    // Handle standard moves (e.g., "Be2", "Nf3")
    for legal_move in legal_moves {
        let move_piece = board.piece_at_sq(legal_move.get_src());
        let dest_square = legal_move.get_dest().to_string();

        // Parse PGN for piece type and destination
        let mut piece_type = 'P'; // Default to pawn
        let mut destination = pgn_move;

        if pgn_move.chars().next().unwrap().is_uppercase() {
            piece_type = pgn_move.chars().next().unwrap();
            destination = &pgn_move[1..];
        }

        // Get the `Piece` enum for the current turn
        let piece_enum = char_to_piece(piece_type, board.turn() == pleco::Player::White)?;

        // Match the piece type and destination
        if move_piece == piece_enum && dest_square == destination {
            return Some(legal_move.to_string());
        }
    }

    // No matching move found
    None
}



/// A simple PGN moves extractor (naive approach, assumes single game in PGN)
pub fn extract_moves(pgn: &str) -> Option<Vec<String>> {
    let moves_start = pgn.find("\n\n")?; // Locate the moves section
    let moves_str = &pgn[moves_start..]
        .replace("\n", " ") // Normalize newlines
        .replace("{", "") // Remove comments
        .replace("}", "");

    let mut moves = Vec::new();
    for token in moves_str.split_whitespace() {
        // Skip move numbers and results
        if token.ends_with('.') || token == "1-0" || token == "0-1" || token == "1/2-1/2" {
            continue;
        }
        moves.push(token.to_string());
    }
    Some(moves)
}



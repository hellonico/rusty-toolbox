use pleco::{Board, Piece};
use crate::utils::convert::{char_to_piece, piece_to_char};

pub fn uci_to_pgn(board: &Board, uci_move: &str) -> Option<String> {
    let legal_moves = board.generate_moves();
    let uci_move = uci_move.trim();

    // Find the matching legal move
    let matching_move = legal_moves.iter().find(|m| m.to_string() == uci_move)?;
    let move_piece = board.piece_at_sq(matching_move.get_src());
    let dest_square = matching_move.get_dest().to_string();

    let mut pgn_move = String::new();

    // Check if the move is a promotion
    let promotion = if uci_move.len() == 5 {
        let promo_char = uci_move.chars().nth(4)?; // Get the 5th character
        match promo_char {
            'q' => Some('Q'),
            'r' => Some('R'),
            'b' => Some('B'),
            'n' => Some('N'),
            _ => None, // Invalid promotion character
        }
    } else {
        None
    };

    // Handle promotion moves
    if let Some(promo) = promotion {
        return Some(format!("{}={}", dest_square, promo));
    }

    // Handle castling
    if matching_move.is_castle() {
        let src_file = matching_move.get_src().file();
        let dest_file = matching_move.get_dest().file();

        if dest_file > src_file {
            return Some("O-O".to_string()); // King-side castling
        } else {
            return Some("O-O-O".to_string()); // Queen-side castling
        }
    }


    // Determine the piece notation
    if move_piece != Piece::WhitePawn && move_piece != Piece::BlackPawn {
        pgn_move.push(piece_to_char(move_piece).parse().unwrap());
    }

    // Handle disambiguation (e.g., Nbd2)
    let mut disambiguation = String::new();
    let same_dest_moves: Vec<_> = legal_moves
        .iter()
        .filter(|m| m.get_dest() == matching_move.get_dest() && board.piece_at_sq(m.get_src()) == move_piece)
        .collect();

    if same_dest_moves.len() > 1 {
        let src_square = matching_move.get_src().to_string();
        let same_file = same_dest_moves.iter().any(|m| m.get_src().file() == matching_move.get_src().file());
        let same_rank = same_dest_moves.iter().any(|m| m.get_src().rank() == matching_move.get_src().rank());

        if same_file && same_rank {
            disambiguation.push_str(&src_square);
        } else if same_file {
            disambiguation.push(src_square.chars().nth(1).unwrap());
        } else {
            disambiguation.push(src_square.chars().nth(0).unwrap());
        }
    }

    pgn_move.push_str(&disambiguation);

    // Handle captures
    if matching_move.is_capture() {
        let piece = board.piece_at_sq(matching_move.get_src());
        // if let Some(piece) = piece {
            if move_piece == Piece::WhitePawn || move_piece == Piece::BlackPawn {
                pgn_move.push(piece_to_char(piece).parse().unwrap());
            }
        // }

            // pgn_move.push(piece_to_char(matching_move.get_src()));
        pgn_move.push('x');
    }

    // Append destination square
    pgn_move.push_str(&dest_square);

    if matching_move.is_promo() {
        pgn_move.push('=');
        // Convert `PieceType` to PGN string
        if let promo_char = matching_move.promo_piece().to_string() {
            if let Some(promo_piece) = char_to_piece(promo_char.parse().unwrap(), board.turn() == pleco::Player::White) {
                pgn_move.push(piece_to_char(promo_piece).chars().next().unwrap());
            }
        }
        return Some(pgn_move);
    }

    // Append check or checkmate notation
    let mut temp_board = board.shallow_clone();
    temp_board.apply_move(*matching_move);
    if temp_board.checkmate() {
        pgn_move.push('#');
    } else if temp_board.in_check() {
        pgn_move.push('+');
    }

    Some(pgn_move)
}

pub fn generate_pgn(move_history: Vec<String>) -> String {
    let mut pgn = String::new();

    // Add PGN headers
    pgn.push_str("[Event \"Casual Game\"]\n");
    pgn.push_str("[Site \"Unknown\"]\n");
    pgn.push_str("[Date \"2024.12.05\"]\n");
    pgn.push_str("[Round \"1\"]\n");
    pgn.push_str("[White \"Player1\"]\n");
    pgn.push_str("[Black \"Player2\"]\n");
    pgn.push_str("[Result \"*\"]\n\n");

    // Add moves
    let mut move_counter = 1;
    let mut pgn_board = Board::default();
    for (i, mv) in move_history.iter().enumerate() {
        if i % 2 == 0 {
            pgn.push_str(&format!("{}. ", move_counter));
            move_counter += 1;
        }
        pgn.push_str(&uci_to_pgn(&pgn_board, mv).unwrap());
        pgn.push(' ');
        pgn_board.apply_uci_move(mv);
    }

    pgn.push('\n');
    pgn
}
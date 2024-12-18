use std::vec;
use pleco::{BitMove, MoveList, Piece, PieceType};

pub fn char_to_piece(c: char, is_white: bool) -> Option<Piece> {
    match c {
        'P' => Some(if is_white { Piece::WhitePawn } else { Piece::BlackPawn }),
        'N' => Some(if is_white { Piece::WhiteKnight } else { Piece::BlackKnight }),
        'B' => Some(if is_white { Piece::WhiteBishop } else { Piece::BlackBishop }),
        'R' => Some(if is_white { Piece::WhiteRook } else { Piece::BlackRook }),
        'Q' => Some(if is_white { Piece::WhiteQueen } else { Piece::BlackQueen }),
        'K' => Some(if is_white { Piece::WhiteKing } else { Piece::BlackKing }),
        _ => None,
    }
}


// Convert Pleco piece to a character for display
pub fn piece_to_char(piece: Piece) -> String {
    match piece {
        Piece::None => "".to_string(),
        Piece::WhitePawn => "♙".to_string(),
        Piece::WhiteKnight => "♘".to_string(),
        Piece::WhiteBishop => "♗".to_string(),
        Piece::WhiteRook => "♖".to_string(),
        Piece::WhiteQueen => "♕".to_string(),
        Piece::WhiteKing => "♔".to_string(),
        Piece::BlackPawn => "♟".to_string(),
        Piece::BlackKnight => "♞".to_string(),
        Piece::BlackBishop => "♝".to_string(),
        Piece::BlackRook => "♜".to_string(),
        Piece::BlackQueen => "♛".to_string(),
        Piece::BlackKing => "♚".to_string(),
    }
}

pub fn char_to_piecetype(c: char) -> Option<PieceType> {
    match c {
        'Q' => Some(PieceType::Q),
        'R' => Some(PieceType::R),
        'B' => Some(PieceType::B),
        'N' => Some(PieceType::K),
        _ => None,
    }
}
pub fn bitmove_to_uci(mv: BitMove) -> String {
    let from_sq = mv.get_src().to_string(); // Source square
    let to_sq = mv.get_dest().to_string(); // Destination square

    if mv.is_promo() {
        // Add promotion piece type if it's a promotion move

        let promo_piece = match mv.promo_piece() {
            pleco::PieceType::Q => "q",
            pleco::PieceType::R => "r",
            pleco::PieceType::B => "b",
            pleco::PieceType::K => "n", // Knight uses 'n'
            _ => "",
        };
        format!("{}{}{}", from_sq, to_sq, promo_piece)
    } else {
        // Normal move
        format!("{}{}", from_sq, to_sq)
    }
}
pub fn convert_moves_to_uci(moves: Vec<BitMove>) -> Vec<String> {
    moves.iter().map(|bit_move| bitmove_to_uci(*bit_move)).collect()
}

pub fn pretty_print_moves(moves: Vec<BitMove>) {
    let uci_moves = convert_moves_to_uci(moves);

    println!("Legal moves for the current board position:");
    for (index, uci_move) in uci_moves.iter().enumerate() {
        println!("{:>3}. {}", index + 1, uci_move);
    }
}


pub fn to_usize(best_moves: vec::Vec<BitMove>) -> vec::Vec<usize> {
    best_moves.iter().map(|mv| vec![mv.get_dest().0 as usize, mv.get_src().0 as usize]).flatten().collect()
}
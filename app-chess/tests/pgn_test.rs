#[cfg(test)]
mod tests {
    use app_chess::pgn::export::uci_to_pgn;
    use app_chess::pgn::import::{extract_moves, pgn_to_uci};
    use pleco::Board;

    #[test]
    fn first_moves() {
        let board = Board::default();
        let move_list = board.generate_moves();
        assert_eq!(move_list.len(), 20);
    }

    #[test]
    fn simple_pgn_moves() {
        let mut board = Board::default();

        let move1 = pgn_to_uci(&board, "b4").unwrap();
        assert_eq!(move1, "b2b4");
        assert!(board.apply_uci_move(&move1));

        let move2 = pgn_to_uci(&board, "a6").unwrap();
        assert_eq!(move2, "a7a6");
        assert!(board.apply_uci_move(&move2));

        let move3 = pgn_to_uci(&board, "d4").unwrap();
        assert_eq!(move3, "d2d4");
        assert!(board.apply_uci_move(&move3));

        board.pretty_print();
    }

    #[test]
    fn load_pgn_file() {
        match std::fs::read_to_string("resources/test.pgn") {
            Ok(pgn_content) => match extract_moves(&pgn_content).unwrap() {
                moves => {
                    assert_eq!(117, moves.iter().len());
                    println!("{:?}", moves);
                }
            },
            Err(e) => {
                eprintln!("{}", e);
                assert!(false);
            }
        }
    }

    #[test]
    fn pgn_to_uci_to_pgn_one() {
        let board = Board::default();
        let uci_move1 = pgn_to_uci(&board, "b4").unwrap();
        println!("{:?}", uci_move1);
        let pgn_move1 = uci_to_pgn(&board, &uci_move1).unwrap();
        println!("{:?}", pgn_move1);
    }

    fn pgn_to_uci_to_pgn(board: &Board, uci_move: &str) -> String {
        let uci_move1 = pgn_to_uci(&board, uci_move).unwrap();
        let pgn_move1 = uci_to_pgn(&board, &uci_move1).unwrap();
        pgn_move1.to_string()
    }
    #[test]
    fn pgn_to_uci_to_pgn_two() {
        match std::fs::read_to_string("resources/test.pgn") {
            Ok(pgn_content) => match extract_moves(&pgn_content).unwrap() {
                moves => {
                    let mut board = Board::default();
                    for mo in moves {
                        let uci = pgn_to_uci(&board, &mo).unwrap();
                        println!("PGN:{:?} -> UCI:{:?} ", mo, uci);
                        println!("{:?}", pgn_to_uci_to_pgn(&board, &mo));
                        board.apply_uci_move(&uci);
                    }
                }
            },
            Err(e) => {
                eprintln!("{}", e);
                assert!(false);
            }
        }
    }
}

use eframe::egui;
use eframe::egui::{Button, RichText};
use egui::Color32;
use pleco::board::Board;
use pleco::tools::eval::Eval;
use pleco::{Player, SQ};
use rand::rng;
use rand::seq::SliceRandom;

fn main() -> Result<(), eframe::Error> {
    let board = Board::default();
    eframe::run_native(
        "Chess AI",
        eframe::NativeOptions::default(),
        Box::new(|_| {
            Ok(Box::new(ChessApp {
                board,
                selected_square: None,
                highlighted_moves: vec![],
                best_moves: vec![],
                move_history: vec![],
                player_vs_ai: true,
                ai_personality: "Balanced".to_string(),
                ai_elo: 1200,
                status_message: Some("".to_string()),
            }))
        }),
    )
}

struct ChessApp {
    board: Board,
    selected_square: Option<usize>,
    highlighted_moves: Vec<usize>,
    best_moves: Vec<usize>,
    move_history: Vec<String>,
    player_vs_ai: bool,
    ai_personality: String,
    ai_elo: usize,
    status_message: Option<String>,
}

impl eframe::App for ChessApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            self.render_menu(ui);
            ui.horizontal(|ui| {
                let can_undo = self.move_history.len() > 0;
                if ui.add_enabled(can_undo, Button::new("Undo Move")).clicked() {
                    self.undo_last_move();
                };

                if ui.add_enabled(can_undo, Button::new("New Game")).clicked() {
                    self.reset_game(); // Call the reset method
                }
                ui.checkbox(&mut self.player_vs_ai, "Player vs AI");

                // ELO Range Selector
                egui::ComboBox::from_label("ELO")
                    .selected_text(format!("{}-{}", self.ai_elo - 200, self.ai_elo))
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut self.ai_elo, 800, "600-800");
                        ui.selectable_value(&mut self.ai_elo, 1200, "1000-1200");
                        ui.selectable_value(&mut self.ai_elo, 1500, "1200-1500");
                        ui.selectable_value(&mut self.ai_elo, 2000, "1800+ (Grandmaster)");
                    });

                // Personality Selector
                egui::ComboBox::from_label("Personality")
                    .selected_text(&self.ai_personality)
                    .show_ui(ui, |ui| {
                        ui.selectable_value(
                            &mut self.ai_personality,
                            "Childish".to_string(),
                            "Childish",
                        );
                        ui.selectable_value(
                            &mut self.ai_personality,
                            "Aggressive".to_string(),
                            "Aggressive",
                        );
                        ui.selectable_value(
                            &mut self.ai_personality,
                            "Defensive".to_string(),
                            "Defensive",
                        );
                        ui.selectable_value(
                            &mut self.ai_personality,
                            "Exchange-Prone".to_string(),
                            "Exchange-Prone",
                        );
                        ui.selectable_value(
                            &mut self.ai_personality,
                            "Balanced".to_string(),
                            "Balanced",
                        );
                    });
            });

            ui.separator();
            self.render_board(ui);
            self.check_game_over(ui);
            ui.separator();
            ui.heading("Move History");
            let moves = self.moves_to_notation();
            for turn in moves {
                ui.label(turn);
            }
        });
    }
}

impl ChessApp {
    fn render_menu(&mut self, ui: &mut egui::Ui) {
        ui.menu_button("File", |ui| {
            if ui.button("Import PGN").clicked() {
                self.import_pgn();
                ui.close_menu();
            }
            if ui.button("Export PGN").clicked() {
                self.export_pgn();
                ui.close_menu();
            }
        });
    }

    fn generate_pgn(&self) -> String {
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
        for (i, mv) in self.move_history.iter().enumerate() {
            if i % 2 == 0 {
                pgn.push_str(&format!("{}. ", move_counter));
                move_counter += 1;
            }
            pgn.push_str(mv);
            pgn.push(' ');
        }

        pgn.push('\n');
        pgn
    }

    fn parse_pgn(&mut self, pgn: &str) -> Result<(), String> {
        let lines: Vec<&str> = pgn.lines().collect();
        let mut headers = true;
        self.move_history.clear();

        for line in lines {
            if headers && line.starts_with('[') {
                // Ignore metadata headers (e.g., [Event "Game"])
                continue;
            }

            headers = false;

            let moves: Vec<&str> = line.split_whitespace().collect();
            for mv in moves {
                if mv.ends_with('.') {
                    // Skip move numbers like "1.", "2.", etc.
                    continue;
                }

                println!("Replay:{mv}");
                // Apply the move to the board
                self.board.apply_uci_move(mv);

                // Store the move in PGN format for later generation
                self.move_history.push(mv.to_string());
            }
        }

        Ok(())
    }

    fn import_pgn(&mut self) {
        // Open file dialog to select a PGN file
        if let Some(path) = rfd::FileDialog::new()
            .add_filter("PGN files", &["pgn"])
            .pick_file()
        {
            match std::fs::read_to_string(path) {
                Ok(contents) => match self.parse_pgn(&contents) {
                    Ok(_) => self.status_message = Some("PGN imported successfully!".to_string()),
                    Err(err) => self.status_message = Some(format!("Error importing PGN: {err}")),
                },
                Err(err) => self.status_message = Some(format!("Error reading file: {err}")),
            }
        }
    }

    fn export_pgn(&mut self) {
        // Open file dialog to save a PGN file
        if let Some(path) = rfd::FileDialog::new()
            .add_filter("PGN files", &["pgn"])
            .save_file()
        {
            let pgn = self.generate_pgn();
            match std::fs::write(path, pgn) {
                Ok(_) => self.status_message = Some("PGN exported successfully!".to_string()),
                Err(err) => self.status_message = Some(format!("Error writing file: {err}")),
            }
        }
    }
    fn reset_game(&mut self) {
        self.board = Board::default(); // Reset the board to the default starting position
        self.selected_square = None; // Clear any selected squares
        self.highlighted_moves.clear(); // Clear highlighted moves
        self.best_moves.clear(); // Clear best moves
        self.move_history.clear(); // Clear move history
        self.player_vs_ai = true; // Reset Player vs AI toggle (optional)
        self.ai_personality = "Balanced".to_string(); // Reset AI personality (optional)
        self.ai_elo = 1200; // Reset AI ELO (optional)
    }
    fn check_game_over(&self, ui: &mut egui::Ui) {
        if self.board.checkmate() {
            let winner = if self.board.turn() == Player::White {
                "Black"
            } else {
                "White"
            };
            ui.label(format!("Game Over: {} wins by checkmate!", winner));
        }
        if self.board.stalemate() {
            ui.label("Game Over: Stalemate!");
        }
        if self.board.in_check() {
            ui.label("Check!");
        }
    }
    fn update_best_moves(&mut self) {
        let moves = self.board.generate_moves();
        let mut best_score = i32::MIN;
        self.best_moves.clear();

        for mv in moves.iter() {
            let mut board_clone = self.board.clone();
            board_clone.apply_move(*mv);

            let score = Eval::eval_low(&board_clone);
            if score > best_score {
                best_score = score;
                self.best_moves.clear();
                self.best_moves.push(mv.get_dest().0 as usize);
            } else if score == best_score {
                self.best_moves.push(mv.get_dest().0 as usize);
            }
        }
    }
    fn render_board(&mut self, ui: &mut egui::Ui) {
        egui::Grid::new("chess_board")
            .spacing([4.0, 4.0])
            .show(ui, |ui| {
                for rank in (0..8).rev() {
                    for file in 0..8 {
                        let square_index = rank * 8 + file;
                        let piece = self.board.piece_at_sq(SQ::from(square_index));
                        let square_color = self.get_square_color(
                            square_index as usize,
                            rank as usize,
                            file as usize,
                        );
                        let piece_char = piece_to_char(piece);

                        if ui
                            .add_sized(
                                [50.0, 50.0],
                                egui::Button::new(RichText::new(piece_char).size(40.0).strong())
                                    .fill(square_color),
                            )
                            .clicked()
                        {
                            self.handle_square_click(square_index as usize);
                        }
                    }
                    ui.end_row();
                }
            });
    }

    fn get_square_color(&self, square_index: usize, rank: usize, file: usize) -> Color32 {
        let default_color = if (rank + file) % 2 == 0 {
            Color32::LIGHT_GRAY
        } else {
            Color32::DARK_GRAY
        };

        if Some(square_index) == self.selected_square {
            return Color32::KHAKI;
        }

        if self.highlighted_moves.contains(&square_index) {
            return Color32::BLUE;
        }

        if self.best_moves.contains(&square_index) {
            return Color32::GOLD;
        }

        default_color
    }

    fn handle_square_click(&mut self, square_index: usize) {
        if let Some(selected) = self.selected_square {
            let possible_moves = self.board.generate_moves();
            if let Some(mv) = possible_moves.iter().find(|&m| {
                m.get_src().0 as usize == selected && m.get_dest() == SQ::from(square_index as u8)
            }) {
                self.board.apply_move(*mv);
                self.selected_square = None;
                self.highlighted_moves.clear();
                self.update_best_moves();
                self.log_move(mv.to_string());

                // if self.player_vs_ai && !self.board.turn() {
                if self.player_vs_ai && self.board.turn() == Player::Black {
                    self.make_ai_move();
                }
            } else {
                self.selected_square = None;
                self.highlighted_moves.clear();
            }
        } else {
            self.selected_square = Some(square_index);
            self.highlighted_moves = self.get_legal_moves(square_index);
        }
    }

    fn get_legal_moves(&self, square_index: usize) -> Vec<usize> {
        let moves = self.board.generate_moves();
        moves
            .iter()
            .filter(|m| m.get_src().0 as usize == square_index)
            .map(|m| m.get_dest().0 as usize)
            .collect()
    }

    fn log_move(&mut self, mv: String) {
        self.move_history.push(mv);
    }

    fn undo_last_move(&mut self) {
        self.board.undo_move();
        self.move_history.pop();
    }

    fn make_ai_move(&mut self) {
        let mut moves = self.board.generate_moves();
        moves.shuffle(&mut rng());

        let ai_move = match self.ai_personality.as_str() {
            "Childish" => moves.first().cloned(),
            "Aggressive" => moves
                .iter()
                .max_by_key(|m| self.evaluate_aggressiveness(m))
                .cloned(),
            "Defensive" => moves
                .iter()
                .min_by_key(|m| self.evaluate_aggressiveness(m))
                .cloned(),
            "Exchange-Prone" => moves.iter().find(|m| self.is_exchange_move(m)).cloned(),
            _ => moves.iter().max_by_key(|m| self.evaluate_board(m)).cloned(),
        };

        if let Some(mv) = ai_move {
            self.board.apply_move(mv);
            self.log_move(mv.to_string());
        }
    }

    fn evaluate_aggressiveness(&self, mv: &pleco::BitMove) -> i32 {
        // Clone the board to simulate the move
        let mut board_clone = self.board.clone();
        board_clone.apply_move(*mv);

        // Calculate material score for both sides
        let material_score = self.material_value(&board_clone);

        // Combine material advantage and general board evaluation
        material_score + Eval::eval_low(&board_clone)
    }

    // Helper function to calculate the material value
    fn material_value(&self, board: &Board) -> i32 {
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

    fn is_exchange_move(&self, mv: &pleco::BitMove) -> bool {
        let mut cloned_board = self.board.clone();
        cloned_board.apply_move(*mv);
        mv.is_capture() && !cloned_board.checkers().is_empty()
    }

    fn evaluate_board(&self, mv: &pleco::BitMove) -> i32 {
        // General board evaluation for balanced personality
        let mut board_clone = self.board.clone();
        board_clone.apply_move(*mv);
        Eval::eval_low(&board_clone)
    }

    fn moves_to_notation(&mut self) -> Vec<String> {
        let mut moves = Vec::new();
        for (i, chunk) in self.move_history.chunks(2).enumerate() {
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
}

// Convert Pleco piece to a character for display
fn piece_to_char(piece: pleco::core::Piece) -> String {
    match piece {
        pleco::core::Piece::None => "".to_string(),
        pleco::core::Piece::WhitePawn => "♙".to_string(),
        pleco::core::Piece::WhiteKnight => "♘".to_string(),
        pleco::core::Piece::WhiteBishop => "♗".to_string(),
        pleco::core::Piece::WhiteRook => "♖".to_string(),
        pleco::core::Piece::WhiteQueen => "♕".to_string(),
        pleco::core::Piece::WhiteKing => "♔".to_string(),
        pleco::core::Piece::BlackPawn => "♟".to_string(),
        pleco::core::Piece::BlackKnight => "♞".to_string(),
        pleco::core::Piece::BlackBishop => "♝".to_string(),
        pleco::core::Piece::BlackRook => "♜".to_string(),
        pleco::core::Piece::BlackQueen => "♛".to_string(),
        pleco::core::Piece::BlackKing => "♚".to_string(),
    }
}

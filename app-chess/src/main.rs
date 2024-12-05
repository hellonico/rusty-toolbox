use eframe::egui;
use pleco::board::Board;
use pleco::tools::eval::Eval;
use pleco::core::movenum::MoveList;
use pleco::SQ;

fn main() -> Result<(), eframe::Error> {
    let board = Board::default();
    eframe::run_native(
        "Chess AI - Personalities",
        eframe::NativeOptions::default(),
        Box::new(|_| Box::new(ChessApp {
            board,
            selected_square: None,
            highlighted_moves: vec![],
            best_moves: vec![],
            move_history: vec![],
            player_vs_ai: true,
            ai_personality: "Balanced".to_string(),
            ai_elo: 1200,
        })),
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
}

impl eframe::App for ChessApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.heading("Chess AI - Personalities");

                if ui.button("Undo Move").clicked() {
                    self.undo_last_move();
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
                        ui.selectable_value(&mut self.ai_personality, "Childish".to_string(), "Childish");
                        ui.selectable_value(&mut self.ai_personality, "Aggressive".to_string(), "Aggressive");
                        ui.selectable_value(&mut self.ai_personality, "Defensive".to_string(), "Defensive");
                        ui.selectable_value(&mut self.ai_personality, "Exchange-Prone".to_string(), "Exchange-Prone");
                        ui.selectable_value(&mut self.ai_personality, "Balanced".to_string(), "Balanced");
                    });
            });

            ui.separator();
            self.render_board(ui);
            ui.separator();
            ui.heading("Move History");
            for (i, mv) in self.move_history.iter().enumerate() {
                ui.label(format!("{}. {}", i + 1, mv));
            }
        });
    }
}

impl ChessApp {
    fn render_board(&mut self, ui: &mut egui::Ui) {
        egui::Grid::new("chess_board")
            .spacing([4.0, 4.0])
            .show(ui, |ui| {
                for rank in (0..8).rev() {
                    for file in 0..8 {
                        let square_index = rank * 8 + file;
                        let piece = self.board.piece_at_sq(SQ::from(square_index));
                        let square_color = self.get_square_color(square_index, rank, file);
                        let piece_char = piece_to_char(piece);

                        if ui
                            .add_sized(
                                [40.0, 40.0],
                                egui::Button::new(piece_char).fill(square_color),
                            )
                            .clicked()
                        {
                            self.handle_square_click(square_index);
                        }
                    }
                    ui.end_row();
                }
            });
    }

    fn get_square_color(&self, square_index: usize, rank: usize, file: usize) -> egui::Color32 {
        let default_color = if (rank + file) % 2 == 0 {
            egui::Color32::LIGHT_GRAY
        } else {
            egui::Color32::DARK_GRAY
        };

        if Some(square_index) == self.selected_square {
            return egui::Color32::YELLOW;
        }

        if self.highlighted_moves.contains(&square_index) {
            return egui::Color32::GREEN;
        }

        if self.best_moves.contains(&square_index) {
            return egui::Color32::BLUE;
        }

        default_color
    }

    fn handle_square_click(&mut self, square_index: usize) {
        if let Some(selected) = self.selected_square {
            let possible_moves = self.board.generate_moves();
            if let Some(mv) = possible_moves.iter().find(|&m| {
                m.get_src() as usize == selected && m.get_dest() as usize == square_index
            }) {
                self.board.apply_move(mv);
                self.selected_square = None;
                self.highlighted_moves.clear();
                self.update_best_moves();
                self.log_move(mv.to_string());

                if self.player_vs_ai && !self.board.turn() {
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
            .filter(|m| m.get_src() as usize == square_index)
            .map(|m| m.get_dest() as usize)
            .collect()
    }

    fn log_move(&mut self, mv: String) {
        self.move_history.push(mv);
    }

    fn undo_last_move(&mut self) {
        if let Some(_) = self.board.undo_move() {
            self.move_history.pop();
        }
    }

    fn make_ai_move(&mut self) {
        let mut moves = self.board.generate_moves();
        moves.shuffle(&mut rand::thread_rng());

        let ai_move = match self.ai_personality.as_str() {
            "Childish" => moves.first().cloned(),
            "Aggressive" => moves.iter().max_by_key(|m| self.evaluate_aggressiveness(m)).cloned(),
            "Defensive" => moves.iter().min_by_key(|m| self.evaluate_aggressiveness(m)).cloned(),
            "Exchange-Prone" => moves.iter().find(|m| self.is_exchange_move(m)).cloned(),
            _ => moves.iter().max_by_key(|m| self.evaluate_board(m)).cloned(),
        };

        if let Some(mv) = ai_move {
            self.board.apply_move(&mv);
            self.log_move(mv.to_string());
        }
    }

    fn evaluate_aggressiveness(&self, mv: &pleco::Move) -> i32 {
        // Custom scoring for aggressive moves (favor captures and threats)
        let mut board_clone = self.board.clone();
        board_clone.apply_move(mv);
        let captures = board_clone.count_material_loss();
        captures + Eval::evaluate(&board_clone)
    }

    fn is_exchange_move(&self, mv: &pleco::Move) -> bool {
        // Check if the move results in an exchange
        mv.is_capture() && !mv.is_check()
    }

    fn evaluate_board(&self, mv: &pleco::Move) -> i32 {
        // General board evaluation for balanced personality
        let mut board_clone = self.board.clone();
        board_clone.apply_move(mv);
        Eval::eval_low(&board_clone)
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
        _ => "?".to_string(),
    }
}

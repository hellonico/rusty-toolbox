use app_chess::ai::moves::compute_ai_move;
use app_chess::ai::score::{method2};
use app_chess::pgn::export::generate_pgn;
use app_chess::pgn::import::{extract_moves, pgn_to_uci};
use app_chess::utils::convert::{piece_to_char, pretty_print_moves, to_usize};
use app_chess::utils::moves::{
    first_pgn_not_in_history, get_legal_moves, simple_function_to_export_moves,
};
use eframe::egui;
use eframe::egui::{Button, RichText};
use egui::Color32;
use lib_egui_utils::my_default_options;
use pleco::board::Board;
use pleco::{BitMove, Player, SQ};
use std::process::exit;

fn main() -> Result<(), eframe::Error> {
    let board = Board::default();
    let options = my_default_options(900.0, 600.0, include_bytes!("../../icon.png"));
    eframe::run_native(
        "Chess AI",
        options,
        Box::new(|_| {
            Ok(Box::new(ChessApp {
                board,
                selected_square: None,
                highlighted_moves: vec![],
                best_moves: vec![],
                pgn_moves: vec![],
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
    best_moves: std::vec::Vec<BitMove>,
    move_history: Vec<String>,
    player_vs_ai: bool,
    ai_personality: String,
    ai_elo: usize,
    status_message: Option<String>,
    pgn_moves: Vec<String>,
}

impl eframe::App for ChessApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            self.render_bar_menu(ui);
            ui.horizontal(|ui| {
                let can_undo = self.move_history.len() > 0;
                if ui.add_enabled(can_undo, Button::new("Undo Move")).clicked() {
                    self.undo_last_move();
                };
                ui.separator();

                let can_pgn = self.pgn_moves.len() > 0;
                if can_pgn {
                    if ui.add_enabled(can_pgn, Button::new("PGN Move")).clicked() {
                        let first_not_in_history = first_pgn_not_in_history(
                            &self.pgn_moves.clone(),
                            self.move_history.clone(),
                        );
                        match first_not_in_history {
                            Some(move_str) => {
                                let uci = pgn_to_uci(&self.board, move_str.as_str());
                                let ok = self.board.apply_uci_move(uci.unwrap().as_str());
                                self.log_move(move_str.to_string());
                            }
                            None => println!("All moves in pgn_moves are in move_history"),
                        }
                    };
                    if ui.add(Button::new("PGN Start")).clicked() {
                        while self.move_history.len() > 0 {
                            self.undo_last_move();
                        }
                    };
                    ui.separator();
                }

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
                        // ui.selectable_value(
                        //     &mut self.ai_personality,
                        //     "Childish".to_string(),
                        //     "Childish",
                        // );
                        // ui.selectable_value(
                        //     &mut self.ai_personality,
                        //     "Aggressive".to_string(),
                        //     "Aggressive",
                        // );
                        // ui.selectable_value(
                        //     &mut self.ai_personality,
                        //     "Defensive".to_string(),
                        //     "Defensive",
                        // );
                        // ui.selectable_value(
                        //     &mut self.ai_personality,
                        //     "Exchange-Prone".to_string(),
                        //     "Exchange-Prone",
                        // );
                        // ui.selectable_value(
                        //     &mut self.ai_personality,
                        //     "Balanced".to_string(),
                        //     "Balanced",
                        // );
                        let personalities = vec![
                            "Childish",
                            "Balanced",
                            "Defensive",
                            "Exchange-Prone",
                            "Method1",
                            "Method2",
                        ];

                        // Add a selectable value for each personality
                        for personality in &personalities {
                            ui.selectable_value(
                                &mut self.ai_personality,
                                personality.to_string(),
                                *personality,
                            );
                        }
                    });
            });

            ui.separator();

            ui.horizontal(|ui| {
                ui.vertical(|ui| {
                    self.render_board(ui);
                    self.check_game_over(ui);
                });
                ui.vertical(|ui| {
                    ui.heading("Move History");
                    let moves = simple_function_to_export_moves(self.move_history.clone());
                    for turn in moves {
                        ui.label(turn);
                    }
                })
            });
        });
    }
}

impl ChessApp {
    fn render_bar_menu(&mut self, ui: &mut egui::Ui) {
        ui.menu_button("File", |ui| {
            if ui.button("New Game").clicked() {
                self.reset_game();
            }
            if ui.button("Import PGN").clicked() {
                self.import_pgn();
                ui.close_menu();
            }
            if ui.button("Export PGN").clicked() {
                self.export_pgn();
                ui.close_menu();
            }
            if ui.button("Quit").clicked() {
                exit(0);
            }
        });
    }

    fn parse_pgn(&mut self, pgn_content: &str) -> Result<(), String> {
        self.move_history.clear();
        self.pgn_moves = extract_moves(&pgn_content).unwrap();

        if let moves_section = &self.pgn_moves {
            for pgn_move in moves_section {
                if let Some(uci_move) = pgn_to_uci(&self.board, &pgn_move) {
                    self.board.apply_uci_move(&uci_move);
                    self.move_history.push(pgn_move.to_string());
                    println!("Played move: {} -> {}", pgn_move, uci_move);
                } else {
                    eprintln!("Invalid move: {}", pgn_move);
                    break;
                }
            }

            println!("Final board position:\n{}", self.board);
        }

        self.move_history.extend_from_slice(&*self.pgn_moves);
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
            let pgn = generate_pgn(self.move_history.clone());
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
        self.best_moves.clear();
        self.best_moves = method2(self.board.clone());
        pretty_print_moves(self.best_moves.clone());
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

        if to_usize(self.best_moves.clone()).contains(&square_index) {
            return Color32::LIGHT_RED;
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
                self.log_move(mv.to_string());

                // if self.player_vs_ai && !self.board.turn() {
                if self.player_vs_ai && self.board.turn() == Player::Black {
                    if let Some(mv) = compute_ai_move(&self.board, &self.ai_personality) {
                        self.board.apply_move(mv);
                        self.log_move(mv.to_string());
                        self.update_best_moves();
                    }
                }
            } else {
                self.selected_square = None;
                self.highlighted_moves.clear();
            }
        } else {
            self.selected_square = Some(square_index);
            self.highlighted_moves = get_legal_moves(self.board.clone(), square_index);
        }
    }

    fn log_move(&mut self, mv: String) {
        self.move_history.push(mv);
    }

    fn undo_last_move(&mut self) {
        self.board.undo_move();
        self.move_history.pop();
    }
}

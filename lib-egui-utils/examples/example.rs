use eframe::egui;
use egui::{Color32, Vec2};
use lib_egui_utils::mywidgets::{RoundedLabel};

// Define the application
struct MyApp;

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Chatting bubbles");

            let bubble = RoundedLabel::orange_bubble("This is some really long text" );
            if ui.add(&bubble).clicked() {
                println!("Clicked BlueBubble with text: {}", bubble.text);
            }

            let bubble = RoundedLabel::blue_bubble("This is some really long text" );
            // Use a reference to the text field
            if ui.add(&bubble).clicked() {
                println!("Clicked BlueBubble with text: {}", bubble.text);
            }

        });
    }
}

// Main entry point
fn main() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions::default();
    eframe::run_native("RoundedLabel Example", options, Box::new(|_cc| Ok(Box::new(MyApp))))
}

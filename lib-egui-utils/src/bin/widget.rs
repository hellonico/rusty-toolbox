use eframe::egui;
use egui::{Response, Ui, Widget};

// Define a struct for the custom widget
pub struct CustomButton {
    label: String,
}

// Implement methods for the widget
impl CustomButton {
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
        }
    }
}

// Implement the `Widget` trait for the custom button
impl Widget for CustomButton {
    fn ui(self, ui: &mut Ui) -> Response {
        let desired_size = ui.available_width() * egui::vec2(0.5, 0.1); // Adjust size as needed
        let (rect, response) = ui.allocate_at_least(desired_size, egui::Sense::click());

        if response.clicked() {
            println!("CustomButton clicked!");
        }

        // Draw the button
        let visuals = ui.style().interact(&response);
        ui.painter()
            .rect(rect, 5.0, visuals.bg_fill, visuals.bg_stroke);
        ui.painter().text(
            rect.center(),
            egui::Align2::CENTER_CENTER,
            &self.label,
            egui::TextStyle::Button.resolve(ui.style()),
            visuals.text_color(),
        );

        response
    }
}

fn main() {
    let options = eframe::NativeOptions::default();
    eframe::run_native("Custom Widget Example", options, Box::new(|_cc| Ok(Box::new(MyApp::default()))));
}

#[derive(Default)]
struct MyApp;

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            // Use the custom widget
            if CustomButton::new("Click Me!").ui(ui).clicked() {
                println!("Action triggered from main app!");
            }
        });
    }
}

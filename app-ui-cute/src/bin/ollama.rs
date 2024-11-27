use eframe::egui::{self, Align, Color32, Context, Layout, ScrollArea, TextBuffer, TextEdit, TopBottomPanel, Ui};
use lib_ollama_utils::ollama;
use pollster::FutureExt;
use std::sync::{Arc, Mutex};
use std::thread;
use tokio::runtime::Builder;
use lib_egui_utils::my_default_options;

fn configure_fonts(ctx: &egui::Context) {
    let mut fonts = egui::FontDefinitions::default();
    fonts.font_data.insert(
        "CuteFont".to_owned(),
        egui::FontData::from_static(include_bytes!("../../SourGummy-Thin.ttf")),
    );
    fonts
        .families
        .get_mut(&egui::FontFamily::Proportional)
        .unwrap()
        .insert(0, "CuteFont".to_owned());
    ctx.set_fonts(fonts);
}

pub struct CuteChatApp {
    messages: Arc<Mutex<Vec<(String, String)>>>, // Messages to be displayed
    input_text: Arc<Mutex<String>>,              // User's input
    streaming_message: Arc<Mutex<Option<String>>>, // Message to stream
    streamed_words: Arc<Mutex<Vec<String>>>,     // Words being streamed
    stream_index: Arc<Mutex<usize>>,             // Index for current word in stream
    stop_streaming: Arc<Mutex<bool>>,            // Whether to stop streaming
}

impl CuteChatApp {
    pub fn new() -> Self {
        Self {
            messages: Arc::new(Mutex::new(vec![
                ("friend".to_owned(), "Hi there! 💖".to_owned()),
                ("user".to_owned(), "Hello! How are you? 😊".to_owned()),
                (
                    "friend".to_owned(),
                    "I’m great! How about you? 🌸".to_owned(),
                ),
            ])),
            input_text: Arc::new(Mutex::new(String::new())),
            streaming_message: Arc::new(Mutex::new(None)),
            streamed_words: Arc::new(Mutex::new(Vec::new())),
            stream_index: Arc::new(Mutex::new(0)),
            stop_streaming: Arc::new(Mutex::new(true)),
        }
    }

    fn start_streaming(&self, input: String) {
        // let streaming_message = self.streaming_message.clone();
        let streamed_words = self.streamed_words.clone();
        // let stream_index = self.stream_index.clone();
        let messages = self.messages.clone();
        let input_field = self.input_text.clone();

        // TODO: handle properly
        // Save the stop signal to indicate we are not stopping streaming
        // *self.stop_streaming.lock().unwrap() = false;

        tokio::spawn(async move {
            let input_guard = input.clone();
            let ii: &str = input_guard.as_ref();

            ollama("llama3.2", ii.as_str(), |token| {
                streamed_words.lock().unwrap().push(token.parse().unwrap());
            })
            .await
            .unwrap();

            // Finalize the message when all words are streamed
            let final_message = streamed_words.lock().unwrap().join(" ");
            messages
                .lock()
                .unwrap()
                .push(("friend".to_owned(), final_message));

            streamed_words.lock().unwrap().clear();

            *input_field.clone().lock().unwrap() = String::from("");
        });
    }

    fn chat_ui(&self, ui: &mut egui::Ui, ctx: &Context) {

        ui.with_layout(Layout::top_down(Align::LEFT), |ui| {

            // Chat header
            ui.horizontal(|ui| {
                ui.colored_label(Color32::BLACK, "💬 Chat with CuteBot");
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label("🐾");
                });
            });

            ScrollArea::vertical().show(ui, |ui| {

                ui.vertical(|ui| {
                    let messages = self.messages.lock().unwrap();
                    for (sender, msg) in messages.iter() {
                        if sender == "friend" {
                            ui.with_layout(egui::Layout::top_down(Align::Min), |ui| {
                                ui.colored_label(Color32::DARK_GREEN, format!("🐾 {}", msg));
                            });
                        } else {
                            ui.with_layout(Layout::right_to_left(Align::Min), |ui| {
                                ui.colored_label(Color32::ORANGE, format!("{} 🌟", msg));
                            });
                        }
                    }

                    if let Ok(streamed_words) = self.streamed_words.lock() {
                        let streamed = streamed_words.join(" "); // Join in the UI thread only for displaying
                        if !streamed_words.is_empty() {
                            ui.colored_label(Color32::DARK_BLUE, format!("🐾 {}", streamed));
                        }
                    }
                });
            });
        });

        ctx.request_repaint();
    }

    fn bottom_bar(&self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.set_min_height(50.0);
            ui.set_max_width(ui.available_width());
            let mut input_text = self.input_text.lock().unwrap();
            let text_edit = ui.add(
                TextEdit::singleline(&mut *input_text)
                    .hint_text("Type something cute...")
                    .frame(true),
            );

            // Detect Enter key press
            if text_edit.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                self.start_streaming(input_text.clone());
                self.send_message(&input_text);
            }

            // Send button
            if ui.button("❤️ Send").clicked() {
                self.start_streaming(input_text.clone());
                self.send_message(&input_text);
                // self.start_streaming(&input_text, );
            }
        });
    }

    fn send_message(&self, input_text: &str) {
        // Move necessary data (messages, input_text) into the thread
        let input_text = input_text.to_string();
        let messages = self.messages.clone();

        let trimmed_text = input_text.trim();
        if !trimmed_text.is_empty() {
            messages
                .lock()
                .unwrap()
                .push(("user".to_owned(), trimmed_text.to_string()));
        }
    }
}

// Implement the eframe::App trait
impl eframe::App for CuteChatApp {
    fn update(&mut self, ctx: &egui::Context, _: &mut eframe::Frame) {
        configure_fonts(ctx); // Apply the font configuration
        ctx.set_pixels_per_point(3.0);

        TopBottomPanel::bottom("bottom_panel").show(ctx, |ui| {
            self.bottom_bar(ui);
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            self.chat_ui(ui, ctx);
        });
    }
}

// Main method
#[tokio::main]
async fn main() -> Result<(), eframe::Error> {
    let options =
        my_default_options(800.0, 500.0, include_bytes!("../../icon.png"));

    eframe::run_native(
        "Cute Chat App",
        options,
        Box::new(|_cc| Ok(Box::<CuteChatApp>::new(CuteChatApp::new()))),
    )
}

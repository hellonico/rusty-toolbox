use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use eframe::egui::{self, Color32, TextEdit, ScrollArea};

fn configure_fonts(ctx: &egui::Context) {
    let mut fonts = egui::FontDefinitions::default();
    fonts.font_data.insert(
        "CuteFont".to_owned(),
        egui::FontData::from_static(include_bytes!("../../ui-fonts/SourGummy-Thin.ttf")),
    );
    fonts.families.get_mut(&egui::FontFamily::Proportional).unwrap().insert(0, "CuteFont".to_owned());
    ctx.set_fonts(fonts);
}

pub struct CuteChatApp {
    messages: Arc<Mutex<Vec<(String, String)>>>,  // Messages to be displayed
    input_text: Arc<Mutex<String>>,               // User's input
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
                ("friend".to_owned(), "I’m great! How about you? 🌸".to_owned()),
            ])),
            input_text: Arc::new(Mutex::new(String::new())),
            streaming_message: Arc::new(Mutex::new(None)),
            streamed_words: Arc::new(Mutex::new(Vec::new())),
            stream_index: Arc::new(Mutex::new(0)),
            stop_streaming: Arc::new(Mutex::new(true)),
        }

    }
    fn start_streaming(&self) {
        thread::sleep(Duration::from_millis(150));

        let streaming_message = self.streaming_message.clone();
        let streamed_words = self.streamed_words.clone();
        let stream_index = self.stream_index.clone();
        let messages = self.messages.clone();

        // Save the stop signal to indicate we are not stopping streaming
        *self.stop_streaming.lock().unwrap() = false;

        // Spawn a background thread for streaming
        thread::spawn(move || {
            loop {
                let mut message_guard = streaming_message.lock().unwrap();
                if let Some(streaming_msg) = message_guard.take() {
                    let words = streaming_msg.split_whitespace().collect::<Vec<_>>();
                    let mut index_guard = stream_index.lock().unwrap();

                    // Stream words with a delay
                    while *index_guard < words.len() {
                        // Add the next word to the streamed_words
                        streamed_words.lock().unwrap().push(words[*index_guard].to_string());
                        *index_guard += 1;

                        // Wait for 500ms before adding the next word
                        thread::sleep(Duration::from_millis(50));
                    }

                    // Finalize the message when all words are streamed
                    let final_message = streamed_words.lock().unwrap().join(" ");
                    messages.lock().unwrap().push(("friend".to_owned(), final_message));

                    // Reset for the next message
                    *message_guard = None;
                    *index_guard = 0;
                    streamed_words.lock().unwrap().clear();
                } else {
                    break; // Exit if no streaming message is available
                }
            }
        });
    }


    fn chat_ui(&self, ui: &mut egui::Ui) {
        ui.vertical(|ui| {
            ui.add_space(10.0); // Top padding

            // Chat header
            ui.horizontal(|ui| {
                ui.colored_label(Color32::LIGHT_BLUE, "💬 Chat with CuteBot");
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label("🐾");
                });
            });

            ui.add_space(10.0);

            // Chat messages (Scrollable)
            ScrollArea::vertical().show(ui, |ui| {
                let messages = self.messages.lock().unwrap();
                for (sender, msg) in messages.iter() {
                    ui.horizontal(|ui| {
                        if sender == "friend" {
                            ui.colored_label(Color32::LIGHT_BLUE, format!("🐾 {}", msg));
                        } else {
                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Min), |ui| {
                                ui.colored_label(Color32::ORANGE, format!("{} 🌟", msg));
                            });
                        }
                    });
                }

                // Display the message currently being streamed
                // Inside the UI update logic (UI thread):
                if let Ok(streamed_words) = self.streamed_words.lock() {
                    if !streamed_words.is_empty() {
                        // Join words and display the result
                        let streamed = streamed_words.join(" "); // Join in the UI thread only for displaying
                        ui.colored_label(Color32::LIGHT_BLUE, format!("🐾 {}", streamed)); // Display the result
                    }
                    ui.ctx().request_repaint();
                }
            });

            ui.add_space(10.0);

            // Input Area
            ui.horizontal(|ui| {
                let mut input_text = self.input_text.lock().unwrap();
                let text_edit = ui.add(
                    TextEdit::singleline(&mut *input_text)
                        .hint_text("Type something cute...")
                        .frame(true),
                );

                // Detect Enter key press
                if text_edit.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                    self.send_message(&input_text);
                    self.start_streaming();
                }

                // Send button
                if ui.button("❤️ Send").clicked() {
                    self.send_message(&input_text);
                    self.start_streaming();
                }
            });
        });
    }


    fn send_message(&self, input_text: &str) {
        // Move necessary data (messages, input_text) into the thread
        let input_text = input_text.to_string();
        let messages = self.messages.clone();
        let streaming_message = self.streaming_message.clone();
        let sinput_text = self.input_text.clone();

        // Spawn a background thread to process message and trigger streaming
        thread::spawn(move || {
            let trimmed_text = input_text.trim();
            if !trimmed_text.is_empty() {
                // Add user message to chat
                messages.lock().unwrap().push(("user".to_owned(), trimmed_text.to_string()));

                // Start streaming the bot response
                let bot_response = format!("That’s adorable! 🥰 \"{}\"", trimmed_text);
                *streaming_message.lock().unwrap() = Some(bot_response);

                // Start the streaming process in the background
                // self.start_streaming();
            }

            // Clear input box after processing the message
            let mut input_text_lock = sinput_text.lock().unwrap();
            *input_text_lock = String::new();
        });
    }
}

// Implement the eframe::App trait
impl eframe::App for CuteChatApp {
    fn update(&mut self, ctx: &egui::Context, _: &mut eframe::Frame) {
        configure_fonts(ctx); // Apply the font configuration
        egui::CentralPanel::default().show(ctx, |ui| {
            self.chat_ui(ui);
        });
    }
}

// Main method
fn main() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions::default();
    eframe::run_native(
        "Cute Chat App",
        options,
        Box::new(|_cc| Ok(Box::<CuteChatApp>::new(CuteChatApp::new()))),
    )
}

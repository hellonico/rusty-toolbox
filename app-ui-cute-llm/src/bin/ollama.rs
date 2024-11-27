use eframe::egui::{self, menu, Align, Color32, ComboBox, Context, Layout, RichText, ScrollArea, TextBuffer, TextEdit, TopBottomPanel, Ui};
use lib_egui_utils::my_default_options;
use lib_ollama_utils::{fetch_models, ollama, ollama_with_messages};
use std::sync::{Arc, Mutex};

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
    ollama_url: Arc<Mutex<String>>,              // Ollama API URL
    ollama_model: Arc<Mutex<String>>,            // Ollama model name
    show_config_dialog: Arc<Mutex<bool>>,        // Whether to show the config dialog
    ollama_system_prompt: Arc<Mutex<String>>,
    chat_mode: Arc<Mutex<bool>>,        // Whether to show the config dialog
    available_models: Arc<Mutex<Vec<String>>>,
}

impl CuteChatApp {
    pub fn new() -> Self {
        Self {
            messages: Arc::new(Mutex::new(vec![
                // ("assistant".to_owned(), "Hi there! 💖".to_owned()),
                // ("user".to_owned(), "Hello! How are you? 😊".to_owned()),
                // (
                //     "assistant".to_owned(),
                //     "I’m great! How about you? 🌸".to_owned(),
                // ),
            ]
            )),
            input_text: Arc::new(Mutex::new(String::new())),
            streaming_message: Arc::new(Mutex::new(None)),
            streamed_words: Arc::new(Mutex::new(Vec::new())),
            stream_index: Arc::new(Mutex::new(0)),
            stop_streaming: Arc::new(Mutex::new(true)),
            ollama_url: Arc::new(Mutex::new("http://localhost:11434".to_owned())), // Default URL
            ollama_model: Arc::new(Mutex::new("llama3.2".to_owned())),
            show_config_dialog: Arc::new(Mutex::new(false)),
            ollama_system_prompt: Arc::new(Mutex::new(String::from("You are a young dyamic and talkative assistant"))),
            chat_mode: Arc::new(Mutex::new(true)),
            available_models: Arc::new(Mutex::new(Vec::new())),
        }
    }

    fn load_models(&self) {
        let ollama_url = self.ollama_url.lock().unwrap().clone();
        let available_models = self.available_models.clone();

        tokio::spawn(async move {
            let models = fetch_models(ollama_url).await;
            *available_models.lock().unwrap() = models;
        });
    }

    fn start_streaming(&self, input: String) {
        // let streaming_message = self.streaming_message.clone();
        let streamed_words = self.streamed_words.clone();
        // let stream_index = self.stream_index.clone();
        let messages = self.messages.clone();
        let input_field = self.input_text.clone();

        let ollama_url = self.ollama_url.lock().unwrap().clone();
        let ollama_model = self.ollama_model.lock().unwrap().clone();

        let system_message = {
            // Lock `system_prompt` and clone its value
            self.ollama_system_prompt.lock().unwrap().clone()
        };

        let mut ollama_messages = self.messages
            .lock()
            .unwrap()
            .clone();
        ollama_messages.insert(0, ("system".to_owned(), system_message));

        let chat_mode = self.chat_mode.lock().unwrap().clone();

        // TODO: handle properly
        // Save the stop signal to indicate we are not stopping streaming
        // *self.stop_streaming.lock().unwrap() = false;

        tokio::spawn(async move {
            // let input_guard = input.clone();

            if chat_mode {
                ollama_with_messages(&*ollama_url, &ollama_model, &ollama_messages, |token| {
                    streamed_words.lock().unwrap().push(token.parse().unwrap());
                })
                    .await
                    .expect("error");

            } else {
                ollama(&*ollama_url, &ollama_model, &input, |token| {
                    streamed_words.lock().unwrap().push(token.parse().unwrap());
                })
                    .await
                    .expect("error");
            }


            // Finalize the message when all words are streamed
            let final_message = streamed_words.lock().unwrap().join(" ");
            messages
                .lock()
                .unwrap()
                .push(("assistant".to_owned(), final_message));

            streamed_words.lock().unwrap().clear();

            *input_field.clone().lock().unwrap() = String::from("");
        });
    }

    fn config_dialog_ui(&self, ctx: &Context) {
        self.load_models();

        let mut show_dialog = self.show_config_dialog.lock().unwrap();
        if *show_dialog {
            let mut url = self.ollama_url.lock().unwrap().clone(); // Get a copy to work with
            let mut model = self.ollama_model.lock().unwrap().clone();
            let mut ollama_system_prompt = self.ollama_system_prompt.lock().unwrap().clone();
            let mut chat_mode = self.chat_mode.lock().unwrap().clone();
            let available_models = self.available_models.lock().unwrap().clone();
            // let model_names = fetch_models(String::from("http://localhost:11434"));
            egui::Window::new("Ollama Configuration")
                .collapsible(false)
                .resizable(false)
                .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                .show(ctx, |ui| {
                    ui.label("Set Ollama URL:");
                    if ui.text_edit_singleline(&mut url).changed() {
                        *self.ollama_url.lock().unwrap() = url.clone(); // Write back changes
                    }

                    ui.label("Set Ollama Model:");
                    if ComboBox::from_label("Select Model")
                        .selected_text(model.clone()) // Use `model.clone()` here since `selected_text` expects `String` or `&str`
                        .show_ui(ui, |ui| {
                            for name in available_models.iter() {
                                ui.selectable_value(&mut model, name.clone(), name); // `&mut model` ensures the value is updated
                            }
                        })
                        .inner
                        .is_some()
                    {
                        *self.ollama_model.lock().unwrap() = model.clone(); // Write back changes
                    }

                    // if ui.checkbox(&mut chat_mode, "Chat Mode:").changed() {
                    //     *self.chat_mode.lock().unwrap() = chat_mode.clone(); // Write back changes
                    // };

                    ui.label("System Prompt");
                    if ui.text_edit_multiline(&mut ollama_system_prompt).changed() {
                        *self.ollama_system_prompt.lock().unwrap() = ollama_system_prompt.clone(); // Write back changes
                    }

                    ui.horizontal(|ui| {
                        if ui.button("Close").clicked() {
                            *show_dialog = false;
                        }
                    });
                });
        }
    }


    fn menu_bar(&self, ctx: &Context) {

        // TopBottomPanel::top("menu_bar").show(ctx, |ui| {
        //     ui.horizontal(|ui| {
        //         if ui.button("Settings").clicked() {
        //             *self.show_config_dialog.lock().unwrap() = true;
        //         }
        //     });
        // });
        //
        // menu::bar(ui, |ui| {
        //     ui.menu_button("Menu", |ui| {
        //         if ui.button("Settings").clicked() {
        //             *self.show_config_dialog.lock().unwrap() = true;
        //         }
        //     });
        // });
    }

    fn display_configuration(&self, ui: &mut egui::Ui) {
        // Read the current values
        let ollama_url = self.ollama_url.lock().unwrap();
        let ollama_model = self.ollama_model.lock().unwrap();
        let chat_mode = self.chat_mode.lock().unwrap();

        // Add a small section with the configuration details
        ui.horizontal(|ui| {
            ui.add_space(10.0); // Add some space for neat alignment
            ui.label(
                RichText::new(format!("URL: {}", ollama_url))
                    .small() // Make the font smaller
                    .color(egui::Color32::GRAY),
            );
            ui.add_space(20.0); // Add some spacing between URL and Model
            ui.label(
                RichText::new(format!("Model: {}", ollama_model))
                    .small() // Make the font smaller
                    .color(egui::Color32::GRAY),
            );
            ui.add_space(20.0); // Add some spacing between URL and Model
            ui.label(
                RichText::new(format!("Chat Mode: {}", chat_mode))
                    .small() // Make the font smaller
                    .color(egui::Color32::GRAY),
            );
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
                        if sender == "assistant" {
                            ui.with_layout(egui::Layout::top_down(Align::Min), |ui| {
                                let pink = Color32::from_rgb(255, 105, 180); // RGB values for a bright pink
                                ui.colored_label(pink, format!("🐾 {}", msg));
                            });
                        } else {
                            ui.with_layout(Layout::top_down(Align::Max), |ui| {
                                ui.colored_label(Color32::ORANGE, format!("{} 🌟", msg));
                            });
                        }
                    }

                    if let Ok(streamed_words) = self.streamed_words.lock() {
                        let streamed = streamed_words.join(" "); // Join in the UI thread only for displaying
                        if !streamed_words.is_empty() {
                            ui.colored_label(Color32::LIGHT_BLUE, format!("🐾 {}", streamed));
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
                self.send_message(&input_text);
                self.start_streaming(input_text.clone());
            }

            if ui.button("❤ Send").clicked() {
                self.send_message(&input_text);
                self.start_streaming(input_text.clone());
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
            self.display_configuration(ui);

            // self.menu_bar(ctx); // Add the menu bar
        });

        egui::CentralPanel::default().show(ctx, |ui| {

            menu::bar(ui, |ui| {
                ui.menu_button("Menu", |ui| {
                    if ui.button("Settings").clicked() {
                        *self.show_config_dialog.lock().unwrap() = true;
                    }
                    if ui.button("Clear History").clicked() {
                        self.messages.lock().unwrap().clear();
                    }
                });
            });

            self.config_dialog_ui(ctx); // Display configuration dialog if needed


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

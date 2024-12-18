use eframe::egui::{self, menu, Align, Color32, ComboBox, Context, CursorIcon, Frame, Layout, RichText, ScrollArea, Style, TextBuffer, TextEdit, TopBottomPanel, Ui, Visuals};
use egui::Window;
use egui_extras::{Column, TableBuilder};
use lib_egui_utils::{add_font, configure_text_styles, my_default_options};
use lib_ollama_utils::{fetch_models, model_download, ollama, ollama_with_messages};
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::{Read, Write};
use std::path::PathBuf;
use std::process::exit;
use std::sync::{Arc, Mutex};
use std::{env, fs, thread};
use std::collections::HashSet;
use arboard::Clipboard;
use serde_json::Value;
use lib_egui_utils::mywidgets::{RoundedLabel};

#[derive(Serialize, Deserialize)]
struct ConfigData {
    ollama_system_prompt: String,
    messages: Vec<(String, String)>,
    ollama_url: String,
    chat_mode: bool,
    ollama_model: String,
}

#[derive(Clone)]
pub struct CuteChatApp {
    input_text: Arc<Mutex<String>>,                // User's input
    streaming_message: Arc<Mutex<Option<String>>>, // Message to stream
    streamed_words: Arc<Mutex<Vec<String>>>,       // Words being streamed
    stream_index: Arc<Mutex<usize>>,               // Index for current word in stream
    stop_streaming: Arc<Mutex<bool>>,              // Whether to stop streaming
    show_config_dialog: Arc<Mutex<bool>>,          // Whether to show the config dialog
    available_models: Arc<Mutex<Vec<String>>>,
    show_load_dialog: Arc<Mutex<bool>>,
    simple_ui: bool,
    show_prompt_dialog: bool,

    ollama_system_prompt: Arc<Mutex<String>>,
    messages: Arc<Mutex<Vec<(String, String)>>>, // Messages to be displayed
    ollama_url: Arc<Mutex<String>>,              // Ollama API URL
    chat_mode: Arc<Mutex<bool>>,                 // Whether to show the config dialog
    ollama_model: Arc<Mutex<String>>,            // Ollama model name

    download_status: Arc<Mutex<String>>,
    pull_model: Arc<Mutex<String>>,
}

impl CuteChatApp {
    fn new() -> Self {
        Self {
            messages: Arc::new(Mutex::new(vec![
                // ("assistant".to_owned(), "Hi there! 💖".to_owned()),
                // ("user".to_owned(), "Hello! How are you? 😊".to_owned()),
                // (
                //     "assistant".to_owned(),
                //     "I’m great! How about you? 🌸".to_owned(),
                // ),
            ])),
            input_text: Arc::new(Mutex::new(String::new())),
            streaming_message: Arc::new(Mutex::new(None)),
            streamed_words: Arc::new(Mutex::new(Vec::new())),
            stream_index: Arc::new(Mutex::new(0)),
            stop_streaming: Arc::new(Mutex::new(true)),
            ollama_url: Arc::new(Mutex::new("http://localhost:11434".to_owned())), // Default URL
            ollama_model: Arc::new(Mutex::new("llama3.2".to_owned())),
            show_config_dialog: Arc::new(Mutex::new(false)),
            ollama_system_prompt: Arc::new(Mutex::new(String::from(
                "You are a young dyamic and talkative assistant",
            ))),
            chat_mode: Arc::new(Mutex::new(true)),
            available_models: Arc::new(Mutex::new(Vec::new())),
            show_load_dialog: Arc::new(Mutex::new(false)),
            simple_ui: false,
            show_prompt_dialog: false,
            download_status: Arc::new(Mutex::new(String::new())),
            pull_model: Arc::new(Mutex::new(String::new())),
        }
    }

    pub fn save_to_file(&self) -> std::io::Result<()> {
        let messages = self.messages.lock().unwrap().clone();

        // Ensure there's at least one message to name the file
        if let Some((_, first_message)) = messages.first() {
            let sanitized_filename = Self::sanitize_filename(first_message);
            let file_path = Self::get_config_file_path(&sanitized_filename)?;
            let config_folder = file_path.parent().unwrap();

            // Ensure the directory exists
            fs::create_dir_all(config_folder)?;

            // Gather data into a serializable struct
            let data = ConfigData {
                ollama_system_prompt: self.ollama_system_prompt.lock().unwrap().clone(),
                messages,
                ollama_url: self.ollama_url.lock().unwrap().clone(),
                chat_mode: *self.chat_mode.lock().unwrap(),
                ollama_model: self.ollama_model.lock().unwrap().clone(),
            };

            // Serialize and save to file
            let json_data = serde_json::to_string_pretty(&data)?;
            let mut file = File::create(file_path)?;
            file.write_all(json_data.as_bytes())?;
        }

        Ok(())
    }

    /// Load the state from a JSON file
    pub fn load_from_file(&self, file_path: Option<&str>) -> std::io::Result<()> {
        let file_path = if let Some(path) = file_path {
            PathBuf::from(path)
        } else {
            // No parameter provided, find the most recent file
            let config_folder = Self::get_config_folder()?;
            fs::read_dir(&config_folder)?
                .filter_map(|entry| entry.ok()) // Ignore any invalid entries
                .filter(|entry| {
                    entry.path().extension().and_then(|ext| ext.to_str()) == Some("json")
                })
                .max_by_key(|entry| entry.metadata().and_then(|meta| meta.modified()).ok())
                .map(|entry| entry.path())
                .ok_or_else(|| {
                    std::io::Error::new(std::io::ErrorKind::NotFound, "No JSON files found")
                })?
        };

        // Read and deserialize the JSON data
        if file_path.exists() {
            let mut file = File::open(file_path)?;
            let mut json_data = String::new();
            file.read_to_string(&mut json_data)?;

            let data: ConfigData = serde_json::from_str(&json_data)?;

            // Populate the fields
            *self.ollama_system_prompt.lock().unwrap() = data.ollama_system_prompt;
            *self.messages.lock().unwrap() = data.messages;
            *self.ollama_url.lock().unwrap() = data.ollama_url;
            *self.chat_mode.lock().unwrap() = data.chat_mode;
            *self.ollama_model.lock().unwrap() = data.ollama_model;
        }

        Ok(())
    }

    fn get_config_folder() -> std::io::Result<PathBuf> {
        let home_dir = env::var("HOME").map(PathBuf::from).unwrap();
        Ok(home_dir.join(".config/cutellama"))
    }

    /// Helper function to get the config file path
    fn get_config_file_path(filename: &str) -> std::io::Result<PathBuf> {
        Ok(Self::get_config_folder()?.join(format!("{}.json", filename)))
    }
    /// Helper function to sanitize a filename
    fn sanitize_filename(name: &str) -> String {
        name.chars()
            .filter(|c| c.is_alphanumeric() || *c == ' ')
            .collect::<String>()
            .trim()
            .replace(' ', "_")
    }
    //////

    fn load_models(&self) {
        let ollama_url = self.ollama_url.lock().unwrap().clone();
        let available_models = self.available_models.clone();

        tokio::spawn(async move {
            let models = fetch_models(ollama_url).await;
            *available_models.lock().unwrap() = models.iter().map(|model| model.name.clone()).collect();
        });
    }

    fn download_model(&self) {
        let ollama_url = self.ollama_url.lock().unwrap().clone();
        let model = self.pull_model.lock().unwrap().clone();
        let download_status = Arc::clone(&self.download_status); // Share the Arc<Mutex<String>>
        println!("Downloading model from: {}", &ollama_url);
        tokio::spawn(async move {
            let models = model_download(&*ollama_url, &model, |s| {
                let mut status = download_status.lock().unwrap();
                *status = s.to_string();
                println!("{:}",s);
            }).await;
        });
    }

    pub fn show_load_dialog(&mut self, ctx: &egui::Context) {
        // let show_load_dialog = self.show_load_dialog;
        if self.show_load_dialog.lock().unwrap().clone() {
            Window::new("Load Session")
                .collapsible(false)
                .min_width(500.0)
                //.open(show_load_dialog) // Use local copy
                .show(ctx, |ui| {
                    let config_folder = match Self::get_config_folder() {
                        Ok(path) => path,
                        Err(err) => {
                            ui.label(format!("Failed to access config folder: {}", err));
                            return;
                        }
                    };

                    let files = match Self::get_json_files_with_metadata(&config_folder) {
                        Ok(files) => files,
                        Err(err) => {
                            ui.label(format!("Failed to read files: {}", err));
                            return;
                        }
                    };

                    // Create a scrollable table
                    TableBuilder::new(ui)
                        .striped(true) // Optional: Stripe the table rows
                        .column(Column::remainder()) // File name column
                        .column(Column::remainder()) // Modified date column
                        .header(20.0, |mut header| {
                            header.col(|ui| {
                                ui.label("File Name");
                            });
                            header.col(|ui| {
                                ui.label("Last Modified");
                            });
                        })
                        .body(|mut body| {
                            for (file_path, modified_time) in files {
                                if let Some(file_name) =
                                    file_path.file_stem().and_then(|f| f.to_str())
                                {
                                    body.row(20.0, |mut row| {
                                        row.col(|ui| {
                                            let label =
                                                ui.label(file_name.clone().replace("_", " "));
                                            if label.hovered() {
                                                let row_rect = ui.available_rect_before_wrap();
                                                ui.painter().rect_filled(
                                                    row_rect,
                                                    0.0,
                                                    ui.visuals().hyperlink_color,
                                                );
                                            }
                                            if label.clicked() {
                                                let file_path_string =
                                                    file_path.to_str().unwrap().to_string();
                                                if let Err(err) =
                                                    self.load_from_file(Some(&file_path_string))
                                                {
                                                    eprintln!(
                                                        "Failed to load file '{}': {}",
                                                        file_name, err
                                                    );
                                                }
                                                *self.show_load_dialog.lock().unwrap() = false;
                                            }
                                        });
                                        row.col(|ui| {
                                            ui.label(&modified_time);
                                        });
                                    });
                                }
                            }
                        });
                    // });
                });
        }

        // self.show_load_dialog.lock() = show_load_dialog;
    }


    pub fn toggle_load_dialog(self) {
        let mut lock = self.show_load_dialog.lock().unwrap();
        *lock = !*lock;
    }
    pub fn toggle_config_dialog(self) {
        let mut lock = self.show_config_dialog.lock().unwrap();
        *lock = !*lock;
    }

    fn get_all_prompts(folder: &PathBuf) -> Vec<String> {
        let mut prompts_set = HashSet::new();

        if let Ok(entries) = fs::read_dir(folder) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().and_then(|ext| ext.to_str()) == Some("json") {
                    if let Ok(content) = fs::read_to_string(&path) {
                        if let Ok(json) = serde_json::from_str::<Value>(&content) {
                            if let Some(prompt) = json["ollama_system_prompt"].as_str() {
                                prompts_set.insert(prompt.to_string());
                            }
                        }
                    }
                }
            }
        }


        // Convert HashSet to Vec and sort for consistent UI display
        let mut prompts: Vec<String> = prompts_set.into_iter().collect();
        prompts.sort();
        prompts
        // prompts
    }

    fn get_json_files_with_metadata(folder: &PathBuf) -> std::io::Result<Vec<(PathBuf, String)>> {
        let mut files = Vec::new();
        for entry in fs::read_dir(folder)? {
            if let Ok(entry) = entry {
                let path = entry.path();
                if path.extension().and_then(|ext| ext.to_str()) == Some("json") {
                    if let Ok(metadata) = fs::metadata(&path) {
                        if let Ok(modified) = metadata.modified() {
                            if let datetime = chrono::DateTime::<chrono::Local>::from(modified) {
                                files
                                    .push((path, datetime.format("%Y-%m-%d %H:%M:%S").to_string()));
                            }
                        }
                    }
                }
            }
        }
        files.sort_by(|a, b| b.1.cmp(&a.1));
        Ok(files)
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

        let mut ollama_messages = self.messages.lock().unwrap().clone();
        ollama_messages.insert(0, ("system".to_owned(), system_message));

        let chat_mode = self.chat_mode.lock().unwrap().clone();

        // TODO: handle properly
        // Save the stop signal to indicate we are not stopping streaming
        // *self.stop_streaming.lock().unwrap() = false;
        // let clone = self.clone();

        tokio::spawn(async move {
            // let input_guard = input.clone();

            if chat_mode {
                ollama_with_messages(&*ollama_url, &ollama_model, &ollama_messages, |token| {
                    // println!("{:?}", token);
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
            let final_message = streamed_words.lock().unwrap().join("");
            messages
                .lock()
                .unwrap()
                .push(("assistant".to_owned(), final_message));

            streamed_words.lock().unwrap().clear();

            *input_field.clone().lock().unwrap() = String::from("");
            // self.save_to_file().unwrap();
        });
    }

    fn prompt_dialog_ui(&mut self, ctx: &Context) {
        let mut show_dialog = self.show_prompt_dialog;

        if show_dialog {
            let prompts = Self::get_all_prompts(&Self::get_config_folder().unwrap());
            Window::new("Prompts")
                .collapsible(false)
                .resizable(true)
                .min_width(500.0)
                .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                .show(ctx, |ui| {

                    TableBuilder::new(ui)
                        .striped(true) // Optional: Stripe the table rows
                        .column(Column::remainder()) // File name column
                        // .column(Column::remainder()) // Modified date column
                        // .header(20.0, |mut header| {
                        //     // header.col(|ui| {
                        //     //     ui.label("Prompt");
                        //     // });
                        // })
                        .body(|mut body| {
                            for (prompt) in prompts {
                                    body.row(20.0, |mut row| {
                                        row.col(|ui| {
                                            let label =
                                                ui.label(prompt.clone());
                                            if label.hovered() {
                                                let row_rect = ui.available_rect_before_wrap();
                                                ui.painter().rect_filled(
                                                    row_rect,
                                                    0.0,
                                                    ui.visuals().hyperlink_color,
                                                );
                                            }
                                            if label.clicked() {
                                                *self.ollama_system_prompt.lock().unwrap() = prompt.clone().to_string();
                                                self.show_prompt_dialog = false;
                                            }
                                        });
                                    });
                                }
                        });

                    ui.horizontal(|ui| {
                        if ui.button("Edit").clicked() {
                            self.show_prompt_dialog = false;
                            *self.show_config_dialog.lock().unwrap() = true;
                        }
                        if ui.button("Close").clicked() {
                            self.save_to_file().unwrap();
                            self.show_prompt_dialog = false;
                        }
                    });
                });
        }
    }

    fn config_dialog_ui(&self, ctx: &Context) {
        let mut show_dialog = self.show_config_dialog.lock().unwrap();
        if *show_dialog {
            self.load_models();
            let mut url = self.ollama_url.lock().unwrap().clone(); // Get a copy to work with
            let mut model = self.ollama_model.lock().unwrap().clone();
            let mut md = self.pull_model.lock().unwrap().clone();
            let mut ollama_system_prompt = self.ollama_system_prompt.lock().unwrap().clone();
            let mut chat_mode = self.chat_mode.lock().unwrap().clone();
            let mut download_status = Arc::clone(&self.download_status); // Share the Arc<Mutex<String>>
            let available_models = self.available_models.lock().unwrap().clone();
            // let model_names = fetch_models(String::from("http://localhost:11434"));
            Window::new("Cute Configuration")
                .collapsible(false)
                .resizable(false)
                .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                .show(ctx, |ui| {
                    ui.label("Set Ollama URL:");
                    if ui.text_edit_singleline(&mut url).changed() {
                        *self.ollama_url.lock().unwrap() = url.clone(); // Write back changes
                        self.load_models();
                    }

                    ui.label("Set Ollama Model:");
                    if ComboBox::from_label("")
                        .selected_text(model.clone()) // Use `model.clone()` here since `selected_text` expects `String` or `&str`
                        .show_ui(ui, |ui| {
                            for name in available_models.iter() {
                                ui.selectable_value(&mut model, name.clone(), name);
                                // `&mut model` ensures the value is updated
                            }
                        })
                        .inner
                        .is_some()
                    {
                        *self.ollama_model.lock().unwrap() = model.clone(); // Write back changes
                    }
                    ui.label("Pull Model:");
                    if ui.text_edit_singleline(&mut md).changed() {
                        *self.pull_model.lock().unwrap() = md.clone();
                    };
                    if ui.button("Download").clicked() {
                        self.download_model();
                    }
                    ui.label(download_status.lock().unwrap().clone().to_string());

                    if ui.checkbox(&mut chat_mode, "Chat Mode:").changed() {
                        *self.chat_mode.lock().unwrap() = chat_mode.clone(); // Write back changes
                    };

                    ui.label("System Prompt");
                    if ui.text_edit_multiline(&mut ollama_system_prompt).changed() {
                        *self.ollama_system_prompt.lock().unwrap() = ollama_system_prompt.clone();
                        // Write back changes
                    }

                    ui.horizontal(|ui| {
                        if ui.button("Close").clicked() {
                            *show_dialog = false;
                        }
                    });
                });
        }
    }

    fn panel_session_details(&self, ui: &mut egui::Ui) {

        // Add a small section with the configuration details
        if !self.simple_ui {
            // Read the current values
            let ollama_url = self.ollama_url.lock().unwrap();
            let ollama_model = self.ollama_model.lock().unwrap();
            let chat_mode = self.chat_mode.lock().unwrap();

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

    }

    fn chat_ui(&mut self, ui: &mut egui::Ui, ctx: &Context) {
        ui.with_layout(Layout::top_down(Align::LEFT), |ui| {
            // Chat header
            ui.horizontal(|ui| {
                if !self.simple_ui {
                    //ui.colored_label(Color32::BLACK, "💬 Chat with Cuteness");
                    ui.label("💬 Chat with Cuteness");
                    ui.separator();
                    ui.horizontal_wrapped(|ui| {
                        if ui.label(self.ollama_system_prompt.lock().unwrap().clone()).clicked() {
                            self.show_prompt_dialog = true;
                        };
                    });
                    ui.with_layout(Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.label("🐾");
                    });
                }
            });

            ScrollArea::vertical().show(ui, |ui| {
                ui.vertical(|ui| {
                    let messages = self.messages.lock().unwrap();
                    for (sender, msg) in messages.iter() {
                        if sender == "assistant" {
                            ui.with_layout(egui::Layout::top_down(Align::Min), |ui| {


                                ui.horizontal_wrapped(|ui| {
                                    if ui.label(format!("🐾 {}", msg)).clicked() {
                                        let mut clipboard = Clipboard::new().unwrap();
                                        clipboard.set_text(msg.clone());
                                    }
                                    /*
                                    let bubble = RoundedLabel::blue_bubble(
                                        format!("🐾 {}", msg).as_str(),
                                    );
                                    if ui
                                        .add(&bubble)
                                        .clicked()
                                    {
                                        let mut clipboard = Clipboard::new().unwrap();
                                        clipboard.set_text(msg.clone());
                                    }
                                    &/
                                     */
                                });
                                // Use a reference to the text field

                                //
                                // //let pink = Color32::from_rgb(255, 105, 180); // RGB values for a bright pink
                                // let pink = Color32::DARK_GRAY;
                                // ui.colored_label(pink, format!("🐾 {}", msg));
                                // if ui.label("📃").on_hover_text("Copy").on_hover_cursor(CursorIcon::Copy).clicked() {
                                //     let mut clipboard = Clipboard::new().unwrap();
                                //     clipboard.set_text(msg.clone());
                                // }

                            });
                        } else {
                            ui.with_layout(Layout::top_down(Align::Max), |ui| {
                                //ui.colored_label(Color32::BLACK, format!("{} 🌟", msg));
                                let bubble = RoundedLabel::orange_bubble(
                                    format!("🐾 {}", msg).as_str(),
                                );
                                if ui
                                    .add(&bubble)
                                    .clicked()
                                {
                                    let mut clipboard = Clipboard::new().unwrap();
                                    clipboard.set_text(msg.clone());
                                }
                            });
                        }
                    }

                    if let Ok(streamed_words) = self.streamed_words.lock() {
                        let streamed = streamed_words.join(""); // Join in the UI thread only for displaying
                        if !streamed_words.is_empty() {
                            ui.colored_label(Color32::DARK_BLUE, format!("🐾 {}", streamed));
                        }
                    }
                });
            });
        });

        ctx.request_repaint();
    }

    fn clear_session(&self) {
        self.messages.lock().unwrap().clear();
        //REMINDER: that was where the deadlock was happening
        //self.input_text.lock().unwrap().clear();
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

    fn menu_bar(&mut self, ui: &mut Ui) {
        menu::bar(ui, |ui| {
            ui.menu_button("Menu", |ui| {
                if ui.button("Settings").clicked() {
                    *self.show_config_dialog.lock().unwrap() = true;
                }

                ui.menu_button("Session", |ui| {
                    if ui.button("Prompts ...").clicked() {
                        self.show_prompt_dialog = false;
                        ui.close_menu();
                    };
                    if ui.button("Save").clicked() {
                        self.save_to_file();
                        ui.close_menu();
                    }
                    if ui.button("Clear").clicked() {
                        self.messages.lock().unwrap().clear();
                        ui.close_menu();
                    }
                    if ui.button("Most Recent").clicked() {
                        self.load_from_file(None);
                        ui.close_menu();
                    }
                    if ui.button("Load ...").clicked() {
                        self.clone().toggle_load_dialog();
                        ui.close_menu();
                    }
                    if ui.button("Open Folder").clicked() {
                        open::that(Self::get_config_folder().unwrap()).unwrap_or(());
                        ui.close_menu();
                    }
                });

                ui.checkbox(&mut self.simple_ui, "Zen Mode");

                if ui.button("Quit").clicked() {
                    exit(0);
                }
            });
        });
    }
}

// Implement the eframe::App trait
impl eframe::App for CuteChatApp {
    fn update(&mut self, ctx: &egui::Context, _: &mut eframe::Frame) {
        let visuals = Visuals::light();
        ctx.set_style(Style {
            visuals: Visuals {
                window_fill: Color32::WHITE, // Set the background to white
                ..visuals
            },
            ..Style::default()
        });

        // Show the dialog if open
        self.show_load_dialog(ctx);

        TopBottomPanel::bottom("bottom_panel").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.set_min_height(50.0);
                ui.set_max_width(ui.available_width());
                let mut input_text = self.input_text.lock().unwrap();

                let text_edit = if self.simple_ui {
                    let field = TextEdit::singleline(&mut *input_text)
                        .hint_text("Type something cute...")
                        .frame(true)
                        .min_size(ui.max_rect().size());
                    ui.add(field)
                } else {
                    let field = TextEdit::singleline(&mut *input_text)
                        .hint_text("Type something cute...")
                        .frame(true);
                    ui.add(field)
                };

                // Detect Enter key press
                // let show_any_dialog = *self.show_load_dialog.lock().unwrap() || *self.show_load_dialog.lock().unwrap();
                if text_edit.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                    self.send_message(&input_text);
                    self.start_streaming(input_text.clone());
                }
                if ui.input(|i| i.key_pressed(egui::Key::P) && i.modifiers.ctrl) {
                    self.show_prompt_dialog = true;
                }
                if ui.input(|i| i.key_pressed(egui::Key::S) && i.modifiers.ctrl) {
                    self.clone().save_to_file().unwrap();
                }
                if ui.input(|i| i.key_pressed(egui::Key::L)&& i.modifiers.ctrl) {
                    self.clone().toggle_load_dialog();
                }
                if ui.input(|i| i.key_pressed(egui::Key::R)&& i.modifiers.ctrl) {
                    self.load_from_file(None);
                }
                if ui.input(|i| i.key_pressed(egui::Key::Escape)) {
                    // self.clear_session();
                    self.messages.lock().unwrap().clear();
                    input_text.clear();
                }
                if ui.input(|i| i.key_pressed(egui::Key::E)&& i.modifiers.ctrl) {
                    //self.clone().toggle_config_dialog();
                    let mut lock = self.show_config_dialog.lock().unwrap();
                    *lock = !*lock;
                }
                if ui.input(|i| i.key_pressed(egui::Key::U)&& i.modifiers.ctrl) {
                    self.simple_ui = !self.simple_ui;
                }
                if ui.input(|i| i.key_pressed(egui::Key::Q)&& i.modifiers.ctrl) {
                    exit(0);
                }

                if !self.simple_ui {
                    if self.streamed_words.lock().unwrap().len() == 0 {
                        if ui.button("❤ Send").clicked() {
                            self.send_message(&input_text);
                            self.start_streaming(input_text.clone());
                        }
                        // if !self.simple_ui {
                        if ui.button("⭐ Save").clicked() {
                            self.save_to_file().unwrap();
                        }
                        if ui.button("🧹 Clear").clicked() {
                            self.clear_session();
                            input_text.clear();
                        }
                        // }
                    }
                }
            });
            self.panel_session_details(ui);

            // self.menu_bar(ctx); // Add the menu bar
        });

        // Use a custom frame with no background color
        let frame = Frame {
            fill: Color32::WHITE, // Ensure the frame itself is also white
            ..Frame::default()
        };

        egui::CentralPanel::default().frame(frame).show(ctx, |ui| {
            if !self.simple_ui {
                self.menu_bar(ui);
            }

            self.config_dialog_ui(ctx);
            self.prompt_dialog_ui(ctx);
            self.chat_ui(ui, ctx);
        });
    }
}

// Main method
#[tokio::main]
async fn main() -> Result<(), eframe::Error> {
    let options = my_default_options(800.0, 500.0, include_bytes!("../../icon.png"));

    eframe::run_native(
        "Cute Chat App",
        options,
        Box::new(|cc| {
            add_font(
                &cc.egui_ctx,
                "CuteFont",
                include_bytes!("../../../ui-fonts/NotoSansJP-Regular.ttf"),
            );
            configure_text_styles(&cc.egui_ctx);
            let app = CuteChatApp::new();
            app.load_from_file(None);
            // ctx.set_pixels_per_point(3.0);
            Ok(Box::<CuteChatApp>::new(app))
        }),
    )
}

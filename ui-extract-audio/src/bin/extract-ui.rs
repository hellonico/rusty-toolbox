use eframe::egui;
use lib_egui_utils::my_default_options;
use lib_ffmpeg_utils::utils::{check_ffmpeg, path_for};
use native_dialog::FileDialog;
use std::path::PathBuf;
use std::process::{exit, Command};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

#[derive(Default)]
struct MyApp {
    ffmpeg_version: Option<String>, // Store the detected ffmpeg version
    ffmpeg_not_found: bool,         // Flag to indicate ffmpeg is not found
    exit_time: Option<Instant>,     // Time to quit the application if ffmpeg is not found
    selected_file: Option<PathBuf>,
    processing: Arc<Mutex<bool>>,  // Shared flag for processing status
    current_tab: String,           // Current tab
    selected_format: String,       // Selected audio format
    status_message: Arc<Mutex<Option<String>>>, // Shared status message
}

fn main() -> Result<(), eframe::Error> {
    // let app_icon = icon(include_bytes!("../icon.png"));
    // let native_options = NativeOptions {
    //     viewport: ViewportBuilder::default()
    //         .with_close_button(true)
    //         .with_inner_size(egui::Vec2::new(400.0, 300.0))
    //         .with_icon(app_icon),
    //     ..Default::default()
    // };
    let native_options = my_default_options(400.0, 300.0, include_bytes!("../../icon.png"));

    let mut app = MyApp {
        current_tab: "Extract Audio".to_string(),
        selected_format: "mp3".to_string(), // Auto-select the first format
        ..Default::default()
    };

    // Check for ffmpeg availability
    match check_ffmpeg() {
        Ok(version) => app.ffmpeg_version = Some(version),
        Err(_) => {
            app.ffmpeg_not_found = true;
            app.exit_time = Some(Instant::now() + Duration::from_secs(10));
        }
    }

    eframe::run_native(
        "Extract Audio",
        native_options,
        Box::new(|_cc| Ok(Box::new(app))),
    )
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if self.ffmpeg_not_found {
            egui::CentralPanel::default().show(ctx, |ui| {
                ui.label("FFmpeg is not installed or not in the PATH.");
                ui.label("Please install FFmpeg to use this application.");
                ui.label("The application will quit in 10 seconds.");
            });

            // Check if the application should quit
            if let Some(exit_time) = self.exit_time {
                if Instant::now() >= exit_time {
                    exit(0);
                }
            }
            return;
        }

        let processing = self.processing.clone();

        egui::CentralPanel::default().show(ctx, |ui| {
            // Display FFmpeg version
            if let Some(version) = &self.ffmpeg_version {
                ui.label(format!("{}", version));
            }

            // Display tabs
            egui::TopBottomPanel::top("tab_panel").show_inside(ui, |ui| {
                ui.horizontal(|ui| {
                    if ui.selectable_label(self.current_tab == "Extract Audio", "Extract Audio").clicked() {
                        self.current_tab = "Extract Audio".to_string();
                    }
                    // Add more tabs here in the future
                });
            });

            // Show content for the selected tab
            if self.current_tab == "Extract Audio" {
                ui.label("Select a video file:");

                if ui.button("Browse").clicked() {
                    if let Ok(file) = FileDialog::new().show_open_single_file() {
                        self.selected_file = file;
                    }
                }

                if let Some(ref file) = self.selected_file {
                    ui.label(format!("Selected file: {:?}", file.display()));
                } else {
                    ui.label("No file selected");
                }

                ui.horizontal(|ui| {
                    ui.label("Output format:");
                    egui::ComboBox::from_label("")
                        .selected_text(&self.selected_format)
                        .show_ui(ui, |ui| {
                            ui.selectable_value(&mut self.selected_format, "mp3".to_string(), "MP3");
                            ui.selectable_value(&mut self.selected_format, "aac".to_string(), "AAC");
                            ui.selectable_value(&mut self.selected_format, "wav".to_string(), "WAV");
                        });
                });

                if ui.button("Go").clicked() {
                    if let Some(ref file) = self.selected_file {
                        let output_file = format!("{}.{}", file.display(), self.selected_format);

                        // Set the processing flag to true
                        let processing_clone = self.processing.clone();
                        *processing_clone.lock().unwrap() = true;

                        let status_message_clone = self.status_message.clone();

                        // Run ffmpeg in a background thread
                        let file_clone = file.clone();
                        let format_clone = self.selected_format.clone();
                        thread::spawn(move || {
                            let result = run_ffmpeg(&file_clone, &output_file, &format_clone);
                            *processing_clone.lock().unwrap() = false;
                            *status_message_clone.lock().unwrap() = Some(result);
                        });
                    }
                }

                // Check if the process is running and display a "Processing..." message
                if *processing.lock().unwrap() {
                    ui.label("Processing...");
                }

                if let Some(ref message) = *self.status_message.lock().unwrap() {
                    ui.label(message);
                }
            }
        });

        // Request a UI repaint while processing
        if *processing.lock().unwrap() {
            ctx.request_repaint_after(Duration::from_millis(100));
        }
    }
}


fn run_ffmpeg(input_file: &PathBuf, output_file: &str, format: &str) -> String {
    let input = input_file.to_str().unwrap();
    let output = output_file;

    let args = match format {
        "mp3" => vec!["-i", input, "-vn", "-ac", "2", "-ar", "44100", "-ab", "320k", "-f", "mp3", output],
        "aac" => vec!["-i", input, "-vn", "-ac", "2", "-ar", "44100", "-ab", "192k", "-f", "adts", output],
        "wav" => vec!["-i", input, "-vn", "-ac", "2", "-ar", "44100", "-f", "wav", output],
        _ => vec![], // Default empty args for unsupported formats
    };

    let ffmpeg_command = Command::new(path_for("ffmpeg"))
        .args(args)
        .output()
        .expect("Failed to execute ffmpeg command");

    if ffmpeg_command.status.success() {
        format!("Audio extracted successfully: {}", output)
    } else {
        format!(
            "Error during extraction: {}",
            String::from_utf8_lossy(&ffmpeg_command.stderr)
        )
    }
}

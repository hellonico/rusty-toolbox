use eframe::egui;
use std::process::Command;
use std::path::PathBuf;
use native_dialog::FileDialog;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

#[derive(Default)]
struct MyApp {
    selected_file: Option<PathBuf>,
    processing: Arc<Mutex<bool>>,  // Shared flag for processing status
    current_tab: String,
}

fn main() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions::default();
    eframe::run_native(
        "Extract Audio",
        options,
        Box::new(|_cc| Box::new(MyApp {
            current_tab: "Extract Audio".to_string(),
            ..Default::default()
        })),
    )
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let processing = self.processing.clone();
        egui::CentralPanel::default().show(ctx, |ui| {
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

                if ui.button("Go").clicked() {
                    if let Some(ref file) = self.selected_file {
                        let output_file = format!("{}.mp3", file.display());

                        // Set the processing flag to true
                        let processing_clone = self.processing.clone();
                        *processing_clone.lock().unwrap() = true;

                        // Run ffmpeg in a background thread
                        let file_clone = file.clone();
                        thread::spawn(move || {
                            run_ffmpeg(&file_clone, &output_file);
                            // Set the processing flag to false when done
                            *processing_clone.lock().unwrap() = false;
                        });
                    }
                }

                // Check if the process is running and display a "Processing..." message
                if *processing.lock().unwrap() {
                    ui.label("Processing...");
                }
            }
        });

        // Request a UI repaint while processing
        if *processing.lock().unwrap() {
            ctx.request_repaint_after(Duration::from_millis(100));
        }
    }
}

fn run_ffmpeg(input_file: &PathBuf, output_file: &str) {
    let input = input_file.to_str().unwrap();
    let output = output_file;

    let ffmpeg_command = Command::new("ffmpeg")
        .args(&[
            "-i", input,
            "-vn", "-ac", "2",
            "-ar", "44100",
            "-ab", "320k",
            "-f", "mp3", output,
        ])
        .output()
        .expect("Failed to execute ffmpeg command");

    if ffmpeg_command.status.success() {
        println!("Audio extracted successfully: {}", output);
    } else {
        println!("Error during extraction: {:?}", ffmpeg_command.stderr);
    }
}

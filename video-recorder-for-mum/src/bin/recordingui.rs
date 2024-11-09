use eframe::{egui, App};
use egui_extras::install_image_loaders;
use std::process::Command;
use video_recorder_for_mum::RecordingApp;


pub struct RecordingAppUI {
    recording_app : RecordingApp
}


impl Default for RecordingAppUI {
    fn default() -> Self {
        Self {
            recording_app : RecordingApp::default()
        }
    }
}

impl App for RecordingAppUI {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        install_image_loaders(ctx);

        // Show the UI with a horizontal layout
        egui::CentralPanel::default().show(ctx, |ui| {

            // Create a horizontal layout container
            ui.horizontal(|ui| {
                ui.allocate_ui_with_layout(
                    egui::Vec2::new(ui.available_width(), 200.0), // Width and fixed height
                    egui::Layout::left_to_right(egui::Align::Center),
                    |ui| {
                        // Image on the left side
                        ui.add(
                            egui::Image::new(egui::include_image!("../../src/mafalda.png"))
                                // .max_width(400.0) // Set the max width for the image
                                // .size(vec2[200.0, 300.0])
                                // .tint(egui::Color32::LIGHT_BLUE)
                                .rounding(10.0)
                            // .size(200.0),  // Rounded corners for the image
                        );

                        // Rest of the UI on the right side
                        ui.vertical(|ui| {
                            // Start/Stop recording button
                            let process_lock = self.recording_app.recording_process.clone();
                            if process_lock.lock().unwrap().is_some() {
                                if ui.button("Stop Recording").clicked() {
                                    self.recording_app.stop_recording();
                                }
                            } else {
                                if ui.button("Start Recording").clicked() {
                                    self.recording_app.start_recording();
                                }
                            }

                            // Elapsed time label
                            if let Some(elapsed) = self.recording_app.elapsed_time() {
                                ui.label(format!("{:?}", elapsed));
                            }

                            // Button to open containing folder
                            if let Some(output_file) = &*self.recording_app.last_output_file.lock().unwrap() {
                                ui.label(output_file);
                                if ui.button("Open Folder").clicked() {
                                    let folder_path = std::path::Path::new(output_file)
                                        .parent()
                                        .unwrap();

                                    #[cfg(target_os = "macos")]
                                    Command::new("open")
                                        .arg(folder_path)
                                        .spawn()
                                        .expect("Failed to open folder");

                                    #[cfg(target_os = "windows")]
                                    Command::new("explorer")
                                        .arg(folder_path)
                                        .spawn()
                                        .expect("Failed to open folder");

                                    #[cfg(target_os = "linux")]
                                    Command::new("xdg-open")
                                        .arg(folder_path)
                                        .spawn()
                                        .expect("Failed to open folder");
                                }
                            }
                        });


            });


            });

            // Request repaint for smooth updates
            ctx.request_repaint();
        });
    }
}

fn main() {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_app_id(String::from("mom-screenrecorder"))
            .with_inner_size([500.0,300.0])
            .with_resizable(false)
            // .with_taskbar(false)
            .with_decorations(true)
            .with_position([0.0, 0.0])
            .with_window_level(egui::WindowLevel::AlwaysOnTop),
        ..Default::default()
    };

    eframe::run_native(
        "Screen Recorder for Mum", // Updated app name
        options,
        Box::new(|_cc| Box::new(RecordingAppUI::default())),
    )
        .unwrap();
}

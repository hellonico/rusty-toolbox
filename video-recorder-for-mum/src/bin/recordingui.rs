#![windows_subsystem = "windows"]
use eframe::{egui, App};
use egui_extras::install_image_loaders;
use lib_egui_utils::my_default_options;
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
                                ui.label(format!("Elapsed time: {}", elapsed));
                            }

                            // Button to open containing folder
                            if let Some(output_file) = &*self.recording_app.last_output_file.lock().unwrap() {
                                ui.label(output_file.clone());
                                if ui.button("Open Folder").clicked() {
                                    RecordingApp::open_containing_folder(output_file);
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
    // let options = eframe::NativeOptions {
    //     viewport: egui::ViewportBuilder::default()
    //         .with_app_id(String::from("mom-screenrecorder"))
    //         .with_inner_size([500.0,300.0])
    //         .with_resizable(false)
    //         // .with_taskbar(false)
    //         .with_decorations(true)
    //         .with_position([0.0, 0.0])
    //         .with_window_level(egui::WindowLevel::AlwaysOnTop),
    //     ..Default::default()
    // };
    let native_options =
        my_default_options(500.0, 300.0, include_bytes!("../mafalda.png"));


    eframe::run_native(
        "Screen Recorder for Mum", // Updated app name
        native_options,
        //Box::new(|_cc| Box::new(RecordingAppUI::default())),
        Box::new(|_cc| Ok(Box::new(RecordingAppUI::default()) as Box<dyn eframe::App>)),
    )
        .unwrap();
}

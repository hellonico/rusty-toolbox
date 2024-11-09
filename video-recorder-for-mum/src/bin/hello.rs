use eframe::{egui, App};
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
        egui::CentralPanel::default().show(ctx, |ui| {
            // ui.heading("Desktop and Sound Recording Tool");

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
            if let Some(elapsed) = self.recording_app.elapsed_time() {
                ui.label(format!("{:?}",elapsed));
            }


            // Button to open containing folder
            if let Some(output_file) = &*self.recording_app.last_output_file.lock().unwrap() {
                ui.label(output_file);
                if ui.button("Open Containing Folder").clicked() {
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

            ctx.request_repaint();

        });
    }
}

fn main() {
    let options = eframe::NativeOptions {
        initial_window_size: Some(egui::vec2(300.0, 200.0)), // Set the initial window size (width x height)
        ..Default::default()
    };
    eframe::run_native(
        "Screen Recorder for Mum", // Updated app name
        options,
        Box::new(|_cc| Box::new(RecordingAppUI::default())),
    )
        .unwrap();
}

use eframe::{egui, App};
use std::process::Command;
use std::sync::{Arc, Mutex};
use std::thread;

struct RecordingApp {
    is_recording: Arc<Mutex<bool>>,
}

impl Default for RecordingApp {
    fn default() -> Self {
        Self {
            is_recording: Arc::new(Mutex::new(false)),
        }
    }
}

impl App for RecordingApp {

    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
    //fn update(&mut self, ctx: &egui::CtxRef, _: &mut epi::Frame<'_>) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Desktop and Sound Recording Tool");

            let is_recording = self.is_recording.clone();
            let mut is_recording_locked = is_recording.lock().unwrap();

            if *is_recording_locked {
                if ui.button("Stop Recording").clicked() {
                    *is_recording_locked = false;
                    // Stop the recording process
                    // Here, you could send a signal to the ffmpeg process or kill it
                }
            } else {
                if ui.button("Start Recording").clicked() {
                    *is_recording_locked = true;
                    let is_recording_clone = is_recording.clone();

                    // Start recording in a new thread
                    thread::spawn(move || {
                        let output = "output.mp4";
                        let mut ffmpeg_cmd = Command::new("ffmpeg")
                            .arg("-f")
                            .arg("gdigrab") // Use `gdigrab` for Windows, `x11grab` for Linux, or `avfoundation` for macOS
                            .arg("-i")
                            .arg(":0.0") // Use correct display for your system
                            .arg("-f")
                            .arg("dshow") // Use `dshow` for Windows or `avfoundation` for macOS
                            .arg("-i")
                            .arg("default") // Use correct audio device
                            .arg("-c:v")
                            .arg("libx264")
                            .arg("-c:a")
                            .arg("aac")
                            .arg(output)
                            .spawn()
                            .expect("Failed to start ffmpeg");

                        ffmpeg_cmd.wait().unwrap();

                        // Stop the recording when finished
                        *is_recording_clone.lock().unwrap() = false;
                    });
                }
            }
        });
    }
}
fn main() {

    let options = eframe::NativeOptions::default();
    eframe::run_native(
        "System Info Viewer",
        options,
        Box::new(|_cc| Box::new(RecordingApp::default())),
    )
        .unwrap();
}
use eframe::{egui, App};
use std::process::{Command, Stdio, Child};
use std::sync::{Arc, Mutex};
use std::thread;
use std::io::Write;
use chrono::Local;

struct RecordingApp {
    recording_process: Arc<Mutex<Option<Child>>>,
}

impl Default for RecordingApp {
    fn default() -> Self {
        Self {
            recording_process: Arc::new(Mutex::new(None)),
        }
    }
}

impl App for RecordingApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            // ui.heading("Desktop and Sound Recording Tool");

            let process_lock = self.recording_process.clone();

            if process_lock.lock().unwrap().is_some() {
                if ui.button("Stop Recording").clicked() {
                    self.stop_recording();
                }
            } else {
                if ui.button("Start Recording").clicked() {
                    self.start_recording();
                }
            }
        });
    }
}

impl RecordingApp {
    fn start_recording(&self) {
        let process_lock = self.recording_process.clone();

        thread::spawn(move || {
            let now = Local::now();
            let output_file = format!("screen_recording_{}.mp4", now.format("%Y-%m-%d_%H-%M-%S"));

            #[cfg(target_os = "macos")]
            let ffmpeg_cmd =
                Command::new("ffmpeg")
                .arg("-f")
                .arg("avfoundation")
                .arg("-i")
                .arg("1:0") // Adjust devices for your system
                .arg("-framerate")
                .arg("25")
                .arg(&output_file) // Use dynamically generated filename
                .stdin(Stdio::piped()) // Open stdin for sending commands
                .spawn()
                .expect("Failed to start ffmpeg");

            #[cfg(target_os = "windows")]
            let ffmpeg_cmd =
                Command::new("ffmpeg")
                    .arg("-f")
                    .arg("gdigrab")
                    .arg("-i")
                    .arg("desktop") // Adjust devices for your system
                    .arg("dshow")
                    .arg("-i")
                    .arg("audio=\"Microphone\"")
                    .arg("-framerate")
                    .arg("25")
                    .arg(&output_file) // Use dynamically generated filename
                    .stdin(Stdio::piped()) // Open stdin for sending commands
                    .spawn()
                    .expect("Failed to start ffmpeg");

            // Lock the process and store it
            *process_lock.lock().unwrap() = Some(ffmpeg_cmd);
        });
    }

    fn stop_recording(&self) {
        let mut process_lock = self.recording_process.lock().unwrap();
        if let Some(mut ffmpeg_process) = process_lock.take() {
            if let Some(stdin) = ffmpeg_process.stdin.as_mut() {
                // Send the 'q' command to quit ffmpeg gracefully
                stdin.write_all(b"q\n").expect("Failed to send 'q' to ffmpeg");
                println!("Sent 'q' to ffmpeg to stop recording");
            }

            // Optionally wait for the process to finish
            let _ = ffmpeg_process.wait().expect("Failed to wait on ffmpeg");
            println!("FFmpeg process has stopped");
        }
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
        Box::new(|_cc| Box::new(RecordingApp::default())),
    )
        .unwrap();
}

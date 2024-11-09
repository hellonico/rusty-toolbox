mod recordingcli;

use eframe::{egui, App};
use std::process::{Command, Child};
use std::sync::{Arc, Mutex};
use std::thread;
use chrono::Local;

struct RecordingApp {
    is_recording: Arc<Mutex<bool>>,
    screen_input: String,
    audio_input: String,
    framerate_input: String,
    // recording_process: Arc<Mutex<Option<Child>>>,
    last_output_file: Arc<Mutex<Option<String>>>,
}

impl Default for RecordingApp {
    fn default() -> Self {
        Self {
            is_recording: Arc::new(Mutex::new(false)),
            screen_input: "1".to_string(),
            audio_input: "0".to_string(),
            framerate_input: "25".to_string(),
            // recording_process: Arc::new(Mutex::new(None)),
            last_output_file: Arc::new(Mutex::new(None)),
        }
    }
}

impl App for RecordingApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Desktop and Sound Recording Tool");

            // Input fields for customization
            ui.horizontal(|ui| {
                ui.label("Video Device:");
                ui.text_edit_singleline(&mut self.screen_input);
            });

            ui.horizontal(|ui| {
                ui.label("Audio Device:");
                ui.text_edit_singleline(&mut self.audio_input);
            });

            ui.horizontal(|ui| {
                ui.label("Framerate:");
                ui.text_edit_singleline(&mut self.framerate_input);
            });

            let recording_process: Arc<Mutex<Option<u32>>> = Arc::new(Mutex::new(None));
            let is_recording = self.is_recording.clone();
            let mut is_recording_locked = is_recording.lock().unwrap();

            if *is_recording_locked {
                if ui.button("Stop Recording").clicked() {
                    *is_recording_locked = false;

                    // Stop the recording process
                    if let Some(pid) = recording_process.lock().unwrap().take() {
                        #[cfg(target_os = "unix")]
                        {
                            let _ = Command::new("kill")
                                .arg("-9")
                                .arg(pid.to_string())
                                .spawn();
                        }

                        #[cfg(target_os = "windows")]
                        {
                            let _ = Command::new("taskkill")
                                .arg("/PID")
                                .arg(pid.to_string())
                                .arg("/F")
                                .spawn();
                        }
                    }

                }
            } else {
                if ui.button("Start Recording").clicked() {
                    *is_recording_locked = true;
                    let is_recording_clone = is_recording.clone();
                    let screen_input = self.screen_input.clone();
                    let audio_input = self.audio_input.clone();
                    let framerate_input = self.framerate_input.clone();

                    // Generate output file name
                    let timestamp = Local::now().format("%Y%m%d_%H%M%S").to_string();
                    let output_file = format!("screen_recording_{}.mp4", timestamp);

                    // Store the output file name
                    *self.last_output_file.lock().unwrap() = Some(output_file.clone());

                    // let recording_process = self.recording_process.clone();

                    // Start recording in a new thread
                    thread::spawn(move || {
                        let ffmpeg_cmd_result = {
                            #[cfg(target_os = "macos")]
                            {
                                Command::new("ffmpeg")
                                    .arg("-f")
                                    .arg("avfoundation")
                                    .arg("-i")
                                    .arg(&format!("{}:{}", screen_input, audio_input))
                                    .arg("-framerate")
                                    .arg(&framerate_input)
                                    .arg(&output_file)
                                    .spawn()
                            }

                            #[cfg(target_os = "windows")]
                            {
                                Command::new("ffmpeg")
                                    .arg("-f")
                                    .arg("gdigrab")
                                    .arg("-framerate")
                                    .arg(&framerate_input)
                                    .arg("-i")
                                    .arg(&screen_input)
                                    .arg("-f")
                                    .arg("dshow")
                                    .arg("-i")
                                    .arg(&audio_input)
                                    .arg(&output_file)
                                    .spawn()
                            }

                            #[cfg(target_os = "linux")]
                            {
                                Command::new("ffmpeg")
                                    .arg("-f")
                                    .arg("x11grab")
                                    .arg("-framerate")
                                    .arg(&framerate_input)
                                    .arg("-i")
                                    .arg(&screen_input)
                                    .arg("-f")
                                    .arg("alsa")
                                    .arg("-i")
                                    .arg(&audio_input)
                                    .arg(&output_file)
                                    .spawn()
                            }
                        };

                        match ffmpeg_cmd_result {
                            Ok(mut ffmpeg_cmd) => {
                                {
                                    let mut process_lock = recording_process.lock().unwrap();
                                    *process_lock = Some(ffmpeg_cmd.id()); // Store process ID
                                }

                                let _ = ffmpeg_cmd.wait(); // Wait for the process to finish
                                *is_recording_clone.lock().unwrap() = false;
                            }
                            Err(e) => {
                                eprintln!("Failed to start ffmpeg: {}", e);
                                *is_recording_clone.lock().unwrap() = false;
                            }
                        }
                    });



                }
            }

            // Button to open containing folder
            if let Some(output_file) = &*self.last_output_file.lock().unwrap() {
                if ui.button("Open Containing Folder").clicked() {
                    let folder_path = std::path::Path::new(output_file)
                        .parent()
                        .unwrap_or_else(|| std::path::Path::new("../../.."));

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
    }
}

fn main() {
    let options = eframe::NativeOptions::default();
    eframe::run_native(
        "Screen Recording Tool",
        options,
        Box::new(|_cc| Box::new(RecordingApp::default())),
    )
        .unwrap();
}

use eframe::egui;
use open;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use tokio::task;
use app_ui_probe_videos::{extract_metadata, Metadata};

#[derive(Clone, Default)]
struct VideoFile {
    path: PathBuf,
    metadata: Metadata,
}

#[derive(Default)]
struct VideoApp {
    folder_path: String,
    video_files: Arc<Mutex<Vec<VideoFile>>>, // Shared state for video files
    load_in_progress: Arc<Mutex<bool>>,     // Shared state for loading progress
}

impl VideoApp {
    fn new() -> Self {
        Self {
            folder_path: String::new(),
            video_files: Arc::new(Mutex::new(Vec::new())),
            load_in_progress: Arc::new(Mutex::new(false)),
        }
    }

    fn load_videos(&self) {
        let mut in_progress = self.load_in_progress.lock().unwrap();
        if *in_progress {
            return; // Avoid reloading if already in progress
        }
        *in_progress = true;
        drop(in_progress); // Drop the lock early to avoid blocking

        let folder_path = self.folder_path.clone();
        let video_files = Arc::clone(&self.video_files);
        let load_in_progress = Arc::clone(&self.load_in_progress);

        tokio::spawn(async move {
            if let Ok(entries) = fs::read_dir(&folder_path) {
                let mut tasks = vec![];

                for entry in entries.filter_map(Result::ok) {
                    let path = entry.path();

                    // Filter valid video files
                    if path
                        .extension()
                        .map_or(false, |ext| matches!(ext.to_str(), Some("mp4" | "mov" | "mpeg")))
                        && !path.file_name().map_or(false, |name| name.to_str().map_or(false, |s| s.starts_with('.')))
                    {
                        let video_files = Arc::clone(&video_files);
                        tasks.push(task::spawn(async move {
                            let metadata = extract_metadata(path.to_string_lossy().to_string());
                            let video_file = VideoFile {
                                path: path.clone(),
                                metadata: metadata,
                            };
                            video_files.lock().unwrap().push(video_file);
                        }));
                    }
                }

                // Wait for all tasks to complete
                futures::future::join_all(tasks).await;

                // Reset loading flag
                *load_in_progress.lock().unwrap() = false;
            }
        });
    }
}

impl eframe::App for VideoApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Video Files");

            // Input folder path
            ui.horizontal(|ui| {
                ui.label("Folder Path:");
                ui.text_edit_singleline(&mut self.folder_path);
                if ui.button("Load Videos").clicked() {
                    self.load_videos();
                }
            });

            ui.separator();

            // Scrollable list of video files
            egui::ScrollArea::vertical().show(ui, |ui| {
                let video_files = self.video_files.lock().unwrap().clone();
                for video_file in video_files.iter() {
                    ui.horizontal(|ui| {
                        if ui.button("Open").clicked() {
                            open::that(&video_file.path).unwrap();
                        }
                        ui.label(video_file.path.file_name().unwrap().to_string_lossy().as_ref());
                        if let metadata = &video_file.metadata {
                            ui.label(format!("{:?}",&metadata));
                        }
                    });
                }

                // Show loading message if necessary
                if *self.load_in_progress.lock().unwrap() {
                    ui.label("Loading videos...");
                }
            });
        });
    }
}

#[tokio::main]
async fn main() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions::default();
    eframe::run_native("Video Browser", options, Box::new(|_cc| Ok(Box::new(VideoApp::new()))))
}

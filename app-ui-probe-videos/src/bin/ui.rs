use app_ui_probe_videos::{extract_frame, extract_metadata, Metadata};
use eframe::egui;
use egui::Color32;
use egui_extras::install_image_loaders;
use open;
use rfd::FileDialog;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use tokio::task;
use lib_egui_utils::my_default_options;
// Add this for folder selection

#[derive(Clone, Default)]
struct VideoFile {
    path: PathBuf,
    metadata: Metadata,
    thumbnail: Option<PathBuf>, // Path to the cached thumbnail
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
                let thumbnail_dir = Path::new(&folder_path).join(".thumbnails");
                fs::create_dir_all(&thumbnail_dir).ok(); // Create thumbnail directory if it doesn't exist
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
                        let thumbnail_dir = thumbnail_dir.clone();

                        tasks.push(task::spawn(async move {
                            let metadata = extract_metadata(path.to_string_lossy().to_string());

                            let thumbnail_path = thumbnail_dir.join(
                                path.file_name()
                                    .unwrap_or_default()
                                    .to_string_lossy()
                                    .to_string()
                                    + ".png",
                            );

                            if !thumbnail_path.exists() {
                                // Generate thumbnail at 00:00:01 timestamp
                                let timestamp = "00:00:01";
                                if let Err(err) =
                                    extract_frame(&path.to_string_lossy(), timestamp, &thumbnail_path.to_string_lossy())
                                {
                                    eprintln!("Failed to extract thumbnail: {}", err);
                                }
                            }

                            let video_file = VideoFile {
                                path: path.clone(),
                                metadata,
                                thumbnail: if thumbnail_path.exists() {
                                    Some(thumbnail_path)
                                } else {
                                    None
                                },
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
        install_image_loaders(ctx);
        egui::CentralPanel::default().show(ctx, |ui| {
            // Calculate the number of loaded videos
            let video_count = self.video_files.lock().unwrap().len();

            ui.horizontal(|ui| {
                ui.heading(format!("Video Files ({})", video_count)); // Display video count
            });

            // Input folder path and folder selection
            ui.horizontal(|ui| {
                ui.label("Folder Path:");
                ui.text_edit_singleline(&mut self.folder_path);

                if ui.button("Select Folder").clicked() {
                    if let Some(folder) = FileDialog::new().pick_folder() {
                        self.folder_path = folder.to_string_lossy().to_string();
                        self.load_videos(); // Automatically load videos after selecting folder
                    }
                }

                if ui.button("Load Videos").clicked() {
                    self.load_videos();
                }

                // Show loading message if necessary
                if *self.load_in_progress.lock().unwrap() {
                    ui.label("Loading videos...");
                }
            });

            ui.separator();

            egui::ScrollArea::vertical().show(ui, |ui| {
                let video_files = self.video_files.lock().unwrap().clone();
                ui.horizontal_wrapped(|ui| {
                    for video_file in video_files.iter() {
                        if let Some(thumbnail) = &video_file.thumbnail {
                            let img = egui::ImageButton::new(format!("file://{}", thumbnail.to_str().unwrap())).frame(false);
                            let res = ui.add_sized([200.0, 100.0], img.clone());
                            if res.clicked() {
                                if let Err(err) = open::that(&video_file.path) {
                                    eprintln!("Failed to open video: {:?}", err);
                                }
                            }
                            if res.hovered() {
                                ui.painter().rect_stroke(res.rect, 10.0, egui::Stroke {
                                    width: 1.0,
                                    color: Color32::from_black_alpha(200),
                                });
                            };
                        } else {
                            ui.label(video_file.metadata.format.filename.clone());
                        }
                    }
                });
            });
        });
    }
}

#[tokio::main]
async fn main() -> Result<(), eframe::Error> {
    // let options = eframe::NativeOptions::default();
    let options =
        my_default_options(800.0, 500.0, include_bytes!("../../icon.png"));

    eframe::run_native("Video Browser", options, Box::new(|_cc| Ok(Box::new(VideoApp::new()))))
}

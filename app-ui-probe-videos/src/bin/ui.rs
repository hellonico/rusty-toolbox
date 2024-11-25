use app_ui_probe_videos::{extract_frame, extract_metadata, Metadata};
use eframe::egui;
use egui::{Color32, ComboBox};
use egui_extras::install_image_loaders;
use open;
use rfd::FileDialog;
use std::fs;
use std::fs::File;
use std::io::{Read, Write};
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
    video_files: Arc<Mutex<Vec<VideoFile>>>,
    load_in_progress: Arc<Mutex<bool>>,
    sort_field: SortField,
    sort_asc: bool,
    config_file: PathBuf,
    show_filename: bool,
    show_filesize: bool,
    show_date: bool,
}


#[derive(Default, Clone, Copy, PartialEq, Eq)]
enum SortField {
    #[default]
    Name,
    Date,
    FileSize,
}

impl VideoApp {
    fn new() -> Self {
        let config_file = dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join(".video-gallery");

        let mut app = Self {
            folder_path: String::new(),
            video_files: Arc::new(Mutex::new(Vec::new())),
            load_in_progress: Arc::new(Mutex::new(false)),
            sort_field: SortField::default(),
            sort_asc: true,
            show_filename: false,
            show_filesize: false,
            show_date: false,
            config_file,
        };

        app.load_settings();
        app
    }

    fn load_settings(&mut self) {
        if let Ok(mut file) = File::open(&self.config_file) {
            let mut contents = String::new();
            if file.read_to_string(&mut contents).is_ok() {
                let parts: Vec<&str> = contents.split(',').collect();
                if let Some(folder) = parts.get(0) {
                    self.folder_path = folder.to_string();
                }
                if let Some(field) = parts.get(1) {
                    self.sort_field = match *field {
                        "Date" => SortField::Date,
                        "FileSize" => SortField::FileSize,
                        _ => SortField::Name,
                    };
                }
                if let Some(order) = parts.get(2) {
                    self.sort_asc = *order == "asc";
                }
            }
        }
    }

    fn save_settings(&self) {
        if let Ok(mut file) = File::create(&self.config_file) {
            let field = match self.sort_field {
                SortField::Date => "Date",
                SortField::FileSize => "FileSize",
                SortField::Name => "Name",
            };
            let order = if self.sort_asc { "asc" } else { "desc" };
            let settings = format!("{},{},{}", self.folder_path, field, order);
            let _ = file.write_all(settings.as_bytes());
        }
    }

    fn sort_videos(&mut self) {
        let mut video_files = self.video_files.lock().unwrap();
        video_files.sort_by(|a, b| {
            let order = match self.sort_field {
                SortField::Name => {
                    a.path.file_name().unwrap_or_default().cmp(b.path.file_name().unwrap_or_default())
                }
                SortField::Date => {
                    let a_date = fs::metadata(&a.path).and_then(|m| m.modified()).ok();
                    let b_date = fs::metadata(&b.path).and_then(|m| m.modified()).ok();
                    a_date.cmp(&b_date) // Option implements Ord
                }
                SortField::FileSize => {
                    let a_size = fs::metadata(&a.path).and_then(|m| Ok(m.len())).unwrap_or(0);
                    let b_size = fs::metadata(&b.path).and_then(|m| Ok(m.len())).unwrap_or(0);
                    a_size.cmp(&b_size)
                }
            };

            if self.sort_asc {
                order
            } else {
                order.reverse()
            }
        });
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

                ui.label("Sort By:");
                let cb = ComboBox::from_label("")
                    .selected_text(match self.sort_field {
                        SortField::Name => "Name",
                        SortField::Date => "Date",
                        SortField::FileSize => "File Size",
                    })
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut self.sort_field, SortField::Name, "Name");
                        ui.selectable_value(&mut self.sort_field, SortField::Date, "Date");
                        ui.selectable_value(&mut self.sort_field, SortField::FileSize, "File Size");
                    });


                if ui.button(if self.sort_asc { "Ascending" } else { "Descending" }).clicked() {
                    self.sort_asc = !self.sort_asc;
                }

                // Checkboxes to toggle display options
                ui.checkbox(&mut self.show_filename, "Show Filename");
                ui.checkbox(&mut self.show_filesize, "Show File Size");
                ui.checkbox(&mut self.show_date, "Show Date");

                // Sort videos whenever sort settings change
                self.sort_videos();
            });

            ui.separator();

            egui::ScrollArea::vertical().show(ui, |ui| {
                let video_files = self.video_files.lock().unwrap().clone();
                ui.horizontal_wrapped(|ui| {
                    for video_file in video_files.iter() {
                        // ui.vert(|ui| {
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

                                ui.vertical(|ui| {
                                    if self.show_filename {
                                        ui.add(egui::Label::new(format!(
                                            "{}",
                                            video_file.path.file_name().unwrap_or_default().to_string_lossy()
                                        ))
                                            // .text_style(egui::TextStyle::Small)
                                        );
                                    }
                                    if self.show_filesize {
                                        let file_size = fs::metadata(&video_file.path).map(|m| m.len()).unwrap_or(0);
                                        let size_in_gb = file_size as f64 / 1_073_741_824.0; // Convert bytes to GB
                                        ui.add(egui::Label::new(format!("{:.2} GB", size_in_gb))
                                            // .text_style(egui::TextStyle::Small)
                                        );
                                    }
                                    if self.show_date {
                                        let modified_date = fs::metadata(&video_file.path)
                                            .and_then(|m| m.created())
                                            .ok()
                                            .map(|t| {
                                                chrono::DateTime::<chrono::Local>::from(t).format("%Y-%m-%d %H:%M").to_string()
                                            })
                                            .unwrap_or_else(|| "Unknown".to_string());
                                        ui.add(egui::Label::new(format!("{}", modified_date))
                                            // .text_style(egui::TextStyle::Small)
                                        );
                                    }
                                });
                            } else {
                                ui.label(video_file.metadata.format.filename.clone());
                            }
                        // });
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

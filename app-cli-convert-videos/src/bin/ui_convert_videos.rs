use eframe::egui::ViewportBuilder;
use eframe::{egui, NativeOptions};
use egui::{CentralPanel, ComboBox, ProgressBar, RichText};
use egui_remixicon::{add_to_fonts, icons};
use lib_egui_utils::{format_elapsed_time, format_f64_or_dash, generate_output_path, get_file_size_in_gb, icon};
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::fs::{self};
use std::io::{self};
use std::path::Path;
use std::process::Command;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use thread::sleep;

// Struct for configuration
#[derive(Serialize, Deserialize, Default)]
struct AppConfig {
    input_folder: String,
    output_folder: String,
    video_format: String,
    audio_format: String,
    file_extension: String,
    delete_original: bool,
    skip_if_exists: bool,
    encoding: String,
}

impl AppConfig {
    fn save(&self, path: &str) -> Result<(), io::Error> {
        let yaml = serde_yaml::to_string(self).unwrap();
        fs::write(path, yaml)
    }

    fn load(path: &str) -> Result<Self, io::Error> {
        let yaml = fs::read_to_string(path)?;
        serde_yaml::from_str(&yaml).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
    }
}

// Main application structure
struct MyApp {
    input_folder: String,
    output_folder: String,
    video_format: String,
    audio_format: String,
    file_extension: String,
    delete_original: bool,
    skip_if_exists: bool,
    job_queue: Arc<Mutex<VecDeque<String>>>,
    total_jobs: usize,
    is_encoding: Arc<Mutex<bool>>,
    progress: f32,
    encoding: String,
    file_stats: Arc<Mutex<Vec<FileStat>>>, // Use Arc<Mutex<>>
}

impl MyApp {
    fn new(cc: &eframe::CreationContext) -> Self {
        let config_path = "app_config.yaml";
        let mut app = Self {
            input_folder: String::new(),
            output_folder: String::new(),
            video_format: "mp4".to_string(),
            audio_format: String::new(),
            file_extension: "mp4".to_string(),
            encoding: "libx264".to_string(), // Default encoding
            delete_original: false,
            skip_if_exists: false,
            job_queue: Arc::new(Mutex::new(VecDeque::new())),
            total_jobs: 0,
            is_encoding: Arc::new(Mutex::new(false)),
            progress: 0.0,
            file_stats: Arc::new(Mutex::new(Vec::new())),
        };

        // Load config if it exists
        if let Ok(config) = AppConfig::load(config_path) {
            app.input_folder = config.input_folder;
            app.output_folder = config.output_folder;
            app.video_format = config.video_format;
            app.audio_format = config.audio_format;
            app.file_extension = config.file_extension;
            app.encoding = config.encoding;
            app.delete_original = config.delete_original;
            app.skip_if_exists = config.skip_if_exists;

            // Update the job queue with files from the loaded folder
            app.enqueue_jobs();
        }

        let mut fonts = egui::FontDefinitions::default();
        add_to_fonts(&mut fonts);
        cc.egui_ctx.set_fonts(fonts);

        app
    }


    fn save_config(&self) {
        let config = AppConfig {
            input_folder: self.input_folder.clone(),
            output_folder: self.output_folder.clone(),
            video_format: self.video_format.clone(),
            audio_format: self.audio_format.clone(),
            file_extension: self.file_extension.clone(),
            delete_original: self.delete_original,
            skip_if_exists: self.skip_if_exists,
            encoding: self.encoding.clone(),
        };

        if let Err(e) = config.save("app_config.yaml") {
            eprintln!("Failed to save configuration: {}", e);
        }
    }

    fn enqueue_jobs(&mut self) {
        let files = fs::read_dir(&self.input_folder)
            .unwrap() //_or_else(|_| fs::ReadDir::empty())
            .filter_map(|entry| entry.ok())
            .filter(|entry| {
                let file_name = entry.file_name();
                let file_name = file_name.to_string_lossy();
                !file_name.starts_with('.') // Skip dot files
                    && entry.path().extension().map_or(false, |ext| {
                    ext.to_str().unwrap().eq_ignore_ascii_case(&self.file_extension)
                })
            })
            .map(|entry| entry.path().to_string_lossy().to_string())
            .collect::<Vec<_>>();

        let mut job_queue = self.job_queue.lock().unwrap();
        job_queue.clear();
        job_queue.extend(files.clone());
        self.total_jobs = files.len();
        self.progress = 0.0;

        // Reset file stats
        self.file_stats.lock().unwrap().clear();
    }

    fn start_encoding(&mut self) {
        let is_encoding = Arc::clone(&self.is_encoding);
        let job_queue = Arc::clone(&self.job_queue);
        let file_stats = Arc::clone(&self.file_stats);
        let output_folder = self.output_folder.clone();
        let video_format = self.video_format.clone();
        let encoding = self.encoding.clone();
        let delete_original = self.delete_original;
        let skip_if_exists = self.skip_if_exists;

        *is_encoding.lock().unwrap() = true;

        thread::spawn(move || {
            while let Some(file) = {
                let mut queue = job_queue.lock().unwrap();
                queue.pop_front()
            } {
                let output_file = generate_output_path(&file, output_folder.clone(), video_format.clone());

                // Skip if output file already exists
                if skip_if_exists && Path::new(&output_file).exists() {
                    println!("Skipping {} as output file already exists", file);
                    continue;
                }

                let input_size = get_file_size_in_gb(&file);
                let start_time = std::time::Instant::now();

                {
                    // Add initial entry to file_stats
                    let mut stats = file_stats.lock().unwrap();
                    stats.push(FileStat {
                        input_file: file.clone(),
                        input_size,
                        output_size: None,
                        reduction: None,
                        elapsed_time: None,
                    });
                }

                // Run FFmpeg command
                let status = Command::new("ffmpeg")
                    .arg("-i")
                    .arg(&file)
                    .arg("-c:v")
                    .arg(&encoding)
                    .arg("-preset")
                    .arg("fast")
                    .arg("-c:a")
                    .arg("aac")
                    .arg(&output_file)
                    .status();

                if let Err(e) = status {
                    eprintln!("Failed to run FFmpeg for {}: {}", file, e);
                    continue;
                }
                sleep(Duration::from_secs(1));

                let output_size = get_file_size_in_gb(&output_file);
                let reduction = (input_size - output_size).max(0.0);
                let elapsed_time = start_time.elapsed().as_secs_f64();

                {
                    // Update file_stats entry
                    let mut stats = file_stats.lock().unwrap();
                    if let Some(stat) = stats.iter_mut().find(|stat| stat.input_file == file) {
                        stat.output_size = Some(output_size);
                        stat.reduction = Some(reduction);
                        stat.elapsed_time = Some(elapsed_time);
                    }
                }

                if delete_original {
                    fs::remove_file(&file).unwrap_or_else(|e| eprintln!("Failed to delete file: {}", e));
                }
            }

            *is_encoding.lock().unwrap() = false;
        });
    }

    fn update_progress(&mut self) {
        let queue_size = self.job_queue.lock().unwrap().len();
        self.progress = if self.total_jobs > 0 {
            1.0 - queue_size as f32 / self.total_jobs as f32
        } else {
            0.0
        };
    }

}

struct FileStat {
    input_file: String,
    input_size: f64,
    output_size: Option<f64>,
    reduction: Option<f64>,
    elapsed_time: Option<f64>, // Time in seconds
}
impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.update_progress();

        CentralPanel::default().show(ctx, |ui| {
            ui.horizontal(|ui| {

                // Left panel
                ui.vertical(|ui| {

                    ui.heading(
                        RichText::new(format!("{} Settings", icons::SETTINGS_3_FILL)).size(20.0),
                    );

                    ui.horizontal(|ui| {
                        ui.label("Input Folder:");
                        if ui.button("Select").clicked() {
                            if let Some(folder) = rfd::FileDialog::new().pick_folder() {
                                self.input_folder = folder.display().to_string();
                            }
                        }
                        ui.text_edit_singleline(&mut self.input_folder);
                    });

                    ui.horizontal(|ui| {
                        ui.label("Output Folder:");
                        if ui.button("Select").clicked() {
                            if let Some(folder) = rfd::FileDialog::new().pick_folder() {
                                self.output_folder = folder.display().to_string();
                            }
                        }
                        ui.text_edit_singleline(&mut self.output_folder);
                    });

                    ui.horizontal(|ui| {
                        ui.label("Output Video Container:");
                        ComboBox::from_id_salt("video_format")
                            .selected_text(&self.video_format)
                            .show_ui(ui, |ui| {
                                ui.selectable_value(&mut self.video_format, "mp4".to_string(), "MP4");
                                ui.selectable_value(&mut self.video_format, "mpeg".to_string(), "MPEG");
                            });
                    });

                    ui.horizontal(|ui| {
                        ui.label("Encoding:");
                        ComboBox::from_id_salt("encoding")
                            .selected_text(&self.encoding)
                            .show_ui(ui, |ui| {
                                for option in ["libx264", "libx265", "vp9", "av1"] {
                                    ui.selectable_value(&mut self.encoding, option.to_string(), option);
                                }
                            });
                    });

                    ui.horizontal(|ui| {
                        ui.label("File Extension:");
                        ui.text_edit_singleline(&mut self.file_extension);
                    });

                    ui.horizontal(|ui| {
                        ui.checkbox(&mut self.delete_original, "Delete Original Files");
                        ui.checkbox(&mut self.skip_if_exists, "Skip if Output File Exists");
                    });

                    if ui.button(RichText::new(format!("{} Encode", icons::PLAY_FILL)).size(12.0),).clicked() {
                        self.save_config();
                        self.enqueue_jobs();
                        self.start_encoding();
                    }

                    ui.heading(RichText::new(format!("{} Progress", icons::PROGRESS_1_FILL)).size(20.0), );
                    let remaining_jobs = self.job_queue.lock().unwrap().len();
                    ui.label(format!(
                        "Progress: {}/{} files processed",
                        self.total_jobs - remaining_jobs,
                        self.total_jobs
                    ));
                    ui.add(ProgressBar::new(self.progress).text(format!("{:.0}%", self.progress * 100.0)));

                    ui.heading(RichText::new(format!("{} File Stats", icons::COMPUTER_LINE)).size(20.0), );
                    egui::Grid::new("file_stats_table")
                        //.min_col_width(250.0)
                        .striped(true)
                        .show(ui, |ui| {
                            // Table headers
                            ui.label("Input File");
                            ui.label("Input Size (GB)");
                            ui.label("Output Size (GB)");
                            ui.label("Reduction (GB)");
                            ui.label("Elapsed Time (s)");
                            ui.end_row();

                            // Populate rows
                            let file_stats = self.file_stats.lock().unwrap();
                            for stat in file_stats.iter() {
                                ui.label(&stat.input_file);
                                ui.label(format!("{:.2}", stat.input_size));
                                ui.label(format_f64_or_dash(stat.output_size));
                                ui.label(format_f64_or_dash(stat.reduction));
                                ui.label(format_elapsed_time(stat.elapsed_time));
                                ui.end_row();
                            }
                        });
                });

            });
        });

        ctx.request_repaint();
    }


}

fn main() -> Result<(), eframe::Error> {
    let app_icon = icon(include_bytes!("../../icon.png"));
    let native_options = NativeOptions {
        viewport: ViewportBuilder::default()
            .with_close_button(true)
            .with_inner_size(egui::Vec2::new(400.0, 300.0))
            .with_icon(app_icon),
            ..Default::default()
    };
    eframe::run_native("BeeVEe - Batch Video Encoder", native_options, Box::new(|cc| Ok(Box::new(MyApp::new(cc)))))
}

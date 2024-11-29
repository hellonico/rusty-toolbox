use app_cli_convert_videos::{encode_video, AppConfig, FileStat};
use eframe::egui;
use egui::{include_image, CentralPanel, ComboBox, ProgressBar, RichText};
use egui_extras::install_image_loaders;
use egui_remixicon::{add_to_fonts, icons};
use lib_egui_utils::{format_elapsed_time, format_f64_or_dash, generate_output_path, get_file_name, get_file_size_in_gb, list_files_from_dir2, my_default_options, SortBy};
use std::collections::VecDeque;
use std::fs::{self};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::thread;
use eframe::egui::{Align, Layout};
use rand::Rng;

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

    fn add_debug_stats(&self) {
        let mut stats = self.file_stats.lock().unwrap();
        let mut rng = rand::thread_rng();

        // rng.gen_range()
        for _ in 0..50 {
            let input_size: f64 = rng.gen_range(100.0..1000000.0);
            let output_size: Option<f64> = Some(rng.gen_range(1.0..1000.0));
            let reduction: Option<f64> = Some(rng.gen_range(0.0..10.0));
            let elapsed_time: Option<f64> = Some(rng.gen_range(0.01..3600.0)); // seconds
            let output_file: Option<String> = None; // Always None for now

            stats.push(FileStat {
                input_file: "hello.mp4".to_string(),
                input_size,
                output_size,
                reduction,
                elapsed_time,
                output_file,
            });
        }
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
        //let files = list_files_from_dir(&self.input_folder, &self.file_extension);
        let files = list_files_from_dir2(&self.input_folder, &self.file_extension, SortBy::LastUpdated, true);

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

                let input_size = get_file_size_in_gb(&file).unwrap();
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
                        output_file: None,
                    });
                }

                let status =
                    encode_video(PathBuf::from(&file), &encoding, &output_file, &String::from("aac"));

                if let Err(e) = status {
                    eprintln!("Failed to run FFmpeg for {}: {}", file, e);
                    continue;
                }
                let elapsed_time = start_time.elapsed().as_secs_f64();

                // sleep(Duration::from_secs(3));
                let output_size = get_file_size_in_gb(&output_file).unwrap();
                let reduction = (input_size - output_size).max(0.0);

                {
                    // Update file_stats entry
                    let mut stats = file_stats.lock().unwrap();
                    if let Some(stat) = stats.iter_mut().find(|stat| stat.input_file == file) {
                        stat.output_size = Some(output_size);
                        stat.reduction = Some(reduction);
                        stat.elapsed_time = Some(elapsed_time);
                        stat.output_file = Some(output_file);
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

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        install_image_loaders(ctx);
        self.update_progress();

        egui::TopBottomPanel::top("menu_bar").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                ui.menu_button("Menu", |ui| {
                    // Add the toggleable "Show Command" button
                    if ui.button("Add debug items").clicked() {
                        self.add_debug_stats();
                    }
                    if ui.button("Clean items").clicked() {
                        self.file_stats.lock().unwrap().clear();
                    }
                })
            })
        });

        CentralPanel::default().show(ctx, |ui| {
            ui.with_layout(Layout::top_down(Align::LEFT), |ui| {

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

                    // Lock the mutex to safely access the value
                    let is_encoding_value = *self.is_encoding.lock().unwrap();
                    // need horizontal
                    ui.horizontal(|ui| {
                        if is_encoding_value {
                            ui.set_min_height(50.0);
                            ui.add(egui::Image::new(include_image!("../../cat.gif")));
                        } else {
                            let eb = ui.button(RichText::new(format!("{} Encode", icons::PLAY_CIRCLE_FILL)).size(14.0));
                            if eb.clicked() {
                                self.save_config();
                                self.enqueue_jobs();
                                self.start_encoding();
                            }
                        };
                    });

                    ui.heading(RichText::new(format!("{} Progress", icons::PROGRESS_1_FILL)).size(20.0));
                    let remaining_jobs = self.job_queue.lock().unwrap().len();
                    ui.label(format!(
                        "Progress: {}/{} files processed",
                        self.total_jobs - remaining_jobs,
                        self.total_jobs
                    ));
                    ui.add(ProgressBar::new(self.progress).text(format!("{:.0}%", self.progress * 100.0)));

                    ui.heading(RichText::new(format!("{} File Stats ({})", icons::COMPUTER_LINE, self.file_stats.lock().unwrap().len())).size(20.0));
                    egui::ScrollArea::vertical() // or `horizontal()` or `both()` depending on your needs )
                        // .min_scrolled_height(300.0)
                        // .max_height()
                        .show(ui, |ui| {

                            // ui.set_min_width(ui.available_width());
                            // ui.set_min_height(ui.available_height());

                            egui::Grid::new("file_stats_table")
                                //.min_col_width(250.0)
                                .striped(true)
                                // .min_row_height(100.0)
                                .show(ui, |ui| {
                                    // Table headers
                                    ui.label("Input File");
                                    ui.label("Input Size (GB)");
                                    ui.label("");
                                    ui.label("Output Size (GB)");
                                    ui.label("");
                                    ui.label("Reduction (GB)");
                                    ui.label("Elapsed Time (s)");
                                    ui.end_row();

                                    // Populate rows
                                    let file_stats = self.file_stats.lock().unwrap();
                                    for stat in file_stats.iter() {
                                        ui.label(get_file_name(&stat.input_file).unwrap());
                                        ui.label(format!("{:.2}", stat.input_size));
                                        ui.add(
                                            egui::ImageButton::new(egui::include_image!("../../link_input.png"))
                                                .rounding(10.0)
                                        ).on_hover_text("Open Input File")
                                            .clicked()
                                            .then(|| {
                                                open::that(PathBuf::from(&stat.input_file)).unwrap_or(());
                                            });
                                        // if ui.add(egui::Image::new(include_image!("../../link_input.png"))).clicked() {
                                        //     println!("Opening: {}", stat.input_file);
                                        //
                                        // }
                                        if let Some(output_file) = &stat.output_file {
                                            ui.label(format_f64_or_dash(stat.output_size));
                                            // if ui.add(egui::Image::new(include_image!("../../link_output.png"))).clicked() {
                                            //     open::that(output_file).expect("Cannot open output file");
                                            // };
                                            ui.add(
                                                egui::ImageButton::new(egui::include_image!("../../link_output.png"))
                                                    .rounding(10.0)
                                            ).on_hover_text("Open Output File")
                                                .clicked()
                                                .then(|| {
                                                    open::that(output_file).expect("Cannot open output file");
                                                });
                                        } else {
                                            ui.image(include_image!("../../icons8-loading.gif"));
                                            ui.label("");
                                        }

                                        ui.label(format_f64_or_dash(stat.reduction));
                                        ui.label(format_elapsed_time(stat.elapsed_time));
                                        ui.end_row();
                                    }
                                });
                        });
                });
            });
        });

        ctx.request_repaint();
    }
}

fn main() -> Result<(), eframe::Error> {
    let native_options =
        my_default_options(800.0, 500.0, include_bytes!("../../icon.png"));

    eframe::run_native("BeeVEe - Batch Video Encoder", native_options, Box::new(|cc| Ok(Box::new(MyApp::new(cc)))))
}

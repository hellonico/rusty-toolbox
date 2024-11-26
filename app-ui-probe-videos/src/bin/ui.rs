use app_ui_probe_videos::{extract_frame, extract_metadata, Metadata};
use eframe::egui;
use egui::TextStyle::Small;
use egui::{Color32, ComboBox, Context, FontId, RichText, Rounding, ScrollArea, Ui};
use egui_extras::install_image_loaders;
use lib_egui_utils::my_default_options;
use open;
use rfd::FileDialog;
use std::fs;
use std::fs::File;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use tokio::task;

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
    list_mode: bool,
    file_to_delete: Option<String>,
    show_confirmation: bool,
}


#[derive(Default, Clone, Copy, PartialEq, Eq)]
enum SortField {
    #[default]
    Name,
    Date,
    FileSize,
}

fn configure_text_styles(ctx: &egui::Context) {
    use egui::FontFamily::Proportional;
    use egui::TextStyle::*;

    let mut style = (*ctx.style()).clone();
    style.text_styles = [
        (Heading, FontId::new(30.0, Proportional)),
        (Body, FontId::new(18.0, Proportional)),
        (Monospace, FontId::new(14.0, Proportional)),
        (Button, FontId::new(14.0, Proportional)),
        (Small, FontId::new(10.0, Proportional)),
    ].into();
    ctx.set_style(style);
}

impl VideoApp {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let config_file = dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join(".video-gallery");

        let mut app = Self {
            folder_path: String::new(),
            video_files: Arc::new(Mutex::new(Vec::new())),
            load_in_progress: Arc::new(Mutex::new(false)),
            sort_field: SortField::default(),
            sort_asc: true,
            show_filename: true,
            show_filesize: true,
            show_date: true,
            config_file,
            list_mode: true,
            file_to_delete: None,
            show_confirmation :false,
        };
        configure_text_styles(&cc.egui_ctx);
        install_image_loaders(&cc.egui_ctx);

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
                self.show_filename = parts.get(3).unwrap().parse().unwrap();
                self.show_date = parts.get(4).unwrap().parse().unwrap();
                self.show_filesize = parts.get(5).unwrap().parse().unwrap();
                self.list_mode = parts.get(6).unwrap_or(&"true").parse().unwrap();
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
            let settings = format!("{},{},{},{},{},{},{}", self.folder_path, field, order, self.show_filename, self.show_date, self.show_filesize, self.list_mode);
            let _ = file.write_all(settings.as_bytes());
        }
    }

    fn sort_videos(&mut self) {
        // println!("Sorting Videos, {}", chrono::offset::Local::now());
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

        // TODO: decide to clear or not before.
        video_files.lock().unwrap().clear();

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

    fn layout(&mut self, video_file: &&VideoFile, ui: &mut Ui) {
        if self.show_filename {
            ui.label(RichText::new(format!(
                "{}",
                video_file.path.file_name().unwrap_or_default().to_string_lossy()
            )).text_style(Small));
            // ui.label(RichText::new("Loading videos...").text_style(Small));
        }
        if self.show_filesize {
            let file_size = fs::metadata(&video_file.path).map(|m| m.len()).unwrap_or(0);
            let size_in_gb = file_size as f64 / 1_073_741_824.0; // Convert bytes to GB
            ui.label(RichText::new(format!("{:.2} GB", size_in_gb)).text_style(Small));
        }
        if self.show_date {
            let modified_date = fs::metadata(&video_file.path)
                .and_then(|m| m.created())
                .ok()
                .map(|t| {
                    chrono::DateTime::<chrono::Local>::from(t).format("%Y-%m-%d %H:%M").to_string()
                })
                .unwrap_or_else(|| "Unknown".to_string());
            ui.label(RichText::new(format!("{}", modified_date)).text_style(Small));
        }
    }

    fn video_image(&mut self, video_file: &&VideoFile, thumbnail: &PathBuf, ui: &mut Ui) {
        let _f = format!("file://{}", thumbnail.to_str().unwrap());
        let img =
            egui::ImageButton::new(_f.clone())
                .frame(false)
                .rounding(Rounding::from(10.0));

        let res = ui.add_sized([200.0, 100.0], img);

        if res.secondary_clicked() {
            // trash::delete().unwrap();
            self.file_to_delete = Some(format!("{}",video_file.path.to_string_lossy().to_string()));
            self.show_confirmation = true;
        }


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
    }

    fn trash_dialog(&mut self, ctx: &Context) {
        if self.show_confirmation {
            let fil = &self.file_to_delete.clone().unwrap();
            println!("File to delete: {:?}",fil);
            egui::Window::new("Confirm Trash")
                .collapsible(false)
                .resizable(false)
                .show(ctx, |ui| {
                    ui.label(format!("Are you sure you want to move\n {:?}\n to the trash?", fil));
                    ui.horizontal(|ui| {
                        if ui.button("Yes").clicked() {
                            if let file = fil {
                                if let Err(err) = trash::delete(file) {
                                    eprintln!("Failed to trash file: {}", err);
                                } else {
                                    println!("File '{}' moved to trash.", file);
                                    //self.video_files.lock().unwrap().iter().find(|file| file.path) {
                                    // {
                                    //     let mut files = &self.video_files.lock().unwrap();
                                    //     // let mut toremove;
                                    //     // if let Some(pos) = files.iter().find(|file| file.path.to_str().unwrap() == fil) {
                                    //     //     // if fs::metadata(fil).is_err() {
                                    //     //     //     let _ = files.remove(pos);
                                    //     //     //     println!("Removed: {}", fil);
                                    //     //     // }
                                    //     //     toremove = pos;
                                    //     //     // files.
                                    //     // }
                                    //     &self.video_files.get_mut().
                                    // }

                                }
                            }
                            self.show_confirmation = false;
                            self.file_to_delete = None;
                        }
                        if ui.button("No").clicked() {
                            self.show_confirmation = false;
                            self.file_to_delete = None;
                        }
                    });
                });
        }
    }
}

impl eframe::App for VideoApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.trash_dialog(ctx);

        egui::CentralPanel::default().show(ctx, |ui| {
            // Calculate the number of loaded videos
            let video_count = self.video_files.lock().unwrap().len();


            // Input folder path and folder selection
            ui.collapsing("", |ui| {
                ui.horizontal(|ui| {
                    ui.heading(format!("Video Files ({})", video_count)); // Display video count
                });

                ui.horizontal(|ui| {
                    ui.vertical(|ui| {
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
                            ui.label(RichText::new("Loading videos...").text_style(Small));
                        }
                    });

                    ui.vertical(|ui| {
                        ui.label("Sort By:");
                        ComboBox::from_label("")
                            .selected_text(match self.sort_field {
                                SortField::Name => "Name",
                                SortField::Date => "Date",
                                SortField::FileSize => "File Size",
                            })
                            .show_ui(ui, |ui| {
                                //TODO (re-)factorize this
                                if ui.selectable_value(&mut self.sort_field, SortField::Name, "Name").clicked() {
                                    self.sort_videos();
                                    self.save_settings();
                                };
                                if ui.selectable_value(&mut self.sort_field, SortField::Date, "Date").clicked() {
                                    self.sort_videos();
                                    self.save_settings();
                                };
                                if ui.selectable_value(&mut self.sort_field, SortField::FileSize, "File Size").clicked() {
                                    self.sort_videos();
                                    self.save_settings();
                                };
                            });

                        if ui.button(if self.sort_asc { "Ascending" } else { "Descending" }).clicked() {
                            self.sort_asc = !self.sort_asc;
                            self.save_settings();
                            self.sort_videos();
                        }
                    });

                    ui.vertical(|ui| {
                        // Checkboxes to toggle display options
                        if ui.checkbox(&mut self.show_filename, "Show Filename").clicked() {
                            self.save_settings();
                        };
                        if ui.checkbox(&mut self.show_filesize, "Show File Size").clicked() {
                            self.save_settings();
                        };
                        if ui.checkbox(&mut self.show_date, "Show Date").clicked() {
                            self.save_settings();
                        };
                        if ui.checkbox(&mut self.list_mode, "List Mode").clicked() {
                            self.save_settings();
                        };
                    });
                });
            });

            ui.separator();

            ScrollArea::vertical().show(ui, |ui| {
                let video_files = self.video_files.lock().unwrap().clone();


                ui.set_min_width(ui.available_width());
                ui.set_min_height(ui.available_height());
                //
                // println!("1.{}",ui.available_width());

                ui.horizontal_wrapped(|ui| unsafe {
                    ui.set_min_width(ui.available_width());
                    ui.set_min_height(ui.available_height());
                    ui.set_max_height(150.0);
                    //
                    // println!("2.{}",ui.available_width());

                    for (i, video_file) in video_files.iter().enumerate() {



                            if let Some(thumbnail) = &video_file.thumbnail {

                                    if self.list_mode == true {
                                        ui.horizontal(|ui| {
                                            self.video_image(&video_file, thumbnail, ui);
                                            self.layout(&video_file, ui);
                                        });
                                    } else {
                                        ui.vertical(|ui| {
                                            self.video_image(&video_file, thumbnail, ui);
                                            self.layout(&video_file, ui);
                                        });
                                    }


                                if self.list_mode == true {
                                    ui.end_row();
                                }
                                let rem: usize = (ui.available_width() / 210.0).round() as usize;

                                    if ((i+1) % rem == 0) {

                                            // println!("{}, {}", i, rem);
                                            ui.end_row();


                                    }

                            }


                        // });
                        // });

                        // ui.end_row();

                        // });
                        // ui.horizontal(|ui| {
                        //     ui.set_min_size(vec2(200.0,100.0));


                        // else {
                        //     let img = egui::ImageButton::new(format!("file://{}", video_file.metadata.format.filename)).frame(false);
                        //     let res = ui.add_sized([200.0, 100.0], img.clone());
                        //     if res.hovered() {
                        //         ui.painter().rect_stroke(res.rect, 10.0, egui::Stroke {
                        //             width: 1.0,
                        //             color: Color32::from_black_alpha(200),
                        //         });
                        //     }
                        // }

                        // });
                    };
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

    eframe::run_native("Video Browser", options, Box::new(|_cc| Ok(Box::new(VideoApp::new(_cc)))))
}

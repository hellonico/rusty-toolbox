use app_ui_gitlab_download_file::{GitLabClient, GitlabConfig};
use eframe::{egui, CreationContext};
use eframe::egui::{include_image};
use std::sync::Mutex;
use std::{fs, sync::Arc, thread};
use std::process::exit;
use egui::Image;
use egui_extras::install_image_loaders;
use lib_egui_utils::my_default_options;

struct GitLabDownloaderApp {
    config: GitlabConfig,
    error_message: Option<String>,
    // downloading: bool,
    is_downloading: Arc<Mutex<bool>>,
}

impl GitLabDownloaderApp {
    fn new(_cc: &CreationContext) -> Self {
        install_image_loaders(&_cc.egui_ctx);
        Self {
            config: GitlabConfig::default(),
            error_message: None,
            is_downloading: Arc::new(Mutex::new(false)),
        }
    }

    fn load_config_from_file(&mut self, file_path: &str) {
        match fs::read_to_string(file_path) {
            Ok(content) => match serde_yaml::from_str::<GitlabConfig>(&content) {
                Ok(parsed_config) => {
                    self.config = parsed_config;
                    self.error_message = None;
                },
                Err(err) => self.error_message = Some(format!("Failed to parse YAML: {}", err)),
            },
            Err(err) => self.error_message = Some(format!("Failed to read file: {}", err)),
        }
    }

    fn download_file(&mut self) {
        // Replace with your actual GitLab download logic
        println!(
            "Downloading file '{}' from project '{}'...",
            self.config.file_path, self.config.project_name
        );
        // // Simulate download
        // tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

        let client = GitLabClient::new(
            &self.config.gitlab_host,
            &self.config.personal_token,
            &self.config.project_name,
        );
        client.timed_download(
            &self.config.file_path,
            &self.config.branch,
            &self.config.output_folder,
        );
        client.display_folder_size(&self.config.output_folder);

        println!("Download complete!");
    }
}

impl eframe::App for GitLabDownloaderApp {
    fn update(&mut self, ctx: &egui::Context, _: &mut eframe::Frame) {

        egui::TopBottomPanel::top("menu").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("Load Config File").clicked() {
                        if let Some(file_path) = rfd::FileDialog::new().pick_file() {
                            self.load_config_from_file(file_path.to_str().unwrap());
                        }
                        ui.close_menu();
                    }
                    if ui.button("Open Output").clicked() {
                        open::that(&self.config.output_folder.clone());
                        ui.close_menu();
                    }
                    if ui.button("Quit").clicked() {
                        exit(0);
                    }
                });
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.label("GitLab Downloader Configuration:");

            ui.horizontal(|ui| {
                ui.label("GitLab Host:");
                ui.text_edit_singleline(&mut self.config.gitlab_host);
            });
            ui.horizontal(|ui| {
                ui.label("Personal Token:");
                ui.text_edit_singleline(&mut self.config.personal_token);
            });
            ui.horizontal(|ui| {
                ui.label("Project Name:");
                ui.text_edit_singleline(&mut self.config.project_name);
            });
            ui.horizontal(|ui| {
                ui.label("File Path:");
                ui.text_edit_singleline(&mut self.config.file_path);
            });
            ui.horizontal(|ui| {
                ui.label("Branch:");
                ui.text_edit_singleline(&mut self.config.branch);
            });
            ui.horizontal(|ui| {
                ui.label("Output Folder:");
                ui.text_edit_singleline(&mut self.config.output_folder);
            });

            if ui.button("Download").clicked() {
                let mut config = self.config.clone();
                let is_encoding = Arc::clone(&self.is_downloading);
                thread::spawn(move || {
                    *is_encoding.lock().unwrap() = true;
                    GitLabDownloaderApp {
                        config: config,
                        error_message: None,
                        is_downloading: Arc::new(Mutex::new(false)),
                    }
                    .download_file();
                    *is_encoding.lock().unwrap() = false;
                });
            }

            if let Some(error) = &self.error_message {
                ui.label(format!("Error: {}", error));
            }
            //
            if self.is_downloading.lock().unwrap().clone() {
                ui.horizontal(|ui| {
                    ui.label("Download in progress...");
                    ui.add_sized(
                        [20.0,20.0],
                        Image::new(include_image!("../icons8-loading.gif")));
                });
            }

            // Handle drag and drop
            if let dropped_files = &ctx.input(|i| i.raw.dropped_files.clone()) {
                for file in dropped_files {
                    if let Some(path) = &file.path {
                        if matches!(
                        path.extension().and_then(|s| s.to_str()),
                        Some("yaml") | Some("yml")
                    ) {
                            self.load_config_from_file(path.clone().to_str().unwrap());
                        } else {
                            self.error_message =
                                Some("Only .yaml or .yml files are supported.".to_string());
                        }
                    }
                }
            }
            //
        });
    }
}

#[tokio::main]
async fn main() -> Result<(), eframe::Error> {
    let options =
        my_default_options(800.0, 500.0, include_bytes!("../icon.png"));
    // options.defa
    eframe::run_native(
        "GitLab Downloader",
        options,
        Box::new(|_cc| {
            _cc.egui_ctx.set_theme(egui::Theme::Light);
            Ok(Box::new(GitLabDownloaderApp::new(_cc)))
        })
    )
}

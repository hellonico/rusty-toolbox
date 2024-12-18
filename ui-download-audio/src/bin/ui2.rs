use std::env::home_dir;
use eframe::egui;
use std::process::{Command, Stdio};
use std::sync::{Arc, Mutex};
use std::thread;
use std::io::{BufRead, BufReader};
use lib_egui_utils::my_default_options;
use lib_ffmpeg_utils::utils::path_for;

fn main() -> Result<(), eframe::Error> {
    let options =
        my_default_options(500.0, 300.0, include_bytes!("../../icon.png"));

    eframe::run_native("YouTube Downloader", options, Box::new(|_cc| Ok(Box::new(MyApp::default()))))
}

struct MyApp {
    youtube_url: String,
    is_downloading: Arc<Mutex<bool>>,
    downloaded_file: Arc<Mutex<Option<String>>>,
    download_status: Arc<Mutex<Option<String>>>,
    extract_audio: bool,
}

impl Default for MyApp {
    fn default() -> Self {
        Self {
            youtube_url: String::new(),
            is_downloading: Arc::new(Mutex::new(false)),
            downloaded_file: Arc::new(Mutex::new(None)),
            download_status: Arc::new(Mutex::new(None)),
            extract_audio: false,
        }
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("YouTube Downloader");

            // Text input for YouTube URL
            ui.label("Enter YouTube URL:");
            ui.text_edit_singleline(&mut self.youtube_url);


            // Checkbox for extracting audio
            ui.checkbox(&mut self.extract_audio, "Extract audio");


            let is_downloading = self.is_downloading.clone();
            let download_status = self.download_status.clone();
            let extract_audio = self.extract_audio;

            // Download button
            if ui.button("Download").clicked() && !*is_downloading.lock().unwrap() {
                let youtube_url = self.youtube_url.clone();

                // Start the download process in a separate thread
                *is_downloading.lock().unwrap() = true;
                *download_status.lock().unwrap() = None;
                let downloaded_file = self.downloaded_file.clone();
                *downloaded_file.lock().unwrap() = None;

                thread::spawn(move || {
                    let (result, file_name) = download_youtube_video(&youtube_url, extract_audio);
                    *is_downloading.lock().unwrap() = false;
                    *download_status.lock().unwrap() = result;
                    *downloaded_file.lock().unwrap() = file_name;
                });
            }

            // Display the download status
            let is_downloading = self.is_downloading.clone();
            if *is_downloading.lock().unwrap() {
                ui.label("Downloading...");
            }

            let download_status = self.download_status.clone();
            if let Some(status) = &*download_status.lock().unwrap() {
                ui.label(status);
            }

            // Display the downloaded file name
            let downloaded_file = self.downloaded_file.clone();
            if let Some(file_name) = &*downloaded_file.lock().unwrap() {
                if ui.label(format!("Downloaded File: {}", file_name)).clicked() {
                    println!("Filename: {}", file_name.clone());
                    open::that(file_name).unwrap_or_default();
                }
            };
        });
    }
}
fn download_youtube_video(url: &str, extract_audio: bool) -> (Option<String>, Option<String>) {
    if url.is_empty() {
        return (Some("Please enter a valid URL.".to_string()), None);
    }
    let mut binding = Command::new(path_for("yt-dlp"));
    let mut command = binding
        .arg("--ffmpeg-location")
        .arg(path_for("ffmpeg"))
        .arg("-o")
        //.arg("--force-overwrites")
        .arg(format!("{:}/Desktop/%(title)s.%(ext)s", home_dir().unwrap().to_string_lossy().to_string())); // Dynamic filename based on video title

    println!("{:?}", extract_audio);
    if extract_audio {
        command.arg("-x").arg("--audio-format").arg("mp3"); // Add the -x argument for audio extraction
    }

    command.arg(url).stdout(Stdio::piped());

    if let Ok(mut child) = command.spawn() {
        let stdout = BufReader::new(child.stdout.take().unwrap());
        let mut file_path = None;

        // Read the yt-dlp output line by line
        for line in stdout.lines() {
            if let Ok(line) = line {
                // Detect the line with "Destination: " to extract the file name
                if let Some(path) = line.strip_prefix("[download] Destination: ") {
                    file_path = Some(path.to_string());
                }
            }
        }

        // Wait for the process to finish
        if let Ok(status) = child.wait() {
            if status.success() {
                if let Some(path) = file_path {
                    return (Some("Download successful.".to_string()), Some(path));
                } else {
                    return (Some("Download successful, but file name is missing.".to_string()), None);
                }
            } else {
                return (Some(format!("yt-dlp exited with status: {}", status)), None);
            }
        }
    }

    (Some("Failed to start yt-dlp.".to_string()), None)
}
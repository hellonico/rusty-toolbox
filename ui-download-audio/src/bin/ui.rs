use eframe::egui;
use std::sync::{Arc, Mutex};
use std::thread;

fn main() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions::default();
    eframe::run_native("YouTube Downloader", options, Box::new(|_cc| Ok(Box::new(MyApp::default()))))
}

struct MyApp {
    youtube_url: String,
    is_downloading: Arc<Mutex<bool>>,
    download_status: Arc<Mutex<Option<String>>>,
}

impl Default for MyApp {
    fn default() -> Self {
        Self {
            youtube_url: String::new(),
            is_downloading: Arc::new(Mutex::new(false)),
            download_status: Arc::new(Mutex::new(None)),
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

            let is_downloading = self.is_downloading.clone();
            let download_status = self.download_status.clone();

            // Download button
            if ui.button("Download").clicked() && !*is_downloading.lock().unwrap() {
                let youtube_url = self.youtube_url.clone();

                // Start the download process in a separate thread
                *is_downloading.lock().unwrap() = true;
                *download_status.lock().unwrap() = None;

                thread::spawn(move || {
                    let result = download_youtube_video(&youtube_url);
                    *is_downloading.lock().unwrap() = false;
                    *download_status.lock().unwrap() = result;
                });
            }

            // Display the download status
            let is_downloading = self.is_downloading.clone();
            if *is_downloading.lock().unwrap() {
                ui.label("Downloading...");
            }
            
            if let Some(status) = &*download_status.lock().unwrap() {
                ui.label(status);
            }
        });
    }
}

// Function to download YouTube video using yt-dlp
fn download_youtube_video(url: &str) -> Option<String> {
    if url.is_empty() {
        return Some("Please enter a valid URL.".to_string());
    }

    let status = std::process::Command::new("yt-dlp")
        .arg("-o")
        .arg("%(title)s.%(ext)s") // Dynamic filename based on video title
        .arg(url)
        .status();

    match status {
        Ok(exit_status) if exit_status.success() => Some("Download successful.".to_string()),
        Ok(exit_status) => Some(format!("yt-dlp exited with status: {}", exit_status)),
        Err(e) => Some(format!("Failed to execute yt-dlp: {}", e)),
    }
}

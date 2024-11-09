use std::env;
use eframe::egui;
use egui_extras::install_image_loaders;
use std::path::Path;
use std::process::Command;
use std::time::{Duration, Instant};
use tempfile::Builder;

struct MyApp {
    output_path: String,
    last_update: Instant,
}

impl MyApp {
    fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        Self {
            output_path: String::new(),
            last_update: Instant::now(),
        }

    }

    fn capture_screenshot(&mut self) {
        let output_path = "output.png";

        // Determine the FFmpeg command based on the platform
        #[cfg(target_os = "windows")]
        let ffmpeg_args = vec![
            "-y", // overwrite
            "-f", "gdigrab", // Use GDI Grab for screen capture
            "-i", "desktop", // Input source
            "-frames:v", "1", // Capture a single frame
            output_path, // Output file
        ];

        #[cfg(target_os = "macos")]
        let ffmpeg_args = vec![
            "-y", // overwrite
            "-f", "avfoundation", // Use AVFoundation for macOS
            "-i", "1:0", // Adjust input index based on your devices
            "-frames:v", "1", // Capture a single frame
            output_path, // Output file
        ];

        // Spawn the FFmpeg process
        let status = Command::new("ffmpeg")
            .args(ffmpeg_args)
            .status()
            .expect("Failed to start FFmpeg");

        if status.success() {
            println!("Screenshot saved to: {}", output_path);
        } else {
            println!("Failed to capture screenshot. Check FFmpeg setup.");
        }

        self.output_path = output_path.to_string()
    }

}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _: &mut eframe::Frame) {
        install_image_loaders(ctx);

        let path = Path::new(&self.output_path);
        let path_exist = path.exists();
        let img_path = format!("file://{}", path.to_str().unwrap());

        // Update the screenshot every 5 seconds
        if self.last_update.elapsed() > Duration::from_millis(5000) {
            self.capture_screenshot();
            self.last_update = Instant::now();
            ctx.forget_image(img_path.as_str());
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Screenshot Viewer");

            if path_exist {
                ui.image(img_path);// Show the screenshot as a small image
            } else {
                // println!("Failed to load screenshot. {}",path.display());
                ui.label("No screenshot available.");
            }
        });

        ctx.request_repaint_after(Duration::from_millis(100)); // Keep the app responsive
    }
}

fn main() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions::default();

    eframe::run_native("Screenshot Viewer",
                       options,
                       Box::new(|_cc| Ok(Box::new(MyApp::new(_cc)) as Box<dyn eframe::App>)))
}

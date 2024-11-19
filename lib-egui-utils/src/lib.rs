use std::fs;
use std::path::Path;
use eframe::egui::{IconData, ViewportBuilder};
use eframe::{egui, NativeOptions};
use image::GenericImageView;

pub fn format_f64_or_dash(stat: Option<f64>) -> String {
    stat.map(|t| format!("{:.2}", t)).unwrap_or_else(|| "-".to_string())
}

pub fn generate_output_path(file: &String, output_folder:String, video_format:String) -> String{
    format!(
        "{}/{}.{}",
        output_folder,
        Path::new(&file)
            .file_stem()
            .unwrap_or_default()
            .to_string_lossy(),
        video_format
    )
}

pub fn format_elapsed_time(duration: Option<f64>) -> String {
    if let Some(seconds) = duration {
        let total_seconds = seconds as u64; // Convert to integer for hours, minutes, seconds
        let fractional_seconds = seconds - total_seconds as f64;

        let hours = total_seconds / 3600;
        let minutes = (total_seconds % 3600) / 60;
        let seconds_whole = total_seconds % 60;
        let milliseconds = (fractional_seconds * 1000.0).round() as u64;

        format!(
            "{:02}:{:02}:{:02}.{:03}",
            hours, minutes, seconds_whole, milliseconds
        )
    } else {
        "-".into()
    }

}


// Helper functions
pub fn get_file_size_in_gb(file_path: &str) -> f64 {
    fs::metadata(file_path)
        .map(|meta| meta.len() as f64 / (1024.0 * 1024.0 * 1024.0))
        .unwrap_or(0.0)
}


pub fn icon(icon_bytes: &[u8]) -> IconData {
    // Include the image at compile time
    // let icon_bytes = include_bytes!("../../icon.png");
    // Decode the embedded image into RGBA data
    let icon_image = image::load_from_memory(icon_bytes).expect("Failed to load icon");
    let (width, height) = icon_image.dimensions();
    let rgba = icon_image.to_rgba8();
    let icon = IconData {
        rgba: rgba.into_raw(),
        width,
        height,
    };
    icon
}

pub fn my_default_options(x: f32, y: f32, bytes: &[u8]) -> NativeOptions {
    let app_icon = icon(bytes);
    let options = NativeOptions {
        viewport: ViewportBuilder::default()
            .with_close_button(true)
            .with_inner_size(egui::Vec2::new(x, y))
            .with_icon(app_icon),
        ..Default::default()
    };
    options
}




pub fn list_files_from_dir(input_folder: &String, extension_filter: &String) -> Vec<String> {
    // Attempt to read the directory
    let files = match fs::read_dir(input_folder) {
        Ok(read_dir) => read_dir,
        Err(_) => {
            eprintln!("Directory not found: {}", input_folder);
            return Vec::new(); // Return an empty vector if the folder does not exist
        }
    };

    // Process the directory entries
    files
        .filter_map(|entry| entry.ok())
        .filter(|entry| {
            let file_name = entry.file_name();
            let file_name = file_name.to_string_lossy();
            !file_name.starts_with('.') // Skip dot files
                && entry.path().extension().map_or(false, |ext| {
                ext.to_str().unwrap().eq_ignore_ascii_case(extension_filter)
            })
        })
        .map(|entry| entry.path().to_string_lossy().to_string())
        .collect::<Vec<_>>()
}
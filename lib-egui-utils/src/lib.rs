

use std::{env, fs, io};
use std::fs::File;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::SystemTime;
use eframe::egui::{FontId, IconData, ViewportBuilder};
use eframe::{egui, NativeOptions};
use egui::FontData;
use image::GenericImageView;
use SortBy::Size;

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

pub fn open_containing_folder(output_file: &String) {
    let folder_path = env::current_dir().unwrap().display().to_string();
    eprintln!("Opening: {} {}", folder_path, output_file );

    #[cfg(target_os = "macos")]
    Command::new("open")
        .arg(folder_path.clone())
        .spawn()
        .expect("Failed to open folder");

    #[cfg(target_os = "windows")]
    Command::new("explorer")
        .arg(folder_path)
        .spawn()
        .expect("Failed to open folder");

    #[cfg(target_os = "linux")]
    Command::new("xdg-open")
        .arg(folder_path)
        .spawn()
        .expect("Failed to open folder");
}


// Helper functions
pub fn get_file_size_in_gb(file_path: &str) -> io::Result<f64> {
    let file = File::open(file_path)?;
    let metadata = file.metadata()?;
    Ok(metadata.len() as f64 / (1024.0 * 1024.0 * 1024.0)) // Size in GB
}


pub fn get_file_name(path: &str) -> Option<String> {
    Path::new(path).file_name().map(|name| name.to_string_lossy().into_owned())
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

        // default_theme: eframe::Theme::Light, // Use the light theme
        viewport: ViewportBuilder::default().with_transparent(true)
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

pub enum SortBy {
    FileName,
    Size,
    LastUpdated,
}

pub fn list_files_from_dir2(
    input_folder: &String,
    extension_filter: &String,
    sort_by: SortBy,
    ascending: bool,
) -> Vec<String> {
    // Attempt to read the directory
    let files = match fs::read_dir(input_folder) {
        Ok(read_dir) => read_dir,
        Err(_) => {
            eprintln!("Directory not found: {}", input_folder);
            return Vec::new(); // Return an empty vector if the folder does not exist
        }
    };

    // Collect valid entries into a vector
    let mut file_entries: Vec<(PathBuf, u64, SystemTime)> = files
        .filter_map(|entry| entry.ok())
        .filter_map(|entry| {
            let metadata = entry.metadata().ok()?;
            let modified = metadata.modified().ok()?;
            let file_name = entry.file_name();
            let file_name = file_name.to_string_lossy();

            if file_name.starts_with('.') {
                return None; // Skip dot files
            }

            if entry.path().extension().map_or(false, |ext| {
                ext.to_str().unwrap().eq_ignore_ascii_case(extension_filter)
            }) {
                Some((entry.path(), metadata.len(), modified))
            } else {
                None
            }
        })
        .collect();

    // Sort the vector based on the specified criteria
    match sort_by {
        SortBy::FileName => {
            file_entries.sort_by(|a, b| a.0.file_name().cmp(&b.0.file_name()));
        }
        Size => {
            file_entries.sort_by(|a, b| a.1.cmp(&b.1));
        }
        SortBy::LastUpdated => {
            file_entries.sort_by(|a, b| a.2.cmp(&b.2));
        }
    }

    // Reverse the order if descending
    if !ascending {
        file_entries.reverse();
    }

    // Map the sorted entries to their string paths
    file_entries
        .into_iter()
        .map(|(path, _, _)| path.to_string_lossy().to_string())
        .collect()
}


pub fn add_font(ctx: &egui::Context, name: &str, bytes:&[u8]) {
    let mut fonts = egui::FontDefinitions::default();
    let boxed_bytes = Box::leak(bytes.to_vec().into_boxed_slice());
    fonts.font_data.insert(
        name.to_owned(),
        FontData::from_static(boxed_bytes),
    );

    fonts
        .families
        .get_mut(&egui::FontFamily::Proportional)
        .unwrap()
        .insert(0, name.to_owned());

    ctx.set_fonts(fonts);
}

pub fn configure_text_styles(ctx: &egui::Context) {
    use egui::FontFamily::Proportional;
    use egui::TextStyle::*;

    let mut style = (*ctx.style()).clone();
    style.text_styles = [
        (Heading, FontId::new(20.0, Proportional)),
        (Body, FontId::new(11.0, Proportional)),
        (Monospace, FontId::new(11.0, Proportional)),
        (Button, FontId::new(11.0, Proportional)),
        (Small, FontId::new(10.0, Proportional)),
    ].into();
    ctx.set_style(style);
}

pub mod mywidgets;
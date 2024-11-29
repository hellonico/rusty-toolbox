use std::path::Path;
use std::process::Command;
use serde::Deserialize;
use serde_json::from_str;

#[derive(Debug, Deserialize, Clone, Default)]
pub struct Format {
    pub filename: String,
    nb_streams: u32,
    nb_programs: u32,
    nb_stream_groups: u32,
    format_name: String,
    format_long_name: String,
    start_time: String,
    duration: String,
    size: String,
    bit_rate: String,
    probe_score: u32,
    tags: Tags,
}

#[derive(Debug, Deserialize, Clone, Default)]
pub struct Tags {
    major_brand: String,
    minor_version: String,
    compatible_brands: String,
    encoder: Option<String>,
}

#[derive(Debug, Deserialize, Clone, Default)]
pub struct Metadata {
    pub format: Format,
}


pub fn extract_metadata(video_path: String) -> Metadata {
    // Run the ffprobe command
    let output = Command::new("/opt/homebrew/bin/ffprobe")
        .arg("-v")
        .arg("quiet")
        .arg("-print_format")
        .arg("json")
        .arg("-show_format")
        // .arg("-show_streams")
        .arg(video_path)
        .output()
        .expect("Failed to execute ffprobe");

    // Parse and print the JSON output
    let metadata_string = String::from_utf8_lossy(&output.stdout);
    println!("Extracted metadata:\n{}", metadata_string);

    // Parse JSON into the Metadata struct
    let metadata: Metadata = from_str(metadata_string.as_ref()).expect("Failed to parse JSON");
    metadata
}

pub fn extract_frame(video_path: &str, timestamp: &str, output_image: &str) -> Result<(), String> {
    // Ensure the video file exists
    if !Path::new(video_path).exists() {
        return Err(format!("Video file '{}' does not exist.", video_path));
    }

    // Execute the ffmpeg command
    // let output = Command::new("ffmpeg")
    //     .args(&[
    //         "-i", video_path,          // Input video file
    //         "-ss", timestamp,         // Timestamp to extract (e.g., "00:00:01")
    //         "-vframes", "1",          // Extract one frame
    //         output_image,             // Output image file
    //     ])
    //     .output();
    let output = Command::new("/opt/homebrew/bin/ffmpeg")
        .args(&[
            "-i", video_path,           // Input video file
            "-ss", timestamp,           // Timestamp to extract (e.g., "00:00:01")
            "-vframes", "1",            // Extract one frame
            "-vf", "scale=-1:200",      // Optional: Resize to max height 720px, preserving aspect ratio
            "-q:v", "30",                // Set quality (lower is better, 2-31, where 31 is worst)
            output_image,               // Output image file
        ])
        .output();

    // Check the result of the command
    match output {
        Ok(result) if result.status.success() => {
            println!("Frame saved to {}", output_image);
            Ok(())
        }
        Ok(result) => {
            Err(format!(
                "FFmpeg error: {}",
                String::from_utf8_lossy(&result.stderr)
            ))
        }
        Err(e) => Err(format!("Failed to execute FFmpeg: {}", e)),
    }
}

use std::process::Command;
use std::io;

fn main() {
    // Ask the user for the YouTube video URL
    println!("Enter the YouTube video URL:");
    let mut video_url = String::new();
    io::stdin().read_line(&mut video_url).expect("Failed to read input");
    let video_url = video_url.trim(); // Remove any trailing newline or spaces

    // Define the output format and path
    let output_path = "downloaded_video.mp4";

    // Execute yt-dlp command
    let status = Command::new("yt-dlp")
        .arg("-o")
        .arg(output_path) // Specify the output file path
        .arg(video_url) // Pass the video URL
        .status();

    match status {
        Ok(exit_status) if exit_status.success() => {
            println!("Video downloaded successfully to {}", output_path);
        }
        Ok(exit_status) => {
            eprintln!("yt-dlp exited with status: {}", exit_status);
        }
        Err(e) => {
            eprintln!("Failed to execute yt-dlp: {}", e);
        }
    }
}

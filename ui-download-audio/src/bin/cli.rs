use std::process::Command;
use std::io;
use reqwest;
use scraper::{Html, Selector};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Ask the user for the YouTube video URL
    println!("Enter the YouTube video URL:");
    let mut video_url = String::new();
    io::stdin().read_line(&mut video_url).expect("Failed to read input");
    let video_url = video_url.trim(); // Remove any trailing newline or spaces

    // Fetch the HTML content of the page
    let html_content = reqwest::blocking::get(video_url)?.text()?;

    // Parse the HTML to extract the `meta` tag content
    let document = Html::parse_document(&html_content);
    let selector = Selector::parse(r#"meta[name="twitter:title"]"#).unwrap();

    let title = if let Some(element) = document.select(&selector).next() {
        element.value().attr("content").unwrap_or("video")
    } else {
        "video"
    };

    // Sanitize the title for file name
    let sanitized_title = title
        .replace(|c: char| !c.is_alphanumeric() && c != ' ', "_")
        .trim()
        .to_string();
    let output_path = format!("{}.mp4", sanitized_title);

    // Execute yt-dlp command
    let status = Command::new("yt-dlp")
        .arg("-o")
        .arg(&output_path) // Specify the dynamic output file path
        .arg(video_url)    // Pass the video URL
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

    Ok(())
}

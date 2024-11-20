use app_ui_probe_videos::{extract_frame, extract_metadata};
use std::path::Path;

fn main() {
    let video_path = "/Volumes/518/SCREENS3/Screen Recording 2022-04-27 at 09.01.26.mp4";
    if Path::new(video_path).exists() {
        println!("The file exists!");
    } else {
        println!("The file does not exist.");
    }
    let metadata = extract_metadata(video_path.into());
    // Print the parsed metadata
    println!("{:#?}", metadata);

    let timestamp = "01:00:01"; // Extract frame at 1-second mark
    let output_image = "output_frame.png";

    match extract_frame(video_path, timestamp, output_image) {
        Ok(_) => println!("Frame extraction succeeded!"),
        Err(e) => eprintln!("Error: {}", e),
    }
}

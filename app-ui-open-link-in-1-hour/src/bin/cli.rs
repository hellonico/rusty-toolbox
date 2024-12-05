use tokio::time::{sleep, Duration};
use std::process;

#[tokio::main]
async fn main() {
    let url = "https://us04web.zoom.us/j/72238146825?pwd=6AcxYpeMFbHPhhImbiJP6tJF9h3djM.1"; // Replace with your desired URL
    println!("Waiting for 5 seconds to open the link...");

    // Wait for 1 hour (3600 seconds)
    sleep(Duration::from_secs(5)).await;

    // Open the URL
    if let Err(e) = open::that(url) {
        eprintln!("Failed to open the link: {}", e);
        process::exit(1);
    }

    println!("Link opened: {}", url);
}

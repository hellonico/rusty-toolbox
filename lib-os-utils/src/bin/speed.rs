use lib_os_utils::speed::{test_download_speed, test_upload_speed};

#[tokio::main]
async fn main() {
    println!("Starting internet speed test...");

    match test_download_speed().await {
        Ok(speed) => println!("Download speed: {:.2} Mbps", speed),
        Err(e) => println!("Download test failed: {}", e),
    }

    match test_upload_speed().await {
        Ok(speed) => println!("Upload speed: {:.2} Mbps", speed),
        Err(e) => println!("Upload test failed: {}", e),
    }
}

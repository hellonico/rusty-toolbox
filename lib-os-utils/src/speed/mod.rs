
use std::time::Instant;
use anyhow::Result;
use futures::StreamExt;
use reqwest::Client;

const TEST_URL:&str = "https://storage.googleapis.com/golang/go1.20.5.src.tar.gz";

pub async fn test_download_speed() -> Result<f64, reqwest::Error> {
    let client = Client::new();
    let start = Instant::now();

    let mut response = client.get(TEST_URL).send().await?;
    let mut total_bytes = 0u64;

    let mut stream = response.bytes_stream();
    while let Some(chunk) = stream.next().await {
        let chunk = chunk?; // Handle any errors in the stream.
        total_bytes += chunk.len() as u64;
    }

    let duration = start.elapsed().as_secs_f64();

    if duration > 0.0 {
        let size_mb = total_bytes as f64 / (1024.0 * 1024.0);
        let speed_mbps = (size_mb / duration) * 8.0; // Convert MBps to Mbps.
        Ok(speed_mbps)
    } else {
        Ok(0.0) // Avoid divide-by-zero errors.
    }
}

pub async fn test_upload_speed() -> Result<f64> {
    let client = reqwest::Client::new();
    let data = vec![0u8; 10 * 1024 * 1024]; // 10 MB of random data
    let start = std::time::Instant::now();

    let response = client
        .post("https://httpbin.org/post") // Replace with a preferred test endpoint.
        .body(data)
        .send()
        .await?;

    if response.status().is_success() {
        let duration = start.elapsed().as_secs_f64();
        let size_mb = (10 * 1024 * 1024) as f64 / (1024.0 * 1024.0);
        let speed_mbps = (size_mb / duration) * 8.0;
        Ok(speed_mbps)
    } else {
        Err(anyhow::anyhow!(
            "Upload test failed with status: {}",
            response.status()
        ))
    }
}

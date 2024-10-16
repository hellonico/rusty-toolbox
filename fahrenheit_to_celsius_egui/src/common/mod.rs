use reqwest::Client;
use serde::Deserialize;
use std::sync::{Arc, Mutex};
//
// pub async fn convert_temperature(fahrenheit: String) -> Result<String, Error> {
//     let response: serde_json::Value = reqwest::get(&format!("http://localhost:8000/convert?fahrenheit={}", fahrenheit))
//         .await?
//         .json()
//         .await?;
//
//     Ok(response["celsius"].to_string())
// }


#[derive(Deserialize)]
struct ApiResponse {
    celsius: f64,
}
pub async fn fetch_data(value: String, response: Arc<Mutex<Option<String>>>) {
    let client = Client::new();
    let req = &format!("http://localhost:8000/convert?fahrenheit={}", value);
    match client
        .get(req)
        .send()
        .await
    {
        Ok(res) => {
            if let Ok(json) = res.json::<ApiResponse>().await {
                let mut response_lock = response.lock().unwrap();
                *response_lock = Some(json.celsius.to_string());
            }
        }
        Err(_) => {
            let mut response_lock = response.lock().unwrap();
            *response_lock = Some("Error fetching data".to_string());
        }
    }
}
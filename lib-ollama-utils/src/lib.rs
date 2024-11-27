use futures::StreamExt;
use reqwest::Client;
use serde::Deserialize;
use std::error::Error;

#[derive(Deserialize, Debug)]
struct ResponseData {
    response: String,
    done: bool,
}

/// Function to interact with Ollama API
///
/// # Parameters
/// - `model`: The name of the model to use.
/// - `prompt`: The prompt input for the model.
/// - `on_token`: A function to execute for each received token.

pub async fn ollama<F>(url: &str, model: &str, prompt: &str, on_token: F) -> Result<(), Box<dyn Error>>
where
    F: Fn(&str) + Send + Sync, // `on_token` must be callable from multiple threads
{
    let client = Client::new();

    // Send a POST request
    let response = client
        .post(url.to_string())
        .json(&serde_json::json!({
            "model": model.to_string(),
            "prompt": prompt
        }))
        .send()
        .await?;

    // print!("{}", response.headers().clone().un);

    // Stream the response body line by line
    let mut stream = response.bytes_stream();
    let mut buffer = Vec::new();


    while let Some(chunk) = stream.next().await {
        match chunk {
            Ok(bytes) => {
                buffer.extend_from_slice(&bytes);

                // Attempt to process as valid JSON
                if let Ok(json) = serde_json::from_slice::<ResponseData>(&buffer) {

                    on_token(&json.response);

                    // Stop when `done` is true
                    if json.done {
                        break;
                    }

                    // Clear the buffer after successful parse
                    buffer.clear();
                }
            }
            Err(e) => {
                eprintln!("Error reading chunk: {}", e);
                break;
            }
        }
    }

    Ok(())
}

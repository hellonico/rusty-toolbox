use surf::Client;
use serde::{Deserialize, Serialize};
use serde_json::{Deserializer};
use std::error::Error;
use futures::stream::StreamExt;
use futures::TryFutureExt;

#[derive(Serialize)]
struct RequestBody<'a> {
    model: &'a str,
    prompt: &'a str,
}

#[derive(Deserialize, Debug)]
struct ResponseData {
    model: String,
    created_at: String,
    response: String,
    done: bool,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let client = Client::new();

    // Set up the request body
    let body = RequestBody {
        model: "llama3.2",
        prompt: "Why is the sky blue?",
    };

    // Send the POST request to Ollama server
    let mut res = client
        .post("http://localhost:11434/api/generate")
        .body_json(&body)?
        .send()
        .await?;

    // Read the entire body as bytes
    let body_bytes = res.body_bytes().await?;

    // Create a stream to process the body as individual JSON objects
    let stream = Deserializer::from_slice(&body_bytes)
        .into_iter::<ResponseData>();

    // Process each individual JSON object
    for result in stream {
        match result {
            Ok(response) => {
                print!("{}", response.response);

                // Stop if done is true
                if response.done {
                    break;
                }
            }
            Err(e) => {
                eprintln!("Error deserializing JSON: {}", e);
                break;
            }
        }
    }

    Ok(())
}

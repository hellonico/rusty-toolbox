use futures::StreamExt;
use reqwest::Client;
use serde::de::StdError;
use serde::Deserialize;
use std::error::Error;
use std::future::Future;

#[derive(Deserialize, Debug)]
struct GenerateData {
    response: String,
    done: bool,
}

#[derive(Deserialize, Debug)]
struct MessageData {
    content: String,
    role: String,
}

#[derive(Deserialize, Debug)]
struct ChatData {
    message: MessageData,
    done: bool,
}

/// Function to interact with Ollama API
///
/// # Parameters
/// - `model`: The name of the model to use.
/// - `prompt`: The prompt input for the model.
/// - `on_token`: A function to execute for each received token.
use serde_json::{json, Value};

fn convert_to_json(model: &str, messages: &Vec<(String, String)>) -> serde_json::Value {
    // Transform messages into the desired format
    let formatted_messages: Vec<serde_json::Value> = messages
        .iter()
        .map(|(user, text)| {
            json!({
                "role": user,
                "content": text
            })
        })
        .collect();

    // Build the JSON structure
    json!({
        "model": model,
        "messages": formatted_messages
    })
}

pub async fn ollama<F>(
    base_url: &str,
    model: &str,
    prompt: &str,
    on_token: F,
) -> Result<(), Box<dyn Error>>
where
    F: Fn(&str) + Send + Sync,
{
    let url = format!("{}/api/generate", base_url);
    let json = json!({
        "model": model.to_string(),
        "prompt": prompt
    });

    process_stream(&url, json, on_token, |buffer| {
        serde_json::from_slice::<GenerateData>(&buffer).map(|json| (json.response, json.done))
    })
        .await
}

pub async fn ollama_with_messages<F>(
    base_url: &str,
    model: &str,
    messages: &Vec<(String, String)>,
    on_token: F,
) -> Result<(), Box<dyn Error>>
where
    F: Fn(&str) + Send + Sync,
{
    let url = format!("{}/api/chat", base_url);
    let json = convert_to_json(model, messages);

    process_stream(&url, json, on_token, |buffer| {
        serde_json::from_slice::<ChatData>(&buffer).map(|json| (json.message.content, json.done))
    })
        .await
}

async fn process_stream<F, P>(
    url: &str,
    json: Value,
    on_token: F,
    parse_chunk: P,
) -> Result<(), Box<dyn Error>>
where
    F: Fn(&str) + Send + Sync,
    P: Fn(&[u8]) -> Result<(String, bool), serde_json::Error>,
{
    let client = Client::new();
    let response = client.post(url).json(&json).send().await?;
    let mut stream = response.bytes_stream();
    let mut buffer = Vec::new();

    while let Some(chunk) = stream.next().await {
        match chunk {
            Ok(bytes) => {
                buffer.extend_from_slice(&bytes);
                match parse_chunk(&buffer) {
                    Ok((token, done)) => {
                        on_token(&token);
                        if done {
                            break;
                        }
                        buffer.clear();
                    }
                    Err(_) => {
                        // Wait for more data to parse successfully
                    }
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


////
// Function to fetch models from the API

// Struct to parse the API response
#[derive(Deserialize, Debug)]
struct ModelDetails {
    format: String,
    family: String,
    families: Option<Vec<String>>,
    parameter_size: String,
    quantization_level: String,
}

#[derive(Deserialize, Debug)]
struct Model {
    name: String,
    modified_at: String,
    size: u64,
    digest: String,
    details: ModelDetails,
}

#[derive(Deserialize, Debug)]
struct ModelsResponse {
    models: Vec<Model>,
}
pub async fn fetch_models(base_url:String) -> Vec<String> {
    let url = format!("{}/api/tags",base_url);
    let response = reqwest::get(url).await.unwrap();
    let models_response: ModelsResponse = response.json().await.unwrap();
    let models:Vec<String> = models_response.models.into_iter().map(|m| m.name).collect();
    models
}
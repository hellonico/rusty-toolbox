use clap::{Arg, Command};
use reqwest::Client;
use std::error::Error;
use tokio::runtime::Runtime;
use lib_ollama_utils::{model_download, ollama_with_messages};


async fn pull_model(base_url: &str, model_name: &str) -> Result<(), Box<dyn Error>> {
    model_download(base_url, model_name,|token| {
            println!("{:?}", token);
    }).await.expect("error");
    Ok(())
}

fn main() {
    // Set up CLI arguments
    let matches = Command::new("Model Puller")
        .version("1.0")
        .author("Your Name <your.email@example.com>")
        .about("Pulls a model from the ollama library")
        .arg(
            Arg::new("model")
                .short('m')
                .long("model")
                .default_value("qwen")
                .help("The name of the model to pull"),
        )
        .arg(
            Arg::new("url")
                .short('u')
                .long("url")
                .default_value("http://localhost:11434")
                .help("Base URL of the server (default: http://localhost:11434)"),
        )
        .get_matches();

    let model_name = matches.get_one::<String>("model").unwrap();
    let base_url = matches.get_one::<String>("url").unwrap();

    // Create the async runtime
    let rt = Runtime::new().unwrap();

    // Run the async function
    rt.block_on(async {
        if let Err(e) = pull_model(base_url, model_name).await {
            eprintln!("Error pulling model: {}", e);
        }
    });
}

use clap::Parser;
use lib_ollama_utils::{model_delete, model_info};
use std::error::Error;

/// Fetches model information from a specified server
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// URL of the server
    #[arg(short, long, default_value = "http://localhost:11434")]
    url: String,

    /// Name of the model
    #[arg(short, long, default_value = "llama3.2")]
    model: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Parse command-line arguments
    let args = Args::parse();

    // Call the function with arguments from the CLI
    let model_delete = model_delete(args.url, args.model).await;

    // Serialize the result to JSON and print
    // let json_str = serde_json::to_string_pretty(&model_show)?;
    println!("{:?}", model_delete);

    Ok(())
}


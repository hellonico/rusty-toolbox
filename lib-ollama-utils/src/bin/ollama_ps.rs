use clap::Parser;
use lib_ollama_utils::{model_info, model_ps};
use std::error::Error;

/// Fetches model information from a specified server
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// URL of the server
    #[arg(short, long, default_value = "http://localhost:11434")]
    url: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Parse command-line arguments
    let args = Args::parse();

    // Call the function with arguments from the CLI
    let ps_response = model_ps(args.url).await;

    // Serialize the result to JSON and print
    let json_str = serde_json::to_string_pretty(&ps_response)?;
    println!("{}", json_str);

    Ok(())
}

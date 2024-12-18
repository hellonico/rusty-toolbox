use std::error::Error;
use lib_ollama_utils::fetch_models;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let models = fetch_models("http://localhost:11434".into()).await;
    let model_names : Vec<String> = models.iter().map(|model| model.name.clone()).collect();
    let json_str = serde_json::to_string_pretty(&model_names)?;
    println!("{}", json_str);
    Ok(())
}

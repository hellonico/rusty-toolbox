use std::error::Error;
use std::io;
use std::io::Write;
use lib_ollama_utils::ollama;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    ollama("llama3.2", "Who is super mario?", |token| {
        print!("{}", token);
        io::stdout().flush().unwrap(); // Ensure immediate display
    })
        .await
}

use std::error::Error;
use std::io;
use std::io::Write;
use clap::{Arg, Command};
use lib_ollama_utils::ollama;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let matches = Command::new("My Super Mario CLI")
        .version("0.1.0")
        .author("Your Name <your.email@example.com>")
        .about("Asks OLLAMA for information about Super Mario")
        .arg(Arg::new("ollama_url")
            .short('u')
            .long("url")
            .default_value("http://localhost:11434")
            .value_name("URL")
            .help("Sets the URL of the OLLAMA server")
            // .takes_value(true)
        )
        .arg(Arg::new("model")
            .short('m')
            .long("model")
            .default_value("llama3.2")
            .value_name("MODEL")
            .help("Sets the model to use")
            // .takes_value(true)
        )
        .arg(Arg::new("question")
            // .required(true)
            .index(1)
            .value_name("QUESTION")
            .default_value("Who are the Super Mario main characters?")
            .help("The question to ask OLLAMA"))
        .get_matches();

    let ollama_url = matches.get_one::<String>("ollama_url").unwrap();
    let model = matches.get_one::<String>("model").unwrap();
    let question = matches.get_one::<String>("question").unwrap();

    println!("# SETTINGS:\n{ollama_url}\n{model}\n# QUESTION:\n{question}");

    ollama(ollama_url, model, question, |token| {
        print!("{}", token);
        io::stdout().flush().unwrap(); // Ensure immediate display
    }).await?;

    Ok(())
}

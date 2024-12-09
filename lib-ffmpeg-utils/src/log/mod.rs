use std::fs::OpenOptions;
use std::path::PathBuf;
use chrono::Local;
use dirs::home_dir;
use std::io::Write;

pub fn append_to_log_file(log_file_path: PathBuf, log_message: String) {
    // Open the file in append mode. Create it if it doesn't exist.
    let mut file = OpenOptions::new()
        .create(true) // Create the file if it doesn't exist
        .append(true) // Append to the file
        .open(log_file_path).unwrap();

    let timestamp = Local::now().format("[%Y-%m-%d %H:%M:%S]").to_string();

    // Write the log message followed by a newline
    writeln!(file, "{} {}", timestamp, log_message);

    // Ok(())
}

pub fn append_to_home_log(log_message: String) {
    // Get the home directory
    let home = std::env::home_dir().unwrap_or(PathBuf::from("/tmp"));
    let log_file_path = home.join("mom.log");
    append_to_log_file(log_file_path, log_message)
}

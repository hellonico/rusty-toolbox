use std::fs::OpenOptions;
use std::path::PathBuf;
use std::process::Command;
use chrono::Local;
use std::io::Write; // Import the Write trait
use dirs::home_dir;

fn path_for(binary:&str) -> String {
    // TODO: allow to read from a config file
    #[cfg(target_os = "macos")]
    let path_for_ffmpeg_binaries = "/opt/homebrew/bin/";

    // use the regular path
    #[cfg(not(target_os = "macos"))]
    let path_for_ffmpeg_binaries = "";

    format!("{}{}", path_for_ffmpeg_binaries, binary)
}

pub fn get_base_ffmpeg_command(args:String) -> Command {
    let path = path_for("ffmpeg");
    append_to_home_log(format!("{:?}", &*args.to_string()));
    let args_vec: Vec<&str> = args.split(' ').collect();
    let mut cmd = Command::new(path);
    cmd.args(args_vec);
    cmd
}

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
    let home = home_dir().expect("Failed to find home directory");
    let log_file_path = home.join("mom.log");
    append_to_log_file(log_file_path, log_message)
}

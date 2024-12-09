use std::env;
use std::path::PathBuf;
use std::process::{Command, Output};
use std::time::Duration;
use regex::{Captures, Regex};
use crate::log::append_to_home_log;
//TODO allow

#[cfg(target_os = "macos")]
fn find_ff_binary(binary: &str) -> String {
    // When in the bundled mac APP\
    let current_exe = env::current_exe().unwrap();
    append_to_home_log(current_exe.display().to_string());
    let bin = format!("{}/Resources/resources/{}", current_exe.parent().unwrap().parent().unwrap().to_string_lossy(), binary);
    append_to_home_log(format!("Try {}", bin));
    if PathBuf::from(bin.clone()).exists() {
        bin
    } else {
        // revert to homebrew location
        format!("/opt/homebrew/bin/{}", binary).to_string()
    }
}

#[cfg(not(target_os = "macos"))]
fn find_ff_binary(binary: &str) -> String {
    let bin = format!("{:}", binary);
}

pub fn ffmpeg_binary() -> String {
    find_ff_binary("ffmpeg")
}
pub fn path_for(binary:&str) -> String {
    find_ff_binary(binary)
}

pub fn get_base_ffmpeg_command(args:String) -> Command {
    let path = path_for("ffmpeg");
    append_to_home_log(format!("{}", path));
    append_to_home_log(format!("{}", &*args.to_string()));
    let args_vec: Vec<&str> = args.split(' ').collect();
    let mut cmd = Command::new(path);
    cmd.args(args_vec);
    cmd
}

pub fn clean_up_parameters(params: String) -> Vec<String> {
    params
        .split(' ') // Split the string by spaces
        .map(str::trim) // Trim each item
        .filter(|&s| !s.is_empty()) // Filter out empty strings
        .map(str::to_string) // Trim each item
        .collect() // Collect into a Vec
}


pub fn format_duration(duration: Duration) -> String {
    let total_seconds = duration.as_secs();
    let hours = total_seconds / 3600;
    let minutes = (total_seconds % 3600) / 60;
    let seconds = total_seconds % 60;

    format!("{:02}:{:02}:{:02}", hours, minutes, seconds)
}


pub fn check_ffmpeg() -> Result<String, ()> {
    // Try to execute `ffmpeg -version` to check if ffmpeg is installed
    let output = Command::new(path_for("ffmpeg"))
        .arg("-version")
        .output();

    match output {
        Ok(Output { stdout, .. }) => {
            let version = String::from_utf8_lossy(&stdout);
            let version_line = version.lines().next().unwrap_or("").to_string();
            append_to_home_log(format!("{}", version_line));
            //let re  =  Regex::new(r"version (.*)+ ").unwrap();
            let re = Regex::new(r"version ([.\-a-zA-Z0-9]*)").unwrap();
            if let Some(v) = re.captures(version_line.clone().trim())  {
                return Ok(format!("ffmpeg {}", v.get(1).unwrap().as_str().to_string()));
            }
            Ok("Invalid ffmpeg version".to_string())
        }
        Err(_) => Err(()),
    }
}
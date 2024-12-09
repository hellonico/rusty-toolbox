use std::env;
use std::path::PathBuf;
use std::process::Command;
use crate::log::append_to_home_log;
//TODO combined the functions below !

#[cfg(target_os = "macos")]
pub fn ffmpeg_binary() -> String {
    // When in the bundled mac APP\
    let current_exe = env::current_exe().unwrap();
    append_to_home_log(current_exe.display().to_string());
    let bin = format!("{:}/Resources/resources/ffmpeg", current_exe.parent().unwrap().parent().unwrap().to_string_lossy());
    if PathBuf::from(bin.clone()).exists() {
        bin
    } else {
        // revert to homebrew location
        "/opt/homebrew/bin/ffmpeg".to_string()
    }
}

#[cfg(not(target_os = "macos"))]
pub fn ffmpeg_binary() -> String {
    "ffmpeg".to_string()
}

pub fn path_for(binary:&str) -> String {
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

pub fn clean_up_parameters(params: String) -> Vec<String> {
    params
        .split(' ') // Split the string by spaces
        .map(str::trim) // Trim each item
        .filter(|&s| !s.is_empty()) // Filter out empty strings
        .map(str::to_string) // Trim each item
        .collect() // Collect into a Vec
}
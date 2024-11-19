use std::{fs, io};
use std::path::PathBuf;
use std::process::{Command, ExitStatus};
use serde::{Deserialize, Serialize};

// Struct for configuration
#[derive(Serialize, Deserialize, Default)]
pub struct AppConfig {
    pub input_folder: String,
    pub output_folder: String,
    pub video_format: String,
    pub audio_format: String,
    pub file_extension: String,
    pub delete_original: bool,
    pub skip_if_exists: bool,
    pub encoding: String,
}

impl AppConfig {
    pub fn save(&self, path: &str) -> Result<(), io::Error> {
        let yaml = serde_yaml::to_string(self).unwrap();
        fs::write(path, yaml)
    }

    pub fn load(path: &str) -> Result<Self, io::Error> {
        let yaml = fs::read_to_string(path)?;
        serde_yaml::from_str(&yaml).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
    }
}


pub struct FileStat {
    pub input_file: String,
    pub input_size: f64,
    pub output_size: Option<f64>,
    pub reduction: Option<f64>,
    pub elapsed_time: Option<f64>, // Time in seconds
}


pub fn encode_video(file: PathBuf, encoding: &String, output_file: &String, audio: &String) -> io::Result<ExitStatus> {
    let status = Command::new("ffmpeg")
        .arg("-hwaccel")
        .arg("auto")
        .arg("-i")
        .arg(&file)
        .arg("-c:v")
        .arg(&encoding)
        .arg("-preset")
        .arg("fast")
        .arg("-c:a")
        .arg(audio)
        .arg(&output_file)
        .status();
    status
}
use std::process::Command;
use crate::utils::ffmpeg_binary;

pub struct DeviceLister {
    audio_devices: Vec<String>,
    video_devices: Vec<String>,
}

impl DeviceLister {
    /// Creates a new `DeviceLister` by running the ffmpeg command and parsing the output
    pub fn new() -> Self {
        // Run the ffmpeg command and capture its output
        let output = Command::new(ffmpeg_binary())
            .args(&["-hide_banner", "-list_devices", "true", "-f", "avfoundation", "-i", "dummy"])
            .output()
            .expect("Failed to execute ffmpeg command");

        // Convert the output to a String
        let output_str = String::from_utf8_lossy(&output.stderr);

        // Parse devices from the output
        let (audio_devices, video_devices) = Self::parse_devices(&output_str);

        Self {
            audio_devices,
            video_devices,
        }
    }

    /// Returns the list of audio devices
    pub fn get_audio_devices(&self) -> Vec<String> {
        self.audio_devices.clone()
    }

    /// Returns the list of video devices
    pub fn get_video_devices(&self) -> Vec<String> {
        self.video_devices.clone()
    }

    /// Parses the ffmpeg output to extract audio and video devices
    fn parse_devices(output: &str) -> (Vec<String>, Vec<String>) {
        let mut audio_devices = Vec::new();
        let mut video_devices = Vec::new();
        let mut in_video_section = false;
        let mut in_audio_section = false;

        for line in output.lines() {
            if line.contains("AVFoundation video devices:") {
                in_video_section = true;
                in_audio_section = false;
            } else if line.contains("AVFoundation audio devices:") {
                in_audio_section = true;
                in_video_section = false;
            } else if line.contains("[") && line.contains("]") {
                if in_video_section {
                    if let Some(device) = Self::extract_device_name(line) {
                        video_devices.push(device);
                    }
                } else if in_audio_section {
                    if let Some(device) = Self::extract_device_name(line) {
                        audio_devices.push(device);
                    }
                }
            }
        }

        (audio_devices, video_devices)
    }

    /// Extracts the device name from a line like "[0] FaceTime HD Camera"
    fn extract_device_name(line: &str) -> Option<String> {
        // Ensure the line contains a valid format like "[0] Device Name"
        if let Some(start) = line.find(']') {
            let device_name = line[start + 1..].trim(); // Get the part after the ']'
            if !device_name.is_empty() && !device_name.starts_with("Error") {
                return Some(device_name.to_string());
            }
        }
        None
    }
}

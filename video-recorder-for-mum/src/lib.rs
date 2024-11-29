use chrono::Local;
use std::io::{BufRead, BufReader, Write};
#[cfg(target_os = "windows")]
use std::os::windows::process::CommandExt;
use std::process::{Child, Command, Stdio};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use std::{env, io, thread};
use std::fs::File;
#[cfg(target_os = "windows")]
use winapi::um::winbase::DETACHED_PROCESS;

pub struct RecordingApp {
    pub recording_process: Arc<Mutex<Option<Child>>>,
    pub last_output_file: Arc<Mutex<Option<String>>>,
    pub start_time: Arc<Mutex<Option<Instant>>>, // Added to store start time
}

impl Default for RecordingApp {
    fn default() -> Self {
        Self {
            recording_process: Arc::new(Mutex::new(None)),
            last_output_file: Arc::new(Mutex::new(None)),
            start_time: Arc::new(Mutex::new(None)), // Initialize start time
        }
    }
}

impl RecordingApp {
    pub fn open_containing_folder(output_file: &String) {
        let folder_path = env::current_dir().unwrap().display().to_string();

        #[cfg(target_os = "macos")]
        Command::new("open")
            .arg(folder_path.clone())
            .spawn()
            .expect("Failed to open folder");

        eprintln!("Opening: {} {}", output_file, folder_path);
        #[cfg(target_os = "windows")]
        Command::new("explorer")
            .arg(folder_path)
            .spawn()
            .expect("Failed to open folder");

        #[cfg(target_os = "linux")]
        Command::new("xdg-open")
            .arg(folder_path)
            .spawn()
            .expect("Failed to open folder");
    }
    pub fn start_recording(&self) {
        let process_lock = self.recording_process.clone();
        let start_time_lock = self.start_time.clone();

        let now = Local::now();
        let output_file = format!("screen_recording_{}.mp4", now.format("%Y-%m-%d_%H-%M-%S"));

        // Store the output file name
        *self.last_output_file.lock().unwrap() = Some(output_file.clone());

        // Store the start time
        *start_time_lock.lock().unwrap() = Some(Instant::now());

        #[cfg(target_os = "windows")]
        let default_device = self.get_audio_inputs().unwrap()[0].clone();

        thread::spawn(move || {
            #[cfg(target_os = "macos")]
            let ffmpeg_cmd =
                Command::new("bash").arg("-c").arg(format!("ffmpeg -f avfoundation -i 1.0 -framerate 30 {}", &output_file))
                // Command::new("ffmpeg")
                //     .arg("-f")
                //     .arg("avfoundation")
                //     .arg("-i")
                //     .arg("1:0") // Adjust devices for your system
                //     .arg("-framerate")
                //     .arg("30")
                //     .arg(&output_file) // Use dynamically generated filename
                    .stdin(Stdio::piped()) // Open stdin for sending commands
                    .stdout(Stdio::piped()) // Pipe stdout to capture logs
                    .spawn()
                    .expect("Failed to start ffmpeg");

            #[cfg(target_os = "windows")]
            let ffmpeg_cmd =
                Command::new("ffmpeg")
                    .arg("-f")
                    .arg("gdigrab")
                    .arg("-i")
                    .arg("desktop") // Adjust devices for your system
                    .arg("-f")
                    .arg("dshow")
                    .arg("-i")
                    .arg(format!("audio={}", default_device))
                    .arg("-framerate")
                    .arg("25")
                    .arg(&output_file) // Use dynamically generated filename
                    .creation_flags(winapi::um::winbase::DETACHED_PROCESS)
                    .stdin(Stdio::piped()) // Open stdin for sending commands
                    .spawn()
                    .expect("Failed to start ffmpeg");
            // Lock the process and store it

            *process_lock.lock().unwrap() = Some(ffmpeg_cmd);
        });
    }

    pub fn stop_recording(&self) {
        let mut process_lock = self.recording_process.lock().unwrap();
        if let Some(mut ffmpeg_process) = process_lock.take() {
            if let Some(stdin) = ffmpeg_process.stdin.as_mut() {
                // Send the 'q' command to quit ffmpeg gracefully
                stdin.write_all(b"q\n").expect("Failed to send 'q' to ffmpeg");
                println!("Sent 'q' to ffmpeg to stop recording");
            }

            // Optionally wait for the process to finish
            let _ = ffmpeg_process.wait().expect("Failed to wait on ffmpeg");
            println!("FFmpeg process has stopped");
        }

        // Clear the start time
        *self.start_time.lock().unwrap() = None;
    }

    /// Returns the elapsed time since the recording started.
    pub fn elapsed_time(&self) -> Option<String> {
        let start_time_lock = self.start_time.lock().unwrap();
        if let Some(start_time) = *start_time_lock {
            Some(Self::format_duration(start_time.elapsed()))
        } else {
            None // Return None if no recording is active
        }
    }


    fn format_duration(duration: Duration) -> String {
        let total_seconds = duration.as_secs();
        let hours = total_seconds / 3600;
        let minutes = (total_seconds % 3600) / 60;
        let seconds = total_seconds % 60;

        format!("{:02}:{:02}:{:02}", hours, minutes, seconds)
    }

    /// Retrieves the list of available audio inputs via FFmpeg.
    pub fn get_audio_inputs(&self) -> io::Result<Vec<String>> {
        // Execute FFmpeg to list devices
        let ffmpeg_output = Command::new("ffmpeg")
            .args(&["-f", "dshow", "-list_devices", "true", "-i", "dummy"])
            .stderr(Stdio::piped()) // FFmpeg writes device info to stderr
            .output()?;

        let output = String::from_utf8_lossy(&ffmpeg_output.stderr);
        let mut audio_inputs = Vec::new();

        // Parse FFmpeg's output for audio devices
        for line in output.lines() {
            if line.contains("[dshow") && line.contains("(audio)") {
                if let Some(start) = line.find('"') {
                    if let Some(end) = line[start + 1..].find('"') {
                        let device_name = line[start + 1..start + 1 + end].to_string();
                        audio_inputs.push(device_name);
                    }
                }
            }
        }

        Ok(audio_inputs)
    }

}

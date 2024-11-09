use std::io::Write;
use std::process::{Child, Command, Stdio};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};
use chrono::Local;

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
    pub fn start_recording(&self) {
        let process_lock = self.recording_process.clone();
        let start_time_lock = self.start_time.clone();

        let now = Local::now();
        let output_file = format!("screen_recording_{}.mp4", now.format("%Y-%m-%d_%H-%M-%S"));

        // Store the output file name
        *self.last_output_file.lock().unwrap() = Some(output_file.clone());

        // Store the start time
        *start_time_lock.lock().unwrap() = Some(Instant::now());

        thread::spawn(move || {
            #[cfg(target_os = "macos")]
            let ffmpeg_cmd =
                Command::new("ffmpeg")
                    .arg("-f")
                    .arg("avfoundation")
                    .arg("-i")
                    .arg("1:0") // Adjust devices for your system
                    .arg("-framerate")
                    .arg("25")
                    .arg(&output_file) // Use dynamically generated filename
                    .stdin(Stdio::piped()) // Open stdin for sending commands
                    .spawn()
                    .expect("Failed to start ffmpeg");

            #[cfg(target_os = "windows")]
            let ffmpeg_cmd =
                Command::new("ffmpeg")
                    .arg("-f")
                    .arg("gdigrab")
                    .arg("-i")
                    .arg("desktop") // Adjust devices for your system
                    .arg("dshow")
                    .arg("-i")
                    .arg("audio=\"Microphone\"")
                    .arg("-framerate")
                    .arg("25")
                    .arg(&output_file) // Use dynamically generated filename
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
    pub fn elapsed_time(&self) -> Option<Duration> {
        let start_time_lock = self.start_time.lock().unwrap();
        if let Some(start_time) = *start_time_lock {
            Some(start_time.elapsed())
        } else {
            None // Return None if no recording is active
        }
    }
}

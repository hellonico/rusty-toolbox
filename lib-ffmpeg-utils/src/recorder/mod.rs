use chrono::Local;
use std::io::{BufRead, BufReader, Write};
#[cfg(target_os = "windows")]
use std::os::windows::process::CommandExt;
use std::process::{Child, Command, Stdio};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use std::{env, io, thread};
#[cfg(target_os = "windows")]
use winapi::um::winbase::DETACHED_PROCESS;
use crate::devices::DeviceLister;
use crate::log::append_to_home_log;
use crate::utils::{format_duration, get_base_ffmpeg_command};

pub struct RecordingApp {
    pub recording_process: Arc<Mutex<Option<Child>>>,
    pub last_output_file: Arc<Mutex<Option<String>>>,
    pub start_time: Arc<Mutex<Option<Instant>>>, // Added to store start time
    device_lister: DeviceLister,
}

impl Default for RecordingApp {
    fn default() -> Self {
        Self {
            recording_process: Arc::new(Mutex::new(None)),
            last_output_file: Arc::new(Mutex::new(None)),
            start_time: Arc::new(Mutex::new(None)), // Initialize start time
            device_lister: DeviceLister::new(),
        }
    }
}

impl RecordingApp {

    pub fn start_recording(&self) {
        let process_lock = self.recording_process.clone();
        let start_time_lock = self.start_time.clone();

        let now = Local::now();
        let output_file = format!("screen_recording_{}.mkv", now.format("%Y-%m-%d_%H-%M-%S"));

        // Store the output file name
        *self.last_output_file.lock().unwrap() = Some(output_file.clone());

        // Store the start time
        *start_time_lock.lock().unwrap() = Some(Instant::now());

        #[cfg(target_os = "windows")]
        let default_device = self.get_audio_inputs().unwrap()[0].clone();

        thread::spawn(move || {
            #[cfg(target_os = "linux")]
            let mut ffmpeg_base_cmd = get_base_ffmpeg_command(format!("-video_size 1920x1080 -framerate 30 -f x11grab -i :0.0 -c:v libx264rgb -crf 0 -preset ultrafast -color_range 2 output.mp4 {}", &output_file));

            #[cfg(target_os = "macos")]
            let mut ffmpeg_base_cmd =
                get_base_ffmpeg_command(format!("-y -f avfoundation -i 1:1 -vcodec libx264 -preset veryfast -b:a 196k -s 960x540 -r 20 /Users/niko/Desktop/{}", &output_file));

            #[cfg(not(target_os = "windows"))]
            let ffmpeg_cmd= ffmpeg_base_cmd
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

            append_to_home_log(format!("PID: {}", ffmpeg_cmd.id()));
            // Lock the process and store it
            *process_lock.lock().unwrap() = Some(ffmpeg_cmd);
        });
    }

    pub fn stop_recording(&self) {
        append_to_home_log("Before unwrap lock".to_string());
        let mut process_lock = self.recording_process.lock().unwrap();
        append_to_home_log("Before process lock".to_string());
        if let Some(mut ffmpeg_process) = process_lock.take() {
            append_to_home_log("After process lock".to_string());
            if let Some(stdin) = ffmpeg_process.stdin.as_mut() {
                append_to_home_log("Send 'q' to ffmpeg to stop recording".to_string());
                // Send the 'q' command to quit ffmpeg gracefully
                match stdin.write_all(b"q\n") {
                    Ok(v) => {
                        append_to_home_log("OK Send 'q' to ffmpeg to stop recording".to_string());
                        ()
                    },
                    Err(_0) => {
                        append_to_home_log(format!("FAIL Send 'q' to ffmpeg to stop recording {:}", _0).to_string());
                    },
                } //.expect("Failed to send 'q' to ffmpeg");
            }
            append_to_home_log("Waiting for ffmpeg to finish".to_string());
            // Optionally wait for the process to finish
            let _ = ffmpeg_process.wait().expect("Failed to wait on ffmpeg");
            append_to_home_log("FFmpeg process has stopped".to_string());
        }
        append_to_home_log("Could not take lock".to_string());
        // Clear the start time
        *self.start_time.lock().unwrap() = None;
    }

    /// Returns the elapsed time since the recording started.
    pub fn elapsed_time(&self) -> Option<String> {
        let start_time_lock = self.start_time.lock().unwrap();
        if let Some(start_time) = *start_time_lock {
            Some(format_duration(start_time.elapsed()))
        } else {
            None // Return None if no recording is active
        }
    }

}

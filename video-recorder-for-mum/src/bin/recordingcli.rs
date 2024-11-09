use std::process::{Command, Stdio, Child};
use std::sync::{Arc, Mutex};
use std::thread;
use std::io::Write;

struct RecordingApp {
    recording_process: Arc<Mutex<Option<Child>>>,
}

impl Default for RecordingApp {
    fn default() -> Self {
        Self {
            recording_process: Arc::new(Mutex::new(None)),
        }
    }
}

impl RecordingApp {
    fn start_recording(&self) {
        let process_lock = self.recording_process.clone();

        thread::spawn(move || {
            let output_file = "screen_recording.mp4"; // Adjust the filename generation logic if needed
            let ffmpeg_cmd = Command::new("ffmpeg")
                .arg("-f")
                .arg("avfoundation")
                .arg("-i")
                .arg("1:0") // Adjust devices for your system
                .arg("-framerate")
                .arg("25")
                .arg(output_file)
                .stdin(Stdio::piped()) // Open stdin for sending commands
                .spawn()
                .expect("Failed to start ffmpeg");

            // Lock the process and store it
            *process_lock.lock().unwrap() = Some(ffmpeg_cmd);
        });
    }

    fn stop_recording(&self) {
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
    }
}

fn main() {
    let app = RecordingApp::default();

    // Start recording
    app.start_recording();

    // Stop recording after 10 seconds
    thread::sleep(std::time::Duration::from_secs(10));
    app.stop_recording();
}

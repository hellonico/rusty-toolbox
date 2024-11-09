use std::thread;
use video_recorder_for_mum::RecordingApp;
fn main() {
    let app = RecordingApp::default();

    // Start recording
    app.start_recording();

    // Stop recording after 10 seconds
    thread::sleep(std::time::Duration::from_secs(10));
    app.stop_recording();
}

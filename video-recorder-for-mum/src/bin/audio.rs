use std::process::{Command, Stdio};
use std::io;
use lib_ffmpeg_utils::recorder::RecordingApp;
use video_recorder_for_mum::RecordingApp;

fn main() -> io::Result<()> {
    let rca =  RecordingApp::default();
    // Get the list of audio devices
    let audio_inputs = rca.get_audio_inputs()?;

    if audio_inputs.is_empty() {
        eprintln!("No audio inputs found.");
        return Ok(());
    }

    // Find the default microphone or choose the first one
    let default_microphone = &audio_inputs[0];
    println!("Default Microphone: {}", default_microphone);

    // Record from the default microphone
    println!("Starting audio recording...");
    let record_status = Command::new("ffmpeg")
        .args(&[
            "-f", "dshow",
            "-i", &format!("audio=\"{}\"", default_microphone),
            "-t", "10", // Record for 10 seconds
            "output.wav", // Output file
        ])
        .status()?;

    if record_status.success() {
        println!("Recording saved to output.wav");
    } else {
        eprintln!("Failed to record audio.");
    }

    Ok(())
}

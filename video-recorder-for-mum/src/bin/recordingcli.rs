use std::{thread, time::Duration};
use clap::{Arg, Command};
use chrono::{Local, NaiveTime};
use egui::debug_text::print;
use video_recorder_for_mum::RecordingApp;
use notify_rust::Notification;

fn main() {
    let matches = Command::new("Video Recorder")
        .version("1.0")
        .about("Records video starting at a given time for a specified duration")
        .arg(
            Arg::new("start_time")
                .short('s')
                .long("start-time")
                .help("The start time for recording in HH:MM:SS format. If not provided, recording starts immediately."),
        )
        .arg(
            Arg::new("duration")
                .short('d')
                .long("duration")
                .default_value("60.0")
                .help("The duration of the recording in seconds (floating-point)"),
        )
        .arg(
            Arg::new("notification")
                .short('n')
                .long("notification")
                .default_value("true")
                .help("Whether to send a notification when recording finishes (true or false)."),
        )
        .get_matches();

    // Parse the duration
    let duration: f64 = matches
        .get_one::<String>("duration")
        .unwrap()
        .parse()
        .expect("Invalid duration value, must be a number");

    // Parse the notification option
    let send_notification: bool = matches
        .get_one::<String>("notification")
        .unwrap()
        .parse()
        .expect("Invalid notification value, must be true or false");

    let app = RecordingApp::default();

    // Handle start time
    let delay = if let Some(start_time_str) = matches.get_one::<String>("start_time") {
        let start_time = NaiveTime::parse_from_str(start_time_str, "%H:%M:%S")
            .expect("Invalid time format for start_time");
        let now = Local::now();
        let current_time = now.time();

        // Calculate delay
        if start_time > current_time {
            (start_time - current_time).num_seconds() as u64
        } else {
            0 // Start immediately if the time is in the past
        }
    } else {
        0 // Start immediately if no start_time is provided
    };

    if delay > 0 {
        println!(
            "Waiting {} seconds to start recording for a duration of {} seconds...",
            delay, duration
        );
        thread::sleep(Duration::from_secs(delay));
    }

    // Start recording
    println!("Starting recording...");
    app.start_recording();

    // Stop recording after the specified duration
    thread::sleep(Duration::from_secs_f64(duration));
    println!("Stopping recording...");
    app.stop_recording();

    // Send a notification if enabled
    if send_notification {
        println!("Sending notification...");
        Notification::new()
            .summary("Recording Finished")
            .body(&format!(
                "Your recording lasted {:.1} seconds.",
                duration
            ))
            .show()
            .expect("Failed to send notification");
    }
    // Wait for a while after notification to ensure it's visible long enough
    thread::sleep(Duration::from_secs(1));  // You can adjust this duration
}

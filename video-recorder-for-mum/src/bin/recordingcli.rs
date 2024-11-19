use std::{thread, time::Duration};
use chrono::{Local, NaiveTime};
use clap::{Arg, Command};
use video_recorder_for_mum::RecordingApp;

fn main() {
    let matches = Command::new("Video Recorder")
        .version("1.0")
        .about("Records video starting at a given time for a specified duration")
        .arg(
            Arg::new("start_time")
                .short('s')
                .long("start-time")
                // .takes_value(true)
                .required(true)
                .help("The start time for recording in HH:MM:SS format"),
        )
        .arg(
            Arg::new("duration")
                .short('d')
                .long("duration")
                // .takes_value(true)
                .default_value("60")
                .help("The duration of the recording in seconds"),
        )
        .get_matches();

    let start_time = matches.get_one::<String>("start_time").unwrap();
    let duration: f64 = matches
        .get_one::<String>("duration")
        .unwrap() // Always has a value because of `default_value`
        .parse()
        .expect("Invalid duration value, must be a number");

    println!("Recording programmed. Starting at time {}, Duration {} seconds", start_time, duration);

    let app = RecordingApp::default();

    // Parse the start time
    let start_time = NaiveTime::parse_from_str(start_time, "%H:%M:%S").expect("Invalid time format");

    // Calculate the delay until the start time
    let now = Local::now();
    let current_time = now.time();
    let delay = if start_time > current_time {
        start_time - current_time
    } else {
        // If the start time is in the past, schedule for the next day
        start_time + chrono::Duration::hours(24) - current_time
    };

    println!(
        "Waiting {} to start recording at {} for a duration of {} seconds...",
        delay, start_time, duration
    );

    // Sleep until the start time
    thread::sleep(Duration::from_secs(delay.num_seconds() as u64));

    // Start recording
    println!("Starting recording...");
    app.start_recording();

    // Stop recording after the specified duration
    thread::sleep(Duration::from_secs(duration as u64));
    println!("Stopping recording...");
    app.stop_recording();
}

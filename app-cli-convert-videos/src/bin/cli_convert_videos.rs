use clap::{Arg, Command as ClapCommand};
use indicatif::{ProgressBar, ProgressStyle};
use std::fs;
use std::fs::OpenOptions;
use std::io::Read;
use std::process::Command;

fn main() {
    let matches = ClapCommand::new("Video Encoder")
        .version("1.0")
        .about("Encodes video files using ffmpeg")
        .arg(
            Arg::new("input")
                .short('i')
                .long("input")
                .value_name("INPUT_FOLDER")
                .help("Input folder containing videos")
                .required(true),
        )
        .arg(
            Arg::new("output")
                .short('o')
                .long("output")
                .value_name("OUTPUT_FOLDER")
                .help("Output folder for encoded videos")
                .required(true),
        )
        .arg(
            Arg::new("delete")
                .short('d')
                .long("delete")
                .help("Delete original files after encoding"),
        )
        .arg(
            Arg::new("format")
                .short('f')
                .long("format")
                .value_name("OUTPUT_FORMAT")
                .help("Output video format (mp4 or mpeg)")
                .default_value("mp4"),
        )
        .arg(
            Arg::new("audio")
                .short('a')
                .long("audio")
                .value_name("AUDIO_FORMAT")
                .help("Audio format (aac or mp3)")
                .default_value("aac"),
        )
        .arg(
            Arg::new("extension")
                .short('e')
                .long("extension")
                .value_name("EXTENSION")
                .help("Input file extension (e.g., mov)")
                .default_value("mov"),
        )
        .get_matches();

    let input = matches.get_one::<String>("input").unwrap();
    let output = matches.get_one::<String>("output").unwrap();
    let delete = matches.contains_id("delete");
    let video_format = matches.get_one::<String>("format").unwrap();
    let audio_format = matches.get_one::<String>("audio").unwrap();
    let extension = matches.get_one::<String>("extension").unwrap();

    // Ensure output directory exists
    fs::create_dir_all(output).expect("Failed to create output directory");

    // Get a list of input files
    let files: Vec<_> = fs::read_dir(input)
        .expect("Failed to read input directory")
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.path())
        .filter(|path| path.extension().map_or(false, |ext| extension.eq_ignore_ascii_case(ext.to_str().unwrap())))
        .collect();

    let pb = ProgressBar::new(files.len() as u64);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("{spinner:.green} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {pos}/{len} {msg}").expect("REASON")
            .progress_chars("#>-"),
    );

    // Create or append to the log file
    let log_file_path = "ffmpeg.log";
    let mut log_file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(log_file_path)
        .expect("Failed to open or create log file");

    for file in files {
        let filename = file.file_name().unwrap().to_string_lossy();
        let output_file = format!(
            "{}/{}.{}",
            output,
            file.file_stem().unwrap().to_string_lossy(),
            video_format
        );

        pb.set_message(format!("Encoding: {}", filename));
        let status = Command::new("ffmpeg")
            .args([
                "-y", // Force overwrite
                "-hwaccel", "auto",
                "-i", file.to_str().unwrap(),
                "-c:v", "libx265",
                "-crf", "26",
                "-preset", "fast",
                "-c:a", audio_format,
                &output_file,
            ])
            .stdout(std::process::Stdio::null()) // Suppress stdout
            .stderr(std::process::Stdio::piped()) // Capture stderr for logging
            .spawn()
            .and_then(|mut child| {
                // Capture stderr and write it to the log file
                if let Some(stderr) = child.stderr.take() {
                    std::io::copy(&mut stderr.take(1024 * 1024), &mut log_file)
                        .expect("Failed to write ffmpeg output to log file");
                }
                child.wait()
            })
            .expect("Failed to execute ffmpeg");

        if !status.success() {
            eprintln!("Failed to encode file: {}", filename);
            continue;
        }

        if delete {
            fs::remove_file(&file).expect("Failed to delete original file");
        }

        pb.inc(1);
    }

    pb.finish_with_message("Encoding completed");
    println!("FFmpeg logs have been written to {}", log_file_path);
}

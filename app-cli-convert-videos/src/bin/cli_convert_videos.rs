use app_cli_convert_videos::encode_video;
use clap::{Arg, Command as ClapCommand};
use indicatif::{ProgressBar, ProgressStyle};
use lib_egui_utils::{generate_output_path, list_files_from_dir2, SortBy};
use std::fs;
use std::path::PathBuf;

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
    let files: Vec<_> = list_files_from_dir2(input, extension, SortBy::LastUpdated, true);

    let pb = ProgressBar::new(files.len() as u64);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("{spinner:.green} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {pos}/{len} {msg}").expect("REASON")
            .progress_chars("#>-"),
    );

    for filename in files {
        let file = PathBuf::from(filename.clone());
        let output_file = generate_output_path(&filename.clone(), output.into(), video_format.into());

        pb.set_message(format!("Encoding: {}", filename));

        let status = encode_video(file.clone(), &String::from("libx265"), &output_file, audio_format);

        if let Err(e) = status {
            eprintln!("Failed to run FFmpeg for {:?}: {}", file, e);
            continue;
        }

        if delete {
            fs::remove_file(&file).expect("Failed to delete original file");
        }

        pb.inc(1);
    }

    pb.finish_with_message("Encoding completed");
}

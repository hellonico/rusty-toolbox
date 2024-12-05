use std::sync::{Arc, Mutex};
use std::{env, thread};
use std::ascii::AsciiExt;
use std::thread::sleep;
use eframe::{egui};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use chrono::{Local, NaiveTime};
use eframe::egui::ViewportCommand;

fn main() -> Result<(), eframe::Error> {
    append_to_home_log(env::current_exe().unwrap().to_string_lossy().to_string());
    append_to_home_log(ffmpeg_binary());
    let options =
        my_default_options(800.0, 500.0, include_bytes!("../../icon.png"));
    eframe::run_native("Open URL Scheduler", options, Box::new(|_cc| Ok(Box::new(MyApp::default()))))
}

#[cfg(target_os = "macos")]
fn ffmpeg_binary() -> String {
    let bin = format!("{:}/Resources/resources/ffmpeg", env::current_exe().unwrap().parent().unwrap().parent().unwrap().to_string_lossy());
    if PathBuf::from(bin.clone()).exists() {
        bin
    } else {
        "/opt/homebrew/bin/ffmpeg".to_string()
    }
}
#[cfg(not(target_os = "macos"))]
fn ffmpeg_binary() -> String {
    "ffmpeg".to_string()
}

#[derive(Serialize, Deserialize)]
struct Config {
    start_time: String,
    url: String,
    command: String,
    show_parameter: bool,
}


struct MyApp {
    url: String,
    start_time: String,
    waiting: Arc<Mutex<bool>>,
    wait_duration: Option<Duration>,
    // remaining_time: Option<std::time::Duration>,
    remaining_time: Arc<Mutex<Option<Duration>>>, // Shared state for remaining time
    // extra_command: String,
    command: String,
    show_parameter: bool,
}

impl MyApp {

    fn set_record_time(& mut self, duration: Duration) {
        let reg1 = r"-t \d{2}:\d{2}:\d{2}";
        let reg2 = r"-y ";
        let new1 = format!("-t {}", format_duration(duration));
        self.replace_or_insert(reg1, reg2, new1);
    }

    fn set_codec(& mut self, preset: String) {
        self.replace_or_insert(r"-vcodec \w+", r"(-i \S+)", format!("-vcodec {} ", preset));
    }

    fn set_preset(& mut self, preset: String) {
        let reg1 = r"-preset \w+";
        let reg2 = r"-vcodec \w+";
        let new1 = format!("-preset {}", preset);

        self.replace_or_insert(reg1, reg2, new1);
    }

    fn set_tune(& mut self, preset: String) {
        let reg1 = r"-tune \w+";
        let reg2 = r"-preset \w+";
        let new1 = format!("-tune {}", preset);

        self.replace_or_insert(reg1, reg2, new1);
    }

    fn set_screen(& mut self, preset: String) {
        let reg1 = r"-s \w+";
        let reg2 = r"(-b:a \S+)";
        let new1 = format!("-s {}", preset);

        self.replace_or_insert(reg1, reg2, new1);
    }


    fn replace_or_insert(&mut self, reg1: &str, reg2: &str, new1: String) {
        // Regular expressions
        let preset_re = Regex::new(reg1).unwrap();
        let vcodec_re = Regex::new(reg2).unwrap();

        // Check if '-preset' exists
        if preset_re.is_match(&*self.command) {
            self.command = preset_re.replace(&*self.command, new1).to_string();
        } else if let Some(pos) = vcodec_re.find(&*self.command).map(|m| m.end()) {
            // Insert '-preset ultrafast' after '-vcodec <value>'
            let (before, after) = self.command.split_at(pos);
            self.command = format!("{} {}{}", before, new1, after);
        }
    }

}

fn format_duration(duration: Duration) -> String {
    // Get total seconds from the Duration
    let total_seconds = duration.as_secs();

    // Calculate hours, minutes, and seconds
    let hours = total_seconds / 3600;
    let minutes = (total_seconds % 3600) / 60;
    let seconds = total_seconds % 60;

    // Return the formatted string
    format!("{:02}:{:02}:{:02}", hours, minutes, seconds)
}

impl Default for MyApp {
    fn default() -> Self {
        let start_time = Local::now() + chrono::Duration::seconds(5);
        let formatted_time = start_time.format("%H:%M:%S").to_string();

        Self {
            //url: String::new(),
            url: "https://www.youtube.com/embed/PaSQGeNJx9Q??&autoplay=1".to_string(),
            start_time: formatted_time,
            waiting: Arc::new(Mutex::new(false)),
            wait_duration: None,
            remaining_time: Arc::new(Mutex::new(None)),
            //extra_command: "/Users/niko/Downloads/ffmpeg -y -f avfoundation -i 1:0 -vsync vfr -b:a 196k -t 00:00:10 output.mp4".to_string(),
            command: "-y -f avfoundation -i 1:0 -vcodec libx264 -preset veryfast -b:a 196k -s 960x540 -t 00:00:10 /Users/niko/Desktop/output.mkv".to_string(), // Initialize with an empty command
            show_parameter: false,
        }
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {

            ui.menu_button("Night Menu", |ui| {
                if ui.checkbox(&mut self.show_parameter, "Show Recording Parameters ").clicked() {
                    // self.show_parameter();
                };
                ui.menu_button("Set Time", |ui| {
                    if ui.button("Now").clicked() {
                        self.start_time = Local::now().naive_local().checked_add_signed(chrono::Duration::seconds(0)).unwrap().format("%H:%M:%S").to_string();
                    }
                    if ui.button("In 10 seconds").clicked() {
                        self.start_time = Local::now().naive_local().checked_add_signed(chrono::Duration::seconds(10)).unwrap().format("%H:%M:%S").to_string();
                    }
                    if ui.button("In 10 minutes").clicked() {
                        self.start_time = Local::now().naive_local().checked_add_signed(chrono::Duration::minutes(10)).unwrap().format("%H:%M:%S").to_string();
                    }
                    if ui.button("In 1 hour").clicked() {
                        self.start_time = Local::now().naive_local().checked_add_signed(chrono::Duration::hours(1)).unwrap().format("%H:%M:%S").to_string();
                    }
                });

                ui.menu_button("Record Settings", |ui| {
                    ui.menu_button("Duration", |ui| {
                        if ui.button("2 hours").clicked() {
                            self.set_record_time(Duration::from_secs(3600*2));
                        }
                        if ui.button("1.5 hour").clicked() {
                            self.set_record_time(Duration::from_secs(3600+1800));
                        }
                        if ui.button("1 hour").clicked() {
                            self.set_record_time(Duration::from_secs(3600));
                        }
                        if ui.button("30 minutes").clicked() {
                            self.set_record_time(Duration::from_secs(1800));
                        }
                        if ui.button("5 seconds").clicked() {
                            self.set_record_time(Duration::from_secs(5));
                        }
                    });

                    ui.menu_button("Codec", |ui| {
                        let presets = ["libx264"];

                        for &preset in &presets {
                            if ui.button(preset).clicked() {
                                self.set_codec(preset.to_string());
                            }
                        }
                    });

                    ui.menu_button("Preset", |ui| {
                        let presets = ["ultrafast", "superfast", "veryfast", "faster", "fast", "medium"];

                        for &preset in &presets {
                            if ui.button(preset).clicked() {
                                self.set_preset(preset.to_string());
                            }
                        }
                    });

                    ui.menu_button("Tune", |ui| {
                        let presets = ["film", "animation", "grain", "stillimage", "fastdecode", "zerolatency"];

                        for &preset in &presets {
                            if ui.button(preset).clicked() {
                                self.set_tune(preset.to_string());
                            }
                        }
                    });

                    ui.menu_button("Screen", |ui| {
                        let presets = ["640x480", "960x540", "1024x768"];

                        for &preset in &presets {
                            if ui.button(preset).clicked() {
                                self.set_screen(preset.to_string());
                            }
                        }
                    });

                    //-s 960x540

                });

                ui.menu_button("Config", |ui| {
                    if ui.button("Open").clicked() {
                        // Open File Dialog
                        if let Some(path) = FileDialog::new().pick_file() {
                            if let Ok(config) = load_config(path.to_str().unwrap()) {
                                self.start_time = config.start_time;
                                self.url = config.url;
                                self.command = config.command;
                                self.show_parameter = config.show_parameter;
                            }
                        }
                    }

                    if ui.button("Save").clicked() {
                        // Save File Dialog
                        if let Some(path) = FileDialog::new().save_file() {
                            let config = Config {
                                start_time: self.start_time.clone(),
                                url: self.url.clone(),
                                command: self.command.clone(),
                                show_parameter : self.show_parameter.clone(),
                            };
                            if let Err(e) = save_config(path.to_str().unwrap(), &config) {
                                eprintln!("Error saving file: {}", e);
                            }
                        };
                    };

                });
                if ui.button("Quit").clicked() {
                    exit(0);
                }
            });

            if self.waiting.lock().unwrap().clone() {
                // Display waiting time dynamically
                if let remaining = self.remaining_time.clone().lock().unwrap().unwrap_or_default() {
                    let minutes = remaining.as_secs() / 60;
                    let seconds = remaining.as_secs() % 60;
                    ui.label(format!(
                        "Waiting to open the URL... Time left: {:02}:{:02}",
                        minutes, seconds
                    ));
                } else {
                    ui.label("Waiting to open the URL...");
                }
            } else {
                ui.label("Enter the URL to open:");
                ui.text_edit_singleline(&mut self.url);

                ui.label("Enter the start time (HH:MM:ss, 24-hour format):");
                ui.text_edit_singleline(&mut self.start_time);

                ui.separator();

                if self.show_parameter {
                    ui.label("Enter recording parameters:");
                    ui.text_edit_multiline(&mut self.command);

                    ui.separator();
                }

                if ui.button("Plan Recording").clicked() {
                    if let Ok(duration) = compute_wait_duration(&self.start_time) {
                        let mut waiting_lock = self.waiting.lock().unwrap();
                        *waiting_lock = true;
                        // drop(waiting_lock);  // Explicitly drop the lock to release it

                        self.wait_duration = Some(duration);

                        // Spawn async tasks
                        let remaining_time_clone = Arc::clone(&self.remaining_time);
                        let url_clone = self.url.clone();


                        thread::spawn(move || {
                            update_remaining_time(duration, remaining_time_clone)
                        });
                        let mut waiting_lock = self.waiting.clone();
                        let command = self.command.clone();
                        let cctx = ctx.clone();
                        thread::spawn(move || {

                            open_url_after_delay(duration, url_clone);

                            // Minimize the window
                            cctx.send_viewport_cmd(ViewportCommand::Minimized(true));


                            // Set waiting to false after URL is opened
                            let mut waiting_lock = waiting_lock.lock().unwrap();
                            *waiting_lock = false;

                            thread::spawn(move || {
                                // let timestamp = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
                                // self.replace_or_insert(r"-timestamp ",r"-y", );
                                // let updated = command.replace("-y",format!("-timestamp {}", timestamp).as_str() );;
                                //
                                let date = Local::now().format("%d_%m_%Y_%H_%M");
                                let updated = command.replace("output.mkv", format!("screen_{:}.mkv", date).as_str());
                                println!("Updated : {}", updated);
                                append_to_home_log(format!("{} {}", ffmpeg_binary(), updated));
                                // TODO: extract binary
                                // Replace if ffmpeg

                                let output = Command::new(ffmpeg_binary())
                                    // .arg("-c")
                                    .args(updated.split(" ").collect::<Vec<&str>>())
                                    .output()
                                    .expect("Failed to execute command");

                                if !output.stdout.is_empty() {
                                    append_to_home_log(format!("Command output: {}", String::from_utf8_lossy(&output.stdout)));
                                }
                                if !output.stderr.is_empty() {
                                    append_to_home_log(format!("Command error: {}", String::from_utf8_lossy(&output.stderr)));
                                }
                            })
                        });
                    } else {
                        ui.label("Invalid start time format.");
                    }
                }


                // ctx.request_repaint(); // En
                ctx.request_repaint_after(Duration::from_secs(1));
            }
        });

    }
}
use std::error::Error;
use std::fmt::format;
use std::fs::File;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::process::{exit, Command};
use eframe::egui::UiKind::TopPanel;
use rfd::FileDialog;
use serde::{Deserialize, Serialize};
use lib_egui_utils::my_default_options;
use lib_ffmpeg_utils::append_to_home_log;
use regex::Regex;

fn compute_wait_duration(start_time_str: &str) -> Result<Duration, Box<dyn Error>> {
    // Parse the start time entered by the user
    let current_time = Local::now().naive_local().time();
    let start_time = NaiveTime::parse_from_str(start_time_str, "%H:%M:%S")
        .or_else(|_| NaiveTime::parse_from_str(start_time_str, "%H:%M"))
        .or_else(|_| NaiveTime::parse_from_str(start_time_str, "%H"))
        .map_err(|e| e.to_string())?;

    // Compute the time difference in seconds
    let duration = if start_time >= current_time {
        // If the start time is later today, subtract current time from start time
        start_time - current_time
    } else {
        // If the start time is earlier today, we add 24 hours to the start time
        start_time + chrono::Duration::days(1) - current_time
    };

    Ok(Duration::new(duration.num_seconds() as u64, 0))
}


fn update_remaining_time(
    wait_duration: Duration,
    remaining_time: Arc<Mutex<Option<Duration>>>,
) {
    let mut elapsed = Duration::from_secs(0);
    let interval = Duration::from_secs(1);

    println!("Remaining Time: {}", elapsed.as_secs());

    while elapsed < wait_duration && wait_duration < Duration::from_secs(3600*24*7) {
        {
            let mut remaining = remaining_time.lock().unwrap();
            *remaining = Some(wait_duration - elapsed);

        }

        sleep(interval);
        elapsed += interval;
    }

    // Clear remaining time once the wait is over
    let mut remaining = remaining_time.lock().unwrap();
    *remaining = None;
}

fn open_url_after_delay(wait_duration: Duration, url: String) {
    if wait_duration < Duration::from_secs(3600*24*7) {
        sleep(wait_duration);
    }
    if let Err(e) = open::that(url) {
        eprintln!("Failed to open the URL: {}", e);
    }
}


fn load_config(file_path: &str) -> Result<Config, Box<dyn std::error::Error>> {
    let path = Path::new(file_path);
    if !path.exists() {
        return Err("File does not exist.".into());
    }

    let mut file = File::open(file_path)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;

    let config: Config = serde_yaml::from_str(&contents)?;
    Ok(config)
}

fn save_config(file_path: &str, config: &Config) -> Result<(), Box<dyn std::error::Error>> {
    let yaml = serde_yaml::to_string(config)?;
    let mut file = File::create(file_path)?;
    file.write_all(yaml.as_bytes())?;
    Ok(())
}
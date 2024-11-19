use clap::Parser;
use csv::ReaderBuilder;
use eframe::{egui, Error, NativeOptions};
use egui::{FontId, TextBuffer, TextStyle, ViewportBuilder};
use regex::Regex;
use std::collections::HashMap;
use std::env::home_dir;
use std::fs::File;
use std::io::{Read, Write};
use std::path::PathBuf;
use std::process::{exit, Child, Command, Stdio};
use std::{env, fs};
use egui_extras::install_image_loaders;
use tokio::io::AsyncWriteExt;
use lib_egui_utils::icon;

const RDP_TEMPLATE: &str = r"
screen mode id:i:2
use multimon:i:0
desktopwidth:i:1920
desktopheight:i:1080
session bpp:i:32
winposstr:s:0,1,626,176,1617,879
compression:i:1
keyboardhook:i:2
audiocapturemode:i:0
videoplaybackmode:i:1
connection type:i:7
networkautodetect:i:1
bandwidthautodetect:i:1
displayconnectionbar:i:1
enableworkspacereconnect:i:0
disable wallpaper:i:0
allow font smoothing:i:0
allow desktop composition:i:0
disable full window drag:i:1
disable menu anims:i:1
disable themes:i:0
disable cursor setting:i:0
bitmapcachepersistenable:i:1
audiomode:i:0
redirectprinters:i:1
redirectlocation:i:0
redirectcomports:i:0
redirectsmartcards:i:1
redirectwebauthn:i:1
redirectclipboard:i:1
redirectposdevices:i:0
autoreconnection enabled:i:1
authentication level:i:2
prompt for credentials:i:0
negotiate security layer:i:1
remoteapplicationmode:i:0
alternate shell:s:
shell working directory:s:
gatewayhostname:s:
gatewayusagemethod:i:4
gatewaycredentialssource:i:1
gatewayprofileusagemethod:i:0
promptcredentialonce:i:1
gatewaybrokeringtype:i:0
use redirection server name:i:0
rdgiskdcproxy:i:0
kdcproxyname:s:
enablerdsaadauth:i:0
drivestoredirect:s:

full address:s:localhost:{port}
username:s:{user}
";

struct MyApp {
    is_edit_window_open: bool,
    tunnels: Vec<Tunnel>,
    tunnel_processes: HashMap<String, Option<Child>>, // Tracks running processes
    file_path: PathBuf,

    // is_edit_window_open: bool,
    tunnel_file_content: String,
    // tunnel_file_path: String, // Store the path to the tunnels file
    error_message: Option<String>, // Optional error message for the UI
    show_command: bool,
}

#[derive(Clone)]
struct Tunnel {
    pub name:String,
    pub command:String,
    pub user:String,
    pub port:String
}


const PREF_FILE: &str = ".tunnels_pref";


impl MyApp {

    /// Load the last used tunnel file from the preference file
    fn get_last_used_file() -> Option<PathBuf> {
        if let Some(home_dir) = home_dir() {
            let pref_file = home_dir.join(PREF_FILE);
            if pref_file.exists() {
                if let Ok(mut file) = File::open(pref_file) {
                    let mut content = String::new();
                    if file.read_to_string(&mut content).is_ok() {
                        return Some(PathBuf::from(content.trim()));
                    }
                }
            }
        }
        None
    }

    /// Save the currently used tunnel file to the preference file
    fn save_last_used_file(file_path: &PathBuf) {
        if let Some(home_dir) = home_dir() {
            let pref_file = home_dir.join(PREF_FILE);
            if let Ok(mut file) = File::create(pref_file) {
                let _ = file.write_all(file_path.to_string_lossy().as_bytes());
            }
        }
    }

    /// Initialize MyApp, defaulting to the last used file or an empty state
    fn new_with_fallback(file_path: Option<PathBuf>) -> Self {
        let path = file_path.or_else(MyApp::get_last_used_file);

        if let Some(valid_path) = path {
            if valid_path.exists() {
                println!("Loading tunnels from {:?}", valid_path);
                let tunnels = read_tunnels(valid_path.clone());
                Self::save_last_used_file(&valid_path);
                return Self {
                    is_edit_window_open: false,
                    file_path: valid_path,
                    tunnel_file_content: "".to_string(),
                    tunnels,
                    tunnel_processes: HashMap::new(),
                    error_message: None,
                    show_command: false,
                };
            } else {
                println!("Specified or last used file {:?} does not exist.", valid_path);
            }
        }

        // Fallback to empty tunnels
        println!("No valid tunnel file provided or found. Starting with an empty list.");
        Self {
            is_edit_window_open: false,
            file_path: PathBuf::new(),
            tunnel_file_content: "".to_string(),
            tunnels: vec![],
            tunnel_processes: HashMap::new(),
            error_message: None,
            show_command: false,
        }
    }

    fn new(file_path: PathBuf) -> Self {
        Self {
            is_edit_window_open: false,
            file_path: file_path.clone(),
            tunnel_file_content: fs::read_to_string(&file_path).unwrap_or_else(|_| String::new()),
            tunnels:read_tunnels(file_path),
            tunnel_processes: HashMap::new(),
            error_message: None,
            show_command: false,
        }
    }
    fn get_path_to_rdp(&self, connection: Tunnel) -> PathBuf {
        let rdp_file_content = (RDP_TEMPLATE.replace("{user}", &connection.user)).as_str()
            .replace("{port}", &connection.port);
        println!("{}", rdp_file_content);

        // Generate a temporary file path
        let temp_dir = env::temp_dir();
        let rdp_file_path = temp_dir.join(format!("{}.rdp", connection.name));
        println!("{} - {}", temp_dir.display(), rdp_file_path.display());

        // Write the RDP file content to the temporary file
        let mut file = File::create(&rdp_file_path).unwrap();
        let _ = file.write_all(rdp_file_content.as_bytes());

        rdp_file_path
    }

    #[cfg(target_os = "linux")]
    fn start_rdp(&self, connection: Tunnel) -> std::io::Result<Child> {
        let command = format!("/usr/bin/remmina -c {:?}", self.get_path_to_rdp(connection));
        println!("{}", &command.to_string());
        Command::new("bash")
            .args(&["-c", &command])
            .spawn()
    }

    #[cfg(target_os = "windows")]
    fn start_rdp(&self, connection: Tunnel) -> std::io::Result<Child> {
        // Retrieve the current username from the environment variable
        // let user = env::var("USERNAME").unwrap_or_else(|_| "USER".to_string()); // Fallback to "USER" if not found
        let command = format!("MSTSC {:}", self.get_path_to_rdp(connection).to_str().unwrap());
        Command::new("cmd")
            .args(&["/C", &command])
            .spawn()
    }

    #[cfg(target_os = "macos")]
    fn start_rdp(&self, connection: Tunnel) -> std::io::Result<Child> {
        let command = format!("open -a /Applications/Microsoft\\ Remote\\ Desktop.app {:?}", self.get_path_to_rdp(connection));
        Command::new("bash")
            .args(&["-c", &command])
            .spawn()
    }

    pub fn is_tunnel_running(&self, name: &str) -> bool {
        self.tunnel_processes.contains_key(name)
    }

    // fn get_parent(&self) -> String {
    //     let parent = self.file_path.as_path().parent().unwrap().display().to_string();
    //     parent
    // }


    fn start_ssh_tunnel(&self, command: &str) -> std::io::Result<Child> {
        let mut parts = command.split_whitespace();
        let cmd = parts.next().unwrap();
        let args: Vec<&str> = parts.collect();

        Command::new(cmd)
            .args(&args)
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
    }

    fn stop_ssh_tunnel(&mut self, name: &str) {
        if let Some(Some(mut child)) = self.tunnel_processes.remove(name) {
            println!("Stopping tunnel [{}] with PID: {}", name, child.id());
            let _ = child.kill().expect("Tunnel was killed");
        }
    }

    fn start_all(&mut self) {
        for Tunnel {name, command, ..} in &self.tunnels {
            if !self.tunnel_processes.contains_key(name) {
                match self.start_ssh_tunnel(command) {
                    Ok(child) => {
                        self.tunnel_processes.insert(name.clone(), Some(child));
                        println!("Started tunnel: {}", name);
                    }
                    Err(e) => {
                        eprintln!("Failed to start tunnel {}: {}", name, e);
                    }
                }
            }
        }
    }

    fn stop_all(&mut self) {
        let running_tunnels: Vec<String> = self.tunnel_processes.keys().cloned().collect();
        for name in running_tunnels {
            self.stop_ssh_tunnel(&name);
        }
    }

    /// Refresh tunnels by re-reading the CSV file
    fn refresh_tunnels(&mut self) {
        self.tunnels = read_tunnels(self.file_path.clone());
        println!("Tunnels refreshed!");
    }

    /// Quit the application, stopping all tunnels
    fn quit_application(&mut self, frame: &mut eframe::Frame) {
        println!("Stopping all tunnels and quitting...");
        self.stop_all();
        exit(0);
    }


    fn edit_tunnels_ui(&mut self, ui: &mut egui::Ui) -> bool {
        let mut save_clicked = false;

        ui.label("Edit the tunnels configuration:");
        // ui.add(egui::TextEdit::multiline(&mut self.tunnel_file_content).desired_rows(10));
        // Set a custom width for the multiline text editor
        ui.add(egui::TextEdit::multiline(&mut self.tunnel_file_content)
            .desired_rows(10)
            .desired_width(500.0));  // Adjust this value as needed for the desired width


        ui.horizontal(|ui| {
            if ui.button("Save").clicked() {
                // Save the content back to the file
                if let Err(e) = std::fs::write(&self.file_path, &self.tunnel_file_content) {
                    eprintln!("Failed to save tunnels file: {}", e);
                } else {
                    save_clicked = true; // Signal to close the window and refresh tunnels
                }
            }
            if ui.button("Cancel").clicked() {
                save_clicked = true; // Treat Cancel as a signal to close the window
            }
        });

        save_clicked
    }


    fn save_tunnels_file(&self) -> Result<(), std::io::Error> {
        fs::write(&self.file_path, &self.tunnel_file_content)
    }

    fn toggle_tunnel(&mut self, name: &String, command: &String, is_running: bool) {
        if is_running {
            self.stop_ssh_tunnel(name.as_str());
        } else {
            match self.start_ssh_tunnel(command.as_str()) {
                Ok(child) => {
                    self.tunnel_processes.insert(name.clone(), Some(child));
                    println!("Started tunnel: {}", name);
                }
                Err(e) => {
                    eprintln!("Failed to start tunnel {}: {}", name, e);
                }
            }
        }
    }

    fn click_rdp(&mut self, tunnel: &Tunnel, name: &String, command: &String, is_running: bool) -> bool {
        // Check if the tunnel is running. If not, start it.
        if !is_running {
            match self.start_ssh_tunnel(command) {
                Ok(child) => {
                    self.tunnel_processes.insert(name.clone(), Some(child));
                    println!("Started tunnel: {}", name);
                }
                Err(e) => {
                    eprintln!("Failed to start tunnel {}: {}", name, e);
                    return true;
                }
            }
        }

        // Polling mechanism to wait for the tunnel to start
        let mut retries = 5; // Number of retries
        let delay = std::time::Duration::from_millis(200); // Delay between retries

        while retries > 0 {
            std::thread::sleep(delay);
            if self.is_tunnel_running(&name) { // Check if the tunnel is running
                break;
            }
            println!("Waiting for tunnel to start: {} ({} retries left)", name, retries);
            retries -= 1;
        }

        // If the tunnel is still not running, exit with an error
        if !self.is_tunnel_running(&name) {
            eprintln!("Tunnel did not start in time for {}", name);
            return true;
        }

        // Now proceed with starting the RDP connection
        match self.start_rdp(tunnel.clone()) {
            Ok(_) => {
                println!("Started RDP connection for {}", name);
            }
            Err(e) => {
                eprintln!("Failed to start RDP connection for {}: {}", name, e);
            }
        }
        false
    }
}

impl eframe::App for MyApp {

        fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
            install_image_loaders(ctx);

            let mut style = (*ctx.style()).clone();
            style.text_styles = [
                (TextStyle::Button, FontId::proportional(10.0)),
                (TextStyle::Heading, FontId::proportional(11.0)),
                (TextStyle::Body, FontId::proportional(10.0)),
                (TextStyle::Monospace, FontId::proportional(10.0)),
            ]
                .into();
            ctx.set_style(style);

            egui::TopBottomPanel::top("menu_bar").show(ctx, |ui| {
                egui::menu::bar(ui, |ui| {
                    ui.menu_button("Menu", |ui| {
                        // if ui.button("Debug").clicked() {
                        //     println!("Currently running processes:");
                        //     for (name, child_opt) in &self.tunnel_processes {
                        //         if let Some(child) = child_opt {
                        //             println!("Tunnel: {} | PID: {}", name, child.id());
                        //         }
                        //     }
                        // }

                        // Add the toggleable "Show Command" button
                        let button_text = if self.show_command { "Hide Command" } else { "Show Command" };
                        if ui.button(button_text).clicked() {
                            self.show_command = !self.show_command;
                        }

                        if ui.button("Start All").clicked() {
                            self.start_all();
                        }
                        if ui.button("Stop All").clicked() {
                            self.stop_all();
                        }
                        if ui.button("Refresh Tunnels").clicked() {
                            self.refresh_tunnels(); // Call the new refresh function
                        }
                        if ui.button("Edit Tunnels").clicked() {
                            self.is_edit_window_open = true;
                        }
                        if ui.button("Quit").clicked() {
                            self.quit_application(frame); // Call the quit function
                        }

                    });

                });
            });

            if self.is_edit_window_open {
                let mut is_open = self.is_edit_window_open; // Local mutable variable
                egui::Window::new("Edit Tunnels")
                    .open(&mut is_open)
                    .show(ctx, |ui| {
                        if self.edit_tunnels_ui(ui) {
                            // If "Save" button is clicked, close the window and refresh tunnels
                            self.is_edit_window_open = false;
                            self.refresh_tunnels();
                        }
                    });

                // If the window was closed manually, close the edit window and refresh tunnels
                if !is_open {
                    self.is_edit_window_open = false;
                    self.refresh_tunnels();
                }
            }



            egui::CentralPanel::default().show(ctx, |ui| {
                egui::Grid::new("tunnel_grid").show(ui, |ui| {
                    ui.label("Status");
                    ui.label("Name");
                    if self.show_command {
                        ui.label("Command");
                    }

                    // ui.label("Switch");
                    ui.label("RDP"); // New column for RDP
                    ui.end_row();

                    // Collect names of tunnels first to avoid borrowing issues
                    let tunnels = self.tunnels.clone(); // Clone the tunnels for iteration

                    for  tunnel @ Tunnel {name, command, user, ..} in &tunnels {

                        let is_running = self.tunnel_processes.get(name.as_str()).map_or(false, |c| c.is_some());

                        if is_running {
                            ui.add(
                                egui::ImageButton::new(egui::include_image!("../src/green.png"))
                                    // .max_size(egui::vec2(32.0, 32.0))
                                    .rounding(10.0)
                            ).on_hover_text("Stop")
                                .clicked()
                                .then(|| {
                                    self.toggle_tunnel(name, command, is_running);
                                });
                        } else {
                            ui.add(
                                egui::ImageButton::new(egui::include_image!("../src/red.png"))
                                    // .max_size(egui::vec2(32.0, 32.0))
                                    .rounding(10.0)
                            ).on_hover_text("Start")
                                .clicked()
                                .then(|| {
                                    self.toggle_tunnel(name, command, is_running);
                                });
                        }

                        ui.label(name);
                        if self.show_command {
                            ui.label(command);
                        }


                        // if ui.button(if is_running { "Stop" } else { "Start" }).clicked() {
                        //     self.toggle_tunnel(name, command, is_running);
                        // }

                        if user != "" {
                            // RDP Button
                            ui.add(
                                egui::ImageButton::new(egui::include_image!("../src/remote.png"))
                                    // .max_size(egui::vec2(32.0, 32.0))
                                    .rounding(10.0)
                            ).on_hover_text("RDP")
                                .clicked()
                                .then(|| {
                                    // self.toggle_tunnel(name, command, is_running);
                                    self.click_rdp(tunnel, &name, command, is_running)
                                });
                            // if ui.button("RDP").clicked() {
                            //     if self.click_rdp(tunnel, &name, command, is_running) { return; }
                            // }
                        }


                        ui.end_row();
                    }
                });
            });
        }
    }

fn get_port(input: &str) -> &str {
    let re = Regex::new(r"-L (\d+):").unwrap();

    if let Some(captures) = re.captures(input) {
        if let Some(port) = captures.get(1) {
            // println!("Captured value: {}", port.as_str())
            port.as_str()
        } else {""}
    } else {""}
}
fn read_tunnels(file_path: PathBuf) -> Vec<Tunnel> { // Skip lines starting with #;

    //ReaderBuilder::from_reader()
    let mut rdr = ReaderBuilder::new().from_path(file_path).unwrap();
    let mut tunnels = Vec::new();
    let mut next_port = 3389; // Start with the base port

    for result in rdr.records() {
        let record = result.unwrap();

        // Extract the name and user
        let name = record.get(0).unwrap_or("").to_string();

        if name.starts_with("#") {
            continue;
        }

        let user = record.get(2).unwrap_or("").to_string();

        // Generate the command if empty
        let command = if let Some(cmd) = record.get(1) {
            if cmd.trim().is_empty() {
                // Generate a default command
                let port = next_port;
                next_port += 1; // Increment the port for the next tunnel
                format!("ssh {} -L {}:127.0.0.1:3389", name, port)
            } else {
                cmd.to_string()
            }
        } else {
            // Handle entirely missing column
            let port = next_port;
            next_port += 1;
            format!("ssh {} -L {}:127.0.0.1:3389", name, port)
        };

        // Create the Tunnel instance
        let tunnel = Tunnel {
            name,
            command: command.clone(),
            user,
            port: get_port(&command).into(),
        };

        tunnels.push(tunnel);
    }

    tunnels
}




fn get_default_log_path() -> String {
    let mut path = env::current_dir().unwrap();
    path.push("tunnels.csv");
    path.display().to_string()
}

#[derive(Parser, Debug)]
struct Cli {
    #[arg(
        help = "Path to tunnels definition CSV file. If not provided, uses the last saved file.",
        default_value = ""
    )]
    log_path: String,
}

fn get_last_tunnel_file() -> Option<String> {
    let home_dir = dirs::home_dir().expect("Unable to find home directory");
    let pref_path = home_dir.join(PREF_FILE);

    if pref_path.exists() {
        let mut file = File::open(pref_path).ok()?;
        let mut contents = String::new();
        file.read_to_string(&mut contents).ok()?;
        Some(contents.trim().to_string())
    } else {
        None
    }
}

fn save_last_tunnel_file(path: &str) {
    let home_dir = dirs::home_dir().expect("Unable to find home directory");
    let pref_path = home_dir.join(PREF_FILE);
    let mut file = File::create(pref_path).expect("Unable to create preferences file");
    file.write_all(path.as_bytes()).expect("Unable to save file path");
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    let args = Cli::parse();
    let mut file_path = args.log_path.clone();

    if file_path.is_empty() {
        if let Some(last_path) = get_last_tunnel_file() {
            file_path = last_path;
        } else {
            file_path = rfd::FileDialog::new()
                .add_filter("CSV files", &["csv"])
                .pick_file()
                .map(|p| p.to_string_lossy().to_string())
                .unwrap_or_else(|| {
                    eprintln!("No file selected, exiting.");
                    exit(1);
                });

            save_last_tunnel_file(&file_path);
        }
    }

    let app = MyApp::new(file_path.into());
    // let options = eframe::NativeOptions::default();
    let app_icon = icon(include_bytes!("icon.png"));
    let options = NativeOptions {
        viewport: ViewportBuilder::default()
            .with_close_button(true)
            .with_inner_size(egui::Vec2::new(400.0, 300.0))
            .with_icon(app_icon),
        ..Default::default()
    };
    eframe::run_native("Tunnels",
                       options,
                       Box::new(|_cc| Ok(Box::new(app))))
}

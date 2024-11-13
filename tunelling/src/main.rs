use clap::Parser;
use csv::ReaderBuilder;
use eframe::{egui, Error};
use egui::{FontId, TextBuffer, TextStyle};
use std::collections::HashMap;
use std::env;
use std::fmt::format;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;
use std::process::{exit, Child, Command, Stdio};
use regex::Regex;
use tokio::io::AsyncWriteExt;

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
    tunnels: Vec<Tunnel>,
    tunnel_processes: HashMap<String, Option<Child>>, // Tracks running processes
    file_path: PathBuf,
}

#[derive(Clone)]
struct Tunnel {
    pub name:String,
    pub command:String,
    pub user:String,
    pub port:String
}

impl MyApp {
    fn new(file_path: PathBuf) -> Self {
        Self {
            file_path: file_path.clone(),
            tunnels:read_tunnels(file_path),
            tunnel_processes: HashMap::new(),
        }
    }
    fn get_path_to_rdp2(&self, connection: Tunnel) -> PathBuf {
        let rdp_file_content = (RDP_TEMPLATE.replace("{user}", &connection.user)).as_str()
            .replace("{port}", &connection.port);
        println!("{}", rdp_file_content);

        // Generate a temporary file path
        let temp_dir = env::temp_dir();
        let rdp_file_path = temp_dir.join(format!("{}.rdp", connection.name));

        // Write the RDP file content to the temporary file
        let mut file = File::create(&rdp_file_path).unwrap();
        let _ = file.write_all(rdp_file_content.as_bytes());

        rdp_file_path
    }

    #[cfg(target_os = "linux")]
    fn start_rdp(&self, connection: Tunnel) -> std::io::Result<Child> {
        let command = format!("/usr/bin/remmina -c {:?}", self.get_path_to_rdp2(connection));
        println!("{}", &command.to_string());
        Command::new("bash")
            .args(&["-c", &command])
            .spawn()
    }

    #[cfg(target_os = "windows")]
    fn start_rdp(&self, connection: Tunnel) -> std::io::Result<Child> {
        // Retrieve the current username from the environment variable
        // let user = env::var("USERNAME").unwrap_or_else(|_| "USER".to_string()); // Fallback to "USER" if not found
        let command = format!("MSTSC {:?}", self.get_path_to_rdp2(connection));
        Command::new("cmd")
            .args(&["/C", &command])
            .spawn()
    }

    #[cfg(target_os = "macos")]
    fn start_rdp(&self, connection: Tunnel) -> std::io::Result<Child> {
        let command = format!("open -a /Applications/Microsoft\\ Remote\\ Desktop.app {:?}", self.get_path_to_rdp2(connection));
        Command::new("bash")
            .args(&["-c", &command])
            .spawn()
    }

    fn get_parent(&self) -> String {
        let parent = self.file_path.as_path().parent().unwrap().display().to_string();
        parent
    }
    fn get_path_to_rdp(&self, connection: Tunnel) -> String {
        format!("{}/{}.rdp", self.get_parent(), connection.name)
    }


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

}

impl eframe::App for MyApp {

        fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
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
                        if ui.button("Debug").clicked() {
                            println!("Currently running processes:");
                            for (name, child_opt) in &self.tunnel_processes {
                                if let Some(child) = child_opt {
                                    println!("Tunnel: {} | PID: {}", name, child.id());
                                }
                            }
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
                        if ui.button("Quit").clicked() {
                            self.quit_application(frame); // Call the quit function
                        }
                    });
                });
            });

            egui::CentralPanel::default().show(ctx, |ui| {
                egui::Grid::new("tunnel_grid").show(ui, |ui| {
                    ui.label("Name");
                    ui.label("Command");
                    ui.label("Switch");
                    ui.label("RDP"); // New column for RDP
                    ui.end_row();

                    // Collect names of tunnels first to avoid borrowing issues
                    let tunnels = self.tunnels.clone(); // Clone the tunnels for iteration

                    for  tunnel @ Tunnel {name, command, user, ..} in &tunnels {
                        ui.label(name);
                        ui.label(command);

                        let is_running = self.tunnel_processes.get(name.as_str()).map_or(false, |c| c.is_some());

                        if ui.button(if is_running { "Stop" } else { "Start" }).clicked() {
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

                        if user != "" {
                            // RDP Button
                            if ui.button("RDP").clicked() {
                                // Check if the tunnel is running. If not, start it.
                                if !is_running {
                                    match self.start_ssh_tunnel(command) {
                                        Ok(child) => {
                                            self.tunnel_processes.insert(name.clone(), Some(child));
                                            println!("Started tunnel: {}", name);
                                        }
                                        Err(e) => {
                                            eprintln!("Failed to start tunnel {}: {}", name, e);
                                            return;
                                        }
                                    }
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
                            }
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

fn read_tunnels(file_path: PathBuf) -> Vec<Tunnel> {
    let mut rdr = ReaderBuilder::new().from_path(file_path).unwrap();
    let mut tunnels = Vec::new();

    for result in rdr.records() {
        let record = result.unwrap();
        let tunnel = Tunnel {
            name:  record.get(0).unwrap_or("").to_string(),
            command: record.get(1).unwrap_or("").to_string(),
            user: record.get(2).unwrap_or("").to_string(),
            port: get_port(record.get(1).unwrap_or("")).into(),
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
    #[arg(help="path to tunnels definition csv file.",default_value_t=get_default_log_path())]
    log_path: String,
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    let args = Cli::parse();
    let app = MyApp::new(args.log_path.into());
    let options = eframe::NativeOptions::default();
    eframe::run_native("Tunnels", options, Box::new(|_cc| Box::new(app)))
}

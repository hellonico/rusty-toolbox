use clap::Arg;
use csv::ReaderBuilder;
use eframe::{egui, Error};
use std::collections::HashMap;
use std::env;
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use egui::{FontDefinitions, FontFamily, FontId, TextStyle};

struct MyApp {
    tunnels: Vec<(String, String)>,
    tunnel_processes: HashMap<String, Option<Child>>, // Tracks running processes
    file_path: PathBuf,
}

impl MyApp {
    fn new(file_path: PathBuf) -> Self {
        Self {
            file_path: file_path.clone(),
            tunnels:read_tunnels(file_path),
            tunnel_processes: HashMap::new(),
        }
    }

    #[cfg(target_os = "linux")]
    fn start_rdp(&self, connection: &str) -> std::io::Result<Child> {
        let command = format!("/usr/bin/remmina -c {}", self.get_path_to_rdp(connection));
        println!("{}", &command.to_string());
        Command::new("bash")
            .args(&["-c", &command])
            .spawn()
    }

    #[cfg(target_os = "windows")]
    fn start_rdp(&self, connection: &str) -> std::io::Result<Child> {
        // Retrieve the current username from the environment variable
        // let user = env::var("USERNAME").unwrap_or_else(|_| "USER".to_string()); // Fallback to "USER" if not found
        let command = format!("MSTSC {}", self.get_path_to_rdp(connection));
        Command::new("cmd")
            .args(&["/C", &command])
            .spawn()
    }

    #[cfg(target_os = "macos")]
    fn start_rdp(&self, connection: &str) -> std::io::Result<Child> {
        let command = format!("open -a /Applications/Microsoft\\ Remote\\ Desktop.app {}", self.get_path_to_rdp(connection));
        Command::new("bash")
            .args(&["-c", &command])
            .spawn()
    }

    fn get_parent(&self) -> String {
        let parent = self.file_path.as_path().parent().unwrap().display().to_string();
        parent
    }
    fn get_path_to_rdp(&self, connection: &str) -> String {
        format!("{}/{}.rdp", self.get_parent(), connection)
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
        for (name, command) in &self.tunnels {
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

}

impl eframe::App for MyApp {

        fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
            // ctx.set_style({
            //     let mut style = (*ctx.style()).clone();
            //     style.spacing.window_margin.left = 50.0; // Increase left and right margins
            //     style.text_styles.insert(TextStyle::Body);
            //     style
            // });
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

                    for (name, command) in &tunnels {
                        ui.label(name);
                        ui.label(command);

                        let is_running = self.tunnel_processes.get(name).map_or(false, |c| c.is_some());

                        if ui.button(if is_running { "Stop" } else { "Start" }).clicked() {
                            if is_running {
                                self.stop_ssh_tunnel(name);
                            } else {
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

                        // RDP Button
                        if ui.button("RDP").clicked() {
                            match self.start_rdp(name) {
                                Ok(_) => {
                                    println!("Started RDP connection for {}", name);
                                }
                                Err(e) => {
                                    eprintln!("Failed to start RDP connection for {}: {}", name, e);
                                }
                            }
                        }

                        ui.end_row();
                    }
                });
            });
        }
    }


fn read_tunnels(file_path: PathBuf) -> Vec<(String, String)> {
    let mut rdr = ReaderBuilder::new().from_path(file_path).unwrap();
    let mut tunnels = Vec::new();

    for result in rdr.records() {
        let record = result.unwrap();
        let name = record.get(0).unwrap_or("").to_string();
        let command = record.get(1).unwrap_or("").to_string();
        tunnels.push((name, command));
    }

    tunnels
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    let path = env::current_dir().unwrap();

    let matches = clap::Command::new("Tunnel Manager")
        .version("1.0")
        .author("Your Name")
        .about("Manages SSH tunnels")
        .arg(
            Arg::new("file")
                .short('f')
                .long("file")
                .help("Path to the CSV file with SSH tunnels")
                .default_value("${path.display}/tunnels.csv"),
        )
        .get_matches();

    let file_path = matches.get_one::<String>("file").unwrap().into();

    let app = MyApp::new(file_path);
    let options = eframe::NativeOptions::default();
    eframe::run_native("Tunnels", options, Box::new(|_cc| Box::new(app)))
}

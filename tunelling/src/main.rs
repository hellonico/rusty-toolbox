use eframe::{egui, Error};
use std::process::{Command, Child, Stdio};
use std::collections::HashMap;

use csv::ReaderBuilder;
use log::info;

struct MyApp {
    tunnels: Vec<(String, String)>,
    tunnel_processes: HashMap<String, Option<Child>>, // Tracks running processes
}

impl MyApp {
    fn new(tunnels: Vec<(String, String)>) -> Self {
        Self {
            tunnels,
            tunnel_processes: HashMap::new(),
        }
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
}

impl eframe::App for MyApp {

    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            egui::Grid::new("tunnel_grid").show(ui, |ui| {
                ui.label("Name");
                ui.label("Command");
                ui.label("Switch");
                ui.end_row();

                for (name, command) in &self.tunnels {
                    ui.label(name);
                    ui.label(command);

                    let is_running = self.tunnel_processes.get(name).map_or(false, |c| c.is_some());


                    if ui.button(if is_running { "Stop" } else { "Start" }).clicked() {
                        println!("tunnel is running [{}]", is_running);
                        if is_running {
                            if let Some(Some(mut child)) = self.tunnel_processes.remove(name) {
                                println!("tunnel killing [{}]", child.id());
                                let _ = child.kill().expect("Was killed"); // Stop the tunnel
                            }
                        } else {
                            println!("Start new tunnel [{}]", command);
                            match self.start_ssh_tunnel(command) {
                                Ok(child) => {
                                    self.tunnel_processes.insert(name.clone(), Some(child)); // Store the child process
                                    println!("Started tunnel: {}", name); // Log tunnel start
                                }
                                Err(e) => {
                                    eprintln!("Failed to start tunnel {}: {}", name, e);
                                }
                            }
                        }
                    }


                    ui.end_row();
                }
            });

            // Add a Debug button to print running processes
            if ui.button("Debug").clicked() {
                println!("Currently running processes:");
                for (name, child_opt) in &self.tunnel_processes {
                    if let Some(child) = child_opt {
                        println!("Tunnel: {} | PID: {}", name, child.id());
                    }
                }
            }

        });
    }

}


fn read_tunnels(file_path: &str) -> Vec<(String, String)> {
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
async fn main() -> Result<(),  Error> {
    let tunnels = read_tunnels(&std::env::args().nth(1).expect("Please provide a file path"));

    let app = MyApp::new(tunnels);
    let options = eframe::NativeOptions::default();
    eframe::run_native(
        "Tunnels",
        options,
        Box::new(|_cc| Box::new(app)))
}

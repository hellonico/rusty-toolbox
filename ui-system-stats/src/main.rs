use sysinfo::{CpuExt, DiskExt, System, SystemExt};
use get_if_addrs::get_if_addrs;
use egui::{Color32, Stroke};
use wifi_ssid::get_wifi_ssid;

fn get_ip_address() -> Option<String> {
    if let Ok(interfaces) = get_if_addrs() {
        for iface in interfaces {
            if iface.is_loopback() {
                continue;
            }
            if let std::net::IpAddr::V4(ipv4) = iface.ip() {
                return Some(ipv4.to_string());
            }
        }
    }
    None
}

// fn get_wifi_name() -> Option<String> {
//
//         // Execute the 'netsh wlan show interfaces' command
//         let output = Command::new("netsh")
//             .arg("wlan")
//             .arg("show")
//             .arg("interfaces")
//             .output()
//             .expect("Failed to execute command");
//
//         // Check if the command executed successfully
//         if output.status.success() {
//             // Convert the command output from bytes to a readable string
//             let ssid_info = String::from_utf8_lossy(&output.stdout);
//
//             // Iterate over the lines of the command output
//             for line in ssid_info.lines() {
//                 // Find the line that contains the SSID (ignoring the BSSID line)
//                 if line.trim().starts_with("SSID") && !line.contains("BSSID") {
//                     // Extract and print the SSID
//                     let ssid = line.split(":").nth(1).unwrap().trim();
//                     return Some(ssid.parse().unwrap());
//                     //println!("Connected to Wi-Fi: {}", ssid);
//                     //break;
//                 }
//             }
//             None
//         } else {
//             // Print error if the command failed
//             eprintln!("Error: {}", String::from_utf8_lossy(&output.stderr));
//             None
//         }
//     }


fn main() {
    let mut sys = System::new_all();
    sys.refresh_all();

    let options = eframe::NativeOptions::default();
    eframe::run_native(
        "System Info Viewer",
        options,
        Box::new(|_cc| Box::new(MyApp::new(sys))),
    )
        .unwrap();
}

struct MyApp {
    sys: System,
}

impl MyApp {
    fn new(sys: System) -> Self {
        Self { sys }
    }

    fn total_cpu_usage(&self) -> f32 {
        self.sys.cpus().iter().map(|cpu| cpu.cpu_usage()).sum::<f32>() / self.sys.cpus().len() as f32
    }

    fn get_memory_in_gb(&self, bytes: u64) -> f64 {
        (bytes as f64) / 1_073_741_824.0 // Convert bytes to GB (1 GB = 1024^3 bytes)
    }


    // Helper function to draw a filled circle representing disk usage
    fn draw_disk_usage(&self, ui: &mut egui::Ui, disk: &sysinfo::Disk) {
        let used_space = disk.total_space() - disk.available_space();
        let usage_percent = used_space as f32 / disk.total_space() as f32;

        // Determine the size of the circle
        let circle_radius = 50.0;

        // Background circle (empty space)
        ui.painter().circle_filled(ui.min_rect().center(), circle_radius, Color32::LIGHT_GRAY);

        // Filled part based on usage
        // let filled_angle = usage_percent * std::f32::consts::TAU; // Full circle is 2π (TAU)
        let painter = ui.painter();
        painter.add(egui::epaint::CircleShape {
            center: ui.min_rect().center(),
            radius: circle_radius,
            fill: Color32::from_rgb(100, 200, 100), // Greenish color for used space
            stroke: Stroke::default(),           // Define the stroke (none in this case)
        });

        // Text in the center
        painter.text(
            ui.min_rect().center(),
            egui::Align2::CENTER_CENTER,
            format!("{:.0}%", usage_percent * 100.0),
            egui::FontId::proportional(16.0), // Use FontId with a specific size
            Color32::BLACK,
        );
    }


    // Helper function to format the uptime in hours, minutes, and seconds
    fn format_uptime(&self) -> String {
        let uptime_seconds = self.sys.uptime();
        let hours = uptime_seconds / 3600;
        let minutes = (uptime_seconds % 3600) / 60;
        let seconds = uptime_seconds % 60;
        format!("{:02}:{:02}:{:02}", hours, minutes, seconds)
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Refresh the system information on each update
        self.sys.refresh_all();

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("System Information");

            // OS Version
            ui.separator();
            ui.label("OS Version:");
            ui.monospace(format!("{}", self.sys.long_os_version().unwrap_or_default()));


            // Uptime
            ui.separator();
            ui.label("Uptime:");
            ui.monospace(self.format_uptime());


            // CPU Information and Total CPU Usage
            ui.separator();
            ui.label("CPU Information:");
            for cpu in self.sys.cpus() {
                ui.monospace(format!("{}: {}% usage", cpu.name(), cpu.cpu_usage()));
            }
            ui.monospace(format!(
                "Total CPU Usage: {:.2}%",
                self.total_cpu_usage()
            ));

            // Memory Usage (in GB)
            ui.separator();
            ui.label("Memory Usage:");
            ui.monospace(format!(
                "Total: {:.2} GB, Used: {:.2} GB",
                self.get_memory_in_gb(self.sys.total_memory() * 1024),
                self.get_memory_in_gb(self.sys.used_memory() * 1024)
            ));


            // Disk Usage with Filled Circles
            ui.separator();
            ui.label("Disk Usage:");
            for disk in self.sys.disks() {
                ui.label(format!(
                    "{}: Total: {:.2} GB",
                    disk.name().to_str().unwrap_or_default(),
                    (disk.total_space() as f64) / 1_000_000_000.0
                ));
                self.draw_disk_usage(ui, disk); // Draw the filled circle for each disk
            }

            // IP Address
            ui.separator();
            ui.label("Network Information:");
            ui.label("IP Address:");
            match get_ip_address() {
                Some(ip) => ui.monospace(ip),
                None => ui.monospace("No IP address found"),
            };

            // Wi-Fi Name (if available)
            ui.separator();
            ui.label("Wi-Fi Name:");
            // match lib-wifi-ssid::get_wifi_ssid()() {
            //     Some(wifi_name) => ui.monospace(wifi_name),
            //     None => ui.monospace("No Wi-Fi connection found"),
            // }
            ui.monospace(get_wifi_ssid().unwrap());
        });
    }
}

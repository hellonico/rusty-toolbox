use sysinfo::{CpuExt, DiskExt, System, SystemExt};
use get_if_addrs::get_if_addrs;
use egui::{Color32, FontId, ScrollArea, Style, TextStyle};
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

        // Define a smaller font size style
        let mut style = (*ctx.style()).clone();
        style.text_styles = [
            (TextStyle::Heading, FontId::proportional(10.0)),
            (TextStyle::Body, FontId::proportional(9.0)),
            (TextStyle::Monospace, FontId::proportional(9.0)),
        ]
            .into();
        ctx.set_style(style);

        egui::CentralPanel::default().show(ctx, |ui| {
            // Wrap all components in a scrolling panel
            ScrollArea::vertical().show(ui, |ui| {
                ui.heading("System Information");

                // OS Version
                ui.separator();
                ui.label("OS Version:");
                ui.monospace(format!("{}", self.sys.long_os_version().unwrap_or_default()));

                // Kernel Version
                ui.separator();
                ui.label("Kernel Version:");
                ui.monospace(format!("{}", self.sys.kernel_version().unwrap_or_default()));

                // Hostname
                ui.separator();
                ui.label("Hostname:");
                ui.monospace(format!("{}", self.sys.host_name().unwrap_or_default()));

                // Core Count
                ui.separator();
                ui.label("CPU Core Count:");
                ui.monospace(format!("{}", self.sys.cpus().len()));

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
                ui.monospace(format!("Total CPU Usage: {:.2}%", self.total_cpu_usage()));

                // Memory Usage (in GB)
                ui.separator();
                ui.label("Memory Usage:");
                ui.monospace(format!(
                    "Total: {:.2} GB, Used: {:.2} GB",
                    self.get_memory_in_gb(self.sys.total_memory() * 1024),
                    self.get_memory_in_gb(self.sys.used_memory() * 1024)
                ));

                // Disk Usage
                ui.separator();
                ui.label("Disk Usage:");
                for disk in self.sys.disks() {
                    ui.label(format!(
                        "{}: Total: {:.2} GB",
                        disk.name().to_str().unwrap_or_default(),
                        (disk.total_space() as f64) / 1_000_000_000.0
                    ));
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
                ui.monospace(get_wifi_ssid().unwrap_or("No Wi-Fi connection found".to_string()));
            });
        });
    }
}

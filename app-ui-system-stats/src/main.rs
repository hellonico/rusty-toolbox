use egui::{Color32, FontId, ProgressBar, TextStyle};
use egui_extras::{Column, TableBuilder};
use get_if_addrs::get_if_addrs;
use sysinfo::{CpuExt, DiskExt, System, SystemExt};
use lib_egui_utils::my_default_options;
use lib_os_utils::location::get_location;
use lib_os_utils::serial::get_serial_number;
use lib_os_utils::wifi::get_wifi_ssid;

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

    let options =
        my_default_options(600.0, 600.0, include_bytes!("icon.png"));

    eframe::run_native(
        "System Info Viewer",
        options,
        // Box::new(|cc| Box::new(MyApp::new(sys))),
        Box::new(|_cc| Ok(Box::new(MyApp::new(sys)) as Box<dyn eframe::App>)),
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
        self.sys
            .cpus()
            .iter()
            .map(|cpu| cpu.cpu_usage())
            .sum::<f32>()
            / self.sys.cpus().len() as f32
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

        let mut style = (*ctx.style()).clone();
        style.text_styles = [
            (TextStyle::Button, FontId::proportional(10.0)),
            (TextStyle::Heading, FontId::proportional(11.0)),
            (TextStyle::Body, FontId::proportional(11.0)),
            (TextStyle::Monospace, FontId::proportional(10.0)),
        ]
        .into();
        ctx.set_style(style);

        egui::CentralPanel::default()
            .frame(egui::Frame::default().fill(Color32::from_gray(240)))
            .show(ctx, |ui| {
                // Optional: Set a background color to confirm full width
                ui.heading("System Information");

                TableBuilder::new(ui)
                    .striped(false)
                    .cell_layout(
                        egui::Layout::left_to_right(egui::Align::Max)
                            .with_cross_align(egui::Align::Center),
                    ) // Center align in cell
                    .column(Column::remainder().resizable(true)) // Auto-adjust columns to fit width
                    .column(Column::remainder())
                    .header(20.0, |mut header| {
                        header.col(|ui| {
                            ui.heading("Info");
                        });
                        header.col(|ui| {
                            ui.heading("Details");
                        });
                    })
                    .body(|mut body| {
                        body.row(20.0, |mut row| {
                            row.col(|ui| {
                                ui.label("OS Version:");
                            });
                            row.col(|ui| {
                                ui.label(self.sys.long_os_version().unwrap_or_default());
                            });
                        });

                        body.row(20.0, |mut row| {
                            row.col(|ui| {
                                ui.label("Kernel Version:");
                            });
                            row.col(|ui| {
                                ui.label(self.sys.kernel_version().unwrap_or_default());
                            });
                        });

                        body.row(20.0, |mut row| {
                            row.col(|ui| {
                                ui.label("Hostname:");
                            });
                            row.col(|ui| {
                                ui.label(self.sys.host_name().unwrap_or_default());
                            });
                        });

                        body.row(20.0, |mut row| {
                            row.col(|ui| {
                                ui.label("CPU Core Count:");
                            });
                            row.col(|ui| {
                                ui.label(format!("{}", self.sys.cpus().len()));
                            });
                        });

                        body.row(20.0, |mut row| {
                            row.col(|ui| {
                                ui.label("Uptime:");
                            });
                            row.col(|ui| {
                                ui.label(self.format_uptime());
                            });
                        });

                        body.row(20.0, |mut row| {
                            //row.col(|ui| { ui.label(format!("{}:", cpu.name())); });
                            row.col(|ui| {
                                ui.label(format!("CPUs {}", self.sys.cpus().len()));
                            });
                            row.col(|ui| {
                                for cpu in self.sys.cpus() {
                                    ui.add(
                                        ProgressBar::new(cpu.cpu_usage() / 100.0)
                                            .show_percentage()
                                            .desired_height(10.0)
                                            .desired_width(30.0), // Adjust width here
                                    );
                                }
                            });
                        });
                        // takes all power for the http request
                        let loc = &get_location().unwrap();
                        body.row(20.0, |mut row| {
                            row.col(|ui| {
                                ui.label("Location:");
                            });
                            row.col(|ui| {
                                ui.label(&loc.city);
                                ui.label(&loc.country);
                            });
                        });

                        body.row(20.0, |mut row| {
                            row.col(|ui| {
                                ui.label("Total CPU Usage:");
                            });
                            row.col(|ui| {
                                ui.add(
                                    ProgressBar::new(self.total_cpu_usage() / 100.0)
                                        .show_percentage()
                                        .desired_width(100.0),
                                );
                            });
                        });

                        body.row(20.0, |mut row| {
                            let total_memory =
                                self.get_memory_in_gb(self.sys.total_memory() * 1024);
                            let used_memory = self.get_memory_in_gb(self.sys.used_memory() * 1024);
                            let memory_usage = used_memory / total_memory;

                            row.col(|ui| {
                                ui.label(format!("Memory : {} Gb", total_memory));
                            });
                            row.col(|ui| {
                                ui.add(
                                    ProgressBar::new(memory_usage as f32)
                                        .show_percentage()
                                        .desired_width(100.0),
                                );
                            });
                        });

                        for disk in self.sys.disks() {
                            body.row(20.0, |mut row| {
                                let total_space_gb = (disk.total_space() as f64) / 1_000_000_000.0;
                                let used_space_gb = total_space_gb
                                    - (disk.available_space() as f64) / 1_000_000_000.0;
                                let disk_usage = used_space_gb / total_space_gb;

                                row.col(|ui| {
                                    ui.label(format!(
                                        "Disk ({}) [{} Gb]:",
                                        disk.name().to_str().unwrap_or_default(),
                                        total_space_gb
                                    ));
                                });
                                row.col(|ui| {
                                    ui.add(
                                        ProgressBar::new(disk_usage as f32)
                                            .show_percentage()
                                            .desired_width(100.0)
                                            .fill(Color32::from_rgb(255, 105, 180)), // Pink color for disk usage
                                    );
                                });
                            });
                        }

                        body.row(20.0, |mut row| {
                            row.col(|ui| {
                                ui.label("IP Address:");
                            });
                            row.col(|ui| {
                                if let Some(ip) = get_ip_address() {
                                    ui.label(ip);
                                } else {
                                    ui.label("No IP address found");
                                }
                            });
                        });

                        body.row(20.0, |mut row| {
                            row.col(|ui| {
                                ui.label("Wi-Fi Name:");
                            });
                            row.col(|ui| {
                                ui.label(
                                    get_wifi_ssid()
                                        .unwrap_or("No Wi-Fi connection found".to_string()),
                                );
                            });
                        });

                        body.row(20.0, |mut row| {
                            row.col(|ui| {
                                ui.label("Location:");
                            });
                            row.col(|ui| {
                                let loc = get_location().unwrap();
                                ui.label(format!(
                                    "{} - {} / {}",
                                    loc.city, loc.region_name, loc.country
                                ));
                            });
                        });

                        body.row(20.0, |mut row| {
                            row.col(|ui| {
                                ui.label("Serial:");
                            });
                            row.col(|ui| {
                                let loc = get_serial_number().unwrap();
                                ui.label(format!("{}", loc));
                            });
                        });

                        let manager = battery::Manager::new().unwrap();
                        for battery in manager.batteries().unwrap() {
                            body.row(20.0, |mut row| {
                                row.col(|ui| {
                                    ui.label("Battery:");
                                });
                                row.col(|ui| {
                                    let battery = battery.unwrap();
                                    ui.label(format!(
                                        "Percentage: {:.2}% State: {:?} Capacity: {:.2} Wh",
                                        battery.state_of_charge().value * 100.0,
                                        battery.state(),
                                        battery.energy().value
                                    ));
                                });
                            });
                        }

                        ctx.request_repaint();
                    });
            });
    }
}

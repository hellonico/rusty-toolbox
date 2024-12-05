use app_ui_system_stats::SysStats;
use egui::{menu, Color32, FontId, ProgressBar, TextStyle};
use egui_extras::{Column, TableBuilder};
use lib_egui_utils::my_default_options;
use lib_os_utils::VpnStatus;

fn main() {
    let mut sys = SysStats::new();
    sys.refresh_all();

    let options = my_default_options(900.0, 500.0, include_bytes!("../icon.png"));

    eframe::run_native(
        "System Info Viewer",
        options,
        // Box::new(|cc| Box::new(MyApp::new(sys))),
        Box::new(|_cc| Ok(Box::new(MyApp::new(sys)) as Box<dyn eframe::App>)),
    )
    .unwrap();
}

struct MyApp {
    sys: SysStats,
}

impl MyApp {
    fn new(sys: SysStats) -> Self {
        Self { sys }
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
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

                menu::bar(ui, |ui| {
                    ui.menu_button("Menu", |ui| {
                        if ui.button("Refresh").clicked() {
                            self.sys.refresh_all();
                        }
                        if ui.button("Quit").clicked() {
                            std::process::exit(0);
                        }
                    });
                });

                TableBuilder::new(ui)
                    .striped(false)
                    .cell_layout(
                        egui::Layout::left_to_right(egui::Align::Max)
                            .with_cross_align(egui::Align::Center),
                    ) // Center align in cell
                    .column(Column::remainder().resizable(true)) // Auto-adjust columns to fit width
                    .column(Column::remainder())
                    .body(|mut body| {
                        body.row(20.0, |mut row| {
                            row.col(|ui| {
                                ui.label("OS Version:");
                            });
                            row.col(|ui| {
                                let os = &self.sys.long_os_version;
                                ui.label(os);
                            });
                        });

                        body.row(20.0, |mut row| {
                            row.col(|ui| {
                                ui.label("Kernel Version:");
                            });
                            row.col(|ui| {
                                ui.label(&self.sys.kernel_version);
                            });
                        });

                        body.row(20.0, |mut row| {
                            row.col(|ui| {
                                ui.label("Hostname:");
                            });
                            row.col(|ui| {
                                ui.label(&self.sys.host_name);
                            });
                        });

                        body.row(20.0, |mut row| {
                            row.col(|ui| {
                                ui.label("CPU Core Count:");
                            });
                            row.col(|ui| {
                                ui.label(format!("{}", &self.sys.cpus.len()));
                            });
                        });

                        body.row(20.0, |mut row| {
                            row.col(|ui| {
                                ui.label("Uptime:");
                            });
                            row.col(|ui| {
                                ui.label(self.sys.format_uptime());
                            });
                        });

                        body.row(20.0, |mut row| {
                            //row.col(|ui| { ui.label(format!("{}:", cpu.name())); });
                            row.col(|ui| {
                                ui.label(format!("CPUs {}", self.sys.cpus.len()));
                            });
                            row.col(|ui| {
                                for cpu in &self.sys.cpus {
                                    ui.add(
                                        ProgressBar::new(cpu.usage / 100.0)
                                            .show_percentage()
                                            .desired_height(10.0)
                                            .desired_width(30.0), // Adjust width here
                                    );
                                }
                            });
                        });

                        body.row(20.0, |mut row| {
                            row.col(|ui| {
                                ui.label("Total CPU Usage:");
                            });
                            row.col(|ui| {
                                ui.add(
                                    ProgressBar::new(self.sys.total_cpu_usage() / 100.0)
                                        .show_percentage()
                                        .desired_width(100.0),
                                );
                            });
                        });

                        body.row(20.0, |mut row| {
                            let total_memory = self.sys.memory.total_memory;
                            let used_memory = self.sys.memory.used_memory;
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

                        for disk in &self.sys.disks {
                            body.row(20.0, |mut row| {
                                let total_space_gb = disk.total_space_gb;
                                let used_space_gb = disk.used_space_gb;
                                // let free_space_gb = disk.free_space_gb;
                                let disk_usage = used_space_gb / total_space_gb;

                                row.col(|ui| {
                                    ui.label(format!(
                                        "Disk ({}) [{} Gb]:",
                                        disk.name,
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
                                if let Some(ip) = SysStats::get_ip_address() {
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
                                ui.label(&self.sys.wifi_ssid);
                            });
                        });

                        body.row(20.0, |mut row| {
                            row.col(|ui| {
                                ui.label("Location:");
                            });
                            row.col(|ui| {
                                let loc = &self.sys.location;
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
                                let loc = &self.sys.serial_number;
                                ui.label(loc);
                            });
                        });

                        body.row(20.0, |mut row| {
                            row.col(|ui| {
                                ui.label("VPN");
                            });
                                match &self.sys.vpn {
                                    Some(vpn) => {
                                        row.col(|ui| {
                                            ui.label(format!("{:?}", vpn));
                                        });
                                    }
                                    None => {

                                    }
                                }
                            });


                        for battery in &self.sys.batteries {
                            body.row(20.0, |mut row| {
                                row.col(|ui| {
                                    ui.label("Battery:");
                                });
                                row.col(|ui| {
                                    ui.label(format!(
                                        "State: {:?}\nCapacity: {:.2} Wh\nCycles: {:}\nVoltage: {:.2}",
                                        // battery.state_of_charge().value * 100.0,
                                        battery.state,
                                        battery.capacity,
                                        battery.cycle_count,
                                        battery.voltage,
                                    ));
                                    ui.add(
                                        ProgressBar::new(battery.state_of_charge)
                                            .show_percentage()
                                            .desired_width(100.0),
                                    );
                                });
                            });
                        }

                        // ctx.request_repaint();
                    });
            });
    }
}

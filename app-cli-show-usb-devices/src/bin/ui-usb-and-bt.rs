use std::sync::Arc;
use btleplug::api::Manager;
use eframe::egui;
use tokio::sync::Mutex;
use app_cli_show_usb_devices::bt::fetch_bluetooth_devices;
use app_cli_show_usb_devices::usb::fetch_usb_devices;

#[tokio::main]
async fn main() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions::default();
    eframe::run_native(
        "USB and Bluetooth Device Viewer",
        options,
        Box::new(|_cc| Ok(Box::new(DeviceApp::default()))),
    )
}

struct DeviceApp {
    usb_devices: Vec<String>,
    bluetooth_devices: Arc<Mutex<Vec<String>>>,
    error: Option<String>,
    selected_tab: Tab,
}

enum Tab {
    USB,
    Bluetooth,
}

impl Default for DeviceApp {
    fn default() -> Self {
        Self {
            usb_devices: fetch_usb_devices().unwrap_or_else(|_| vec!["Error fetching USB devices".to_string()]),
            bluetooth_devices : Self::fetchbt(),
            error: None,
            selected_tab: Tab::USB,
        }
    }

}

impl DeviceApp {
    pub fn fetchbt() -> Arc<Mutex<Vec<String>>> {
        let bluetooth_devices = Arc::new(Mutex::new(Vec::new()));
        let cloned_bt_devices = bluetooth_devices.clone();

        // Spawn an asynchronous task without creating a new runtime
        tokio::spawn(async move {
            let manager = btleplug::platform::Manager::new().await.unwrap();
            let adapters = manager.adapters();
            let central = Arc::new(Mutex::new(adapters.await.unwrap().iter().nth(0).unwrap().clone()));
            fetch_bluetooth_devices(cloned_bt_devices, central).await;
        });
        // self.bluetooth_devices =
        bluetooth_devices
    }
}

impl eframe::App for DeviceApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::TopBottomPanel::top("tabs").show(ctx, |ui| {
            ui.horizontal(|ui| {
                if ui.button("USB Devices").clicked() {
                    self.selected_tab = Tab::USB;
                }
                if ui.button("Bluetooth Devices").clicked() {
                    self.selected_tab = Tab::Bluetooth;
                    Self::fetchbt();
                }
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| match self.selected_tab {
            Tab::USB => {
                ui.heading("USB Devices");
                if self.usb_devices.is_empty() {
                    ui.label("No USB devices found.");
                } else {
                    egui::ScrollArea::vertical().show(ui, |ui| {
                        for device in &self.usb_devices {
                            ui.label(device);
                        }
                    });
                }
            }
            Tab::Bluetooth => {
                ui.heading("Bluetooth Devices");
                let bluetooth_devices = self.bluetooth_devices.lock();
                // if bluetooth_devices.is_empty() {
                //     ui.label("Scanning for Bluetooth devices...");
                // } else {
                //     egui::ScrollArea::vertical().show(ui, |ui| {
                //         for device in bluetooth_devices.iter() {
                //             ui.label(device);
                //         }
                //     });
                // }
            }
        });
    }
}


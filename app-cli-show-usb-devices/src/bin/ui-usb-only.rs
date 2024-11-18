use app_cli_show_usb_devices::usb::fetch_usb_devices;
use eframe::{egui, Error};

struct UsbApp {
    devices: Vec<String>,
    error: Option<String>,
}
impl Default for UsbApp {
    fn default() -> Self {
        let (devices, error) = match fetch_usb_devices() {
            Ok(devices) => (devices, None),
            Err(err) => (Vec::new(), Some(err.to_string())),
        };

        Self { devices, error }
    }
}

impl eframe::App for UsbApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("USB Devices");

            if let Some(err) = &self.error {
                ui.colored_label(egui::Color32::RED, format!("Error: {}", err));
            } else if self.devices.is_empty() {
                ui.label("No USB devices found.");
            } else {
                egui::ScrollArea::vertical().show(ui, |ui| {
                    for device in &self.devices {
                        ui.label(device);
                    }
                });
            }
        });
    }
}
#[tokio::main]
async fn main() -> Result<(), Error> {
    let app = UsbApp::default();
    let options = eframe::NativeOptions::default();
    let app = UsbApp::default();
    eframe::run_native(
        "USB Device Viewer",
        options,
        Box::new(|_cc| Ok(Box::new(app)))
    )
}

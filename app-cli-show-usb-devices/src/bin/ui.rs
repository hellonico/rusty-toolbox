use eframe::{egui, Error};
use rusb::{Context, Device, UsbContext};

// fn main() {
//     let native_options = eframe::NativeOptions::default();
//     eframe::run_native("My egui App", native_options, Box::new(|cc| Box::new(MyEguiApp::new(cc))));
// }

// struct MyEguiApp {}

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

fn fetch_usb_devices() -> rusb::Result<Vec<String>> {
    let context = Context::new()?;
    let devices = context.devices()?;
    let mut device_list = Vec::new();

    for device in devices.iter() {
        if let Ok(info) = format_device_info(&device) {
            device_list.push(info);
        }
    }

    Ok(device_list)
}

fn format_device_info(device: &Device<Context>) -> rusb::Result<String> {
    let descriptor = device.device_descriptor()?;

    Ok(format!(
        "Bus {:03} Device {:03} ID {:04x}:{:04x}",
        device.bus_number(),
        device.address(),
        descriptor.vendor_id(),
        descriptor.product_id()
    ))
}

fn main() -> Result<(), Error> {
    let app = UsbApp::default();
    let options = eframe::NativeOptions::default();
    let app = UsbApp::default();
    eframe::run_native(
        "USB Device Viewer",
        options,
        Box::new(|_cc| Ok(Box::new(app)))
    )
}

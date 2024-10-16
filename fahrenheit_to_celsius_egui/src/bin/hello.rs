use egui::{CentralPanel, Context};
use fahrenheit_to_celsius_egui::common;
use std::sync::{Arc, Mutex};

struct MyApp {
    response: Arc<Mutex<Option<String>>>,
}

impl MyApp {
    fn new() -> Self {
        MyApp {
            response: Arc::new(Mutex::new(None)),
        }
    }


    fn update_async(&mut self) {
        let response = self.response.clone();
        tokio::spawn(async move {
            common::fetch_data(String::from("100"), response).await;
        });
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &Context, _frame: &mut eframe::Frame) {
        CentralPanel::default().show(ctx, |ui| {
            ui.heading("Async GET Request Example");

            let response_message = {
                let response_lock = self.response.lock().unwrap();
                response_lock.clone()
            };

            if let Some(message) = response_message {
                ui.label(format!("Response: {}", message));
            } else {
                ui.label("No data fetched yet.");
            }

            if ui.button("Fetch Data").clicked() {
                self.update_async();
            }
        });
    }
}

#[tokio::main]
async fn main() -> Result<(), eframe::Error> {
    let app = MyApp::new();
    let native_options = eframe::NativeOptions::default();
    eframe::run_native("Async GET Example", native_options, Box::new(|_cc| Ok(Box::new(app))))
}

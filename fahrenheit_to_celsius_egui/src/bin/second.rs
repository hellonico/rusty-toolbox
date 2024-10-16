use std::fmt::Debug;
use egui::{CentralPanel, Context};
use fahrenheit_to_celsius_egui::common;
use std::sync::{Arc, Mutex};

struct MyApp {
    response: Arc<Mutex<Option<String>>>,
    fahrenheit: String
}

impl MyApp {
    fn new() -> Self {
        MyApp {
            response: Arc::new(Mutex::new(None)),
            fahrenheit: "100".to_string()
        }
    }


    fn update_async(&mut self, value: String) {
        let response = self.response.clone();
        tokio::spawn(async move {
            common::fetch_data(String::from(value), response).await;
        });
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &Context, _frame: &mut eframe::Frame) {
        CentralPanel::default().show(ctx, |ui| {
            ui.label("Enter temperature in Fahrenheit:");
            ui.text_edit_singleline(&mut self.fahrenheit);

            if ui.button("Convert to celcius").clicked() {
                let m = self.fahrenheit.clone();
                self.update_async(m);
            }

            let response_message = {
                let response_lock = self.response.lock().unwrap();
                response_lock.clone()
            };

            if let Some(message) = response_message {
                ui.label(format!("Celsius: {}", message));
            }
        });
    }
}

#[tokio::main]
async fn main() -> Result<(), eframe::Error> {
    let app = MyApp::new();
    let native_options = eframe::NativeOptions::default();
    eframe::run_native("Fahrenheit to Celsius", native_options, Box::new(|_cc| Ok(Box::new(app))))
}

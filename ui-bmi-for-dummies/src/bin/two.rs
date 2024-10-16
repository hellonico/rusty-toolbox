use eframe::egui;
use chrono::{DateTime, Local};
use serde::{Deserialize, Serialize};
use std::fs::{File, OpenOptions};
use std::io::{BufReader, BufWriter, Write};

// File where BMI records will be saved
const BMI_LOG_FILE: &str = "bmi_log.json";

fn main() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions::default();
    eframe::run_native(
        "BMI Calculator with History",
        options,
        Box::new(|_cc| Box::new(BmiCalculatorApp::load())),
    )
}

#[derive(Serialize, Deserialize)]
struct BmiRecord {
    timestamp: String,
    bmi: f32,
}

#[derive(Default)]
struct BmiCalculatorApp {
    height_cm: f32,
    weight_kg: f32,
    bmi: Option<f32>,
    bmi_history: Vec<BmiRecord>,
}

impl BmiCalculatorApp {
    // Load the app with saved BMI records from file
    fn load() -> Self {
        let bmi_history = match File::open(BMI_LOG_FILE) {
            Ok(file) => {
                let reader = BufReader::new(file);
                serde_json::from_reader(reader).unwrap_or_default()
            }
            Err(_) => vec![], // If file doesn't exist or is corrupted
        };

        Self {
            bmi_history,
            ..Default::default()
        }
    }

    // Calculate BMI and save the result with a timestamp
    fn calculate_bmi(&mut self) {
        if self.height_cm > 0.0 && self.weight_kg > 0.0 {
            let height_m = self.height_cm / 100.0;
            let bmi = self.weight_kg / (height_m * height_m);

            // Get the current timestamp
            let now: DateTime<Local> = Local::now();
            let timestamp = now.format("%Y-%m-%d %H:%M:%S").to_string();

            // Save the record
            self.bmi_history.push(BmiRecord { timestamp, bmi });
            self.save_bmi_history();

            // Keep only the last 10 records
            if self.bmi_history.len() > 10 {
                self.bmi_history.drain(0..self.bmi_history.len() - 10);
            }

            self.bmi = Some(bmi);
        }
    }

    // Save the BMI history to a file
    fn save_bmi_history(&self) {
        let file = OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(BMI_LOG_FILE)
            .unwrap();
        let writer = BufWriter::new(file);
        serde_json::to_writer(writer, &self.bmi_history).unwrap();
    }
}

impl eframe::App for BmiCalculatorApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("BMI Calculator with History");

            // Input fields for height and weight, updating BMI in real-time
            ui.horizontal(|ui| {
                ui.label("Height (cm): ");
                if ui.add(egui::DragValue::new(&mut self.height_cm).speed(1.0)).changed() {
                    self.calculate_bmi(); // Recalculate BMI when height changes
                }
            });

            ui.horizontal(|ui| {
                ui.label("Weight (kg): ");
                if ui.add(egui::DragValue::new(&mut self.weight_kg).speed(1.0)).changed() {
                    self.calculate_bmi(); // Recalculate BMI when weight changes
                }
            });

            // Display BMI result in real-time
            if let Some(bmi_value) = self.bmi {
                ui.label(format!("Your BMI is: {:.2}", bmi_value));
                let category = match bmi_value {
                    bmi if bmi < 18.5 => "Underweight",
                    bmi if bmi < 24.9 => "Normal weight",
                    bmi if bmi < 29.9 => "Overweight",
                    _ => "Obesity",
                };
                ui.label(format!("Category: {}", category));
            }

            // Display last 10 BMI records
            if !self.bmi_history.is_empty() {
                ui.separator();
                ui.heading("Last 10 BMI Records");

                for record in self.bmi_history.iter().rev() {
                    ui.label(format!("{} - BMI: {:.2}", record.timestamp, record.bmi));
                }
            }
        });
    }
}

use eframe::egui;

fn main() -> Result<(), eframe::Error> {
    // Launch the app
    let options = eframe::NativeOptions::default();
    eframe::run_native(
        "BMI Calculator",
        options,
        Box::new(|_cc| Box::new(BmiCalculatorApp::default())),
    )
}

#[derive(Default)]
struct BmiCalculatorApp {
    height_cm: f32,
    weight_kg: f32,
    bmi: Option<f32>,
}

impl eframe::App for BmiCalculatorApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("BMI Calculator");

            // Input fields for height and weight
            ui.horizontal(|ui| {
                ui.label("Height (cm): ");
                ui.add(egui::DragValue::new(&mut self.height_cm).speed(1.0));
            });

            ui.horizontal(|ui| {
                ui.label("Weight (kg): ");
                ui.add(egui::DragValue::new(&mut self.weight_kg).speed(1.0));
            });

            // Button to compute BMI
            if ui.button("Compute BMI").clicked() {
                self.bmi = self.calculate_bmi();
            }

            // Display BMI result
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
        });
    }
}

impl BmiCalculatorApp {
    fn calculate_bmi(&self) -> Option<f32> {
        if self.height_cm > 0.0 && self.weight_kg > 0.0 {
            let height_m = self.height_cm / 100.0;
            Some(self.weight_kg / (height_m * height_m))
        } else {
            None
        }
    }
}

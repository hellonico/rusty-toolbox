use eframe::egui;

#[derive(Default)]
pub struct MyApp {
    input_text: String,
    output_text: String,
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.label("Enter text to echo:");
            ui.text_edit_singleline(&mut self.input_text);

            if ui.button("Echo").clicked() {
                self.run_echo();
            }

            if !self.output_text.is_empty() {
                ui.label(format!("Output: {}", self.output_text));
            }
        });
    }
}

impl MyApp {
    fn run_echo(&mut self) {
        #[cfg(not(target_arch = "wasm32"))]
        {
            // For native platform: Run `echo` command
            use std::process::Command;
            let output = Command::new("echo")
                .arg(&self.input_text)
                .output()
                .expect("failed to execute echo command");

            self.output_text = String::from_utf8_lossy(&output.stdout).trim().to_string();
        }

        #[cfg(target_arch = "wasm32")]
        {
            // For WASM platform: Simply show the input text as output
            self.output_text = self.input_text.clone();
        }
    }
}

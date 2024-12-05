use std::error::Error;
use eframe::{egui};
use std::path::Path;
use app_convert_png_to_icns::png_to_icns;
use lib_egui_utils::my_default_options;
use lib_ffmpeg_utils::append_to_home_log;

fn main() -> Result<(), eframe::Error> {
    let options =
        my_default_options(800.0, 500.0, include_bytes!("../../icon.png"));

    eframe::run_native(
        "PNG to ICNS Converter",
        options,
        Box::new(|_cc| Ok(Box::new(MyApp::default()))),
    )
}

struct MyApp {
    input_path: Option<String>,
    output_path: Option<String>,
    dropped_files: Vec<egui::DroppedFile>,
}

impl Default for MyApp {
    fn default() -> Self {
        Self {
            input_path: None,
            output_path: None,
            dropped_files: Vec::new(),
        }
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Drag and Drop a PNG File to Convert to ICNS");

            // Display the input file if available
            if let Some(input) = &self.input_path {
                ui.label(format!("Input: {}", input));
            } else {
                ui.label("No file selected");
            }
            if let Some(output) = &self.output_path {
                ui.label(format!("Output: {}", output));
            }

            // Handle file drag-and-drop
            if !self.dropped_files.is_empty() {
                for file in &self.dropped_files {
                    if let Some(path) = &file.path {
                        let input = path.to_string_lossy().to_string();
                        self.input_path = Some(input.clone());
                        let (output, res) = Self::do_convert(&input.clone());
                        append_to_home_log(format!("{}\n{:?}", output.clone(),res));
                        self.output_path = Some(output.clone());
                        if let Err(e) = res {
                            ui.label(format!("Error: {}", e));
                        } else {
                            ui.label(format!("Successfully saved as {}", output));
                        }
                        break;
                    }
                }
                self.dropped_files.clear();
            }
        });

        // Collect dropped files
        ctx.input(|i| {
            if !i.raw.dropped_files.is_empty() {
                self.dropped_files = i.raw.dropped_files.clone();
            }
        });
    }
}

impl MyApp {
    fn do_convert(input: &String) -> (String, Result<(), Box<dyn Error>>) {
        let output = Path::new(input)
            .with_extension("icns")
            .to_str()
            .unwrap()
            .to_string();
        let res = png_to_icns(input, &output);
        (output, res)
    }
}

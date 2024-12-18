use app_cli_overlay_text::{blend_text_to_image, load_font, load_image};
use eframe::egui;
use eframe::egui::Image;
use egui_extras::install_image_loaders;
use image::RgbaImage;
use rusttype::Font;
use std::fs;
use std::path::PathBuf;
use tempfile::Builder;

struct ImageOverlayApp {
    image_path: Option<PathBuf>,
    font_path: Option<PathBuf>,
    text: String,
    x: f32,
    y: f32,
    scale_factor: f32,
    thickness: f32,
    output_path: String,
    final_image_path: String,
    color: String,

    font: Option<Font<'static>>,
    image: Option<RgbaImage>,
    // color_image: Option<ColorImage>,
    display_width: f32,
}

impl Default for ImageOverlayApp {
    fn default() -> Self {
        Self {
            // image_path: Some(PathBuf::from(
            //     "/Users/niko/Downloads/christmas-trees-background-free-vector.jpg",
            // )),
            // font_path: Some(PathBuf::from(
            //     "/Users/niko/cool/rusty-toolbox/ui-fonts/Mynerve-Regular.ttf",
            // )),
            image_path: None,
            font_path: None,
            text: "Nico! It was nice to share 2024 with You!".to_string(),
            x: 80.0,
            y: 1190.0,
            scale_factor: 0.07,
            thickness: 2.0,
            output_path: "output.png".to_string(),
            final_image_path: "".to_string(),
            color: "#FF".to_string(),
            display_width:300.0,
            font: None,
            image: None,
            // color_image: None,
        }
        // load_image(sefl)
    }
}

impl eframe::App for ImageOverlayApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Image Overlay Tool");

            // File Selection
            ui.horizontal(|ui| {
                ui.label("Image Path:");
                if let Some(path) = &self.image_path {
                    ui.label(path.display().to_string());
                }
                if ui.button("Select Image").clicked() {
                    if let Some(path) = rfd::FileDialog::new().pick_file() {
                        self.image_path = Some(path.clone());
                        self.image = Some(load_image(&path.clone().to_string_lossy()));
                        if let Some(path) = self.font_path.clone() {
                            self.font = Some(load_font(self.font_path.clone().unwrap()));
                            self.update_image();
                        }
                    }
                }
            });

            ui.horizontal(|ui| {
                ui.label("Font Path:");
                if let Some(path) = &self.font_path {
                    ui.label(path.display().to_string());
                }
                if ui.button("Select Font").clicked() {
                    if let Some(path) = rfd::FileDialog::new().pick_file() {
                        self.font_path = Some(path.clone());
                        self.font = Some(load_font(self.font_path.clone().unwrap()));
                        if let Some(path) = self.image_path.clone() {
                            self.image = Some(load_image(
                                self.image_path.clone().unwrap().to_str().unwrap().trim(),
                            ));

                            self.update_image();
                        }

                    }
                }
            });

            // Parameters
            ui.separator();
            ui.label("Overlay Text:");
            if ui.text_edit_singleline(&mut self.text).changed() {
                self.update_image();
            };

            ui.horizontal(|ui| {
                ui.label("X Position:");
                if ui.add(egui::DragValue::new(&mut self.x)).changed() {
                    self.update_image();
                };
                ui.label("Y Position:");
                if ui.add(egui::DragValue::new(&mut self.y)).changed() {
                    self.update_image();
                };

            });

            ui.horizontal(|ui| {
                ui.label("Scale Factor:");
                if ui
                    .add(egui::DragValue::new(&mut self.scale_factor))
                    .changed()
                {
                    self.update_image();
                };
            });

            ui.horizontal(|ui| {
                ui.label("Thickness:");
                if ui.add(egui::DragValue::new(&mut self.thickness)).changed() {
                    self.update_image();
                };
            });

            ui.horizontal(|ui| {
                ui.label("Color:");
                if ui.text_edit_singleline(&mut self.color).changed() {
                    self.update_image();
                };
            });

            ui.horizontal(|ui| {
                ui.label("Display Size:");
                if ui.add(egui::DragValue::new(&mut self.display_width)).changed() {
                    self.update_image();
                };
            });

            ui.horizontal(|ui| {
                ui.label("Output Path:");
                if ui.text_edit_singleline(&mut self.output_path).changed() {
                    self.update_image();
                };
            });

            // Save Button
            if ui.button("Save Image").clicked() {
                if !self.final_image_path.is_empty() {
                    fs::copy(&self.final_image_path, &self.output_path)
                        .expect("Failed to save image");
                }
            }

            ui.separator();
            ui.heading("Preview:");

            // Render the Image
            if self.final_image_path.is_empty() {
                // ui.image(format!("file://{}", self.final_image_path));
                // if let Some(img) = &self.color_image {
                //     let image_widget = Image::new(ImageSource::from(img))
                //     .max_width(300.0)
                //     .rounding(10.0);
                //     ui.add(image_widget);
                ui.label("No image to display. Please select an image and font.");
            } else {
                let image_widget = Image::new(format!("file://{}", self.final_image_path))
                    .max_width(self.display_width)
                    .rounding(10.0);
                ui.add(image_widget);
            }
        });
    }
}

impl ImageOverlayApp {
    fn update_image(&mut self) {
        if let (Some(image_path), Some(font_path)) = (&self.image_path, &self.font_path) {
            let img = blend_text_to_image(
                self.image.clone().unwrap(),
                self.font.clone().unwrap(),
                &self.text,
                self.x,
                self.y,
                self.scale_factor,
                self.thickness as u32,
                &*self.color,
            );

            self.image = Some(img.clone());
            // self.color_image = Some(load_image_as_bytes_(img));

            let builder = Builder::new().suffix(".png").tempfile();
            let tmp_path = builder.unwrap().path().to_owned();
            img.save(tmp_path.clone())
                .expect("Failed to save temp image");

            println!("Saved temp image to {:?}", tmp_path);
            self.final_image_path = tmp_path.clone().to_string_lossy().to_string();
        }
    }
}
fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions::default();
    eframe::run_native(
        "Image Overlay App",
        options,
        Box::new(|_cc| {
            install_image_loaders(&_cc.egui_ctx);
            // install_image_loaders(&_cc.egui_ctx);
            Ok(Box::new(ImageOverlayApp::default()))
        }),
    )
}

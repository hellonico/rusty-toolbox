use egui::Rangef;
use lib_egui_utils::my_default_options;
use egui_extras::{StripBuilder, Size};
#[derive(Clone, Debug, Copy, Default)]
struct VideoApp2 {

}
impl eframe::App for VideoApp2 {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {

            StripBuilder::new(ui)
                .size(Size::remainder().at_least(100.0)) // top cell
                .size(Size::exact(120.0)) // bottom cell
                .vertical(|mut strip| {
                    strip.strip(|builder| {
                        builder.sizes(Size::remainder(), 1).horizontal(|mut strip| {
                            strip.cell(|ui| {
                                ui. label(format!("One{}",ui.max_rect()));
                            });

                        });
                    });

                    strip.strip(|builder| {
                        builder.sizes(Size::remainder(), 4).horizontal(|mut strip| {
                            strip.cell(|ui| {
                                ui. label(format!("Left{}",ui.min_rect()));
                            });
                            strip.cell(|ui| {
                                ui. label(format!("Center {}",ui.min_rect()));
                            });
                            strip.cell(|ui| {
                                ui. label(format!("Right {}",ui.min_rect()));
                            });

                        });
                    });
                });
        });
    }
}

#[tokio::main]
async fn main() -> Result<(), eframe::Error> {
    let options =
        my_default_options(800.0, 500.0, include_bytes!("../../icon.png"));

    eframe::run_native("Video Browser", options, Box::new(|_cc| Ok(Box::new(VideoApp2::default()))))
}
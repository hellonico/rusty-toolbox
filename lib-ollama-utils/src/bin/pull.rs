use std::sync::{Arc, Mutex};
use eframe::{egui, Frame};
use tokio::runtime::Runtime;
use tokio::sync::mpsc;
// use lib_ollama_utils::ModelDownloader;
// struct App {
//     downloader: Arc<Mutex<ModelDownloader>>,
// }
//
// impl App  {
//
//     fn new() -> Self {
//         Self {
//             downloader: Arc::new(Mutex::new(ModelDownloader::new())),
//         }
//     }
//
//     async fn download(&mut self) {
//         let (tx, mut rx) = mpsc::channel(1);
//         let model_name = self.downloader.lock().unwrap().model_name;
//         tokio::spawn(async move {
//             self.downloader.pull_model(model_name, tx).await;
//             println!("Model {} downloaded successfully!", model_name);
//         });
//     }
// }
//
// impl eframe::App for App {
//
//     fn update(&mut self, ctx: &egui::Context, _frame: &mut Frame) {
//         egui::CentralPanel::default().show(ctx, |ui| {
//             ui.horizontal(|ui| {
//                 ui.label("Model Name:");
//                 ui.text_edit_singleline(&mut self.model_name);
//             });
//
//             if ui.button("Download Model").clicked() && !self.is_downloading {
//                 // Monitor the status updates
//                 self.download();
//                 ctx.request_repaint();
//                 // while let Some(status) = rx.blocking_recv() {
//                 //     self.downloader.status = status;
//                 //     ctx.request_repaint();
//                 //     if self.downloader.status == "success" {
//                 //         break;
//                 //     }
//                 // }
//             }
//
//             ui.label("Status:");
//             ui.label(&self.downloader.status);
//         });
//     }
// }

#[tokio::main]
async fn main() -> eframe::Result {
    // let options = eframe::NativeOptions {
    //     ..Default::default()
    // };
    // eframe::run_native("Model Downloader", options,
    //                    Box::new(|_cc| Ok(Box::<App>::new(App::new()))))
    Ok(())
}

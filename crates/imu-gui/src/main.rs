use eframe::egui;

mod types;
mod state;
mod background;
mod connection;
mod data;
mod config;
mod ui;

use state::ImuApp;

fn main() -> eframe::Result<()> {
    env_logger::init();
    
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1024.0, 768.0])
            .with_title("IMU948 Control Panel"),
        ..Default::default()
    };

    eframe::run_native(
        "IMU948 Control Panel",
        options,
        Box::new(|_cc| Ok(Box::new(ImuApp::default()))),
    )
}

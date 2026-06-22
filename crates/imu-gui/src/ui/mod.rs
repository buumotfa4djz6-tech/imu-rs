pub mod device_panel;
pub mod config_panel;
pub mod visualization;

use eframe::egui;
use crate::state::ImuApp;

impl eframe::App for ImuApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Consume data from channel (non-blocking)
        let mut readings_to_process = Vec::new();
        if let Some(data_rx) = &mut self.data_rx {
            while let Ok(reading) = data_rx.try_recv() {
                readings_to_process.push(reading);
            }
        }
        for reading in readings_to_process {
            self.process_imu_reading(&reading);
        }
        
        // Top menu bar
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("Quit").clicked() {
                        std::process::exit(0);
                    }
                });
                ui.menu_button("Help", |ui| {
                    ui.label("IMU948 Control Panel");
                    ui.label("Version 0.1.0");
                });
            });
        });

        // Bottom status bar
        egui::TopBottomPanel::bottom("status_bar").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label("Status:");
                ui.label(&self.status_message);
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if self.is_connected {
                        ui.label("🟢 Connected");
                        if let Some(battery) = self.battery_level {
                            ui.label(format!("🔋 {}%", battery));
                        }
                    } else {
                        ui.label("🔴 Disconnected");
                    }
                });
            });
        });

        // Left panel for device discovery
        egui::SidePanel::left("device_panel").show(ctx, |ui| {
            self.show_device_panel(ui, ctx);
        });

        // Right panel for configuration (Slice 8)
        egui::SidePanel::right("config_panel").show(ctx, |ui| {
            self.show_config_panel(ui);
        });

        // Central panel for data visualization (Slice 7)
        egui::CentralPanel::default().show(ctx, |ui| {
            self.show_visualization_panel(ui);
        });
    }
}

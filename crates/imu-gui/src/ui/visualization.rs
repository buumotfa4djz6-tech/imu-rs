use eframe::egui;
use crate::state::ImuApp;

impl ImuApp {
    pub fn show_visualization_panel(&mut self, ui: &mut egui::Ui) {
        ui.heading("IMU Data Visualization");
        ui.separator();
        
        if self.is_connected {
            // Acceleration plot
            ui.label("Acceleration (m/s²)");
            if !self.accel_buffer.is_empty() {
                egui::Frame::canvas(ui.style()).show(ui, |ui| {
                    let plot = egui_plot::Plot::new("accel_plot")
                        .height(150.0)
                        .show_x(false)
                        .show_y(true);
                    
                    plot.show(ui, |plot_ui| {
                        let x_data: Vec<[f64; 2]> = self.accel_buffer.iter().enumerate()
                            .map(|(i, (x, _, _))| [i as f64, *x])
                            .collect();
                        let y_data: Vec<[f64; 2]> = self.accel_buffer.iter().enumerate()
                            .map(|(i, (_, y, _))| [i as f64, *y])
                            .collect();
                        let z_data: Vec<[f64; 2]> = self.accel_buffer.iter().enumerate()
                            .map(|(i, (_, _, z))| [i as f64, *z])
                            .collect();
                        
                        plot_ui.line(egui_plot::Line::new(egui_plot::PlotPoints::from(x_data))
                            .name("X").color(egui::Color32::RED));
                        plot_ui.line(egui_plot::Line::new(egui_plot::PlotPoints::from(y_data))
                            .name("Y").color(egui::Color32::GREEN));
                        plot_ui.line(egui_plot::Line::new(egui_plot::PlotPoints::from(z_data))
                            .name("Z").color(egui::Color32::BLUE));
                    });
                });
            } else {
                ui.label("No acceleration data available");
            }
            
            ui.add_space(10.0);
            
            // Angular velocity plot
            ui.label("Angular Velocity (°/s)");
            if !self.gyro_buffer.is_empty() {
                egui::Frame::canvas(ui.style()).show(ui, |ui| {
                    let plot = egui_plot::Plot::new("gyro_plot")
                        .height(150.0)
                        .show_x(false)
                        .show_y(true);
                    
                    plot.show(ui, |plot_ui| {
                        let x_data: Vec<[f64; 2]> = self.gyro_buffer.iter().enumerate()
                            .map(|(i, (x, _, _))| [i as f64, *x])
                            .collect();
                        let y_data: Vec<[f64; 2]> = self.gyro_buffer.iter().enumerate()
                            .map(|(i, (_, y, _))| [i as f64, *y])
                            .collect();
                        let z_data: Vec<[f64; 2]> = self.gyro_buffer.iter().enumerate()
                            .map(|(i, (_, _, z))| [i as f64, *z])
                            .collect();
                        
                        plot_ui.line(egui_plot::Line::new(egui_plot::PlotPoints::from(x_data))
                            .name("X").color(egui::Color32::RED));
                        plot_ui.line(egui_plot::Line::new(egui_plot::PlotPoints::from(y_data))
                            .name("Y").color(egui::Color32::GREEN));
                        plot_ui.line(egui_plot::Line::new(egui_plot::PlotPoints::from(z_data))
                            .name("Z").color(egui::Color32::BLUE));
                    });
                });
            } else {
                ui.label("No gyroscope data available");
            }
            
            ui.add_space(10.0);
            
            // Magnetic field plot
            ui.label("Magnetic Field (μT)");
            if !self.mag_buffer.is_empty() {
                egui::Frame::canvas(ui.style()).show(ui, |ui| {
                    let plot = egui_plot::Plot::new("mag_plot")
                        .height(150.0)
                        .show_x(false)
                        .show_y(true);
                    
                    plot.show(ui, |plot_ui| {
                        let x_data: Vec<[f64; 2]> = self.mag_buffer.iter().enumerate()
                            .map(|(i, (x, _, _))| [i as f64, *x])
                            .collect();
                        let y_data: Vec<[f64; 2]> = self.mag_buffer.iter().enumerate()
                            .map(|(i, (_, y, _))| [i as f64, *y])
                            .collect();
                        let z_data: Vec<[f64; 2]> = self.mag_buffer.iter().enumerate()
                            .map(|(i, (_, _, z))| [i as f64, *z])
                            .collect();
                        
                        plot_ui.line(egui_plot::Line::new(egui_plot::PlotPoints::from(x_data))
                            .name("X").color(egui::Color32::RED));
                        plot_ui.line(egui_plot::Line::new(egui_plot::PlotPoints::from(y_data))
                            .name("Y").color(egui::Color32::GREEN));
                        plot_ui.line(egui_plot::Line::new(egui_plot::PlotPoints::from(z_data))
                            .name("Z").color(egui::Color32::BLUE));
                    });
                });
            } else {
                ui.label("No magnetometer data available");
            }
            
            // Show current data values
            ui.add_space(10.0);
            ui.separator();
            ui.label(format!("Last timestamp: {} ms", self.last_timestamp));
            ui.label(format!("Acceleration buffer: {} points", self.accel_buffer.len()));
            ui.label(format!("Gyroscope buffer: {} points", self.gyro_buffer.len()));
            ui.label(format!("Magnetometer buffer: {} points", self.mag_buffer.len()));
        } else {
            ui.label("Connect to a device to see IMU data");
        }
    }
}

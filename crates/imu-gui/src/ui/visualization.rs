use eframe::egui;
use crate::state::ImuApp;

impl ImuApp {
    /// Draw a simple 3D orientation cube using 2D projection
    fn draw_3d_cube(ui: &mut egui::Ui, roll: f64, pitch: f64, yaw: f64) {
        let size = 120.0_f32;
        let center = egui::pos2(80.0, 80.0);
        
        // Convert degrees to radians
        let roll_rad = roll.to_radians();
        let pitch_rad = pitch.to_radians();
        let yaw_rad = yaw.to_radians();
        
        // Rotation matrices (simplified for 2D projection)
        let cos_r = roll_rad.cos() as f32;
        let sin_r = roll_rad.sin() as f32;
        let cos_p = pitch_rad.cos() as f32;
        let sin_p = pitch_rad.sin() as f32;
        let cos_y = yaw_rad.cos() as f32;
        let sin_y = yaw_rad.sin() as f32;
        
        // Cube vertices (8 corners)
        let h = size / 2.0;
        let vertices = [
            [-h, -h, -h], [h, -h, -h], [h, h, -h], [-h, h, -h],
            [-h, -h, h], [h, -h, h], [h, h, h], [-h, h, h],
        ];
        
        // Apply rotation and project to 2D
        let projected: Vec<egui::Pos2> = vertices.iter().map(|v| {
            let (x, y, z) = (v[0], v[1], v[2]);
            
            // Rotate around Z (yaw)
            let x1 = x * cos_y - y * sin_y;
            let y1 = x * sin_y + y * cos_y;
            let z1 = z;
            
            // Rotate around X (pitch)
            let y2 = y1 * cos_p - z1 * sin_p;
            let z2 = y1 * sin_p + z1 * cos_p;
            
            // Rotate around Y (roll)
            let x3 = x1 * cos_r + z2 * sin_r;
            let z3 = -x1 * sin_r + z2 * cos_r;
            
            // Simple perspective projection
            let scale = 200.0 / (200.0 + z3);
            let px = center.x + x3 * scale;
            let py = center.y + y2 * scale;
            
            egui::pos2(px, py)
        }).collect();
        
        // Draw cube edges
        let edges = [
            (0,1), (1,2), (2,3), (3,0), // back face
            (4,5), (5,6), (6,7), (7,4), // front face
            (0,4), (1,5), (2,6), (3,7), // connecting edges
        ];
        
        let painter = ui.painter();
        for (i, j) in edges {
            painter.line_segment(
                [projected[i], projected[j]],
                egui::Stroke::new(2.0, egui::Color32::from_rgb(100, 180, 255))
            );
        }
        
        // Draw axes
        let axis_len = size * 0.8;
        let origin = projected[0]; // approximate center
        
        // X axis (red)
        let x_end = egui::pos2(origin.x + axis_len * cos_y, origin.y + axis_len * sin_y);
        painter.line_segment([center, x_end], egui::Stroke::new(2.0, egui::Color32::RED));
        
        // Y axis (green)
        let y_end = egui::pos2(center.x - axis_len * sin_y, center.y + axis_len * cos_y);
        painter.line_segment([center, y_end], egui::Stroke::new(2.0, egui::Color32::GREEN));
        
        // Z axis (blue)
        let z_end = egui::pos2(center.x, center.y - axis_len * 0.5);
        painter.line_segment([center, z_end], egui::Stroke::new(2.0, egui::Color32::BLUE));
    }
    
    pub fn show_visualization_panel(&mut self, ui: &mut egui::Ui) {
        ui.heading("IMU Data Visualization");
        ui.separator();
        
        if self.is_connected {
            // 3D Orientation visualization
            ui.horizontal(|ui| {
                ui.label("3D Orientation:");
                ui.label(format!("Roll: {:.1}° Pitch: {:.1}° Yaw: {:.1}°", 
                    self.current_euler.0, self.current_euler.1, self.current_euler.2));
            });
            
            egui::Frame::canvas(ui.style()).show(ui, |ui| {
                let (rect, _) = ui.allocate_exact_size(egui::vec2(160.0, 160.0), egui::Sense::hover());
                let mut child_ui = ui.child_ui(rect, egui::Layout::top_down(egui::Align::LEFT), None);
                Self::draw_3d_cube(&mut child_ui, self.current_euler.0, self.current_euler.1, self.current_euler.2);
            });
            ui.add_space(10.0);
            
            // Channel selection
            ui.horizontal(|ui| {
                ui.label("Channels:");
                ui.checkbox(&mut self.show_accel, "Acceleration");
                ui.checkbox(&mut self.show_gyro, "Angular Velocity");
                ui.checkbox(&mut self.show_mag, "Magnetic Field");
            });
            ui.add_space(10.0);
            
            // Acceleration plot
            if self.show_accel {
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
            }
            
            // Angular velocity plot
            if self.show_gyro {
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
            }
            
            // Magnetic field plot
            if self.show_mag {
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
                ui.add_space(10.0);
            }
            
            // Show current data values
            ui.separator();
            ui.label(format!("Last timestamp: {} ms", self.last_timestamp));
            ui.label(format!("Acceleration buffer: {} points", self.accel_buffer.len()));
            ui.label(format!("Gyroscope buffer: {} points", self.gyro_buffer.len()));
            ui.label(format!("Magnetometer buffer: {} points", self.mag_buffer.len()));
            
            // Data export
            ui.add_space(10.0);
            ui.separator();
            ui.label("Export Data:");
            ui.horizontal(|ui| {
                if ui.button("Export CSV").clicked() {
                    match self.export_csv("imu_data.csv") {
                        Ok(()) => self.status_message = "Data exported to imu_data.csv".to_string(),
                        Err(e) => self.status_message = format!("Export failed: {}", e),
                    }
                }
                if ui.button("Export JSON").clicked() {
                    match self.export_json("imu_data.json") {
                        Ok(()) => self.status_message = "Data exported to imu_data.json".to_string(),
                        Err(e) => self.status_message = format!("Export failed: {}", e),
                    }
                }
            });
        } else {
            ui.label("Connect to a device to see IMU data");
        }
    }
}

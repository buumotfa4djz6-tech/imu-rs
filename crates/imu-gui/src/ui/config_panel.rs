use eframe::egui;
use crate::state::ImuApp;

impl ImuApp {
    pub fn show_config_panel(&mut self, ui: &mut egui::Ui) {
        ui.heading("Configuration");
        ui.separator();
        
        if self.is_connected {
            ui.label("Sensor Settings");
            ui.add_space(5.0);
            
            // Report rate
            ui.horizontal(|ui| {
                ui.label("Report Rate:");
                if ui.add(egui::DragValue::new(&mut self.config_report_rate)
                    .range(1..=1000)
                    .prefix("")
                    .suffix(" Hz")).changed() {
                    self.config_modified = true;
                }
            });
            
            ui.add_space(5.0);
            
            // Accelerometer range
            ui.horizontal(|ui| {
                ui.label("Accel Range:");
                egui::ComboBox::from_id_source("accel_range")
                    .selected_text(format!("±{}G", self.config_accel_range))
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut self.config_accel_range, 2, "±2G");
                        ui.selectable_value(&mut self.config_accel_range, 4, "±4G");
                        ui.selectable_value(&mut self.config_accel_range, 8, "±8G");
                        ui.selectable_value(&mut self.config_accel_range, 16, "±16G");
                    });
                if ui.add(egui::DragValue::new(&mut self.config_accel_range)
                    .range(2..=16)).changed() {
                    self.config_modified = true;
                }
            });
            
            ui.add_space(5.0);
            
            // Gyroscope range
            ui.horizontal(|ui| {
                ui.label("Gyro Range:");
                egui::ComboBox::from_id_source("gyro_range")
                    .selected_text(format!("±{}°/s", self.config_gyro_range))
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut self.config_gyro_range, 2, "±2°/s");
                        ui.selectable_value(&mut self.config_gyro_range, 4, "±4°/s");
                        ui.selectable_value(&mut self.config_gyro_range, 8, "±8°/s");
                        ui.selectable_value(&mut self.config_gyro_range, 16, "±16°/s");
                    });
                if ui.add(egui::DragValue::new(&mut self.config_gyro_range)
                    .range(2..=16)).changed() {
                    self.config_modified = true;
                }
            });
            
            ui.add_space(5.0);
            
            // Magnetometer range
            ui.horizontal(|ui| {
                ui.label("Mag Range:");
                egui::ComboBox::from_id_source("mag_range")
                    .selected_text(format!("±{} Gauss", self.config_mag_range))
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut self.config_mag_range, 1, "±1 Gauss");
                        ui.selectable_value(&mut self.config_mag_range, 2, "±2 Gauss");
                        ui.selectable_value(&mut self.config_mag_range, 3, "±3 Gauss");
                        ui.selectable_value(&mut self.config_mag_range, 4, "±4 Gauss");
                    });
                if ui.add(egui::DragValue::new(&mut self.config_mag_range)
                    .range(1..=4)).changed() {
                    self.config_modified = true;
                }
            });
            
            ui.add_space(5.0);
            
            // Filter level
            ui.horizontal(|ui| {
                ui.label("Filter Level:");
                if ui.add(egui::Slider::new(&mut self.config_filter_level, 0..=10)).changed() {
                    self.config_modified = true;
                }
            });
            
            ui.add_space(10.0);
            ui.separator();
            ui.add_space(10.0);
            
            // Apply and Save buttons
            ui.horizontal(|ui| {
                if ui.button("Apply").clicked() {
                    self.apply_configuration();
                }
                if ui.button("Save").clicked() {
                    self.save_configuration();
                }
                if ui.button("Load").clicked() {
                    self.load_configuration();
                }
            });
            
            if self.config_modified {
                ui.colored_label(egui::Color32::YELLOW, "⚠ Configuration modified");
            }
            
            ui.add_space(20.0);
            ui.separator();
            ui.add_space(10.0);
            
            // Calibration section
            ui.label("Calibration");
            ui.add_space(5.0);
            
            if ui.button("Calibrate Accelerometer").clicked() {
                self.start_accel_calibration();
            }
            
            if ui.button("Calibrate Gyroscope").clicked() {
                self.start_gyro_calibration();
            }
            
            if ui.button("Calibrate Magnetometer").clicked() {
                self.start_mag_calibration();
            }
            
            if !self.calibration_status.is_empty() {
                ui.add_space(5.0);
                ui.colored_label(egui::Color32::LIGHT_BLUE, &self.calibration_status);
            }
            
            ui.add_space(20.0);
            ui.separator();
            ui.add_space(10.0);
            
            // Device Control section
            ui.label("Device Control");
            ui.add_space(5.0);
            
            if ui.button("📊 Query Device Status").clicked() {
                self.query_device_status();
            }
            
            ui.add_space(5.0);
            ui.horizontal(|ui| {
                if ui.button("▶ Start Auto Report").clicked() {
                    self.start_auto_report();
                }
                if ui.button("⏸ Stop Auto Report").clicked() {
                    self.stop_auto_report();
                }
            });
        } else {
            ui.label("Connect to a device to configure settings");
        }
    }
}

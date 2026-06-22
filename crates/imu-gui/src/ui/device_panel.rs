use eframe::egui;
use crate::state::ImuApp;
use crate::types::ConnectionType;

impl ImuApp {
    pub fn show_device_panel(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        ui.heading("Device Discovery");
        ui.separator();

        // Connection type selection
        ui.horizontal(|ui| {
            ui.radio_value(&mut self.connection_type, ConnectionType::Serial, "Serial");
            ui.radio_value(&mut self.connection_type, ConnectionType::Ble, "BLE");
        });
        ui.add_space(10.0);

        match self.connection_type {
            ConnectionType::Serial => {
                ui.label("Serial Ports:");
                if ui.button("🔄 Refresh").clicked() {
                    self.refresh_serial_ports();
                }
                ui.add_space(5.0);

                if self.serial_ports.is_empty() {
                    ui.label("No serial ports found");
                } else {
                    egui::ComboBox::from_label("Select Port")
                        .selected_text(self.selected_serial_port.clone().unwrap_or_else(|| "None".to_string()))
                        .show_ui(ui, |ui| {
                            for port in &self.serial_ports {
                                ui.selectable_value(&mut self.selected_serial_port, Some(port.clone()), port);
                            }
                        });
                }

                ui.add_space(10.0);
                ui.horizontal(|ui| {
                    ui.label("Baud Rate:");
                    egui::ComboBox::from_id_source("baud_rate")
                        .selected_text(format!("{}", self.baud_rate))
                        .show_ui(ui, |ui| {
                            ui.selectable_value(&mut self.baud_rate, 9600, "9600");
                            ui.selectable_value(&mut self.baud_rate, 115200, "115200");
                            ui.selectable_value(&mut self.baud_rate, 460800, "460800");
                            ui.selectable_value(&mut self.baud_rate, 921600, "921600");
                        });
                });
            }
            ConnectionType::Ble => {
                ui.label("BLE Devices:");
                if ui.button("🔍 Scan").clicked() && !self.scanning_ble {
                    self.scan_ble_devices(ctx);
                }
                ui.add_space(5.0);

                if self.scanning_ble {
                    ui.label("Scanning...");
                } else if self.ble_devices.is_empty() {
                    ui.label("No BLE devices found");
                } else {
                    egui::ComboBox::from_label("Select Device")
                        .selected_text(self.selected_ble_device.clone().unwrap_or_else(|| "None".to_string()))
                        .show_ui(ui, |ui| {
                            for device in &self.ble_devices {
                                let label = device.name.as_deref().unwrap_or("Unknown");
                                let value = &device.address;
                                ui.selectable_value(&mut self.selected_ble_device, Some(value.clone()), 
                                    format!("{} ({})", label, value));
                            }
                        });
                }
            }
        }

        ui.add_space(20.0);
        ui.separator();
        ui.add_space(10.0);

        // Connect/Disconnect buttons
        if self.is_connected {
            if ui.button("🔌 Disconnect").clicked() {
                self.disconnect();
            }
        } else {
            if ui.button("🔗 Connect").clicked() {
                self.connect();
            }
        }
    }
}

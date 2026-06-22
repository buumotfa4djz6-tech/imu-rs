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
                ui.label("BLE Devices (IM948):");
                
                // Scan/Stop button
                if self.scanning_ble {
                    if ui.button("⏹ Stop Scan").clicked() {
                        self.stop_ble_discovery();
                    }
                } else if ui.button("🔍 Start Scan").clicked() {
                    self.start_ble_discovery(ctx);
                }
                ui.add_space(5.0);

                if self.scanning_ble {
                    ui.colored_label(egui::Color32::GREEN, "Scanning...");
                } else if self.ble_devices.is_empty() {
                    ui.label("No IM948 devices found");
                }
                
                // Sort devices by RSSI (strongest first), then by name
                let mut devices: Vec<_> = self.ble_devices.values().collect();
                devices.sort_by(|a, b| {
                    // Sort by RSSI descending (None last), then by name
                    match (a.rssi, b.rssi) {
                        (Some(r1), Some(r2)) => r2.cmp(&r1),
                        (Some(_), None) => std::cmp::Ordering::Less,
                        (None, Some(_)) => std::cmp::Ordering::Greater,
                        (None, None) => a.name.cmp(&b.name),
                    }
                });

                if !devices.is_empty() {
                    ui.label(format!("{} device(s) found:", devices.len()));
                    ui.add_space(5.0);
                    
                    egui::ScrollArea::vertical().max_height(200.0).show(ui, |ui| {
                        for device in &devices {
                            let label = device.name.as_deref().unwrap_or("Unknown");
                            let rssi_str = device.rssi.map(|r| format!("{} dBm", r)).unwrap_or_else(|| "N/A".to_string());
                            let addr = &device.address;
                            let display = format!("{} ({}) [{}]", label, addr, rssi_str);
                            
                            if ui.selectable_label(
                                self.selected_ble_device.as_deref() == Some(addr.as_str()),
                                &display
                            ).clicked() {
                                self.selected_ble_device = Some(addr.clone());
                            }
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
        } else if ui.button("🔗 Connect").clicked() {
            self.connect();
        }
    }
}

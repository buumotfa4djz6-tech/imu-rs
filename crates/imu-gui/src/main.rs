use eframe::egui;
use imu_transport::{BleTransport, BleDeviceInfo, list_serial_ports};

/// Connection type
#[derive(Debug, Clone, PartialEq)]
enum ConnectionType {
    Serial,
    Ble,
}

/// Application state
struct ImuApp {
    // Connection state
    connection_type: ConnectionType,
    is_connected: bool,
    
    // Serial port state
    serial_ports: Vec<String>,
    selected_serial_port: Option<String>,
    baud_rate: u32,
    
    // BLE state
    ble_devices: Vec<BleDeviceInfo>,
    selected_ble_device: Option<String>,
    scanning_ble: bool,
    
    // Status
    status_message: String,
    battery_level: Option<u8>,
    
    // Runtime for async operations
    rt: tokio::runtime::Runtime,
}

impl Default for ImuApp {
    fn default() -> Self {
        Self {
            connection_type: ConnectionType::Serial,
            is_connected: false,
            serial_ports: Vec::new(),
            selected_serial_port: None,
            baud_rate: 115200,
            ble_devices: Vec::new(),
            selected_ble_device: None,
            scanning_ble: false,
            status_message: "Ready".to_string(),
            battery_level: None,
            rt: tokio::runtime::Runtime::new().unwrap(),
        }
    }
}

impl ImuApp {
    /// Refresh serial port list
    fn refresh_serial_ports(&mut self) {
        self.serial_ports = list_serial_ports()
            .into_iter()
            .map(|p| p.name)
            .collect();
        self.status_message = format!("Found {} serial port(s)", self.serial_ports.len());
    }

    /// Scan for BLE devices
    fn scan_ble_devices(&mut self, _ctx: &egui::Context) {
        self.scanning_ble = true;
        self.status_message = "Scanning for BLE devices...".to_string();
        
        // Perform synchronous scan
        match self.rt.block_on(BleTransport::discover_devices()) {
            Ok(devices) => {
                self.ble_devices = devices;
                self.status_message = format!("Found {} BLE device(s)", self.ble_devices.len());
            }
            Err(e) => {
                self.status_message = format!("BLE scan error: {}", e);
            }
        }
        
        self.scanning_ble = false;
    }

    /// Connect to selected device
    fn connect(&mut self) {
        match self.connection_type {
            ConnectionType::Serial => {
                if let Some(port) = &self.selected_serial_port {
                    self.status_message = format!("Connecting to {}...", port);
                    // TODO: Implement actual connection
                    self.is_connected = true;
                    self.status_message = format!("Connected to {}", port);
                } else {
                    self.status_message = "Please select a serial port".to_string();
                }
            }
            ConnectionType::Ble => {
                if let Some(device) = &self.selected_ble_device {
                    self.status_message = format!("Connecting to {}...", device);
                    // TODO: Implement actual connection
                    self.is_connected = true;
                    self.status_message = format!("Connected to {}", device);
                } else {
                    self.status_message = "Please select a BLE device".to_string();
                }
            }
        }
    }

    /// Disconnect from device
    fn disconnect(&mut self) {
        self.is_connected = false;
        self.status_message = "Disconnected".to_string();
        self.battery_level = None;
    }
}

impl eframe::App for ImuApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
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
        });

        // Central panel for data visualization (placeholder for Slice 7)
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("IMU Data Visualization");
            ui.separator();
            
            if self.is_connected {
                ui.label("Data visualization will appear here...");
                ui.label("(To be implemented in Slice 7)");
            } else {
                ui.label("Connect to a device to see IMU data");
            }
        });
    }
}

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

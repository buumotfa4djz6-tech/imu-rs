use eframe::egui;
use imu_transport::{BleTransport, BleDeviceInfo, list_serial_ports};
use imu_core::ImuReading;
use std::collections::VecDeque;

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
    
    // Data visualization (Slice 7)
    accel_buffer: VecDeque<(f64, f64, f64)>,  // (x, y, z) acceleration data
    gyro_buffer: VecDeque<(f64, f64, f64)>,   // (x, y, z) gyroscope data
    mag_buffer: VecDeque<(f64, f64, f64)>,    // (x, y, z) magnetometer data
    data_buffer_len: usize,
    last_timestamp: u32,
    
    // Configuration (Slice 8)
    show_config_panel: bool,
    config_report_rate: u16,
    config_accel_range: u8,
    config_gyro_range: u8,
    config_mag_range: u8,
    config_filter_level: u8,
    config_modified: bool,
    
    // Calibration state
    calibrating_accel: bool,
    calibrating_gyro: bool,
    calibrating_mag: bool,
    calibration_status: String,
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
            accel_buffer: VecDeque::with_capacity(1000),
            gyro_buffer: VecDeque::with_capacity(1000),
            mag_buffer: VecDeque::with_capacity(1000),
            data_buffer_len: 1000,
            last_timestamp: 0,
            show_config_panel: false,
            config_report_rate: 100,
            config_accel_range: 2,
            config_gyro_range: 4,
            config_mag_range: 3,
            config_filter_level: 2,
            config_modified: false,
            calibrating_accel: false,
            calibrating_gyro: false,
            calibrating_mag: false,
            calibration_status: String::new(),
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
        // Clear data buffers
        self.accel_buffer.clear();
        self.gyro_buffer.clear();
        self.mag_buffer.clear();
    }
    
    /// Process incoming IMU reading and update visualization buffers
    pub fn process_imu_reading(&mut self, reading: &ImuReading) {
        // Extract acceleration data
        if let Some(accel) = reading.acceleration {
            let point = (accel.x as f64, accel.y as f64, accel.z as f64);
            self.accel_buffer.push_back(point);
            if self.accel_buffer.len() > self.data_buffer_len {
                self.accel_buffer.pop_front();
            }
        }
        
        // Extract gyroscope data
        if let Some(gyro) = reading.angular_velocity {
            let point = (gyro.x as f64, gyro.y as f64, gyro.z as f64);
            self.gyro_buffer.push_back(point);
            if self.gyro_buffer.len() > self.data_buffer_len {
                self.gyro_buffer.pop_front();
            }
        }
        
        // Extract magnetometer data
        if let Some(mag) = reading.magnetic_field {
            let point = (mag.x as f64, mag.y as f64, mag.z as f64);
            self.mag_buffer.push_back(point);
            if self.mag_buffer.len() > self.data_buffer_len {
                self.mag_buffer.pop_front();
            }
        }
        
        self.last_timestamp = reading.timestamp_ms;
    }
    
    // Slice 8 methods: Configuration and Calibration
    
    pub fn start_accel_calibration(&mut self) {
        self.calibrating_accel = true;
        self.calibration_status = "Accelerometer calibration in progress...".to_string();
        // In a real implementation, this would send a calibration command to the device
        // For now, just simulate the process
    }
    
    pub fn start_gyro_calibration(&mut self) {
        self.calibrating_gyro = true;
        self.calibration_status = "Gyroscope calibration in progress...".to_string();
        // In a real implementation, this would send a calibration command to the device
    }
    
    pub fn start_mag_calibration(&mut self) {
        self.calibrating_mag = true;
        self.calibration_status = "Magnetometer calibration in progress...".to_string();
        // In a real implementation, this would send a calibration command to the device
    }
    
    pub fn apply_configuration(&mut self) {
        self.config_modified = false;
        self.status_message = format!(
            "Configuration applied: {}Hz, Accel:{}G, Gyro:{}°/s, Mag:{}Gauss",
            self.config_report_rate,
            self.config_accel_range,
            self.config_gyro_range,
            self.config_mag_range
        );
        // In a real implementation, this would send configuration commands to the device
    }
    
    pub fn save_configuration(&self) {
        // Save configuration to file
        let config = format!(
            "{{\n  \"report_rate\": {},\n  \"accel_range\": {},\n  \"gyro_range\": {},\n  \"mag_range\": {},\n  \"filter_level\": {}\n}}",
            self.config_report_rate,
            self.config_accel_range,
            self.config_gyro_range,
            self.config_mag_range,
            self.config_filter_level
        );
        
        if let Ok(mut file) = std::fs::File::create("imu_config.json") {
            use std::io::Write;
            let _ = file.write_all(config.as_bytes());
        }
    }
    
    pub fn load_configuration(&mut self) {
        // Load configuration from file
        if let Ok(content) = std::fs::read_to_string("imu_config.json") {
            // Simple JSON parsing (in a real app, use serde_json)
            if let Some(rate) = content.find("\"report_rate\":") {
                if let Some(end) = content[rate..].find(',') {
                    if let Ok(val) = content[rate + 14..rate + end].trim().parse() {
                        self.config_report_rate = val;
                    }
                }
            }
            // Similar parsing for other fields...
            self.status_message = "Configuration loaded".to_string();
        }
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

        // Right panel for configuration (Slice 8)
        egui::SidePanel::right("config_panel").show(ctx, |ui| {
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
            } else {
                ui.label("Connect to a device to configure settings");
            }
        });

        // Central panel for data visualization (Slice 7)
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("IMU Data Visualization");
            ui.separator();
            
            if self.is_connected {
                // Add tabs for different data types
                egui::TopBottomPanel::top("data_tabs").show_inside(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.label("Acceleration");
                        ui.label("Angular Velocity");
                        ui.label("Magnetic Field");
                        ui.label("Orientation");
                    });
                });
                
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

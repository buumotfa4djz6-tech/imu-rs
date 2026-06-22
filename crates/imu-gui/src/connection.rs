use eframe::egui;
use imu_transport::{BleManager, BleTransport, Device, SerialTransport, list_serial_ports};
use std::sync::Arc;
use tokio::sync::{Mutex as TokioMutex, mpsc};

use crate::background::background_task;
use crate::state::ImuApp;
use crate::types::*;

impl ImuApp {
    /// Refresh serial port list
    pub fn refresh_serial_ports(&mut self) {
        self.serial_ports = list_serial_ports()
            .into_iter()
            .map(|p| p.name)
            .collect();
        self.status_message = format!("Found {} serial port(s)", self.serial_ports.len());
    }

    /// Scan for BLE devices
    pub fn scan_ble_devices(&mut self, _ctx: &egui::Context) {
        self.scanning_ble = true;
        self.status_message = "Scanning for BLE devices...".to_string();
        
        // Initialize BLE manager if not already done
        if self.ble_manager.is_none() {
            match self.rt.block_on(async {
                let mut manager = BleManager::new();
                manager.init().await?;
                Ok::<_, imu_transport::TransportError>(manager)
            }) {
                Ok(manager) => {
                    self.ble_manager = Some(Arc::new(TokioMutex::new(manager)));
                }
                Err(e) => {
                    self.status_message = format!("BLE init error: {}", e);
                    self.scanning_ble = false;
                    return;
                }
            }
        }
        
        // Perform scan
        if let Some(manager) = &self.ble_manager {
            let manager_clone = manager.clone();
            match self.rt.block_on(async {
                manager_clone.lock().await.scan(3).await
            }) {
                Ok(devices) => {
                    self.ble_devices = devices;
                    self.status_message = format!("Found {} BLE device(s)", self.ble_devices.len());
                }
                Err(e) => {
                    self.status_message = format!("BLE scan error: {}", e);
                }
            }
        }
        
        self.scanning_ble = false;
    }

    /// Connect to selected device
    pub fn connect(&mut self) {
        match self.connection_type {
            ConnectionType::Serial => {
                if let Some(port_name) = self.selected_serial_port.clone() {
                    self.status_message = format!("Connecting to {}...", port_name);
                    
                    // Create channels
                    let (command_tx, command_rx) = mpsc::channel::<DeviceCommand>(32);
                    let (data_tx, data_rx) = mpsc::channel::<imu_core::ImuReading>(100);
                    
                    // Create transport and device
                    let transport = SerialTransport::new(&port_name, self.baud_rate);
                    let device = Device::new(transport);
                    
                    // Spawn background task
                    let rt = self.rt.handle().clone();
                    rt.spawn(async move {
                        background_task(device, command_rx, data_tx).await;
                    });
                    
                    // Send connect command
                    self.rt.block_on(command_tx.send(DeviceCommand::Connect(ConnectionParams::Serial {
                        port: port_name.clone(),
                        baud_rate: self.baud_rate,
                    }))).unwrap();
                    
                    // Store state
                    self.command_tx = Some(command_tx);
                    self.data_rx = Some(data_rx);
                    self.is_connected = true;
                    self.status_message = format!("Connected to {}", port_name);
                } else {
                    self.status_message = "Please select a serial port".to_string();
                }
            }
            ConnectionType::Ble => {
                if let Some(device_addr) = self.selected_ble_device.clone() {
                    self.status_message = format!("Connecting to {}...", device_addr);
                    
                    // Check if we have a BLE manager
                    if let Some(manager) = &self.ble_manager {
                        // Get the peripheral from manager
                        let manager_clone = manager.clone();
                        let addr_clone = device_addr.clone();
                        
                        match self.rt.block_on(async {
                            let mgr = manager_clone.lock().await;
                            mgr.get_peripheral(&addr_clone).await
                        }) {
                            Some(peripheral) => {
                                // Create channels
                                let (command_tx, command_rx) = mpsc::channel::<DeviceCommand>(32);
                                let (data_tx, data_rx) = mpsc::channel::<imu_core::ImuReading>(100);
                                
                                // Create transport and device
                                let mut transport = BleTransport::new(&device_addr);
                                
                                // Connect using the peripheral
                                match self.rt.block_on(transport.connect_with_peripheral(peripheral)) {
                                    Ok(()) => {
                                        let device = Device::new(transport);
                                        
                                        // Spawn background task
                                        let rt = self.rt.handle().clone();
                                        rt.spawn(async move {
                                            background_task(device, command_rx, data_tx).await;
                                        });
                                        
                                        // Store state
                                        self.command_tx = Some(command_tx);
                                        self.data_rx = Some(data_rx);
                                        self.is_connected = true;
                                        self.status_message = format!("Connected to {}", device_addr);
                                    }
                                    Err(e) => {
                                        self.status_message = format!("BLE connect error: {}", e);
                                    }
                                }
                            }
                            None => {
                                self.status_message = "BLE device not found. Please scan again.".to_string();
                            }
                        }
                    } else {
                        self.status_message = "BLE manager not initialized. Please scan first.".to_string();
                    }
                } else {
                    self.status_message = "Please select a BLE device".to_string();
                }
            }
        }
    }

    /// Disconnect from device
    pub fn disconnect(&mut self) {
        // Send disconnect command to background task
        if let Some(command_tx) = &self.command_tx {
            let _ = self.rt.block_on(command_tx.send(DeviceCommand::Disconnect));
        }
        
        // Clear state
        self.command_tx = None;
        self.data_rx = None;
        self.is_connected = false;
        self.status_message = "Disconnected".to_string();
        self.battery_level = None;
        
        // Clear data buffers
        self.accel_buffer.clear();
        self.gyro_buffer.clear();
        self.mag_buffer.clear();
    }
}

use imu_core::ImuReading;
use imu_transport::{BleDeviceInfo, BleDeviceEvent, BleManager};
use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use tokio::sync::{Mutex as TokioMutex, mpsc, oneshot};

use crate::types::*;

/// Application state
pub struct ImuApp {
    // Connection state
    pub connection_type: ConnectionType,
    pub is_connected: bool,
    
    // Channels for async communication
    pub data_rx: Option<mpsc::Receiver<ImuReading>>,
    pub command_tx: Option<mpsc::Sender<DeviceCommand>>,
    
    // Serial port state
    pub serial_ports: Vec<String>,
    pub selected_serial_port: Option<String>,
    pub baud_rate: u32,
    
    // BLE state
    pub ble_manager: Option<Arc<TokioMutex<BleManager>>>,
    pub ble_devices: HashMap<String, BleDeviceInfo>,
    pub selected_ble_device: Option<String>,
    pub scanning_ble: bool,
    pub ble_discovery_rx: Option<mpsc::Receiver<BleDeviceEvent>>,
    pub ble_stop_tx: Option<oneshot::Sender<()>>,
    pub ble_filter: String,  // Filter by device name or MAC address
    
    // Status
    pub status_message: String,
    pub battery_level: Option<u8>,
    
    // Runtime for async operations
    pub rt: tokio::runtime::Runtime,
    
    // Data visualization (Slice 7)
    pub accel_buffer: VecDeque<(f64, f64, f64)>,  // (x, y, z) acceleration data
    pub gyro_buffer: VecDeque<(f64, f64, f64)>,   // (x, y, z) gyroscope data
    pub mag_buffer: VecDeque<(f64, f64, f64)>,    // (x, y, z) magnetometer data
    pub data_buffer_len: usize,
    pub last_timestamp: u32,
    
    // Orientation for 3D visualization
    pub current_euler: (f64, f64, f64),  // (roll, pitch, yaw) in degrees
    
    // Configuration (Slice 8)
    pub config_report_rate: u16,
    pub config_accel_range: u8,
    pub config_gyro_range: u8,
    pub config_mag_range: u8,
    pub config_filter_level: u8,
    pub config_modified: bool,
    
    // Calibration state
    pub calibrating_accel: bool,
    pub calibrating_gyro: bool,
    pub calibrating_mag: bool,
    pub calibration_status: String,
    
    // Channel visibility
    pub show_accel: bool,
    pub show_gyro: bool,
    pub show_mag: bool,
}

impl Default for ImuApp {
    fn default() -> Self {
        Self {
            connection_type: ConnectionType::Serial,
            is_connected: false,
            data_rx: None,
            command_tx: None,
            serial_ports: Vec::new(),
            selected_serial_port: None,
            baud_rate: 115200,
            ble_manager: None,
            ble_devices: HashMap::new(),
            selected_ble_device: None,
            scanning_ble: false,
            ble_discovery_rx: None,
            ble_stop_tx: None,
            ble_filter: String::new(),
            status_message: "Ready".to_string(),
            battery_level: None,
            rt: tokio::runtime::Runtime::new().unwrap(),
            accel_buffer: VecDeque::with_capacity(1000),
            gyro_buffer: VecDeque::with_capacity(1000),
            mag_buffer: VecDeque::with_capacity(1000),
            data_buffer_len: 1000,
            last_timestamp: 0,
            current_euler: (0.0, 0.0, 0.0),
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
            show_accel: true,
            show_gyro: true,
            show_mag: true,
        }
    }
}

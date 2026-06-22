use imu_transport::{BleTransport, Device, SerialTransport};

/// Connection type
#[derive(Debug, Clone, PartialEq)]
pub enum ConnectionType {
    Serial,
    Ble,
}

/// Connected device (holds the actual transport)
#[allow(dead_code)]
pub enum ConnectedDevice {
    Serial(Device<SerialTransport>),
    Ble(Device<BleTransport>),
}

/// Commands that can be sent to the background task
pub enum DeviceCommand {
    Connect(ConnectionParams),
    Disconnect,
    SetConfig(DeviceConfig),
    Calibrate(CalibrationType),
    QueryStatus,
    StartAutoReport,
    StopAutoReport,
}

#[allow(dead_code)]
pub enum ConnectionParams {
    Serial { port: String, baud_rate: u32 },
    Ble { address: String },
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct DeviceConfig {
    pub report_rate: u16,
    pub accel_range: u8,
    pub gyro_range: u8,
    pub mag_range: u8,
    pub filter_level: u8,
}

#[derive(Debug, Clone)]
pub enum CalibrationType {
    Accelerometer,
    Gyroscope,
    Magnetometer,
}

use crate::state::ImuApp;
use crate::types::*;

impl ImuApp {
    pub fn start_accel_calibration(&mut self) {
        self.calibrating_accel = true;
        self.calibration_status = "Accelerometer calibration in progress...".to_string();
        
        if let Some(command_tx) = &self.command_tx {
            let _ = self.rt.block_on(command_tx.send(DeviceCommand::Calibrate(CalibrationType::Accelerometer)));
        }
    }
    
    pub fn start_gyro_calibration(&mut self) {
        self.calibrating_gyro = true;
        self.calibration_status = "Gyroscope calibration in progress...".to_string();
        
        if let Some(command_tx) = &self.command_tx {
            let _ = self.rt.block_on(command_tx.send(DeviceCommand::Calibrate(CalibrationType::Gyroscope)));
        }
    }
    
    pub fn start_mag_calibration(&mut self) {
        self.calibrating_mag = true;
        self.calibration_status = "Magnetometer calibration in progress...".to_string();
        
        if let Some(command_tx) = &self.command_tx {
            let _ = self.rt.block_on(command_tx.send(DeviceCommand::Calibrate(CalibrationType::Magnetometer)));
        }
    }
    
    pub fn apply_configuration(&mut self) {
        let config = DeviceConfig {
            report_rate: self.config_report_rate,
            accel_range: self.config_accel_range,
            gyro_range: self.config_gyro_range,
            mag_range: self.config_mag_range,
            filter_level: self.config_filter_level,
        };
        
        if let Some(command_tx) = &self.command_tx {
            let _ = self.rt.block_on(command_tx.send(DeviceCommand::SetConfig(config)));
        }
        
        self.config_modified = false;
        self.status_message = format!(
            "Configuration applied: {}Hz, Accel:{}G, Gyro:{}°/s, Mag:{}Gauss",
            self.config_report_rate,
            self.config_accel_range,
            self.config_gyro_range,
            self.config_mag_range
        );
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
    
    pub fn query_device_status(&mut self) {
        if let Some(command_tx) = &self.command_tx {
            let _ = self.rt.block_on(command_tx.send(DeviceCommand::QueryStatus));
            self.status_message = "Querying device status...".to_string();
        }
    }
    
    pub fn start_auto_report(&mut self) {
        if let Some(command_tx) = &self.command_tx {
            let _ = self.rt.block_on(command_tx.send(DeviceCommand::StartAutoReport));
            self.status_message = "Starting auto report...".to_string();
        }
    }
    
    pub fn stop_auto_report(&mut self) {
        if let Some(command_tx) = &self.command_tx {
            let _ = self.rt.block_on(command_tx.send(DeviceCommand::StopAutoReport));
            self.status_message = "Stopping auto report...".to_string();
        }
    }
}

use imu_core::ImuReading;

use crate::state::ImuApp;

impl ImuApp {
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
        
        // Extract euler angles for 3D visualization
        if let Some(euler) = reading.euler_angles {
            self.current_euler = (euler.x as f64, euler.y as f64, euler.z as f64);
        }
        
        self.last_timestamp = reading.timestamp_ms;
    }
    
    /// Export data to CSV format
    pub fn export_csv(&self, path: &str) -> Result<(), String> {
        use std::io::Write;
        
        let max_len = self.accel_buffer.len()
            .max(self.gyro_buffer.len())
            .max(self.mag_buffer.len());
        
        let mut file = std::fs::File::create(path)
            .map_err(|e| format!("Failed to create file: {}", e))?;
        
        // Write header
        writeln!(file, "timestamp_ms,accel_x,accel_y,accel_z,gyro_x,gyro_y,gyro_z,mag_x,mag_y,mag_z")
            .map_err(|e| format!("Failed to write header: {}", e))?;
        
        // Write data rows
        for i in 0..max_len {
            let accel = self.accel_buffer.get(i).copied().unwrap_or((0.0, 0.0, 0.0));
            let gyro = self.gyro_buffer.get(i).copied().unwrap_or((0.0, 0.0, 0.0));
            let mag = self.mag_buffer.get(i).copied().unwrap_or((0.0, 0.0, 0.0));
            
            writeln!(file, "{},{},{},{},{},{},{},{},{},{}",
                self.last_timestamp.saturating_sub((max_len - i) as u32),
                accel.0, accel.1, accel.2,
                gyro.0, gyro.1, gyro.2,
                mag.0, mag.1, mag.2
            ).map_err(|e| format!("Failed to write row: {}", e))?;
        }
        
        Ok(())
    }
    
    /// Export data to JSON format
    pub fn export_json(&self, path: &str) -> Result<(), String> {
        let max_len = self.accel_buffer.len()
            .max(self.gyro_buffer.len())
            .max(self.mag_buffer.len());
        
        let mut json = String::from("{\n  \"data\": [\n");
        
        for i in 0..max_len {
            let accel = self.accel_buffer.get(i).copied().unwrap_or((0.0, 0.0, 0.0));
            let gyro = self.gyro_buffer.get(i).copied().unwrap_or((0.0, 0.0, 0.0));
            let mag = self.mag_buffer.get(i).copied().unwrap_or((0.0, 0.0, 0.0));
            
            json.push_str(&format!(
                "    {{\"timestamp\": {}, \"accel\": [{:.6}, {:.6}, {:.6}], \"gyro\": [{:.6}, {:.6}, {:.6}], \"mag\": [{:.6}, {:.6}, {:.6}]}}",
                self.last_timestamp.saturating_sub((max_len - i) as u32),
                accel.0, accel.1, accel.2,
                gyro.0, gyro.1, gyro.2,
                mag.0, mag.1, mag.2
            ));
            
            if i < max_len - 1 {
                json.push(',');
            }
            json.push('\n');
        }
        
        json.push_str("  ]\n}");
        
        std::fs::write(path, json)
            .map_err(|e| format!("Failed to write JSON: {}", e))?;
        
        Ok(())
    }
}

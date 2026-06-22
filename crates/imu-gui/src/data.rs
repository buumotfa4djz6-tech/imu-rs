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
        
        self.last_timestamp = reading.timestamp_ms;
    }
}

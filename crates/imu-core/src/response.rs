use crate::types::*;
use crate::error::ProtocolError;

/// IM948 response definitions
#[derive(Debug, Clone, PartialEq)]
pub enum ImuResponse {
    // 0x02 - Sleep Ack
    SleepAck,
    
    // 0x03 - Wake Ack
    WakeAck,
    
    // 0x04 - Compass Calibration End
    CompassCalibrationEnd,
    
    // 0x05 - Z-axis Zeroed
    ZeroZAngleAck,
    
    // 0x06 - World XYZ Zeroed
    ZeroWorldXYZAck,
    
    // 0x07 - Accelerometer Calibration In Progress (9 seconds)
    SimpleAccelCalibrationInProgress,
    
    // 0x08 - Axes Reset
    ResetAxesAck,
    
    // 0x10 - Device Status
    DeviceStatus(DeviceStatus),
    
    // 0x11 - Sensor Data Report
    SensorData(ImuReading),
    
    // 0x12 - Set Parameters Ack
    SetParamsAck,
    
    // 0x13 - INS Position Cleared
    ClearPositionAck,
    
    // 0x14 - Factory Calibration Restored
    RestoreFactoryCalibrationAck,
    
    // 0x15 - Factory Calibration Saved
    SaveFactoryCalibrationAck,
    
    // 0x16 - Step Count Cleared
    ClearStepCountAck,
    
    // 0x17 - High-Precision Calibration Status
    CalibrationStatus(CalibrationStatus),
    
    // 0x18 - Auto Report Off
    AutoReportOff,
    
    // 0x19 - Auto Report On
    AutoReportOn,
    
    // Unknown response
    Unknown { tag: u8, payload: Vec<u8> },
}

/// Device status response (0x10)
#[derive(Debug, Clone, PartialEq)]
pub struct DeviceStatus {
    pub still_threshold: u8,
    pub still_zero_speed: u8,
    pub move_zero_speed: u8,
    pub compass_on: bool,
    pub barometer_filter: u8,
    pub imu_on: bool,
    pub auto_report_on: bool,
    pub fps: u8,
    pub gyro_filter: u8,
    pub accel_filter: u8,
    pub compass_filter: u8,
    pub subscription_tag: u16,
    pub charged_state: u8,
    pub battery_level: u8,
    pub battery_voltage_mv: u16,
    pub mac_address: [u8; 6],
    pub firmware_version: String,
    pub product_model: String,
}

/// Calibration status (0x17)
#[derive(Debug, Clone, PartialEq)]
pub enum CalibrationStatus {
    Running,
    Collected(u8),  // 1-250 points collected
    Finishing,      // 255 - finishing calibration
    GyroError,      // 254
    AccelError,     // 253
    CompassError,   // 252
    NotStarted,     // 251
}

/// Parse response from device
pub fn parse_response(data: &[u8]) -> Result<ImuResponse, ProtocolError> {
    if data.is_empty() {
        return Err(ProtocolError::TooShort { expected: 1, actual: 0 });
    }
    
    let tag = data[0];
    
    match tag {
        0x02 => Ok(ImuResponse::SleepAck),
        0x03 => Ok(ImuResponse::WakeAck),
        0x04 => Ok(ImuResponse::CompassCalibrationEnd),
        0x05 => Ok(ImuResponse::ZeroZAngleAck),
        0x06 => Ok(ImuResponse::ZeroWorldXYZAck),
        0x07 => Ok(ImuResponse::SimpleAccelCalibrationInProgress),
        0x08 => Ok(ImuResponse::ResetAxesAck),
        0x10 => parse_device_status(data),
        0x11 => {
            let reading = crate::parser::parse_imu_reading(data)?;
            Ok(ImuResponse::SensorData(reading))
        }
        0x12 => Ok(ImuResponse::SetParamsAck),
        0x13 => Ok(ImuResponse::ClearPositionAck),
        0x14 => Ok(ImuResponse::RestoreFactoryCalibrationAck),
        0x15 => Ok(ImuResponse::SaveFactoryCalibrationAck),
        0x16 => Ok(ImuResponse::ClearStepCountAck),
        0x17 => parse_calibration_status(data),
        0x18 => Ok(ImuResponse::AutoReportOff),
        0x19 => Ok(ImuResponse::AutoReportOn),
        _ => Ok(ImuResponse::Unknown {
            tag,
            payload: data[1..].to_vec(),
        }),
    }
}

fn parse_device_status(data: &[u8]) -> Result<ImuResponse, ProtocolError> {
    if data.len() < 33 {
        return Err(ProtocolError::TooShort { expected: 33, actual: data.len() });
    }
    
    let still_threshold = data[1];
    let still_zero_speed = data[2];
    let move_zero_speed = data[3];
    
    let byte4 = data[4];
    let compass_on = (byte4 & 0x01) != 0;
    let barometer_filter = (byte4 >> 1) & 0x03;
    let imu_on = ((byte4 >> 3) & 0x01) != 0;
    let auto_report_on = ((byte4 >> 4) & 0x01) != 0;
    
    let fps = data[5];
    let gyro_filter = data[6];
    let accel_filter = data[7];
    let compass_filter = data[8];
    
    let subscription_tag = u16::from_le_bytes([data[9], data[10]]);
    
    let charged_state = data[11];
    let battery_level = data[12];
    let battery_voltage_mv = u16::from_le_bytes([data[13], data[14]]);
    
    let mac_address = [data[15], data[16], data[17], data[18], data[19], data[20]];
    
    // Firmware version: 6 bytes string (null-terminated)
    let firmware_version = String::from_utf8_lossy(&data[21..27])
        .trim_end_matches('\0')
        .to_string();
    
    // Product model: 6 bytes string (null-terminated)
    let product_model = String::from_utf8_lossy(&data[27..33])
        .trim_end_matches('\0')
        .to_string();
    
    Ok(ImuResponse::DeviceStatus(DeviceStatus {
        still_threshold,
        still_zero_speed,
        move_zero_speed,
        compass_on,
        barometer_filter,
        imu_on,
        auto_report_on,
        fps,
        gyro_filter,
        accel_filter,
        compass_filter,
        subscription_tag,
        charged_state,
        battery_level,
        battery_voltage_mv,
        mac_address,
        firmware_version,
        product_model,
    }))
}

fn parse_calibration_status(data: &[u8]) -> Result<ImuResponse, ProtocolError> {
    if data.len() < 2 {
        return Err(ProtocolError::TooShort { expected: 2, actual: data.len() });
    }
    
    let status = match data[1] {
        0 => CalibrationStatus::Running,
        1..=250 => CalibrationStatus::Collected(data[1]),
        251 => CalibrationStatus::NotStarted,
        252 => CalibrationStatus::CompassError,
        253 => CalibrationStatus::AccelError,
        254 => CalibrationStatus::GyroError,
        255 => CalibrationStatus::Finishing,
    };
    
    Ok(ImuResponse::CalibrationStatus(status))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_responses() {
        assert_eq!(parse_response(&[0x02]).unwrap(), ImuResponse::SleepAck);
        assert_eq!(parse_response(&[0x03]).unwrap(), ImuResponse::WakeAck);
        assert_eq!(parse_response(&[0x04]).unwrap(), ImuResponse::CompassCalibrationEnd);
        assert_eq!(parse_response(&[0x05]).unwrap(), ImuResponse::ZeroZAngleAck);
        assert_eq!(parse_response(&[0x06]).unwrap(), ImuResponse::ZeroWorldXYZAck);
        assert_eq!(parse_response(&[0x07]).unwrap(), ImuResponse::SimpleAccelCalibrationInProgress);
        assert_eq!(parse_response(&[0x08]).unwrap(), ImuResponse::ResetAxesAck);
        assert_eq!(parse_response(&[0x12]).unwrap(), ImuResponse::SetParamsAck);
        assert_eq!(parse_response(&[0x13]).unwrap(), ImuResponse::ClearPositionAck);
        assert_eq!(parse_response(&[0x14]).unwrap(), ImuResponse::RestoreFactoryCalibrationAck);
        assert_eq!(parse_response(&[0x15]).unwrap(), ImuResponse::SaveFactoryCalibrationAck);
        assert_eq!(parse_response(&[0x16]).unwrap(), ImuResponse::ClearStepCountAck);
        assert_eq!(parse_response(&[0x18]).unwrap(), ImuResponse::AutoReportOff);
        assert_eq!(parse_response(&[0x19]).unwrap(), ImuResponse::AutoReportOn);
    }

    #[test]
    fn test_parse_device_status() {
        // Construct a 33-byte device status response
        let mut data = vec![0x10];
        data.push(5);       // still_threshold
        data.push(255);     // still_zero_speed
        data.push(0);       // move_zero_speed
        // byte4: compass_on=1(bit0), barometer_filter=2(bits1-2), imu_on=1(bit3), auto_report_on=1(bit4)
        // = 0x01 | (2<<1) | 0x08 | 0x10 = 0x01 | 0x04 | 0x08 | 0x10 = 0x1D
        data.push(0x1D);
        data.push(60);      // fps
        data.push(1);       // gyro_filter
        data.push(3);       // accel_filter
        data.push(5);       // compass_filter
        data.extend_from_slice(&[0xFF, 0x0F]); // subscription_tag = 0x0FFF
        data.push(2);       // charged_state (charging)
        data.push(85);      // battery_level 85%
        data.extend_from_slice(&[0x10, 0x0E]); // battery_voltage_mv = 3600
        data.extend_from_slice(&[0xAA, 0xBB, 0xCC, 0xDD, 0xEE, 0xFF]); // MAC
        data.extend_from_slice(b"V1.2.3");     // firmware version
        data.extend_from_slice(b"IM948\x00\x00");     // product model (null-terminated)
        
        let response = parse_response(&data).unwrap();
        
        if let ImuResponse::DeviceStatus(status) = response {
            assert_eq!(status.still_threshold, 5);
            assert_eq!(status.still_zero_speed, 255);
            assert_eq!(status.move_zero_speed, 0);
            assert!(status.compass_on);
            assert_eq!(status.barometer_filter, 2);
            assert!(status.imu_on);
            assert!(status.auto_report_on);
            assert_eq!(status.fps, 60);
            assert_eq!(status.gyro_filter, 1);
            assert_eq!(status.accel_filter, 3);
            assert_eq!(status.compass_filter, 5);
            assert_eq!(status.subscription_tag, 0x0FFF);
            assert_eq!(status.charged_state, 2);
            assert_eq!(status.battery_level, 85);
            assert_eq!(status.battery_voltage_mv, 3600);
            assert_eq!(status.mac_address, [0xAA, 0xBB, 0xCC, 0xDD, 0xEE, 0xFF]);
            assert_eq!(status.firmware_version, "V1.2.3");
            assert_eq!(status.product_model, "IM948");
        } else {
            panic!("Expected DeviceStatus response");
        }
    }

    #[test]
    fn test_parse_calibration_status() {
        assert_eq!(
            parse_response(&[0x17, 0]).unwrap(),
            ImuResponse::CalibrationStatus(CalibrationStatus::Running)
        );
        assert_eq!(
            parse_response(&[0x17, 100]).unwrap(),
            ImuResponse::CalibrationStatus(CalibrationStatus::Collected(100))
        );
        assert_eq!(
            parse_response(&[0x17, 251]).unwrap(),
            ImuResponse::CalibrationStatus(CalibrationStatus::NotStarted)
        );
        assert_eq!(
            parse_response(&[0x17, 252]).unwrap(),
            ImuResponse::CalibrationStatus(CalibrationStatus::CompassError)
        );
        assert_eq!(
            parse_response(&[0x17, 253]).unwrap(),
            ImuResponse::CalibrationStatus(CalibrationStatus::AccelError)
        );
        assert_eq!(
            parse_response(&[0x17, 254]).unwrap(),
            ImuResponse::CalibrationStatus(CalibrationStatus::GyroError)
        );
        assert_eq!(
            parse_response(&[0x17, 255]).unwrap(),
            ImuResponse::CalibrationStatus(CalibrationStatus::Finishing)
        );
    }

    #[test]
    fn test_parse_unknown_response() {
        let data = vec![0x99, 0x01, 0x02, 0x03];
        let response = parse_response(&data).unwrap();
        assert_eq!(
            response,
            ImuResponse::Unknown {
                tag: 0x99,
                payload: vec![0x01, 0x02, 0x03],
            }
        );
    }
}

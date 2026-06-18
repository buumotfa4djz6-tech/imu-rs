/// IM948 command definitions
#[derive(Debug, Clone, PartialEq)]
pub enum ImuCommand {
    // 0x02 - Sleep
    Sleep,
    
    // 0x03 - Wake
    Wake,
    
    // 0x04 - Stop Compass Calibration
    StopCompassCalibration,
    
    // 0x05 - Zero Z-axis Angle
    ZeroZAngle,
    
    // 0x06 - Zero World XYZ Angles
    ZeroWorldXYZ,
    
    // 0x07 - Simple Accelerometer Calibration (9 seconds)
    SimpleAccelCalibration,
    
    // 0x08 - Reset Axes
    ResetAxes,
    
    // 0x10 - Query Device Status
    QueryStatus,
    
    // 0x12 - Set Parameters
    SetParams(SetParamsCmd),
    
    // 0x18 - Stop Auto Report
    StopAutoReport,
    
    // 0x19 - Start Auto Report
    StartAutoReport,
}

/// Parameters for SetParams command (0x12)
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SetParamsCmd {
    pub still_threshold: u8,
    pub still_zero_speed: u8,
    pub move_zero_speed: u8,
    pub compass_on: bool,
    pub barometer_filter: u8,
    pub fps: u8,
    pub gyro_filter: u8,
    pub accel_filter: u8,
    pub compass_filter: u8,
    pub subscription_tag: u16,
}

impl ImuCommand {
    /// Encode command to byte vector
    pub fn encode(&self) -> Vec<u8> {
        match self {
            ImuCommand::Sleep => vec![0x02],
            ImuCommand::Wake => vec![0x03],
            ImuCommand::StopCompassCalibration => vec![0x04],
            ImuCommand::ZeroZAngle => vec![0x05],
            ImuCommand::ZeroWorldXYZ => vec![0x06],
            ImuCommand::SimpleAccelCalibration => vec![0x07],
            ImuCommand::ResetAxes => vec![0x08],
            ImuCommand::QueryStatus => vec![0x10],
            ImuCommand::SetParams(params) => {
                let mut data = vec![0x12];
                data.push(params.still_threshold);
                data.push(params.still_zero_speed);
                data.push(params.move_zero_speed);
                
                // Byte 4: compass_on (bit 0) | barometer_filter (bits 1-2)
                let byte4 = ((params.barometer_filter & 0x03) << 1) | (params.compass_on as u8);
                data.push(byte4);
                
                data.push(params.fps);
                data.push(params.gyro_filter);
                data.push(params.accel_filter);
                data.push(params.compass_filter);
                
                // Subscription tag (little endian)
                data.push((params.subscription_tag & 0xFF) as u8);
                data.push(((params.subscription_tag >> 8) & 0xFF) as u8);
                
                data
            }
            ImuCommand::StopAutoReport => vec![0x18],
            ImuCommand::StartAutoReport => vec![0x19],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encode_simple_commands() {
        assert_eq!(ImuCommand::Sleep.encode(), vec![0x02]);
        assert_eq!(ImuCommand::Wake.encode(), vec![0x03]);
        assert_eq!(ImuCommand::StopCompassCalibration.encode(), vec![0x04]);
        assert_eq!(ImuCommand::ZeroZAngle.encode(), vec![0x05]);
        assert_eq!(ImuCommand::ZeroWorldXYZ.encode(), vec![0x06]);
        assert_eq!(ImuCommand::SimpleAccelCalibration.encode(), vec![0x07]);
        assert_eq!(ImuCommand::ResetAxes.encode(), vec![0x08]);
        assert_eq!(ImuCommand::QueryStatus.encode(), vec![0x10]);
        assert_eq!(ImuCommand::StopAutoReport.encode(), vec![0x18]);
        assert_eq!(ImuCommand::StartAutoReport.encode(), vec![0x19]);
    }

    #[test]
    fn test_encode_set_params() {
        let params = SetParamsCmd {
            still_threshold: 5,
            still_zero_speed: 255,
            move_zero_speed: 0,
            compass_on: true,
            barometer_filter: 2,
            fps: 60,
            gyro_filter: 1,
            accel_filter: 3,
            compass_filter: 5,
            subscription_tag: 0x0FFF,
        };
        
        let cmd = ImuCommand::SetParams(params);
        let encoded = cmd.encode();
        
        // Expected: [0x12, 5, 255, 0, 0x05, 60, 1, 3, 5, 0xFF, 0x0F]
        // byte4 = (2 << 1) | 1 = 0x05
        assert_eq!(encoded, vec![0x12, 5, 255, 0, 0x05, 60, 1, 3, 5, 0xFF, 0x0F]);
    }
}

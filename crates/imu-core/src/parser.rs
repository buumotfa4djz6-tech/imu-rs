use crate::types::*;
use crate::error::ParseError;

// Scale factors
const SCALE_ACCEL: f32 = 0.004_785_156;
const SCALE_ANGLE_SPEED: f32 = 0.061_035_156;
const SCALE_MAG: f32 = 0.151_062_01;
const SCALE_TEMPERATURE: f32 = 0.01;
const SCALE_AIR_PRESSURE: f32 = 0.000_238_418_58;
const SCALE_HEIGHT: f32 = 0.001_072_883_6;
const SCALE_QUAT: f32 = 0.000_030_517_578;
const SCALE_ANGLE: f32 = 0.005_493_164;

pub fn parse_imu_reading(data: &[u8]) -> Result<ImuReading, ParseError> {
    if data.len() < 7 {
        return Err(ParseError::TooShort(data.len()));
    }

    if data[0] != 0x11 {
        return Err(ParseError::InvalidHeader(data[0]));
    }

    let ctl = u16::from_le_bytes([data[1], data[2]]);
    let timestamp_ms = u32::from_le_bytes([data[3], data[4], data[5], data[6]]);

    let mut pos = 7;
    let mut reading = ImuReading {
        timestamp_ms,
        acceleration: None,
        acceleration_raw: None,
        angular_velocity: None,
        magnetic_field: None,
        environment: None,
        quaternion: None,
        euler_angles: None,
        position: None,
        activity: None,
        acceleration_nav: None,
        adc_mv: None,
        gpio: None,
    };

    // 0x0001: acceleration (去重力)
    if ctl & 0x0001 != 0 {
        reading.acceleration = Some(read_vec3_scaled(data, &mut pos, SCALE_ACCEL)?);
    }

    // 0x0002: acceleration_raw (含重力)
    if ctl & 0x0002 != 0 {
        reading.acceleration_raw = Some(read_vec3_scaled(data, &mut pos, SCALE_ACCEL)?);
    }

    // 0x0004: angular_velocity
    if ctl & 0x0004 != 0 {
        reading.angular_velocity = Some(read_vec3_scaled(data, &mut pos, SCALE_ANGLE_SPEED)?);
    }

    // 0x0008: magnetic_field
    if ctl & 0x0008 != 0 {
        reading.magnetic_field = Some(read_vec3_scaled(data, &mut pos, SCALE_MAG)?);
    }

    // 0x0010: environment (temp + pressure + altitude)
    if ctl & 0x0010 != 0 {
        check_len(data, pos, 8)?;
        let temperature = read_i16(data, pos) as f32 * SCALE_TEMPERATURE;
        let pressure = read_i24(data, pos + 2) as f32 * SCALE_AIR_PRESSURE;
        let altitude = read_i24(data, pos + 5) as f32 * SCALE_HEIGHT;
        reading.environment = Some(Environment { temperature, pressure, altitude });
        pos += 8;
    }

    // 0x0020: quaternion
    if ctl & 0x0020 != 0 {
        reading.quaternion = Some(read_quat_scaled(data, &mut pos, SCALE_QUAT)?);
    }

    // 0x0040: euler_angles
    if ctl & 0x0040 != 0 {
        reading.euler_angles = Some(read_vec3_scaled(data, &mut pos, SCALE_ANGLE)?);
    }

    // 0x0080: position (mm -> m)
    if ctl & 0x0080 != 0 {
        reading.position = Some(read_vec3_mm(data, &mut pos)?);
    }

    // 0x0100: activity
    if ctl & 0x0100 != 0 {
        check_len(data, pos, 5)?;
        let steps = read_u32(data, pos);
        let flags = data[pos + 4];
        reading.activity = Some(Activity {
            steps,
            walking: flags & 0x01 != 0,
            running: flags & 0x02 != 0,
            biking: flags & 0x04 != 0,
            driving: flags & 0x08 != 0,
        });
        pos += 5;
    }

    // 0x0200: acceleration_nav
    if ctl & 0x0200 != 0 {
        reading.acceleration_nav = Some(read_vec3_scaled(data, &mut pos, SCALE_ACCEL)?);
    }

    // 0x0400: adc_mv
    if ctl & 0x0400 != 0 {
        check_len(data, pos, 2)?;
        reading.adc_mv = Some(read_u16(data, pos));
        pos += 2;
    }

    // 0x0800: gpio
    if ctl & 0x0800 != 0 {
        check_len(data, pos, 1)?;
        reading.gpio = Some(data[pos]);
    }

    Ok(reading)
}

fn check_len(data: &[u8], pos: usize, needed: usize) -> Result<(), ParseError> {
    if pos + needed > data.len() {
        Err(ParseError::Truncated {
            expected: pos + needed,
            actual: data.len(),
        })
    } else {
        Ok(())
    }
}

fn read_vec3_scaled(data: &[u8], pos: &mut usize, scale: f32) -> Result<Vec3, ParseError> {
    check_len(data, *pos, 6)?;
    let x = read_i16(data, *pos) as f32 * scale;
    let y = read_i16(data, *pos + 2) as f32 * scale;
    let z = read_i16(data, *pos + 4) as f32 * scale;
    *pos += 6;
    Ok(Vec3 { x, y, z })
}

fn read_vec3_mm(data: &[u8], pos: &mut usize) -> Result<Vec3, ParseError> {
    check_len(data, *pos, 6)?;
    let x = read_i16(data, *pos) as f32 / 1000.0;
    let y = read_i16(data, *pos + 2) as f32 / 1000.0;
    let z = read_i16(data, *pos + 4) as f32 / 1000.0;
    *pos += 6;
    Ok(Vec3 { x, y, z })
}

fn read_quat_scaled(data: &[u8], pos: &mut usize, scale: f32) -> Result<Quat4, ParseError> {
    check_len(data, *pos, 8)?;
    let w = read_i16(data, *pos) as f32 * scale;
    let x = read_i16(data, *pos + 2) as f32 * scale;
    let y = read_i16(data, *pos + 4) as f32 * scale;
    let z = read_i16(data, *pos + 6) as f32 * scale;
    *pos += 8;
    Ok(Quat4 { w, x, y, z })
}

fn read_i16(data: &[u8], offset: usize) -> i16 {
    i16::from_le_bytes([data[offset], data[offset + 1]])
}

fn read_u16(data: &[u8], offset: usize) -> u16 {
    u16::from_le_bytes([data[offset], data[offset + 1]])
}

fn read_u32(data: &[u8], offset: usize) -> u32 {
    u32::from_le_bytes([data[offset], data[offset + 1], data[offset + 2], data[offset + 3]])
}

fn read_i24(data: &[u8], offset: usize) -> i32 {
    let raw = (data[offset] as u32)
        | ((data[offset + 1] as u32) << 8)
        | ((data[offset + 2] as u32) << 16);
    
    // Sign extension: if bit 23 is set, extend to 32-bit negative
    if raw & 0x800000 != 0 {
        (raw | 0xFF000000) as i32
    } else {
        raw as i32
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_complete_frame() {
        // 来自 data.py 的真实字节序列
        let data = vec![
            0x11, 0xFF, 0x0F, 0xF5, 0xF5, 0x49, 0x47, 0x10, 0x00,
            0xF6, 0xFF, 0xD5, 0xFF, 0x5A, 0xFF, 0x1F, 0xFD, 0x4A, 0x07,
            0x09, 0x00, 0x0A, 0x00, 0x05, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x3A, 0x0B, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x50, 0x78, 0x9F, 0xEB, 0x5C, 0x0C, 0x64, 0xDB, 0x2C, 0xF1,
            0xB0, 0x03, 0x40, 0xE7, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x08, 0x00, 0x00, 0x00, 0x00, 0xFC, 0xFF, 0xE4, 0xFF, 0xDA,
            0xFF, 0xFE, 0x09, 0x01
        ];

        let reading = parse_imu_reading(&data).unwrap();

        // ctl = 0x0FFF, 所有 12 个通道都有数据
        assert!(reading.acceleration.is_some());
        assert!(reading.acceleration_raw.is_some());
        assert!(reading.angular_velocity.is_some());
        assert!(reading.magnetic_field.is_some());
        assert!(reading.environment.is_some());
        assert!(reading.quaternion.is_some());
        assert!(reading.euler_angles.is_some());
        assert!(reading.position.is_some());
        assert!(reading.activity.is_some());
        assert!(reading.acceleration_nav.is_some());
        assert!(reading.adc_mv.is_some());
        assert!(reading.gpio.is_some());

        // 验证 timestamp
        assert_eq!(reading.timestamp_ms, 0x4749F5F5);
    }

    #[test]
    fn test_parse_partial_frame() {
        // ctl = 0x0005 (只启用 acceleration + angular_velocity)
        let data = vec![
            0x11, 0x05, 0x00, 0x01, 0x00, 0x00, 0x00, // header + ctl + timestamp
            // acceleration (6 bytes)
            0xE8, 0x03, 0xD0, 0x07, 0xB8, 0x0B, // 1000, 2000, 3000
            // angular_velocity (6 bytes)
            0x10, 0x27, 0x20, 0x4E, 0x30, 0x75  // 10000, 20000, 30000
        ];

        let reading = parse_imu_reading(&data).unwrap();

        // 只启用的通道有数据
        assert!(reading.acceleration.is_some());
        assert!(reading.angular_velocity.is_some());

        // 未启用的通道为 None
        assert!(reading.acceleration_raw.is_none());
        assert!(reading.magnetic_field.is_none());
        assert!(reading.environment.is_none());
        assert!(reading.quaternion.is_none());
        assert!(reading.euler_angles.is_none());
        assert!(reading.position.is_none());
        assert!(reading.activity.is_none());
        assert!(reading.acceleration_nav.is_none());
        assert!(reading.adc_mv.is_none());
        assert!(reading.gpio.is_none());
    }

    #[test]
    fn test_scale_factors() {
        // ctl = 0x0001 (只启用 acceleration)
        let data = vec![
            0x11, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00,
            // acceleration: raw value 1000
            0xE8, 0x03, 0x00, 0x00, 0x00, 0x00
        ];

        let reading = parse_imu_reading(&data).unwrap();
        let accel = reading.acceleration.unwrap();

        // 1000 * 0.00478515625 = 4.78515625
        assert!((accel.x - 4.78515625).abs() < 1e-6);
    }

    #[test]
    fn test_i24_sign_extension() {
        // ctl = 0x0010 (只启用 environment)
        let data = vec![
            0x11, 0x10, 0x00, 0x00, 0x00, 0x00, 0x00,
            // temperature: 2500 (25.00°C)
            0xC4, 0x09,
            // pressure: 0x800000 (negative, bit 23 set) -> should be negative
            0x00, 0x00, 0x80,
            // altitude: 0x7FFFFF (positive, max value)
            0xFF, 0xFF, 0x7F
        ];

        let reading = parse_imu_reading(&data).unwrap();
        let env = reading.environment.unwrap();

        // temperature: 2500 * 0.01 = 25.0
        assert!((env.temperature - 25.0).abs() < 1e-6);

        // pressure: 0x800000 -> sign extend to 0xFF800000 -> -8388608
        // -8388608 * 0.0002384185791 ≈ -1999.999
        assert!(env.pressure < -1999.0);
        assert!(env.pressure > -2000.1);

        // altitude: 0x7FFFFF = 8388607
        // 8388607 * 0.0010728836 ≈ 8999.99
        assert!(env.altitude > 8999.0);
        assert!(env.altitude < 9000.1);
    }

    #[test]
    fn test_error_handling() {
        // 空帧
        let data = vec![];
        assert!(matches!(parse_imu_reading(&data), Err(ParseError::TooShort(0))));

        // 太短（少于 7 字节）
        let data = vec![0x11, 0x01, 0x00];
        assert!(matches!(parse_imu_reading(&data), Err(ParseError::TooShort(3))));

        // 错误的头字节
        let data = vec![0x12, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00];
        assert!(matches!(parse_imu_reading(&data), Err(ParseError::InvalidHeader(0x12))));

        // 截断帧（声明了 acceleration 但数据不够）
        let data = vec![0x11, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0xE8, 0x03];
        assert!(matches!(parse_imu_reading(&data), Err(ParseError::Truncated { .. })));
    }

    #[test]
    fn test_accessor_methods() {
        // 测试 has_xxx() 和 xxx_or_zero() 方法
        let reading = ImuReading {
            timestamp_ms: 0,
            acceleration: Some(Vec3 { x: 1.0, y: 2.0, z: 3.0 }),
            acceleration_raw: None,
            angular_velocity: Some(Vec3 { x: 4.0, y: 5.0, z: 6.0 }),
            magnetic_field: None,
            environment: None,
            quaternion: None,
            euler_angles: Some(Vec3 { x: 7.0, y: 8.0, z: 9.0 }),
            position: None,
            activity: None,
            acceleration_nav: Some(Vec3 { x: 10.0, y: 11.0, z: 12.0 }),
            adc_mv: None,
            gpio: None,
        };

        // has_xxx()
        assert!(reading.has_acceleration());
        assert!(!reading.has_acceleration_raw());
        assert!(reading.has_angular_velocity());
        assert!(!reading.has_magnetic_field());
        assert!(reading.has_euler_angles());
        assert!(!reading.has_position());
        assert!(reading.has_acceleration_nav());

        // xxx_or_zero()
        let accel = reading.acceleration_or_zero();
        assert_eq!(accel, Vec3 { x: 1.0, y: 2.0, z: 3.0 });

        let accel_raw = reading.acceleration_raw_or_zero();
        assert_eq!(accel_raw, Vec3::ZERO);

        let gyro = reading.angular_velocity_or_zero();
        assert_eq!(gyro, Vec3 { x: 4.0, y: 5.0, z: 6.0 });

        let euler = reading.euler_angles_or_zero();
        assert_eq!(euler, Vec3 { x: 7.0, y: 8.0, z: 9.0 });

        let pos = reading.position_or_zero();
        assert_eq!(pos, Vec3::ZERO);

        let nav = reading.acceleration_nav_or_zero();
        assert_eq!(nav, Vec3 { x: 10.0, y: 11.0, z: 12.0 });
    }
}

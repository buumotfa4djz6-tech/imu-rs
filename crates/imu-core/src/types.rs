#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Vec3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl Vec3 {
    pub const ZERO: Self = Self { x: 0.0, y: 0.0, z: 0.0 };
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Quat4 {
    pub w: f32,
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Environment {
    pub temperature: f32,
    pub pressure: f32,
    pub altitude: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Activity {
    pub steps: u32,
    pub walking: bool,
    pub running: bool,
    pub biking: bool,
    pub driving: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ImuReading {
    pub timestamp_ms: u32,
    pub acceleration: Option<Vec3>,
    pub acceleration_raw: Option<Vec3>,
    pub angular_velocity: Option<Vec3>,
    pub magnetic_field: Option<Vec3>,
    pub environment: Option<Environment>,
    pub quaternion: Option<Quat4>,
    pub euler_angles: Option<Vec3>,
    pub position: Option<Vec3>,
    pub activity: Option<Activity>,
    pub acceleration_nav: Option<Vec3>,
    pub adc_mv: Option<u16>,
    pub gpio: Option<u8>,
}

impl ImuReading {
    pub fn has_acceleration(&self) -> bool {
        self.acceleration.is_some()
    }

    pub fn acceleration_or_zero(&self) -> Vec3 {
        self.acceleration.unwrap_or(Vec3::ZERO)
    }

    pub fn has_acceleration_raw(&self) -> bool {
        self.acceleration_raw.is_some()
    }

    pub fn acceleration_raw_or_zero(&self) -> Vec3 {
        self.acceleration_raw.unwrap_or(Vec3::ZERO)
    }

    pub fn has_angular_velocity(&self) -> bool {
        self.angular_velocity.is_some()
    }

    pub fn angular_velocity_or_zero(&self) -> Vec3 {
        self.angular_velocity.unwrap_or(Vec3::ZERO)
    }

    pub fn has_magnetic_field(&self) -> bool {
        self.magnetic_field.is_some()
    }

    pub fn magnetic_field_or_zero(&self) -> Vec3 {
        self.magnetic_field.unwrap_or(Vec3::ZERO)
    }

    pub fn has_euler_angles(&self) -> bool {
        self.euler_angles.is_some()
    }

    pub fn euler_angles_or_zero(&self) -> Vec3 {
        self.euler_angles.unwrap_or(Vec3::ZERO)
    }

    pub fn has_position(&self) -> bool {
        self.position.is_some()
    }

    pub fn position_or_zero(&self) -> Vec3 {
        self.position.unwrap_or(Vec3::ZERO)
    }

    pub fn has_acceleration_nav(&self) -> bool {
        self.acceleration_nav.is_some()
    }

    pub fn acceleration_nav_or_zero(&self) -> Vec3 {
        self.acceleration_nav.unwrap_or(Vec3::ZERO)
    }
}

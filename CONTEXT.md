# IMU-RS Context

## Domain

IMU-RS is a Rust workspace for interfacing with the IM948 IMU sensor. The system provides:

- **Protocol parsing** — converting raw sensor bytes into structured data types
- **Transport abstraction** — supporting both serial (UART) and BLE (Bluetooth Low Energy) connections
- **Real-time visualization** — GUI application for monitoring sensor data

## Domain Glossary

### Sensor Terms

- **IM948** — The specific IMU sensor model this library supports
- **Subscription tag (ctl)** — A 16-bit bitmask indicating which data channels are active in a sensor reading
- **Frame** — A complete payload received from the sensor (via BLE notification or serial stream)
- **Auto-report** — Continuous data streaming mode where the sensor pushes readings at a configured FPS
- **Command/Response** — Request-response pattern: host sends a command byte sequence, sensor replies with a matching response

### Data Channels

- **Acceleration (aX/aY/aZ)** — Linear acceleration with gravity removed (m/s²)
- **Acceleration raw (AX/AY/AZ)** — Linear acceleration including gravity (m/s²)
- **Angular velocity (GX/GY/GZ)** — Gyroscope readings (°/s)
- **Magnetic field (CX/CY/CZ)** — Magnetometer readings (μT)
- **Environment** — Temperature, air pressure, altitude
- **Quaternion (w/x/y/z)** — Orientation in quaternion form
- **Euler angles (angleX/Y/Z)** — Orientation in Euler angles (°)
- **Position (offsetX/Y/Z)** — Spatial displacement (mm → m)
- **Activity** — Step count, walking/running/biking/driving detection
- **Acceleration nav (asX/asY/asZ)** — Acceleration in navigation coordinate system

### Architecture Terms

- **imu-core** — Pure protocol parsing crate, no IO dependencies
- **imu-transport** — Transport trait + serial/BLE implementations
- **imu-gui** — egui-based GUI application
- **Transport trait** — Async abstraction for serial and BLE communication
- **ImuReading** — Structured sensor data with optional fields per channel
- **ImuCommand** — Typed enum of all sensor commands
- **ImuResponse** — Typed enum of all sensor responses

use crate::transport::{Transport, TransportError};
use imu_core::{ImuCommand, ImuResponse, parse_response};
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::time::{timeout, Duration};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum DeviceError {
    #[error("Transport error: {0}")]
    Transport(#[from] TransportError),
    #[error("Protocol error: {0}")]
    Protocol(#[from] imu_core::ProtocolError),
    #[error("Command timeout")]
    Timeout,
    #[error("Unexpected response tag: expected 0x{expected:02x}, got 0x{actual:02x}")]
    UnexpectedResponse { expected: u8, actual: u8 },
}

pub struct Device<T: Transport> {
    transport: Arc<Mutex<T>>,
    command_timeout: Duration,
}

impl<T: Transport> Device<T> {
    pub fn new(transport: T) -> Self {
        Self {
            transport: Arc::new(Mutex::new(transport)),
            command_timeout: Duration::from_secs(5),
        }
    }

    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.command_timeout = timeout;
        self
    }

    pub async fn connect(&self) -> Result<(), DeviceError> {
        let mut transport = self.transport.lock().await;
        transport.connect().await?;
        Ok(())
    }

    pub async fn disconnect(&self) -> Result<(), DeviceError> {
        let mut transport = self.transport.lock().await;
        transport.disconnect().await?;
        Ok(())
    }

    pub async fn is_connected(&self) -> bool {
        let transport = self.transport.lock().await;
        transport.is_connected()
    }

    /// Send a command and wait for the matching response
    pub async fn send_command(&self, cmd: &ImuCommand) -> Result<ImuResponse, DeviceError> {
        let expected_tag = Self::expected_response_tag(cmd);
        
        // Encode and send command
        let encoded = cmd.encode();
        {
            let mut transport = self.transport.lock().await;
            transport.send(&encoded).await?;
        }
        
        // Wait for response
        let response_data = timeout(self.command_timeout, async {
            loop {
                let mut transport = self.transport.lock().await;
                match transport.receive().await {
                    Ok(data) => {
                        drop(transport);
                        return Ok::<Vec<u8>, DeviceError>(data);
                    }
                    Err(TransportError::ReceiveFailed(_)) => {
                        drop(transport);
                        // No data available, wait a bit and retry
                        tokio::time::sleep(Duration::from_millis(10)).await;
                        continue;
                    }
                    Err(e) => return Err(DeviceError::Transport(e)),
                }
            }
        }).await.map_err(|_| DeviceError::Timeout)??;
        
        // Parse response
        let response = parse_response(&response_data)?;
        
        // Verify response tag matches
        if let Some(actual_tag) = Self::response_tag(&response) {
            if actual_tag != expected_tag {
                return Err(DeviceError::UnexpectedResponse {
                    expected: expected_tag,
                    actual: actual_tag,
                });
            }
        }
        
        Ok(response)
    }

    fn expected_response_tag(cmd: &ImuCommand) -> u8 {
        match cmd {
            ImuCommand::Sleep => 0x02,
            ImuCommand::Wake => 0x03,
            ImuCommand::StopCompassCalibration => 0x04,
            ImuCommand::ZeroZAngle => 0x05,
            ImuCommand::ZeroWorldXYZ => 0x06,
            ImuCommand::SimpleAccelCalibration => 0x07,
            ImuCommand::ResetAxes => 0x08,
            ImuCommand::QueryStatus => 0x10,
            ImuCommand::SetParams(_) => 0x12,
            ImuCommand::StopAutoReport => 0x18,
            ImuCommand::StartAutoReport => 0x19,
        }
    }

    fn response_tag(response: &ImuResponse) -> Option<u8> {
        match response {
            ImuResponse::SleepAck => Some(0x02),
            ImuResponse::WakeAck => Some(0x03),
            ImuResponse::CompassCalibrationEnd => Some(0x04),
            ImuResponse::ZeroZAngleAck => Some(0x05),
            ImuResponse::ZeroWorldXYZAck => Some(0x06),
            ImuResponse::SimpleAccelCalibrationInProgress => Some(0x07),
            ImuResponse::ResetAxesAck => Some(0x08),
            ImuResponse::DeviceStatus(_) => Some(0x10),
            ImuResponse::SensorData(_) => Some(0x11),
            ImuResponse::SetParamsAck => Some(0x12),
            ImuResponse::ClearPositionAck => Some(0x13),
            ImuResponse::RestoreFactoryCalibrationAck => Some(0x14),
            ImuResponse::SaveFactoryCalibrationAck => Some(0x15),
            ImuResponse::ClearStepCountAck => Some(0x16),
            ImuResponse::CalibrationStatus(_) => Some(0x17),
            ImuResponse::AutoReportOff => Some(0x18),
            ImuResponse::AutoReportOn => Some(0x19),
            ImuResponse::Unknown { tag, .. } => Some(*tag),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mock::MockTransport;

    #[tokio::test]
    async fn test_device_connect_disconnect() {
        let transport = MockTransport::new();
        let device = Device::new(transport);
        
        assert!(!device.is_connected().await);
        
        device.connect().await.unwrap();
        assert!(device.is_connected().await);
        
        device.disconnect().await.unwrap();
        assert!(!device.is_connected().await);
    }

    #[tokio::test]
    async fn test_device_send_command() {
        let mut transport = MockTransport::new();
        transport.connect().await.unwrap();
        
        // Add a SleepAck response
        transport.add_response(vec![0x02]).await;
        
        let device = Device::new(transport);
        
        // Send Sleep command
        let response = device.send_command(&ImuCommand::Sleep).await.unwrap();
        assert_eq!(response, ImuResponse::SleepAck);
    }

    #[tokio::test]
    async fn test_device_query_status() {
        let mut transport = MockTransport::new();
        transport.connect().await.unwrap();
        
        // Add a DeviceStatus response (33 bytes)
        let mut status_response = vec![0x10];
        status_response.extend_from_slice(&[5, 255, 0]); // threshold, zero speeds
        status_response.push(0x1D); // flags
        status_response.extend_from_slice(&[60, 1, 3, 5]); // filters, fps
        status_response.extend_from_slice(&[0xFF, 0x0F]); // subscription tag
        status_response.extend_from_slice(&[2, 85]); // battery
        status_response.extend_from_slice(&[0x10, 0x0E]); // voltage
        status_response.extend_from_slice(&[0xAA, 0xBB, 0xCC, 0xDD, 0xEE, 0xFF]); // MAC
        status_response.extend_from_slice(b"V1.2.3"); // firmware
        status_response.extend_from_slice(b"IM948\x00\x00"); // model
        
        transport.add_response(status_response).await;
        
        let device = Device::new(transport);
        
        let response = device.send_command(&ImuCommand::QueryStatus).await.unwrap();
        
        if let ImuResponse::DeviceStatus(status) = response {
            assert_eq!(status.still_threshold, 5);
            assert_eq!(status.fps, 60);
            assert_eq!(status.battery_level, 85);
        } else {
            panic!("Expected DeviceStatus response");
        }
    }

    #[tokio::test]
    async fn test_device_unexpected_response() {
        let mut transport = MockTransport::new();
        transport.connect().await.unwrap();
        
        // Add wrong response (WakeAck instead of SleepAck)
        transport.add_response(vec![0x03]).await;
        
        let device = Device::new(transport);
        
        // Send Sleep command but get WakeAck
        let result = device.send_command(&ImuCommand::Sleep).await;
        assert!(matches!(result, Err(DeviceError::UnexpectedResponse { expected: 0x02, actual: 0x03 })));
    }
}

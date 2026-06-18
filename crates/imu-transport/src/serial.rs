use crate::transport::{Transport, TransportError};
use async_trait::async_trait;
use serialport::SerialPort;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio::task;

/// Serial port information
#[derive(Debug, Clone)]
pub struct SerialPortInfo {
    pub name: String,
}

/// List available serial ports
pub fn list_serial_ports() -> Vec<SerialPortInfo> {
    serialport::available_ports()
        .unwrap_or_default()
        .into_iter()
        .map(|p| SerialPortInfo { name: p.port_name })
        .collect()
}

/// Serial transport implementation
pub struct SerialTransport {
    port: Option<Arc<Mutex<Box<dyn SerialPort>>>>,
    port_name: String,
    baud_rate: u32,
}

impl SerialTransport {
    /// Create a new serial transport (does not open the port yet)
    pub fn new(port_name: &str, baud_rate: u32) -> Self {
        Self {
            port: None,
            port_name: port_name.to_string(),
            baud_rate,
        }
    }

    /// Get the port name
    pub fn port_name(&self) -> &str {
        &self.port_name
    }

    /// Get the baud rate
    pub fn baud_rate(&self) -> u32 {
        self.baud_rate
    }
}

#[async_trait]
impl Transport for SerialTransport {
    async fn connect(&mut self) -> Result<(), TransportError> {
        if self.port.is_some() {
            return Ok(()); // Already connected
        }

        let port_name = self.port_name.clone();
        let baud_rate = self.baud_rate;

        let port = task::spawn_blocking(move || {
            serialport::new(&port_name, baud_rate)
                .timeout(Duration::from_millis(100))
                .open()
                .map_err(|e| TransportError::ConnectionFailed(e.to_string()))
        })
        .await
        .map_err(|e| TransportError::ConnectionFailed(e.to_string()))??;

        self.port = Some(Arc::new(Mutex::new(port)));
        Ok(())
    }

    async fn disconnect(&mut self) -> Result<(), TransportError> {
        self.port = None;
        Ok(())
    }

    async fn send(&mut self, data: &[u8]) -> Result<(), TransportError> {
        let port = self
            .port
            .as_ref()
            .ok_or(TransportError::NotConnected)?
            .clone();

        let data = data.to_vec();

        task::spawn_blocking(move || {
            let mut port = port
                .lock()
                .map_err(|_| TransportError::SendFailed("Failed to acquire port lock".to_string()))?;

            port.write_all(&data)
                .map_err(|e| TransportError::SendFailed(e.to_string()))
        })
        .await
        .map_err(|e| TransportError::SendFailed(e.to_string()))??;

        Ok(())
    }

    async fn receive(&mut self) -> Result<Vec<u8>, TransportError> {
        let port = self
            .port
            .as_ref()
            .ok_or(TransportError::NotConnected)?
            .clone();

        task::spawn_blocking(move || {
            let mut port = port.lock().map_err(|_| {
                TransportError::ReceiveFailed("Failed to acquire port lock".to_string())
            })?;

            let mut buf = [0u8; 256];
            match port.read(&mut buf) {
                Ok(n) if n > 0 => Ok(buf[..n].to_vec()),
                Ok(_) => Err(TransportError::ReceiveFailed("No data available".to_string())),
                Err(ref e) if e.kind() == std::io::ErrorKind::TimedOut => Err(TransportError::Timeout),
                Err(e) => Err(TransportError::ReceiveFailed(e.to_string())),
            }
        })
        .await
        .map_err(|e| TransportError::ReceiveFailed(e.to_string()))?
    }

    fn is_connected(&self) -> bool {
        self.port.is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_list_serial_ports() {
        // This test just ensures the function doesn't panic
        let ports = list_serial_ports();
        // Result depends on system, just verify it returns a Vec
        let _ = ports.len();
    }

    #[test]
    fn test_serial_transport_creation() {
        let transport = SerialTransport::new("COM1", 115200);
        assert_eq!(transport.port_name(), "COM1");
        assert_eq!(transport.baud_rate(), 115200);
        assert!(!transport.is_connected());
    }

    #[tokio::test]
    async fn test_serial_transport_not_connected() {
        let mut transport = SerialTransport::new("COM1", 115200);

        // Should fail when not connected
        let result = transport.send(&[0x01]).await;
        assert!(matches!(result, Err(TransportError::NotConnected)));

        let result = transport.receive().await;
        assert!(matches!(result, Err(TransportError::NotConnected)));
    }

    #[tokio::test]
    async fn test_serial_transport_disconnect_when_not_connected() {
        let mut transport = SerialTransport::new("COM1", 115200);
        // Disconnect should succeed even when not connected
        let result = transport.disconnect().await;
        assert!(result.is_ok());
    }
}

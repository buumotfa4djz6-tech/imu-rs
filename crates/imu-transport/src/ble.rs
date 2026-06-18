use crate::transport::{Transport, TransportError};
use async_trait::async_trait;
use btleplug::api::{Central, Characteristic, Manager as _, Peripheral as _, ScanFilter};
use btleplug::platform::{Manager, Peripheral};
use std::sync::Arc;
use tokio::sync::Mutex;
use uuid::Uuid;

// IM948 Custom Service and Characteristics
const IM948_SERVICE_UUID: &str = "0000ae00-0000-1000-8000-00805f9b34fb";
const IM948_WRITE_CHAR_UUID: &str = "0000ae01-0000-1000-8000-00805f9b34fb";
const IM948_NOTIFY_CHAR_UUID: &str = "0000ae02-0000-1000-8000-00805f9b34fb";

/// BLE device information
#[derive(Debug, Clone)]
pub struct BleDeviceInfo {
    pub name: Option<String>,
    pub address: String,
}

/// BLE transport implementation using btleplug
pub struct BleTransport {
    peripheral: Arc<Mutex<Option<Peripheral>>>,
    write_char: Arc<Mutex<Option<Characteristic>>>,
    notify_char: Arc<Mutex<Option<Characteristic>>>,
    mac_address: String,
    connected: bool,
}

impl BleTransport {
    /// Create a new BLE transport (does not connect yet)
    pub fn new(mac_address: &str) -> Self {
        Self {
            peripheral: Arc::new(Mutex::new(None)),
            write_char: Arc::new(Mutex::new(None)),
            notify_char: Arc::new(Mutex::new(None)),
            mac_address: mac_address.to_string(),
            connected: false,
        }
    }

    /// Get the MAC address
    pub fn mac_address(&self) -> &str {
        &self.mac_address
    }

    /// Discover available BLE devices
    pub async fn discover_devices() -> Result<Vec<BleDeviceInfo>, TransportError> {
        let manager = Manager::new()
            .await
            .map_err(|e| TransportError::ConnectionFailed(format!("Failed to create manager: {}", e)))?;

        let adapters = manager.adapters()
            .await
            .map_err(|e| TransportError::ConnectionFailed(format!("Failed to get adapters: {}", e)))?;

        if adapters.is_empty() {
            return Err(TransportError::ConnectionFailed("No BLE adapters found".to_string()));
        }

        let central = adapters.into_iter().next().unwrap();
        
        central.start_scan(ScanFilter::default())
            .await
            .map_err(|e| TransportError::ConnectionFailed(format!("Failed to start scan: {}", e)))?;

        // Wait a bit for devices to be discovered
        tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;

        let peripherals = central.peripherals()
            .await
            .map_err(|e| TransportError::ConnectionFailed(format!("Failed to get peripherals: {}", e)))?;

        let mut devices = Vec::new();
        for peripheral in peripherals {
            let properties = peripheral.properties()
                .await
                .map_err(|e| TransportError::ConnectionFailed(format!("Failed to get properties: {}", e)))?;

            if let Some(props) = properties {
                let name = props.local_name;
                let address = props.address.to_string();
                
                devices.push(BleDeviceInfo {
                    name,
                    address,
                });
            }
        }

        central.stop_scan()
            .await
            .map_err(|e| TransportError::ConnectionFailed(format!("Failed to stop scan: {}", e)))?;

        Ok(devices)
    }
}

#[async_trait]
impl Transport for BleTransport {
    async fn connect(&mut self) -> Result<(), TransportError> {
        if self.connected {
            return Ok(());
        }

        let manager = Manager::new()
            .await
            .map_err(|e| TransportError::ConnectionFailed(format!("Failed to create manager: {}", e)))?;

        let adapters = manager.adapters()
            .await
            .map_err(|e| TransportError::ConnectionFailed(format!("Failed to get adapters: {}", e)))?;

        let central = adapters.into_iter().next()
            .ok_or_else(|| TransportError::ConnectionFailed("No BLE adapter found".to_string()))?;

        // Find the peripheral by MAC address
        let peripherals = central.peripherals()
            .await
            .map_err(|e| TransportError::ConnectionFailed(format!("Failed to get peripherals: {}", e)))?;

        let target_peripheral = peripherals.into_iter()
            .find(|p| p.address().to_string() == self.mac_address)
            .ok_or_else(|| TransportError::ConnectionFailed(format!("Device {} not found", self.mac_address)))?;

        // Connect to the peripheral
        target_peripheral.connect()
            .await
            .map_err(|e| TransportError::ConnectionFailed(format!("Failed to connect: {}", e)))?;

        // Discover services
        target_peripheral.discover_services()
            .await
            .map_err(|e| TransportError::ConnectionFailed(format!("Failed to discover services: {}", e)))?;

        // Find characteristics
        let characteristics = target_peripheral.characteristics();

        let service_uuid = Uuid::parse_str(IM948_SERVICE_UUID)
            .map_err(|e| TransportError::ConnectionFailed(format!("Invalid service UUID: {}", e)))?;

        let write_uuid = Uuid::parse_str(IM948_WRITE_CHAR_UUID)
            .map_err(|e| TransportError::ConnectionFailed(format!("Invalid write char UUID: {}", e)))?;

        let notify_uuid = Uuid::parse_str(IM948_NOTIFY_CHAR_UUID)
            .map_err(|e| TransportError::ConnectionFailed(format!("Invalid notify char UUID: {}", e)))?;

        let mut write_char = None;
        let mut notify_char = None;

        for char in characteristics {
            if char.service_uuid == service_uuid {
                if char.uuid == write_uuid {
                    write_char = Some(char.clone());
                } else if char.uuid == notify_uuid {
                    notify_char = Some(char.clone());
                }
            }
        }

        let write_char = write_char
            .ok_or_else(|| TransportError::ConnectionFailed("Write characteristic not found".to_string()))?;

        let notify_char = notify_char
            .ok_or_else(|| TransportError::ConnectionFailed("Notify characteristic not found".to_string()))?;

        // Subscribe to notifications
        target_peripheral.subscribe(&notify_char)
            .await
            .map_err(|e| TransportError::ConnectionFailed(format!("Failed to subscribe: {}", e)))?;

        *self.peripheral.lock().await = Some(target_peripheral);
        *self.write_char.lock().await = Some(write_char);
        *self.notify_char.lock().await = Some(notify_char);
        self.connected = true;

        Ok(())
    }

    async fn disconnect(&mut self) -> Result<(), TransportError> {
        if let Some(peripheral) = self.peripheral.lock().await.take() {
            peripheral.disconnect()
                .await
                .map_err(|e| TransportError::ConnectionFailed(format!("Failed to disconnect: {}", e)))?;
        }

        *self.write_char.lock().await = None;
        *self.notify_char.lock().await = None;
        self.connected = false;

        Ok(())
    }

    async fn send(&mut self, data: &[u8]) -> Result<(), TransportError> {
        if !self.connected {
            return Err(TransportError::NotConnected);
        }

        let peripheral = self.peripheral.lock().await.clone()
            .ok_or(TransportError::NotConnected)?;

        let write_char = self.write_char.lock().await.clone()
            .ok_or(TransportError::NotConnected)?;

        peripheral.write(&write_char, data, btleplug::api::WriteType::WithoutResponse)
            .await
            .map_err(|e| TransportError::SendFailed(e.to_string()))?;

        Ok(())
    }

    async fn receive(&mut self) -> Result<Vec<u8>, TransportError> {
        if !self.connected {
            return Err(TransportError::NotConnected);
        }

        let peripheral = self.peripheral.lock().await.clone()
            .ok_or(TransportError::NotConnected)?;

        let notify_char = self.notify_char.lock().await.clone()
            .ok_or(TransportError::NotConnected)?;

        // Wait for notification with timeout
        match tokio::time::timeout(
            tokio::time::Duration::from_secs(5),
            peripheral.notifications()
        ).await {
            Ok(Ok(mut stream)) => {
                use futures::StreamExt;
                if let Some(notification) = stream.next().await {
                    if notification.uuid == notify_char.uuid {
                        Ok(notification.value)
                    } else {
                        Err(TransportError::ReceiveFailed("Received notification from wrong characteristic".to_string()))
                    }
                } else {
                    Err(TransportError::ReceiveFailed("Notification stream ended".to_string()))
                }
            }
            Ok(Err(e)) => Err(TransportError::ReceiveFailed(e.to_string())),
            Err(_) => Err(TransportError::Timeout),
        }
    }

    fn is_connected(&self) -> bool {
        self.connected
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ble_transport_creation() {
        let transport = BleTransport::new("AA:BB:CC:DD:EE:FF");
        assert_eq!(transport.mac_address(), "AA:BB:CC:DD:EE:FF");
        assert!(!transport.is_connected());
    }

    #[tokio::test]
    async fn test_ble_transport_not_connected() {
        let mut transport = BleTransport::new("AA:BB:CC:DD:EE:FF");

        let result = transport.send(&[0x01]).await;
        assert!(matches!(result, Err(TransportError::NotConnected)));

        let result = transport.receive().await;
        assert!(matches!(result, Err(TransportError::NotConnected)));
    }

    #[tokio::test]
    async fn test_ble_transport_disconnect_when_not_connected() {
        let mut transport = BleTransport::new("AA:BB:CC:DD:EE:FF");
        let result = transport.disconnect().await;
        assert!(result.is_ok());
    }
}

use crate::transport::{Transport, TransportError};
use async_trait::async_trait;
use btleplug::api::{Central, Characteristic, Manager as _, Peripheral as _, ScanFilter};
use btleplug::platform::{Adapter, Manager, Peripheral};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{Mutex, mpsc};
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
    pub rssi: Option<i16>,
}

/// Global BLE manager to handle scanning and connection
pub struct BleManager {
    central: Option<Adapter>,
    discovered_peripherals: Arc<Mutex<Vec<Peripheral>>>,
}

impl BleManager {
    pub fn new() -> Self {
        Self {
            central: None,
            discovered_peripherals: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub async fn init(&mut self) -> Result<(), TransportError> {
        let manager = Manager::new()
            .await
            .map_err(|e| TransportError::ConnectionFailed(format!("Failed to create manager: {}", e)))?;

        let adapters = manager.adapters()
            .await
            .map_err(|e| TransportError::ConnectionFailed(format!("Failed to get adapters: {}", e)))?;

        if adapters.is_empty() {
            return Err(TransportError::ConnectionFailed("No BLE adapters found".to_string()));
        }

        self.central = Some(adapters.into_iter().next().unwrap());
        Ok(())
    }

    pub async fn scan(&self, duration_secs: u64) -> Result<Vec<BleDeviceInfo>, TransportError> {
        let central = self.central.as_ref()
            .ok_or_else(|| TransportError::ConnectionFailed("BLE manager not initialized".to_string()))?;

        // Clear previously discovered peripherals
        self.discovered_peripherals.lock().await.clear();

        // Start scanning
        central.start_scan(ScanFilter::default())
            .await
            .map_err(|e| TransportError::ConnectionFailed(format!("Failed to start scan: {}", e)))?;

        // Wait for specified duration
        tokio::time::sleep(Duration::from_secs(duration_secs)).await;

        // Stop scanning
        central.stop_scan()
            .await
            .map_err(|e| TransportError::ConnectionFailed(format!("Failed to stop scan: {}", e)))?;

        // Get discovered peripherals
        let peripherals = central.peripherals()
            .await
            .map_err(|e| TransportError::ConnectionFailed(format!("Failed to get peripherals: {}", e)))?;

        let mut devices = Vec::new();
        let mut discovered_list = self.discovered_peripherals.lock().await;

        for peripheral in peripherals {
            let properties = peripheral.properties()
                .await
                .map_err(|e| TransportError::ConnectionFailed(format!("Failed to get properties: {}", e)))?;

            if let Some(props) = properties {
                let name = props.local_name.clone();
                let address = props.address.to_string();
                let rssi = props.rssi;

                devices.push(BleDeviceInfo {
                    name,
                    address,
                    rssi,
                });

                discovered_list.push(peripheral);
            }
        }

        Ok(devices)
    }

    pub async fn get_peripheral(&self, address: &str) -> Option<Peripheral> {
        let discovered = self.discovered_peripherals.lock().await;
        for peripheral in discovered.iter() {
            if peripheral.address().to_string() == address {
                return Some(peripheral.clone());
            }
        }
        None
    }
}

/// BLE transport implementation using btleplug
pub struct BleTransport {
    peripheral: Option<Peripheral>,
    write_char: Option<Characteristic>,
    notify_char: Option<Characteristic>,
    mac_address: String,
    connected: bool,
    notification_receiver: Option<mpsc::Receiver<Vec<u8>>>,
    notification_task: Option<tokio::task::JoinHandle<()>>,
}

impl BleTransport {
    /// Create a new BLE transport (does not connect yet)
    pub fn new(mac_address: &str) -> Self {
        Self {
            peripheral: None,
            write_char: None,
            notify_char: None,
            mac_address: mac_address.to_string(),
            connected: false,
            notification_receiver: None,
            notification_task: None,
        }
    }

    /// Get the MAC address
    pub fn mac_address(&self) -> &str {
        &self.mac_address
    }

    /// Connect to a peripheral from a BLE manager
    pub async fn connect_with_peripheral(&mut self, peripheral: Peripheral) -> Result<(), TransportError> {
        if self.connected {
            return Ok(());
        }

        // Connect to the peripheral
        peripheral.connect()
            .await
            .map_err(|e| TransportError::ConnectionFailed(format!("Failed to connect: {}", e)))?;

        // Discover services
        peripheral.discover_services()
            .await
            .map_err(|e| TransportError::ConnectionFailed(format!("Failed to discover services: {}", e)))?;

        // Find characteristics
        let characteristics = peripheral.characteristics();

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
        peripheral.subscribe(&notify_char)
            .await
            .map_err(|e| TransportError::ConnectionFailed(format!("Failed to subscribe: {}", e)))?;

        // Create a channel for receiving notifications
        let (tx, rx) = mpsc::channel(100);

        // Spawn a task to handle notifications
        let peripheral_clone = peripheral.clone();
        let notify_uuid_clone = notify_char.uuid;
        let task_handle = tokio::spawn(async move {
            let mut notifications = match peripheral_clone.notifications().await {
                Ok(stream) => stream,
                Err(e) => {
                    eprintln!("Failed to get notification stream: {}", e);
                    return;
                }
            };

            use futures::StreamExt;
            while let Some(notification) = notifications.next().await {
                if notification.uuid == notify_uuid_clone {
                    if tx.send(notification.value).await.is_err() {
                        // Receiver dropped, stop the task
                        break;
                    }
                }
            }
        });

        self.peripheral = Some(peripheral);
        self.write_char = Some(write_char);
        self.notify_char = Some(notify_char);
        self.notification_receiver = Some(rx);
        self.notification_task = Some(task_handle);
        self.connected = true;

        Ok(())
    }
}

impl Drop for BleTransport {
    fn drop(&mut self) {
        // Cancel the notification task when dropping
        if let Some(task) = self.notification_task.take() {
            task.abort();
        }
    }
}

#[async_trait]
impl Transport for BleTransport {
    async fn connect(&mut self) -> Result<(), TransportError> {
        // This method should not be used directly for BLE
        // Use connect_with_peripheral instead
        Err(TransportError::ConnectionFailed(
            "BLE transport requires connect_with_peripheral() - use BleManager to scan and get peripheral first".to_string()
        ))
    }

    async fn disconnect(&mut self) -> Result<(), TransportError> {
        // Cancel notification task
        if let Some(task) = self.notification_task.take() {
            task.abort();
        }
        self.notification_receiver = None;

        if let Some(peripheral) = self.peripheral.take() {
            peripheral.disconnect()
                .await
                .map_err(|e| TransportError::ConnectionFailed(format!("Failed to disconnect: {}", e)))?;
        }

        self.write_char = None;
        self.notify_char = None;
        self.connected = false;

        Ok(())
    }

    async fn send(&mut self, data: &[u8]) -> Result<(), TransportError> {
        if !self.connected {
            return Err(TransportError::NotConnected);
        }

        let peripheral = self.peripheral.as_ref()
            .ok_or(TransportError::NotConnected)?;

        let write_char = self.write_char.as_ref()
            .ok_or(TransportError::NotConnected)?;

        peripheral.write(write_char, data, btleplug::api::WriteType::WithResponse)
            .await
            .map_err(|e| TransportError::SendFailed(e.to_string()))?;

        Ok(())
    }

    async fn receive(&mut self) -> Result<Vec<u8>, TransportError> {
        if !self.connected {
            return Err(TransportError::NotConnected);
        }

        let receiver = self.notification_receiver.as_mut()
            .ok_or(TransportError::NotConnected)?;

        // Wait for notification with timeout
        match tokio::time::timeout(Duration::from_secs(5), receiver.recv()).await {
            Ok(Some(data)) => Ok(data),
            Ok(None) => Err(TransportError::ReceiveFailed("Notification stream ended".to_string())),
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

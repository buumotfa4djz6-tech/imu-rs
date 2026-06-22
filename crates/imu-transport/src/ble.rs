use crate::transport::{Transport, TransportError};
use async_trait::async_trait;
use btleplug::api::{Central, Characteristic, Manager as _, Peripheral as _, ScanFilter};
use btleplug::platform::{Adapter, Manager, Peripheral};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
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
    pub last_seen: Instant,
}

/// BLE discovery event
#[derive(Debug, Clone)]
pub enum BleDeviceEvent {
    Discovered(BleDeviceInfo),
    Lost(String), // MAC address
}

/// Global BLE manager to handle scanning and connection
pub struct BleManager {
    central: Option<Adapter>,
    discovered_peripherals: Arc<Mutex<Vec<Peripheral>>>,
}

impl Default for BleManager {
    fn default() -> Self {
        Self::new()
    }
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

        // Start scanning with service UUID filter for IM948
        let service_uuid = Uuid::parse_str(IM948_SERVICE_UUID)
            .map_err(|e| TransportError::ConnectionFailed(format!("Invalid service UUID: {}", e)))?;
        
        central.start_scan(ScanFilter {
            services: vec![service_uuid],
        })
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
                // Only include devices that have the IM948 service
                let has_im948_service = props.services.iter().any(|uuid| uuid.to_string() == IM948_SERVICE_UUID);
                
                if !has_im948_service {
                    continue;
                }
                
                let name = props.local_name.clone();
                let address = props.address.to_string();
                let rssi = props.rssi;

                devices.push(BleDeviceInfo {
                    name,
                    address,
                    rssi,
                    last_seen: Instant::now(),
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

    /// Start continuous BLE device discovery with streaming events.
    /// Returns a receiver for BleDeviceEvent (Discovered/Lost).
    /// Sends a stop signal via the returned StopHandle to end discovery.
    pub async fn start_discovery(
        &self,
    ) -> Result<(mpsc::Receiver<BleDeviceEvent>, tokio::sync::oneshot::Sender<()>), TransportError> {
        let central = self.central.as_ref()
            .ok_or_else(|| TransportError::ConnectionFailed("BLE manager not initialized".to_string()))?;

        let (event_tx, event_rx) = mpsc::channel(32);
        let (stop_tx, mut stop_rx) = tokio::sync::oneshot::channel();

        // Clone what we need for the background task
        let central_clone = central.clone();
        let known_peripherals = self.discovered_peripherals.clone();

        // Spawn background task for continuous scanning
        tokio::spawn(async move {
            let mut known_devices: HashMap<String, BleDeviceInfo> = HashMap::new();
            let scan_interval = Duration::from_secs(3);
            let device_timeout = Duration::from_secs(10);

            loop {
                // Check if we should stop
                if stop_rx.try_recv().is_ok() {
                    // Stop scanning before exiting
                    let _ = central_clone.stop_scan().await;
                    break;
                }

                // Clear previously discovered peripherals for this scan
                known_peripherals.lock().await.clear();

                // Start scan
                let service_uuid = Uuid::parse_str(IM948_SERVICE_UUID).unwrap();
                if central_clone.start_scan(ScanFilter {
                    services: vec![service_uuid],
                }).await.is_err() {
                    tokio::time::sleep(scan_interval).await;
                    continue;
                }

                // Wait for scan duration
                tokio::time::sleep(Duration::from_secs(2)).await;

                // Stop scan
                let _ = central_clone.stop_scan().await;

                // Get discovered peripherals
                let peripherals = match central_clone.peripherals().await {
                    Ok(p) => p,
                    Err(_) => continue,
                };

                let mut current_devices: HashMap<String, BleDeviceInfo> = HashMap::new();
                let mut discovered_list = known_peripherals.lock().await;

                for peripheral in peripherals {
                    if let Ok(Some(props)) = peripheral.properties().await {
                        // Filter by IM948 service UUID or name
                        let has_im948_service = props.services.iter()
                            .any(|uuid| uuid.to_string() == IM948_SERVICE_UUID);
                        let has_im948_name = props.local_name.as_ref()
                            .map(|name| name.contains("IM948") || name.contains("IMU"))
                            .unwrap_or(false);

                        if has_im948_service || has_im948_name {
                            let info = BleDeviceInfo {
                                name: props.local_name.clone(),
                                address: props.address.to_string(),
                                rssi: props.rssi,
                                last_seen: Instant::now(),
                            };
                            current_devices.insert(info.address.clone(), info);
                            discovered_list.push(peripheral);
                        }
                    }
                }
                drop(discovered_list);

                // Check for newly discovered devices
                for (addr, info) in &current_devices {
                    if !known_devices.contains_key(addr) {
                        let _ = event_tx.send(BleDeviceEvent::Discovered(info.clone())).await;
                    } else {
                        // Update last_seen for known devices
                        if let Some(known) = known_devices.get_mut(addr) {
                            known.last_seen = info.last_seen;
                            known.rssi = info.rssi;
                        }
                    }
                }

                // Check for lost devices (timeout)
                let lost_addrs: Vec<String> = known_devices.iter()
                    .filter(|(addr, info)| {
                        !current_devices.contains_key(*addr) && info.last_seen.elapsed() > device_timeout
                    })
                    .map(|(addr, _)| addr.clone())
                    .collect();

                for addr in &lost_addrs {
                    let _ = event_tx.send(BleDeviceEvent::Lost(addr.clone())).await;
                    known_devices.remove(addr);
                }

                // Update known devices
                known_devices = current_devices;

                // Wait before next scan
                tokio::time::sleep(scan_interval).await;
            }
        });

        Ok((event_rx, stop_tx))
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
                if notification.uuid == notify_uuid_clone && tx.send(notification.value).await.is_err() {
                    // Receiver dropped, stop the task
                    break;
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

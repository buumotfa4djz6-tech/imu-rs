use crate::transport::{Transport, TransportError};
use async_trait::async_trait;
use std::collections::VecDeque;
use tokio::sync::Mutex;

/// MockTransport for testing without real hardware
pub struct MockTransport {
    connected: bool,
    responses: Mutex<VecDeque<Vec<u8>>>,
    sent_data: Mutex<Vec<Vec<u8>>>,
}

impl MockTransport {
    pub fn new() -> Self {
        Self {
            connected: false,
            responses: Mutex::new(VecDeque::new()),
            sent_data: Mutex::new(Vec::new()),
        }
    }

    /// Add a response that will be returned by receive()
    pub async fn add_response(&self, data: Vec<u8>) {
        self.responses.lock().await.push_back(data);
    }

    /// Get all data that was sent via send()
    pub async fn get_sent_data(&self) -> Vec<Vec<u8>> {
        self.sent_data.lock().await.clone()
    }

    /// Clear sent data
    pub async fn clear_sent_data(&self) {
        self.sent_data.lock().await.clear();
    }
}

impl Default for MockTransport {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Transport for MockTransport {
    async fn connect(&mut self) -> Result<(), TransportError> {
        self.connected = true;
        Ok(())
    }

    async fn disconnect(&mut self) -> Result<(), TransportError> {
        self.connected = false;
        Ok(())
    }

    async fn send(&mut self, data: &[u8]) -> Result<(), TransportError> {
        if !self.connected {
            return Err(TransportError::NotConnected);
        }
        self.sent_data.lock().await.push(data.to_vec());
        Ok(())
    }

    async fn receive(&mut self) -> Result<Vec<u8>, TransportError> {
        if !self.connected {
            return Err(TransportError::NotConnected);
        }
        
        match self.responses.lock().await.pop_front() {
            Some(data) => Ok(data),
            None => Err(TransportError::ReceiveFailed("No more responses".to_string())),
        }
    }

    fn is_connected(&self) -> bool {
        self.connected
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_mock_transport_connect_disconnect() {
        let mut transport = MockTransport::new();
        
        assert!(!transport.is_connected());
        
        transport.connect().await.unwrap();
        assert!(transport.is_connected());
        
        transport.disconnect().await.unwrap();
        assert!(!transport.is_connected());
    }

    #[tokio::test]
    async fn test_mock_transport_send_receive() {
        let mut transport = MockTransport::new();
        transport.connect().await.unwrap();
        
        // Add a response
        transport.add_response(vec![0x10, 0x01, 0x02]).await;
        
        // Send data
        transport.send(&[0x02]).await.unwrap();
        
        // Check sent data
        let sent = transport.get_sent_data().await;
        assert_eq!(sent, vec![vec![0x02]]);
        
        // Receive response
        let received = transport.receive().await.unwrap();
        assert_eq!(received, vec![0x10, 0x01, 0x02]);
    }

    #[tokio::test]
    async fn test_mock_transport_not_connected_error() {
        let mut transport = MockTransport::new();
        
        // Should fail when not connected
        let result = transport.send(&[0x02]).await;
        assert!(matches!(result, Err(TransportError::NotConnected)));
        
        let result = transport.receive().await;
        assert!(matches!(result, Err(TransportError::NotConnected)));
    }
}

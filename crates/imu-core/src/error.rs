use thiserror::Error;

#[derive(Error, Debug, Clone, PartialEq)]
pub enum ProtocolError {
    #[error("Frame too short: expected at least {expected} bytes, got {actual}")]
    TooShort { expected: usize, actual: usize },
    #[error("Invalid frame header: expected 0x{expected:02x}, got 0x{actual:02x}")]
    InvalidHeader { expected: u8, actual: u8 },
    #[error("Truncated frame: need {expected} bytes, got {actual}")]
    Truncated { expected: usize, actual: usize },
    #[error("Unknown command tag: 0x{0:02x}")]
    UnknownCommand(u8),
    #[error("Unknown response tag: 0x{0:02x}")]
    UnknownResponse(u8),
}

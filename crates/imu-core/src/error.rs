use thiserror::Error;

#[derive(Error, Debug, Clone, PartialEq)]
pub enum ParseError {
    #[error("Frame too short: expected at least 7 bytes, got {0}")]
    TooShort(usize),
    #[error("Invalid frame header: expected 0x11, got 0x{0:02x}")]
    InvalidHeader(u8),
    #[error("Truncated frame: need {expected} bytes, got {actual}")]
    Truncated { expected: usize, actual: usize },
    #[error("Unknown channel at offset {offset}")]
    UnknownChannel { offset: usize },
}

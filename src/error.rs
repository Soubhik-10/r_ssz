//! Error variants for ssz
use thiserror::Error;

#[derive(Debug, Error, PartialEq, Eq)]
pub enum SSZError {
    #[error("Invalid length: expected {expected}, got {got}")]
    InvalidLength { expected: usize, got: usize },

    #[error("Invalid byte for boolean deserialization")]
    InvalidBooleanByte,

    #[error("Invalid byte length : expected {expected}, got {got}")]
    InvalidByteLength { expected: usize, got: usize },

    #[error("Deserialization error: {0}")]
    DeserializationError(String),

    #[error("Serialization error: {0}")]
    SerializationError(String),

    #[error("Expected delimiter bit not found")]
    ExpectedDelimiterBit,

    #[error("Offset out of bounds for data length")]
    OffsetOutOfBounds,

    #[error("Invalid offset range: start {start} is greater than end {end}")]
    InvalidOffsetRange { start: usize, end: usize },

    #[error("Unknown error occurred")]
    Unknown,
}

//! Error variants for ssz
use thiserror::Error;

#[derive(Debug, Error, PartialEq, Eq)]
pub enum SSZError {
    #[error("Invalid length: expected {expected}, got {got}")]
    InvalidLength { expected: usize, got: usize },

    #[error("Invalid byte for boolean deserialization")]
    InvalidBooleanByte,

    #[error("Deserialization error: {0}")]
    DeserializationError(String),

    #[error("Serialization error: {0}")]
    SerializationError(String),

    #[error("Unknown error occurred")]
    Unknown,
}

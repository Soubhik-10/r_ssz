//! Error variants for SSZ.

use alloc::string::String;
use thiserror::Error;

#[derive(Debug, Error, PartialEq, Eq)]
pub enum SSZError {
    #[error("Invalid length: expected {expected}, got {got}")]
    InvalidLength { expected: usize, got: usize },

    #[error("Invalid byte for boolean deserialization")]
    InvalidBooleanByte,

    #[error("Invalid byte length : expected {expected}, got {got}")]
    InvalidByteLength { expected: usize, got: usize },

    #[error("Invalid byte for deserialization")]
    InvalidByte,

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

    #[error("Invalid Chunk Size")]
    InvalidChunkSize,

    #[error("Invalid Chunk Count: limit {limit}, got {count}")]
    ChunkCountExceedsLimit { count: usize, limit: usize },

    #[error("Unknown selector: {selector}")]
    InvalidInput { selector: usize },

    #[error("Expected further input")]
    ExpectedFurtherInput,

    #[error("{reason} for {selector}")]
    InvalidSelector { reason: String, selector: usize },

    #[error("Invalid bitvector")]
    InvalidBitvector,

    #[error("Unknown error occurred")]
    Unknown,
}

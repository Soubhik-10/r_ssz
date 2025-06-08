//! This module provides a simple serialization and deserialization mechanism for data structures.

use crate::SSZError;

/// The `SimpleSerialize` trait defines methods for serializing and deserializing data structures
pub trait SimpleSerialize: Sized {
    /// Serializes the data structure into a byte vector.
    fn serialize(&self) -> Result<Vec<u8>, SSZError>;
    /// Deserializes the data structure from a byte slice.
    fn deserialize(data: &[u8]) -> Result<Self, SSZError>;
}

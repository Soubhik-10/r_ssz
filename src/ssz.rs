//! This module provides a simple serialization and deserialization mechanism for data structures.

use crate::SSZError;

/// The `SimpleSerialize` trait defines methods for serializing and deserializing data structures
pub trait SimpleSerialize: Sized {
    /// Serializes the data structure into a byte vector.
    fn serialize(&self) -> Result<Vec<u8>, SSZError>;
    /// Deserializes the data structure from a byte slice.
    fn deserialize(data: &[u8]) -> Result<Self, SSZError>;
}

/// The `SszTypeInfo` trait provides information about the size characteristics of a type.
pub trait SszTypeInfo {
    /// If Some(size), then type is fixed-size with known size in bytes.
    /// If None, then it's variable-size (e.g. Vec<u8>, String, etc).
    fn is_fixed_size() -> bool;
    fn fixed_size() -> Option<usize>;
}

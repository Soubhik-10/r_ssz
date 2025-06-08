//! This module provides a simple serialization and deserialization mechanism for data structures.

/// The `SimpleSerialize` trait defines methods for serializing and deserializing data structures
pub trait SimpleSerialize {
    /// Serializes the data structure into a byte vector.
    fn serialize(&self) -> Vec<u8>;
    /// Deserializes the data structure from a byte slice.
    fn deserialize(data: &[u8]) -> Self;
}

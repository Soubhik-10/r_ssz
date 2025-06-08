//! This module provides a simple serialization and deserialization mechanism for data structures.

/// The `SimpleSerialize` trait defines methods for serializing and deserializing data structures
pub trait SimpleSerialize {
    fn serialize(&self) -> Vec<u8>;
    fn deserialize(data: &[u8]) -> Self;
}
